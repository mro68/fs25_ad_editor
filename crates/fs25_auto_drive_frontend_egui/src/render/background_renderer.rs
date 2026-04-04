//! Background-Renderer fuer Map-Hintergrund.

use crate::shared::RenderCamera;
use eframe::{egui_wgpu, wgpu};
use image::{DynamicImage, GenericImageView};
use wgpu::util::DeviceExt;

/// Ab diesem Wert wird auf pixelgenaues Sampling umgeschaltet.
/// 1.0 bedeutet: ein Hintergrund-Texel belegt mindestens einen Screen-Pixel.
const NEAREST_SAMPLING_TEXEL_THRESHOLD_PX: f32 = 1.0;

/// Weltkoordinaten-Bereich des Hintergrund-Quads im Render-Vertrag.
#[derive(Debug, Clone, Copy)]
pub struct BackgroundWorldBounds {
    /// Linke Kante in Weltkoordinaten.
    pub min_x: f32,
    /// Rechte Kante in Weltkoordinaten.
    pub max_x: f32,
    /// Untere Kante in Weltkoordinaten.
    pub min_y: f32,
    /// Obere Kante in Weltkoordinaten.
    pub max_y: f32,
}

impl BackgroundWorldBounds {
    fn scaled(self, scale: f32) -> Self {
        let half_w = (self.max_x - self.min_x) * 0.5;
        let half_h = (self.max_y - self.min_y) * 0.5;
        let cx = (self.min_x + self.max_x) * 0.5;
        let cy = (self.min_y + self.max_y) * 0.5;

        Self {
            min_x: cx - half_w * scale,
            max_x: cx + half_w * scale,
            min_y: cy - half_h * scale,
            max_y: cy + half_h * scale,
        }
    }
}

/// Uniforms fuer Background-Rendering
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

/// Vertex fuer Background-Quad
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

/// Renderer fuer Background-Map
pub struct BackgroundRenderer {
    pipeline: wgpu::RenderPipeline,
    bind_group_layout: wgpu::BindGroupLayoutDescriptor<'static>,
    vertex_buffer: wgpu::Buffer,
    uniform_buffer: wgpu::Buffer,

    // Optional: aktuelle Background-Map
    texture: Option<wgpu::Texture>,
    linear_sampler: Option<wgpu::Sampler>,
    nearest_sampler: Option<wgpu::Sampler>,
    linear_bind_group: Option<wgpu::BindGroup>,
    nearest_bind_group: Option<wgpu::BindGroup>,
    current_bounds: Option<BackgroundWorldBounds>,
    texture_dimensions: Option<[u32; 2]>,
}

fn screen_pixels_per_background_texel(
    bounds: &BackgroundWorldBounds,
    texture_dimensions: [u32; 2],
    camera: &RenderCamera,
    viewport_size: [f32; 2],
) -> f32 {
    let world_width = (bounds.max_x - bounds.min_x).abs().max(f32::EPSILON);
    let world_height = (bounds.max_y - bounds.min_y).abs().max(f32::EPSILON);
    let texel_world_x = world_width / texture_dimensions[0].max(1) as f32;
    let texel_world_y = world_height / texture_dimensions[1].max(1) as f32;
    let texel_world = texel_world_x.max(texel_world_y);
    let world_per_pixel = camera.world_per_pixel(viewport_size[1].max(1.0));
    texel_world / world_per_pixel
}

fn should_use_nearest_background_sampling(
    bounds: &BackgroundWorldBounds,
    texture_dimensions: [u32; 2],
    camera: &RenderCamera,
    viewport_size: [f32; 2],
) -> bool {
    screen_pixels_per_background_texel(bounds, texture_dimensions, camera, viewport_size)
        >= NEAREST_SAMPLING_TEXEL_THRESHOLD_PX
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

        // Bind-Group-Layout fuer Background (group(1))
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
                count: 4, // Muss mit Node-Renderer uebereinstimmen
                mask: !0,
                alpha_to_coverage_enabled: false,
            },
            multiview: None,
            cache: None,
        });

        // Vertex-Buffer fuer ein Dummy-Quad (wird spaeter bei set_background() aktualisiert)
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
            linear_sampler: None,
            nearest_sampler: None,
            linear_bind_group: None,
            nearest_bind_group: None,
            current_bounds: None,
            texture_dimensions: None,
        }
    }

    /// Setzt die Background-Map und laedt die Texture hoch
    pub fn set_background(
        &mut self,
        device: &wgpu::Device,
        queue: &wgpu::Queue,
        image: &DynamicImage,
        world_bounds: BackgroundWorldBounds,
        scale: f32,
    ) {
        log::info!("BackgroundRenderer: Lade Background-Texture...");

        // Erstelle Texture aus Image
        let (texture, linear_sampler) =
            super::texture::create_texture_from_image(device, queue, image, "Background Texture");
        let nearest_sampler = super::texture::create_sampler(
            device,
            "Background Texture nearest_sampler",
            wgpu::FilterMode::Nearest,
        );
        let texture_view = texture.create_view(&wgpu::TextureViewDescriptor::default());

        // Erstelle Bind-Groups fuer lineares und pixelgenaues Sampling.
        let bind_group_layout = device.create_bind_group_layout(&self.bind_group_layout);
        let linear_bind_group = device.create_bind_group(&wgpu::BindGroupDescriptor {
            label: Some("Background Linear Bind Group"),
            layout: &bind_group_layout,
            entries: &[
                wgpu::BindGroupEntry {
                    binding: 0,
                    resource: self.uniform_buffer.as_entire_binding(),
                },
                wgpu::BindGroupEntry {
                    binding: 1,
                    resource: wgpu::BindingResource::TextureView(&texture_view),
                },
                wgpu::BindGroupEntry {
                    binding: 2,
                    resource: wgpu::BindingResource::Sampler(&linear_sampler),
                },
            ],
        });
        let nearest_bind_group = device.create_bind_group(&wgpu::BindGroupDescriptor {
            label: Some("Background Nearest Bind Group"),
            layout: &bind_group_layout,
            entries: &[
                wgpu::BindGroupEntry {
                    binding: 0,
                    resource: self.uniform_buffer.as_entire_binding(),
                },
                wgpu::BindGroupEntry {
                    binding: 1,
                    resource: wgpu::BindingResource::TextureView(&texture_view),
                },
                wgpu::BindGroupEntry {
                    binding: 2,
                    resource: wgpu::BindingResource::Sampler(&nearest_sampler),
                },
            ],
        });

        // Update Vertex-Buffer mit korrekten Quad-Koordinaten (Weltkoordinaten)
        let bounds = world_bounds.scaled(scale);
        let vertices = [
            // Dreieck 1
            BackgroundVertex {
                position: [bounds.min_x, bounds.min_y],
            },
            BackgroundVertex {
                position: [bounds.max_x, bounds.min_y],
            },
            BackgroundVertex {
                position: [bounds.max_x, bounds.max_y],
            },
            // Dreieck 2
            BackgroundVertex {
                position: [bounds.min_x, bounds.min_y],
            },
            BackgroundVertex {
                position: [bounds.max_x, bounds.max_y],
            },
            BackgroundVertex {
                position: [bounds.min_x, bounds.max_y],
            },
        ];

        queue.write_buffer(&self.vertex_buffer, 0, bytemuck::cast_slice(&vertices));

        // Speichere skalierte Bounds (fuer konsistentes UV-Mapping im Shader)
        self.texture = Some(texture);
        self.linear_sampler = Some(linear_sampler);
        self.nearest_sampler = Some(nearest_sampler);
        self.linear_bind_group = Some(linear_bind_group);
        self.nearest_bind_group = Some(nearest_bind_group);
        self.current_bounds = Some(bounds);
        self.texture_dimensions = Some([image.dimensions().0, image.dimensions().1]);

        log::info!(
            "BackgroundRenderer: Texture geladen ({}x{})",
            image.dimensions().0,
            image.dimensions().1
        );
    }

    /// Entfernt die aktuelle Background-Map
    pub fn clear_background(&mut self) {
        self.texture = None;
        self.linear_sampler = None;
        self.nearest_sampler = None;
        self.linear_bind_group = None;
        self.nearest_bind_group = None;
        self.current_bounds = None;
        self.texture_dimensions = None;
        log::info!("BackgroundRenderer: Background entfernt");
    }

    /// Rendert die Background-Map
    pub fn render(
        &self,
        queue: &wgpu::Queue,
        render_pass: &mut wgpu::RenderPass<'static>,
        camera: &RenderCamera,
        viewport_size: [f32; 2],
        visible: bool,
        opacity: f32,
    ) {
        // Nichts zu rendern, wenn kein Background oder nicht visible
        if !visible || opacity <= 0.0 {
            return;
        }
        let Some(linear_bind_group) = self.linear_bind_group.as_ref() else {
            return;
        };
        let Some(nearest_bind_group) = self.nearest_bind_group.as_ref() else {
            return;
        };
        let Some(bounds) = self.current_bounds.as_ref() else {
            return;
        };
        let Some(texture_dimensions) = self.texture_dimensions else {
            return;
        };

        let bind_group = if should_use_nearest_background_sampling(
            bounds,
            texture_dimensions,
            camera,
            viewport_size,
        ) {
            nearest_bind_group
        } else {
            linear_bind_group
        };

        // Update Uniforms
        let view_proj = super::types::build_view_projection(camera, viewport_size);
        let uniforms = BackgroundUniforms {
            view_proj: view_proj.to_cols_array_2d(),
            opacity: opacity.clamp(0.0, 1.0),
            _padding1: 0.0,
            _padding2: 0.0,
            _padding3: 0.0,
            texture_bounds: [bounds.min_x, bounds.max_x, bounds.min_y, bounds.max_y],
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn nearest_sampling_is_used_when_texels_are_magnified() {
        let bounds = BackgroundWorldBounds {
            min_x: -1024.0,
            max_x: 1024.0,
            min_y: -1024.0,
            max_y: 1024.0,
        };
        let mut camera = RenderCamera::new(glam::Vec2::ZERO, 1.0);
        camera.zoom = 8.0;

        assert!(should_use_nearest_background_sampling(
            &bounds,
            [2048, 2048],
            &camera,
            [800.0, 600.0],
        ));
    }

    #[test]
    fn linear_sampling_is_kept_when_texels_are_subpixel() {
        let bounds = BackgroundWorldBounds {
            min_x: -1024.0,
            max_x: 1024.0,
            min_y: -1024.0,
            max_y: 1024.0,
        };
        let camera = RenderCamera::new(glam::Vec2::ZERO, 1.0);

        assert!(!should_use_nearest_background_sampling(
            &bounds,
            [2048, 2048],
            &camera,
            [800.0, 600.0],
        ));
    }
}
