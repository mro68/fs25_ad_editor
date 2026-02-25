//! Background-Renderer für Map-Hintergrund.

use crate::{BackgroundMap, Camera2D, WorldBounds};
use eframe::{egui_wgpu, wgpu};
use wgpu::util::DeviceExt;

/// Uniforms für Background-Rendering
#[repr(C)]
#[derive(Copy, Clone, Debug, bytemuck::Pod, bytemuck::Zeroable)]
struct BackgroundUniforms {
    view_proj: [[f32; 4]; 4], // mat4x4
    opacity: f32,
    _padding1: f32,
    _padding2: f32,
    _padding3: f32,
    texture_bounds: [f32; 4], // min_x, max_x, min_z, max_z
}

/// Vertex für Background-Quad
#[repr(C)]
#[derive(Copy, Clone, Debug, bytemuck::Pod, bytemuck::Zeroable)]
struct BackgroundVertex {
    position: [f32; 2],
}

impl BackgroundVertex {
    fn desc() -> wgpu::VertexBufferLayout<'static> {
        wgpu::VertexBufferLayout {
            array_stride: std::mem::size_of::<BackgroundVertex>() as wgpu::BufferAddress,
            step_mode: wgpu::VertexStepMode::Vertex,
            attributes: &[wgpu::VertexAttribute {
                offset: 0,
                shader_location: 0,
                format: wgpu::VertexFormat::Float32x2,
            }],
        }
    }
}

/// Renderer für Background-Map
pub struct BackgroundRenderer {
    pipeline: wgpu::RenderPipeline,
    bind_group_layout: wgpu::BindGroupLayoutDescriptor<'static>,
    vertex_buffer: wgpu::Buffer,
    uniform_buffer: wgpu::Buffer,

    // Optional: aktuelle Background-Map
    texture: Option<wgpu::Texture>,
    sampler: Option<wgpu::Sampler>,
    bind_group: Option<wgpu::BindGroup>,
    current_bounds: Option<WorldBounds>,
}

impl BackgroundRenderer {
    /// Erstellt einen neuen Background-Renderer
    pub fn new(render_state: &egui_wgpu::RenderState, shader: &wgpu::ShaderModule) -> Self {
        let device = &render_state.device;

        // Uniform-Buffer erstellen
        let uniform_buffer = device.create_buffer(&wgpu::BufferDescriptor {
            label: Some("Background Uniform Buffer"),
            size: std::mem::size_of::<BackgroundUniforms>() as u64,
            usage: wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST,
            mapped_at_creation: false,
        });

        // Bind-Group-Layout für Background (group(1))
        let bind_group_layout_desc = wgpu::BindGroupLayoutDescriptor {
            label: Some("Background Bind Group Layout"),
            entries: &[
                // Uniforms
                wgpu::BindGroupLayoutEntry {
                    binding: 0,
                    visibility: wgpu::ShaderStages::VERTEX_FRAGMENT,
                    ty: wgpu::BindingType::Buffer {
                        ty: wgpu::BufferBindingType::Uniform,
                        has_dynamic_offset: false,
                        min_binding_size: None,
                    },
                    count: None,
                },
                // Texture
                wgpu::BindGroupLayoutEntry {
                    binding: 1,
                    visibility: wgpu::ShaderStages::FRAGMENT,
                    ty: wgpu::BindingType::Texture {
                        multisampled: false,
                        view_dimension: wgpu::TextureViewDimension::D2,
                        sample_type: wgpu::TextureSampleType::Float { filterable: true },
                    },
                    count: None,
                },
                // Sampler
                wgpu::BindGroupLayoutEntry {
                    binding: 2,
                    visibility: wgpu::ShaderStages::FRAGMENT,
                    ty: wgpu::BindingType::Sampler(wgpu::SamplerBindingType::Filtering),
                    count: None,
                },
            ],
        };

        let bind_group_layout = device.create_bind_group_layout(&bind_group_layout_desc);

        // Pipeline-Layout
        let pipeline_layout = device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
            label: Some("Background Pipeline Layout"),
            bind_group_layouts: &[&bind_group_layout],
            push_constant_ranges: &[],
        });

        // Render-Pipeline
        let pipeline = device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
            label: Some("Background Pipeline"),
            layout: Some(&pipeline_layout),
            vertex: wgpu::VertexState {
                module: shader,
                entry_point: Some("vs_background"),
                buffers: &[BackgroundVertex::desc()],
                compilation_options: Default::default(),
            },
            fragment: Some(wgpu::FragmentState {
                module: shader,
                entry_point: Some("fs_background"),
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
                count: 4, // Muss mit Node-Renderer übereinstimmen
                mask: !0,
                alpha_to_coverage_enabled: false,
            },
            multiview: None,
            cache: None,
        });

        // Vertex-Buffer für ein Dummy-Quad (wird später bei set_background() aktualisiert)
        let vertices = [
            BackgroundVertex {
                position: [0.0, 0.0],
            },
            BackgroundVertex {
                position: [0.0, 0.0],
            },
            BackgroundVertex {
                position: [0.0, 0.0],
            },
            BackgroundVertex {
                position: [0.0, 0.0],
            },
            BackgroundVertex {
                position: [0.0, 0.0],
            },
            BackgroundVertex {
                position: [0.0, 0.0],
            },
        ];

        let vertex_buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some("Background Vertex Buffer"),
            contents: bytemuck::cast_slice(&vertices),
            usage: wgpu::BufferUsages::VERTEX | wgpu::BufferUsages::COPY_DST,
        });

        Self {
            pipeline,
            bind_group_layout: bind_group_layout_desc,
            vertex_buffer,
            uniform_buffer,
            texture: None,
            sampler: None,
            bind_group: None,
            current_bounds: None,
        }
    }

    /// Setzt die Background-Map und lädt die Texture hoch
    pub fn set_background(
        &mut self,
        device: &wgpu::Device,
        queue: &wgpu::Queue,
        bg_map: &BackgroundMap,
        scale: f32,
    ) {
        log::info!("BackgroundRenderer: Lade Background-Texture...");

        // Erstelle Texture aus Image
        let (texture, sampler) = super::texture::create_texture_from_image(
            device,
            queue,
            bg_map.image_data(),
            "Background Texture",
        );

        // Erstelle Bind-Group
        let bind_group_layout = device.create_bind_group_layout(&self.bind_group_layout);
        let bind_group = device.create_bind_group(&wgpu::BindGroupDescriptor {
            label: Some("Background Bind Group"),
            layout: &bind_group_layout,
            entries: &[
                wgpu::BindGroupEntry {
                    binding: 0,
                    resource: self.uniform_buffer.as_entire_binding(),
                },
                wgpu::BindGroupEntry {
                    binding: 1,
                    resource: wgpu::BindingResource::TextureView(
                        &texture.create_view(&wgpu::TextureViewDescriptor::default()),
                    ),
                },
                wgpu::BindGroupEntry {
                    binding: 2,
                    resource: wgpu::BindingResource::Sampler(&sampler),
                },
            ],
        });

        // Update Vertex-Buffer mit korrekten Quad-Koordinaten (Weltkoordinaten)
        let bounds = bg_map.world_bounds();
        let half_w = (bounds.max_x - bounds.min_x) * 0.5;
        let half_h = (bounds.max_z - bounds.min_z) * 0.5;
        let cx = (bounds.min_x + bounds.max_x) * 0.5;
        let cz = (bounds.min_z + bounds.max_z) * 0.5;
        let min_x = cx - half_w * scale;
        let max_x = cx + half_w * scale;
        let min_z = cz - half_h * scale;
        let max_z = cz + half_h * scale;
        let vertices = [
            // Dreieck 1
            BackgroundVertex {
                position: [min_x, min_z],
            },
            BackgroundVertex {
                position: [max_x, min_z],
            },
            BackgroundVertex {
                position: [max_x, max_z],
            },
            // Dreieck 2
            BackgroundVertex {
                position: [min_x, min_z],
            },
            BackgroundVertex {
                position: [max_x, max_z],
            },
            BackgroundVertex {
                position: [min_x, max_z],
            },
        ];

        queue.write_buffer(&self.vertex_buffer, 0, bytemuck::cast_slice(&vertices));

        // Speichere skalierte Bounds (für konsistentes UV-Mapping im Shader)
        self.texture = Some(texture);
        self.sampler = Some(sampler);
        self.bind_group = Some(bind_group);
        self.current_bounds = Some(WorldBounds {
            min_x,
            max_x,
            min_z,
            max_z,
        });

        log::info!(
            "BackgroundRenderer: Texture geladen ({}x{})",
            bg_map.dimensions().0,
            bg_map.dimensions().1
        );
    }

    /// Entfernt die aktuelle Background-Map
    pub fn clear_background(&mut self) {
        self.texture = None;
        self.sampler = None;
        self.bind_group = None;
        self.current_bounds = None;
        log::info!("BackgroundRenderer: Background entfernt");
    }

    /// Rendert die Background-Map
    pub fn render(
        &self,
        queue: &wgpu::Queue,
        render_pass: &mut wgpu::RenderPass<'static>,
        camera: &Camera2D,
        viewport_size: [f32; 2],
        visible: bool,
        opacity: f32,
    ) {
        // Nichts zu rendern, wenn kein Background oder nicht visible
        if !visible || opacity <= 0.0 {
            return;
        }
        let Some(bind_group) = self.bind_group.as_ref() else {
            return;
        };
        let Some(bounds) = self.current_bounds.as_ref() else {
            return;
        };

        // Update Uniforms
        let view_proj = super::types::build_view_projection(camera, viewport_size);
        let uniforms = BackgroundUniforms {
            view_proj: view_proj.to_cols_array_2d(),
            opacity: opacity.clamp(0.0, 1.0),
            _padding1: 0.0,
            _padding2: 0.0,
            _padding3: 0.0,
            texture_bounds: [bounds.min_x, bounds.max_x, bounds.min_z, bounds.max_z],
        };

        queue.write_buffer(&self.uniform_buffer, 0, bytemuck::cast_slice(&[uniforms]));

        // Render
        render_pass.set_pipeline(&self.pipeline);
        render_pass.set_bind_group(0, bind_group, &[]);
        render_pass.set_vertex_buffer(0, self.vertex_buffer.slice(..));
        render_pass.draw(0..6, 0..1);

        log::trace!("BackgroundRenderer: Gerendert");
    }
}
