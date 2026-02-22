//! Connection-Renderer für Verbindungen und Richtungspfeile.
//!
//! Aufgeteilt in:
//! - `culling` — Viewport-Culling-Geometrie
//! - `mesh` — Vertex-Generierung (Linien, Pfeile)

mod culling;
mod mesh;

use super::types::{ConnectionVertex, RenderContext, Uniforms};
use crate::{Camera2D, ConnectionDirection, RoadMap};
use eframe::{egui_wgpu, wgpu};
use glam::Vec2;

use culling::{point_in_rect, segment_intersects_rect};
use mesh::{connection_color, push_arrow, push_line_quad};

/// Renderer für Connection-Linien inkl. Pfeilspitzen.
pub struct ConnectionRenderer {
    pipeline: wgpu::RenderPipeline,
    uniform_buffer: wgpu::Buffer,
    bind_group: wgpu::BindGroup,
    vertex_buffer: Option<wgpu::Buffer>,
    vertex_capacity: usize,
    /// Wiederverwendbarer Scratch-Buffer für Vertex-Daten (vermeidet per-Frame-Allokation)
    vertex_scratch: Vec<ConnectionVertex>,
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
            vertex_scratch: Vec::new(),
        }
    }

    /// Rendert alle sichtbaren Verbindungen inkl. Pfeilspitzen.
    ///
    /// Führt vor dem Draw-Call Viewport-Culling durch und aktualisiert
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

        let aspect = viewport_width / viewport_height;
        let zoom_scale = 1.0 / ctx.camera.zoom;
        let base_extent = Camera2D::BASE_WORLD_EXTENT;
        let half_height = base_extent * zoom_scale;
        let half_width = half_height * aspect;
        let padding = ctx.camera.world_per_pixel(viewport_height) * 8.0;
        let visible_min = Vec2::new(
            ctx.camera.position.x - half_width - padding,
            ctx.camera.position.y - half_height - padding,
        );
        let visible_max = Vec2::new(
            ctx.camera.position.x + half_width + padding,
            ctx.camera.position.y + half_height + padding,
        );

        let view_proj = super::types::build_view_projection(ctx.camera, ctx.viewport_size);
        ctx.queue.write_buffer(
            &self.uniform_buffer,
            0,
            bytemuck::cast_slice(&[Uniforms {
                view_proj: view_proj.to_cols_array_2d(),
                aa_params: [1.0, 0.0, 0.0, 0.0],
            }]),
        );

        let mut vertices = std::mem::take(&mut self.vertex_scratch);
        vertices.clear();
        for connection in road_map.connections_iter() {
            let Some(start) = road_map
                .nodes
                .get(&connection.start_id)
                .map(|node| node.position)
            else {
                continue;
            };
            let Some(end) = road_map
                .nodes
                .get(&connection.end_id)
                .map(|node| node.position)
            else {
                continue;
            };

            if !point_in_rect(start, visible_min, visible_max)
                && !point_in_rect(end, visible_min, visible_max)
                && !segment_intersects_rect(start, end, visible_min, visible_max)
            {
                continue;
            }

            let delta = end - start;
            let length = delta.length();
            if length < f32::EPSILON {
                continue;
            }

            let direction = delta / length;
            let color = connection_color(connection.direction, connection.priority, ctx.options);
            let thickness = match connection.priority {
                crate::ConnectionPriority::Regular => ctx.options.connection_thickness_world,
                crate::ConnectionPriority::SubPriority => {
                    ctx.options.connection_thickness_subprio_world
                }
            };

            push_line_quad(&mut vertices, start, end, thickness, color);

            match connection.direction {
                ConnectionDirection::Regular => {
                    let center = start + direction * (length * 0.5);
                    push_arrow(
                        &mut vertices,
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
                        &mut vertices,
                        center,
                        direction,
                        ctx.options.arrow_length_world,
                        ctx.options.arrow_width_world,
                        color,
                    );
                }
                ConnectionDirection::Dual => {
                    let offset = ctx.options.arrow_length_world * 0.6;
                    let forward_center = start + direction * (length * 0.5 + offset);
                    let backward_center = start + direction * (length * 0.5 - offset);
                    push_arrow(
                        &mut vertices,
                        forward_center,
                        direction,
                        ctx.options.arrow_length_world,
                        ctx.options.arrow_width_world,
                        color,
                    );
                    push_arrow(
                        &mut vertices,
                        backward_center,
                        -direction,
                        ctx.options.arrow_length_world,
                        ctx.options.arrow_width_world,
                        color,
                    );
                }
            }
        }

        if vertices.is_empty() {
            self.vertex_scratch = vertices;
            return;
        }

        if self.vertex_buffer.is_none() || vertices.len() > self.vertex_capacity {
            let vertex_size = std::mem::size_of::<ConnectionVertex>() as u64;
            let buffer_size = (vertices.len() as u64) * vertex_size;
            self.vertex_buffer = Some(ctx.device.create_buffer(&wgpu::BufferDescriptor {
                label: Some("Connection Vertex Buffer"),
                size: buffer_size,
                usage: wgpu::BufferUsages::VERTEX | wgpu::BufferUsages::COPY_DST,
                mapped_at_creation: false,
            }));
            self.vertex_capacity = vertices.len();
        }

        if let Some(vertex_buffer) = &self.vertex_buffer {
            ctx.queue
                .write_buffer(vertex_buffer, 0, bytemuck::cast_slice(&vertices));
        }

        let Some(vertex_buffer) = self.vertex_buffer.as_ref() else {
            log::error!("ConnectionRenderer: missing vertex buffer before draw call");
            self.vertex_scratch = vertices;
            return;
        };

        render_pass.set_pipeline(&self.pipeline);
        render_pass.set_bind_group(0, &self.bind_group, &[]);
        render_pass.set_vertex_buffer(0, vertex_buffer.slice(..));
        render_pass.draw(0..vertices.len() as u32, 0..1);

        self.vertex_scratch = vertices;
    }
}
