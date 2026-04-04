//! Connection-Renderer fuer Verbindungen und Richtungspfeile.
//!
//! Aufgeteilt in:
//! - `culling` — Viewport-Culling-Geometrie
//! - `mesh` — Vertex-Generierung (Linien, Pfeile)

mod culling;
mod mesh;

use super::fingerprint::RenderFingerprint;
use super::types::{compute_visible_rect, ConnectionVertex, RenderContext, Uniforms};
use super::RendererTargetConfig;
use crate::shared::{RenderConnectionDirection, RenderConnectionPriority, RenderMap};

use culling::{point_in_rect, segment_intersects_rect_cached};
use mesh::{connection_color, push_arrow, push_line_quad};

/// Renderer fuer Connection-Linien inkl. Pfeilspitzen.
pub struct ConnectionRenderer {
    pipeline: wgpu::RenderPipeline,
    uniform_buffer: wgpu::Buffer,
    bind_group: wgpu::BindGroup,
    vertex_buffer: Option<wgpu::Buffer>,
    vertex_capacity: usize,
    /// Wiederverwendbarer Scratch-Buffer fuer Vertex-Daten (vermeidet per-Frame-Allokation)
    vertex_scratch: Vec<ConnectionVertex>,
    /// Fingerabdruck der letzten Render-Inputs fuer Buffer-Skip-Detection.
    last_fingerprint: Option<RenderFingerprint>,
    /// Vertex-Anzahl des letzten Render-Passes (fuer Draw-Call bei Skip).
    last_vertex_count: u32,
}

impl ConnectionRenderer {
    /// Erstellt einen neuen Connection-Renderer.
    pub fn new(
        device: &wgpu::Device,
        shader: &wgpu::ShaderModule,
        target_config: RendererTargetConfig,
    ) -> Self {

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
                    format: target_config.color_format,
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
                count: target_config.sample_count,
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
            last_fingerprint: None,
            last_vertex_count: 0,
        }
    }

    /// Rendert alle sichtbaren Verbindungen inkl. Pfeilspitzen.
    ///
    /// Fuehrt vor dem Draw-Call Viewport-Culling durch und aktualisiert
    /// den Vertex-Buffer nur bei Bedarf.
    pub fn render(
        &mut self,
        ctx: &RenderContext,
        render_pass: &mut wgpu::RenderPass<'_>,
        render_map: &RenderMap,
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

        if render_map.connections().is_empty() {
            return;
        }

        // Fingerabdruck berechnen und mit dem letzten Frame vergleichen.
        // Bei Uebereinstimmung koennen O(n)-Loop und GPU-Upload uebersprungen werden.
        let new_fp = RenderFingerprint::from_context(ctx, render_map);

        let skip_rebuild = self.last_fingerprint.as_ref() == Some(&new_fp);
        if skip_rebuild {
            // Inputs unveraendert — Draw-Call mit gespeichertem Ergebnis wiederholen.
            if self.last_vertex_count == 0 || self.vertex_buffer.is_none() {
                return; // nichts zu zeichnen
            }
        } else {
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

            // Zoom-Kompensationsfaktor einmalig pro Frame berechnen (nicht pro Verbindung).
            let compensation = ctx.options.zoom_compensation(ctx.camera.zoom);
            // Pixel -> Welteinheiten-Faktor fuer Mindestgroessen.
            let wpp = ctx.camera.world_per_pixel(ctx.viewport_size[1]);
            let min_thickness = ctx.options.min_connection_width_px * wpp;
            let min_arrow = ctx.options.min_arrow_size_px * wpp;

            // Precompute viewport corners once for the culling calls.
            let bottom_left = glam::Vec2::new(visible_min.x, visible_min.y);
            let bottom_right = glam::Vec2::new(visible_max.x, visible_min.y);
            let top_right = glam::Vec2::new(visible_max.x, visible_max.y);
            let top_left = glam::Vec2::new(visible_min.x, visible_max.y);

            for connection in render_map.connections() {
                if ctx.hidden_node_ids.contains(&connection.start_id)
                    || ctx.hidden_node_ids.contains(&connection.end_id)
                {
                    continue;
                }

                let start = connection.start_pos;
                let end = connection.end_pos;

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
                let thickness = (match connection.priority {
                    RenderConnectionPriority::Regular => ctx.options.connection_thickness_world,
                    RenderConnectionPriority::SubPriority => {
                        ctx.options.connection_thickness_subprio_world
                    }
                } * compensation)
                    .max(min_thickness);

                push_line_quad(&mut self.vertex_scratch, start, end, thickness, color);

                match connection.direction {
                    RenderConnectionDirection::Regular | RenderConnectionDirection::Reverse => {
                        let arrow_dir =
                            if connection.direction == RenderConnectionDirection::Reverse {
                                -direction
                            } else {
                                direction
                            };
                        let center = start + direction * (length * 0.5);
                        push_arrow(
                            &mut self.vertex_scratch,
                            center,
                            arrow_dir,
                            (ctx.options.arrow_length_world * compensation).max(min_arrow),
                            (ctx.options.arrow_width_world * compensation).max(min_arrow),
                            color,
                        );
                    }
                    RenderConnectionDirection::Dual => {
                        // Bidirektionale Verbindungen brauchen keine Pfeile —
                        // die Richtung ist implizit, die Farbe unterscheidet sie bereits.
                    }
                }
            }

            if self.vertex_scratch.is_empty() {
                // Fingerabdruck speichern damit bei naechstem identischen Frame fruehzeitig
                // abgebrochen werden kann.
                self.last_fingerprint = Some(new_fp);
                self.last_vertex_count = 0;
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
                ctx.queue.write_buffer(
                    vertex_buffer,
                    0,
                    bytemuck::cast_slice(&self.vertex_scratch),
                );
            }

            self.last_vertex_count = self.vertex_scratch.len() as u32;
            self.last_fingerprint = Some(new_fp);
        }

        // Draw-Call (laeuft immer — sowohl nach Rebuild als auch bei Skip)
        let Some(vertex_buffer) = self.vertex_buffer.as_ref() else {
            log::error!("ConnectionRenderer: missing vertex buffer before draw call");
            return;
        };

        render_pass.set_pipeline(&self.pipeline);
        render_pass.set_bind_group(0, &self.bind_group, &[]);
        render_pass.set_vertex_buffer(0, vertex_buffer.slice(..));
        render_pass.draw(0..self.last_vertex_count, 0..1);
    }
}
