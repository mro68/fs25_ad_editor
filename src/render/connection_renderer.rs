//! Connection-Renderer für Verbindungen und Richtungspfeile.

use super::types::{ConnectionVertex, RenderContext, Uniforms};
use crate::shared::EditorOptions;
use crate::{Camera2D, ConnectionDirection, ConnectionPriority, RoadMap};
use eframe::{egui_wgpu, wgpu};
use glam::Vec2;

/// Renderer für Connection-Linien inkl. Pfeilspitzen.
pub struct ConnectionRenderer {
    pipeline: wgpu::RenderPipeline,
    uniform_buffer: wgpu::Buffer,
    bind_group: wgpu::BindGroup,
    vertex_buffer: Option<wgpu::Buffer>,
    vertex_capacity: usize,
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

        let mut vertices = Vec::new();
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
                ConnectionPriority::Regular => ctx.options.connection_thickness_world,
                ConnectionPriority::SubPriority => ctx.options.connection_thickness_subprio_world,
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
                    // Pfeile leicht versetzt, damit sie sich nicht überdecken
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
            return;
        };

        render_pass.set_pipeline(&self.pipeline);
        render_pass.set_bind_group(0, &self.bind_group, &[]);
        render_pass.set_vertex_buffer(0, vertex_buffer.slice(..));
        render_pass.draw(0..vertices.len() as u32, 0..1);
    }
}

fn point_in_rect(point: Vec2, min: Vec2, max: Vec2) -> bool {
    point.x >= min.x && point.x <= max.x && point.y >= min.y && point.y <= max.y
}

fn segment_intersects_rect(start: Vec2, end: Vec2, min: Vec2, max: Vec2) -> bool {
    if point_in_rect(start, min, max) || point_in_rect(end, min, max) {
        return true;
    }

    let bottom_left = Vec2::new(min.x, min.y);
    let bottom_right = Vec2::new(max.x, min.y);
    let top_right = Vec2::new(max.x, max.y);
    let top_left = Vec2::new(min.x, max.y);

    segments_intersect(start, end, bottom_left, bottom_right)
        || segments_intersect(start, end, bottom_right, top_right)
        || segments_intersect(start, end, top_right, top_left)
        || segments_intersect(start, end, top_left, bottom_left)
}

fn segments_intersect(a1: Vec2, a2: Vec2, b1: Vec2, b2: Vec2) -> bool {
    let o1 = orientation(a1, a2, b1);
    let o2 = orientation(a1, a2, b2);
    let o3 = orientation(b1, b2, a1);
    let o4 = orientation(b1, b2, a2);

    if o1 * o2 < 0.0 && o3 * o4 < 0.0 {
        return true;
    }

    const EPS: f32 = 1e-6;
    (o1.abs() <= EPS && point_on_segment(b1, a1, a2))
        || (o2.abs() <= EPS && point_on_segment(b2, a1, a2))
        || (o3.abs() <= EPS && point_on_segment(a1, b1, b2))
        || (o4.abs() <= EPS && point_on_segment(a2, b1, b2))
}

fn orientation(a: Vec2, b: Vec2, c: Vec2) -> f32 {
    (b.x - a.x) * (c.y - a.y) - (b.y - a.y) * (c.x - a.x)
}

fn point_on_segment(point: Vec2, seg_start: Vec2, seg_end: Vec2) -> bool {
    const EPS: f32 = 1e-6;
    point.x >= seg_start.x.min(seg_end.x) - EPS
        && point.x <= seg_start.x.max(seg_end.x) + EPS
        && point.y >= seg_start.y.min(seg_end.y) - EPS
        && point.y <= seg_start.y.max(seg_end.y) + EPS
}

fn connection_color(
    direction: ConnectionDirection,
    priority: ConnectionPriority,
    options: &EditorOptions,
) -> [f32; 4] {
    let base = match direction {
        ConnectionDirection::Regular => options.connection_color_regular,
        ConnectionDirection::Dual => options.connection_color_dual,
        ConnectionDirection::Reverse => options.connection_color_reverse,
    };

    match priority {
        ConnectionPriority::Regular => base,
        ConnectionPriority::SubPriority => [
            (base[0] + 1.0) * 0.5,
            (base[1] + 1.0) * 0.5,
            (base[2] + 1.0) * 0.5,
            base[3],
        ],
    }
}

fn push_line_quad(
    vertices: &mut Vec<ConnectionVertex>,
    start: Vec2,
    end: Vec2,
    thickness: f32,
    color: [f32; 4],
) {
    let dir = (end - start).normalize();
    let perp = Vec2::new(-dir.y, dir.x) * (thickness * 0.5);

    let v0 = start + perp;
    let v1 = start - perp;
    let v2 = end + perp;
    let v3 = end - perp;

    vertices.push(ConnectionVertex::new([v0.x, v0.y], color));
    vertices.push(ConnectionVertex::new([v1.x, v1.y], color));
    vertices.push(ConnectionVertex::new([v2.x, v2.y], color));

    vertices.push(ConnectionVertex::new([v2.x, v2.y], color));
    vertices.push(ConnectionVertex::new([v1.x, v1.y], color));
    vertices.push(ConnectionVertex::new([v3.x, v3.y], color));
}

fn push_arrow(
    vertices: &mut Vec<ConnectionVertex>,
    center: Vec2,
    direction: Vec2,
    length: f32,
    width: f32,
    color: [f32; 4],
) {
    let dir = direction.normalize();
    let perp = Vec2::new(-dir.y, dir.x);

    let tip = center + dir * (length * 0.5);
    let base = center - dir * (length * 0.5);
    let left = base + perp * (width * 0.5);
    let right = base - perp * (width * 0.5);

    vertices.push(ConnectionVertex::new([tip.x, tip.y], color));
    vertices.push(ConnectionVertex::new([left.x, left.y], color));
    vertices.push(ConnectionVertex::new([right.x, right.y], color));
}

#[cfg(test)]
mod tests {
    use super::{point_in_rect, segment_intersects_rect};
    use glam::Vec2;

    #[test]
    fn test_point_in_rect_inclusive_edges() {
        let min = Vec2::new(-1.0, -1.0);
        let max = Vec2::new(1.0, 1.0);

        assert!(point_in_rect(Vec2::new(0.0, 0.0), min, max));
        assert!(point_in_rect(Vec2::new(1.0, 1.0), min, max));
        assert!(!point_in_rect(Vec2::new(1.1, 1.0), min, max));
    }

    #[test]
    fn test_segment_intersects_rect_when_crossing_view() {
        let min = Vec2::new(-1.0, -1.0);
        let max = Vec2::new(1.0, 1.0);

        let start = Vec2::new(-2.0, 0.0);
        let end = Vec2::new(2.0, 0.0);
        assert!(segment_intersects_rect(start, end, min, max));
    }

    #[test]
    fn test_segment_does_not_intersect_rect_when_fully_outside() {
        let min = Vec2::new(-1.0, -1.0);
        let max = Vec2::new(1.0, 1.0);

        let start = Vec2::new(2.0, 2.0);
        let end = Vec2::new(3.0, 3.0);
        assert!(!segment_intersects_rect(start, end, min, max));
    }
}
