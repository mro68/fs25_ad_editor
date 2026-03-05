//! Connection-Renderer fuer Verbindungen und Richtungspfeile.
//!
//! Aufgeteilt in:
//! - `culling` — Viewport-Culling-Geometrie
//! - `mesh` — Vertex-Generierung (Linien, Pfeile)

mod culling;
mod mesh;

use super::types::{compute_visible_rect, ConnectionVertex, RenderContext, Uniforms};
use crate::{ConnectionDirection, RoadMap};
use eframe::{egui_wgpu, wgpu};

use culling::{point_in_rect, segment_intersects_rect_cached};
use mesh::{connection_color, push_arrow, push_line_quad};
use std::collections::HashMap;

/// Renderer fuer Connection-Linien inkl. Pfeilspitzen.
pub struct ConnectionRenderer {
    pipeline: wgpu::RenderPipeline,
    uniform_buffer: wgpu::Buffer,
    bind_group: wgpu::BindGroup,
    vertex_buffer: Option<wgpu::Buffer>,
    vertex_capacity: usize,
    /// Wiederverwendbarer Scratch-Buffer fuer Vertex-Daten (vermeidet per-Frame-Allokation)
    vertex_scratch: Vec<ConnectionVertex>,
    /// Persistenter Positions-Cache: Node-ID → Weltposition, wird pro Frame per clear() geleert.
    /// Vermeidet wiederholte road_map-Lookups fuer denselben Node innerhalb eines Frames.
    pos_cache: HashMap<u64, glam::Vec2>,
}

impl ConnectionRenderer {
    /// Erstellt einen neuen Connection-Renderer.
    pub fn new(render_state: &egui_wgpu::RenderState, shader: &wgpu::ShaderModule) -> Self {
        let device = &render_state.device;

        let uniform_buffer = device.create_buffer(&wgpu::BufferDescriptor {
            label: Some("Connection Uniform Buffer"),
            size: std::mem::size_of::<Uniforms>() as u64,
            usage: wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST,
            mapped_at_creation: false,
        });

        let bind_group_layout = device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
            label: Some("Connection Bind Group Layout"),
            entries: &[wgpu::BindGroupLayoutEntry {
                binding: 0,
                visibility: wgpu::ShaderStages::VERTEX,
                ty: wgpu::BindingType::Buffer {
                    ty: wgpu::BufferBindingType::Uniform,
                    has_dynamic_offset: false,
                    min_binding_size: None,
                },
                count: None,
            }],
        });

        let bind_group = device.create_bind_group(&wgpu::BindGroupDescriptor {
            label: Some("Connection Bind Group"),
            layout: &bind_group_layout,
            entries: &[wgpu::BindGroupEntry {
                binding: 0,
                resource: uniform_buffer.as_entire_binding(),
            }],
        });

        let pipeline_layout = device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
            label: Some("Connection Pipeline Layout"),
            bind_group_layouts: &[&bind_group_layout],
            push_constant_ranges: &[],
        });

        let pipeline = device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
            label: Some("Connection Pipeline"),
            layout: Some(&pipeline_layout),
            vertex: wgpu::VertexState {
                module: shader,
                entry_point: Some("vs_connection"),
                buffers: &[ConnectionVertex::desc()],
                compilation_options: Default::default(),
            },
            fragment: Some(wgpu::FragmentState {
                module: shader,
                entry_point: Some("fs_connection"),
                targets: &[Some(wgpu::ColorTargetState {
                    format: render_state.target_format,
                    blend: Some(wgpu::BlendState::ALPHA_BLENDING),
                    write_mask: wgpu::ColorWrites::ALL,
                })],
                compilation_options: Default::default(),
            }),
            primitive: wgpu::PrimitiveState {
                topology: wgpu::PrimitiveTopology::TriangleList,
                strip_index_format: None,
                front_face: wgpu::FrontFace::Ccw,
                cull_mode: None,
                unclipped_depth: false,
                polygon_mode: wgpu::PolygonMode::Fill,
                conservative: false,
            },
            depth_stencil: None,
            multisample: wgpu::MultisampleState {
                count: 4,
                mask: !0,
                alpha_to_coverage_enabled: true,
            },
            multiview: None,
            cache: None,
        });

        Self {
            pipeline,
            uniform_buffer,
            bind_group,
            vertex_buffer: None,
            vertex_capacity: 0,
            vertex_scratch: Vec::with_capacity(1024),
            pos_cache: HashMap::with_capacity(256),
        }
    }

    /// Rendert alle sichtbaren Verbindungen inkl. Pfeilspitzen.
    ///
    /// Fuehrt vor dem Draw-Call Viewport-Culling durch und aktualisiert
    /// den Vertex-Buffer nur bei Bedarf.
    pub fn render(
        &mut self,
        ctx: &RenderContext,
        render_pass: &mut wgpu::RenderPass<'static>,
        road_map: &RoadMap,
    ) {
        let viewport_width = ctx.viewport_size[0];
        let viewport_height = ctx.viewport_size[1];
        if !viewport_width.is_finite()
            || !viewport_height.is_finite()
            || viewport_width <= 0.0
            || viewport_height <= 0.0
        {
            return;
        }

        if road_map.connection_count() == 0 {
            return;
        }

        let (visible_min, visible_max) = compute_visible_rect(ctx);

        let view_proj = super::types::build_view_projection(ctx.camera, ctx.viewport_size);
        ctx.queue.write_buffer(
            &self.uniform_buffer,
            0,
            bytemuck::cast_slice(&[Uniforms {
                view_proj: view_proj.to_cols_array_2d(),
                aa_params: [1.0, 0.0, 0.0, 0.0],
            }]),
        );

        // Reuse the scratch buffer and ensure an initial reserve to avoid
        // repeated reallocations for large maps.
        self.vertex_scratch.clear();

        // Persistenten Positions-Cache leeren — Eintraege bleiben allokiert,
        // dadurch entfaellt die per-Frame HashMap-Allokation.
        self.pos_cache.clear();

        // Precompute viewport corners once for the culling calls.
        let bottom_left = glam::Vec2::new(visible_min.x, visible_min.y);
        let bottom_right = glam::Vec2::new(visible_max.x, visible_min.y);
        let top_right = glam::Vec2::new(visible_max.x, visible_max.y);
        let top_left = glam::Vec2::new(visible_min.x, visible_max.y);

        // Separate &mut-Borrows auf Struct-Felder vor dem Loop, damit der
        // Borrow-Checker gleichzeitigen Zugriff auf pos_cache und vertex_scratch
        // akzeptiert (kein simultanes &mut self noetig).
        // Der Block begrenzt die Borrow-Lebensdauer, damit self.vertex_scratch
        // nach dem Loop wieder direkt zugaenglich ist.
        {
            let pos_cache = &mut self.pos_cache;
            let vertex_scratch = &mut self.vertex_scratch;

            for connection in road_map.connections_iter() {
                // Verbindungen zu ausgeblendeten Nodes ueberspringen
                if ctx.hidden_node_ids.contains(&connection.start_id)
                    || ctx.hidden_node_ids.contains(&connection.end_id)
                {
                    continue;
                }
                // Lazy cache lookup/insert — reduziert road_map.nodes HashMap-Lookups
                let start = match pos_cache.get(&connection.start_id) {
                    Some(p) => *p,
                    None => match road_map.nodes.get(&connection.start_id) {
                        Some(n) => {
                            let p = n.position;
                            pos_cache.insert(connection.start_id, p);
                            p
                        }
                        None => continue,
                    },
                };

                let end = match pos_cache.get(&connection.end_id) {
                    Some(p) => *p,
                    None => match road_map.nodes.get(&connection.end_id) {
                        Some(n) => {
                            let p = n.position;
                            pos_cache.insert(connection.end_id, p);
                            p
                        }
                        None => continue,
                    },
                };

                if !point_in_rect(start, visible_min, visible_max)
                    && !point_in_rect(end, visible_min, visible_max)
                    && !segment_intersects_rect_cached(
                        start,
                        end,
                        bottom_left,
                        bottom_right,
                        top_right,
                        top_left,
                    )
                {
                    continue;
                }

                let delta = end - start;
                let length = delta.length();
                if length < f32::EPSILON {
                    continue;
                }

                let direction = delta / length;
                let color =
                    connection_color(connection.direction, connection.priority, ctx.options);
                let thickness = match connection.priority {
                    crate::ConnectionPriority::Regular => ctx.options.connection_thickness_world,
                    crate::ConnectionPriority::SubPriority => {
                        ctx.options.connection_thickness_subprio_world
                    }
                };

                push_line_quad(vertex_scratch, start, end, thickness, color);

                match connection.direction {
                    ConnectionDirection::Regular => {
                        let center = start + direction * (length * 0.5);
                        push_arrow(
                            vertex_scratch,
                            center,
                            direction,
                            ctx.options.arrow_length_world,
                            ctx.options.arrow_width_world,
                            color,
                        );
                    }
                    ConnectionDirection::Reverse => {
                        let center = start + direction * (length * 0.5);
                        push_arrow(
                            vertex_scratch,
                            center,
                            direction,
                            ctx.options.arrow_length_world,
                            ctx.options.arrow_width_world,
                            color,
                        );
                    }
                    ConnectionDirection::Dual => {
                        // Bidirektionale Verbindungen brauchen keine Pfeile —
                        // die Richtung ist implizit, die Farbe unterscheidet sie bereits.
                    }
                }
            }
        } // pos_cache und vertex_scratch borrows enden hier

        if self.vertex_scratch.is_empty() {
            return;
        }

        if self.vertex_buffer.is_none() || self.vertex_scratch.len() > self.vertex_capacity {
            let vertex_size = std::mem::size_of::<ConnectionVertex>() as u64;
            // Use next_power_of_two capacity to reduce future reallocations.
            let new_capacity = self
                .vertex_scratch
                .len()
                .checked_next_power_of_two()
                .unwrap_or(self.vertex_scratch.len());
            let buffer_size = (new_capacity as u64) * vertex_size;
            self.vertex_buffer = Some(ctx.device.create_buffer(&wgpu::BufferDescriptor {
                label: Some("Connection Vertex Buffer"),
                size: buffer_size,
                usage: wgpu::BufferUsages::VERTEX | wgpu::BufferUsages::COPY_DST,
                mapped_at_creation: false,
            }));
            self.vertex_capacity = new_capacity;
        }

        if let Some(vertex_buffer) = &self.vertex_buffer {
            ctx.queue
                .write_buffer(vertex_buffer, 0, bytemuck::cast_slice(&self.vertex_scratch));
        }

        let Some(vertex_buffer) = self.vertex_buffer.as_ref() else {
            log::error!("ConnectionRenderer: missing vertex buffer before draw call");
            return;
        };

        render_pass.set_pipeline(&self.pipeline);
        render_pass.set_bind_group(0, &self.bind_group, &[]);
        render_pass.set_vertex_buffer(0, vertex_buffer.slice(..));
        render_pass.draw(0..self.vertex_scratch.len() as u32, 0..1);
    }
}
