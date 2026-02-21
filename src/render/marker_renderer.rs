//! Marker-Renderer mit GPU-Instancing für Map-Marker (Pin-Symbole).

use super::types::{MarkerInstance, RenderQuality, Uniforms, Vertex};
use crate::shared::EditorOptions;
use crate::RoadMap;
use eframe::{egui_wgpu, wgpu};
use wgpu::util::DeviceExt;

/// Renderer für Map-Marker (Pin-Symbole)
pub struct MarkerRenderer {
    pipeline: wgpu::RenderPipeline,
    vertex_buffer: wgpu::Buffer,
    uniform_buffer: wgpu::Buffer,
    bind_group: wgpu::BindGroup,
    instance_buffer: Option<wgpu::Buffer>,
    instance_capacity: usize,
}

impl MarkerRenderer {
    /// Erstellt einen neuen Marker-Renderer
    pub fn new(render_state: &egui_wgpu::RenderState, shader: &wgpu::ShaderModule) -> Self {
        let device = &render_state.device;

        // Uniform-Buffer erstellen
        let uniform_buffer = device.create_buffer(&wgpu::BufferDescriptor {
            label: Some("Marker Uniform Buffer"),
            size: std::mem::size_of::<Uniforms>() as u64,
            usage: wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST,
            mapped_at_creation: false,
        });

        // Bind-Group-Layout
        let bind_group_layout = device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
            label: Some("Marker Bind Group Layout"),
            entries: &[wgpu::BindGroupLayoutEntry {
                binding: 0,
                visibility: wgpu::ShaderStages::VERTEX_FRAGMENT,
                ty: wgpu::BindingType::Buffer {
                    ty: wgpu::BufferBindingType::Uniform,
                    has_dynamic_offset: false,
                    min_binding_size: None,
                },
                count: None,
            }],
        });

        // Bind-Group erstellen
        let bind_group = device.create_bind_group(&wgpu::BindGroupDescriptor {
            label: Some("Marker Bind Group"),
            layout: &bind_group_layout,
            entries: &[wgpu::BindGroupEntry {
                binding: 0,
                resource: uniform_buffer.as_entire_binding(),
            }],
        });

        // Pipeline-Layout
        let pipeline_layout = device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
            label: Some("Marker Pipeline Layout"),
            bind_group_layouts: &[&bind_group_layout],
            push_constant_ranges: &[],
        });

        // Render-Pipeline
        let pipeline = device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
            label: Some("Marker Pipeline"),
            layout: Some(&pipeline_layout),
            vertex: wgpu::VertexState {
                module: shader,
                entry_point: Some("vs_marker"),
                buffers: &[Vertex::desc(), MarkerInstance::desc()],
                compilation_options: Default::default(),
            },
            fragment: Some(wgpu::FragmentState {
                module: shader,
                entry_point: Some("fs_marker"),
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

        // Vertex-Buffer für Quad (-1..1)
        let vertices = [
            Vertex {
                position: [-1.0, -1.0],
            },
            Vertex {
                position: [1.0, -1.0],
            },
            Vertex {
                position: [1.0, 1.0],
            },
            Vertex {
                position: [-1.0, -1.0],
            },
            Vertex {
                position: [1.0, 1.0],
            },
            Vertex {
                position: [-1.0, 1.0],
            },
        ];

        let vertex_buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some("Marker Vertex Buffer"),
            contents: bytemuck::cast_slice(&vertices),
            usage: wgpu::BufferUsages::VERTEX,
        });

        Self {
            pipeline,
            vertex_buffer,
            uniform_buffer,
            bind_group,
            instance_buffer: None,
            instance_capacity: 0,
        }
    }

    #[allow(clippy::too_many_arguments)]
    /// Rendert alle sichtbaren Map-Marker per GPU-Instancing.
    ///
    /// Marker-Positionen werden über die referenzierte Node-ID aufgelöst.
    pub fn render(
        &mut self,
        device: &wgpu::Device,
        queue: &wgpu::Queue,
        render_pass: &mut wgpu::RenderPass<'static>,
        road_map: &RoadMap,
        camera: &crate::Camera2D,
        viewport_size: [f32; 2],
        render_quality: RenderQuality,
        options: &EditorOptions,
    ) {
        if road_map.map_markers.is_empty() {
            return;
        }

        // Uniforms erstellen (View-Projection-Matrix + AA aus View-Einstellungen)
        let view_proj = super::types::build_view_projection(camera, viewport_size);
        let aa_params = match render_quality {
            RenderQuality::Low => [0.0, 1.0, 0.0, 0.0],
            RenderQuality::Medium => [1.0, 0.0, 0.0, 0.0],
            RenderQuality::High => [1.8, 0.0, 0.0, 0.0],
        };
        let uniforms = Uniforms {
            view_proj: view_proj.to_cols_array_2d(),
            aa_params,
        };

        // Uniforms hochladen
        queue.write_buffer(&self.uniform_buffer, 0, bytemuck::cast_slice(&[uniforms]));

        // Instanz-Daten vorbereiten
        let instances: Vec<MarkerInstance> = road_map
            .map_markers
            .iter()
            .filter_map(|marker| {
                // Position des Markers aus dem Node holen
                let node = road_map.nodes.get(&marker.id)?;
                Some(MarkerInstance::new(
                    [node.position.x, node.position.y],
                    options.marker_color,
                    options.marker_outline_color,
                    options.marker_size_world,
                ))
            })
            .collect();

        if instances.is_empty() {
            return;
        }

        // Instanz-Buffer erstellen oder resizen
        let needed_capacity = instances.len();
        if self.instance_buffer.is_none() || self.instance_capacity < needed_capacity {
            let new_capacity = needed_capacity.max(64).next_power_of_two();
            self.instance_buffer = Some(device.create_buffer(&wgpu::BufferDescriptor {
                label: Some("Marker Instance Buffer"),
                size: (new_capacity * std::mem::size_of::<MarkerInstance>()) as u64,
                usage: wgpu::BufferUsages::VERTEX | wgpu::BufferUsages::COPY_DST,
                mapped_at_creation: false,
            }));
            self.instance_capacity = new_capacity;
        }

        // Daten hochladen
        if let Some(buffer) = &self.instance_buffer {
            queue.write_buffer(buffer, 0, bytemuck::cast_slice(&instances));
        }

        // Rendern
        render_pass.set_pipeline(&self.pipeline);
        render_pass.set_bind_group(0, &self.bind_group, &[]);
        render_pass.set_vertex_buffer(0, self.vertex_buffer.slice(..));
        if let Some(buffer) = &self.instance_buffer {
            render_pass.set_vertex_buffer(1, buffer.slice(..));
        }
        render_pass.draw(0..6, 0..instances.len() as u32);
    }
}
