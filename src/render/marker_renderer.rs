//! Marker-Renderer mit GPU-Instancing fuer Map-Marker (Pin-Symbole).

use super::types::{MarkerInstance, RenderContext, RenderQuality, Uniforms, Vertex};
use crate::RoadMap;
use eframe::{egui_wgpu, wgpu};
use wgpu::util::DeviceExt;

/// Renderer fuer Map-Marker (Pin-Symbole) mit GPU-Instancing und texturbasiertem Rendering.
///
/// Laedt das Pin-Icon `icon_map_pin.png` beim Start als wgpu-Textur (eingebettet via
/// `include_bytes!`). Die BindGroup enthaelt drei Bindings: Uniform-Buffer (0),
/// Textur-View (1) und Sampler (2). Der Fragment-Shader (`fs_marker`) faerbt den Pin
/// per Instanz-Tint — die Textur-Alpha definiert die Pin-Form.
pub struct MarkerRenderer {
    pipeline: wgpu::RenderPipeline,
    vertex_buffer: wgpu::Buffer,
    uniform_buffer: wgpu::Buffer,
    bind_group: wgpu::BindGroup,
    instance_buffer: Option<wgpu::Buffer>,
    instance_capacity: usize,
    /// Wiederverwendbarer Scratch-Buffer fuer Instanz-Daten (verhindert per-Frame-Allokation)
    instance_scratch: Vec<MarkerInstance>,
    // Pin-Icon-Textur (muss gehalten werden, damit GPU-Ressourcen nicht freigegeben werden)
    _texture: wgpu::Texture,
    _sampler: wgpu::Sampler,
}

impl MarkerRenderer {
    /// Erstellt einen neuen Marker-Renderer und laedt das Pin-Icon als wgpu-Textur.
    ///
    /// Die PNG-Datei `assets/icons/icon_map_pin.png` wird per `include_bytes!` statisch
    /// eingebettet und als `wgpu::Texture` hochgeladen. Die BindGroup wird mit drei
    /// Bindings initialisiert: Uniform-Buffer, Textur-View und Sampler.
    pub fn new(render_state: &egui_wgpu::RenderState, shader: &wgpu::ShaderModule) -> Self {
        let device = &render_state.device;
        let queue = &render_state.queue;

        // Pin-Icon-PNG laden und als wgpu-Textur erstellen
        let png_bytes = include_bytes!("../../assets/icons/icon_map_pin.png");
        let img = image::load_from_memory(png_bytes)
            .expect("icon_map_pin.png: konnte PNG nicht dekodieren");
        let (texture, sampler) =
            super::texture::create_texture_from_image(device, queue, &img, "Marker Pin Texture");
        let texture_view = texture.create_view(&wgpu::TextureViewDescriptor::default());

        // Uniform-Buffer erstellen
        let uniform_buffer = device.create_buffer(&wgpu::BufferDescriptor {
            label: Some("Marker Uniform Buffer"),
            size: std::mem::size_of::<Uniforms>() as u64,
            usage: wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST,
            mapped_at_creation: false,
        });

        // Bind-Group-Layout: Uniform + Textur + Sampler
        let bind_group_layout = device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
            label: Some("Marker Bind Group Layout"),
            entries: &[
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
                wgpu::BindGroupLayoutEntry {
                    binding: 2,
                    visibility: wgpu::ShaderStages::FRAGMENT,
                    ty: wgpu::BindingType::Sampler(wgpu::SamplerBindingType::Filtering),
                    count: None,
                },
            ],
        });

        // Bind-Group erstellen
        let bind_group = device.create_bind_group(&wgpu::BindGroupDescriptor {
            label: Some("Marker Bind Group"),
            layout: &bind_group_layout,
            entries: &[
                wgpu::BindGroupEntry {
                    binding: 0,
                    resource: uniform_buffer.as_entire_binding(),
                },
                wgpu::BindGroupEntry {
                    binding: 1,
                    resource: wgpu::BindingResource::TextureView(&texture_view),
                },
                wgpu::BindGroupEntry {
                    binding: 2,
                    resource: wgpu::BindingResource::Sampler(&sampler),
                },
            ],
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

        // Vertex-Buffer fuer Quad (-1..1)
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
            instance_scratch: Vec::with_capacity(256),
            _texture: texture,
            _sampler: sampler,
        }
    }

    /// Rendert alle sichtbaren Map-Marker per GPU-Instancing.
    ///
    /// Marker-Positionen werden ueber die referenzierte Node-ID aufgeloest.
    /// Das Pin-Icon wird als Textur per `textureSample` gezeichnet; Farbe und Groesse
    /// kommen aus den `EditorOptions` und werden zoom-kompensiert skaliert.
    pub fn render(
        &mut self,
        ctx: &RenderContext,
        render_pass: &mut wgpu::RenderPass<'static>,
        road_map: &RoadMap,
        render_quality: RenderQuality,
    ) {
        if road_map.map_markers.is_empty() {
            return;
        }

        // Uniforms erstellen (View-Projection-Matrix + AA aus View-Einstellungen)
        let view_proj = super::types::build_view_projection(ctx.camera, ctx.viewport_size);
        let aa_params = match render_quality {
            RenderQuality::Low => [0.0, 1.0, 0.0, ctx.options.marker_outline_width],
            RenderQuality::Medium => [1.0, 0.0, 0.0, ctx.options.marker_outline_width],
            RenderQuality::High => [1.8, 0.0, 0.0, ctx.options.marker_outline_width],
        };
        let uniforms = Uniforms {
            view_proj: view_proj.to_cols_array_2d(),
            aa_params,
        };

        // Uniforms hochladen
        ctx.queue
            .write_buffer(&self.uniform_buffer, 0, bytemuck::cast_slice(&[uniforms]));

        // Zoom-Kompensation und Mindestgroesse einmalig pro Frame berechnen.
        let compensation = ctx.options.zoom_compensation(ctx.camera.zoom);
        let wpp = ctx.camera.world_per_pixel(ctx.viewport_size[1]);
        let min_marker_world = ctx.options.min_marker_size_px * wpp;

        // Instanz-Daten vorbereiten (Scratch-Buffer wiederverwenden)
        self.instance_scratch.clear();
        self.instance_scratch
            .extend(road_map.map_markers.iter().filter_map(|marker| {
                let node = road_map.nodes.get(&marker.id)?;
                let size = (ctx.options.marker_size_world * compensation).max(min_marker_world);
                Some(MarkerInstance::new(
                    [node.position.x, node.position.y],
                    ctx.options.marker_color,
                    ctx.options.marker_outline_color,
                    size,
                ))
            }));
        let instances = &self.instance_scratch;

        if instances.is_empty() {
            return;
        }

        // Instanz-Buffer erstellen oder resizen
        let needed_capacity = instances.len();
        if self.instance_buffer.is_none() || self.instance_capacity < needed_capacity {
            let new_capacity = needed_capacity.max(64).next_power_of_two();
            self.instance_buffer = Some(ctx.device.create_buffer(&wgpu::BufferDescriptor {
                label: Some("Marker Instance Buffer"),
                size: (new_capacity * std::mem::size_of::<MarkerInstance>()) as u64,
                usage: wgpu::BufferUsages::VERTEX | wgpu::BufferUsages::COPY_DST,
                mapped_at_creation: false,
            }));
            self.instance_capacity = new_capacity;
        }

        // Daten hochladen
        if let Some(buffer) = &self.instance_buffer {
            ctx.queue
                .write_buffer(buffer, 0, bytemuck::cast_slice(instances));
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
