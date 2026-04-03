//! Marker-Renderer mit GPU-Instancing fuer Map-Marker (Pin-Symbole).

use super::fingerprint::RenderFingerprint;
use super::types::{MarkerInstance, RenderContext, RenderQuality, Uniforms, Vertex};
use crate::shared::options::MARKER_OUTLINE_WIDTH;
use crate::shared::RenderMap;
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
    bind_group_layout: wgpu::BindGroupLayout,
    bind_group: wgpu::BindGroup,
    instance_buffer: Option<wgpu::Buffer>,
    instance_capacity: usize,
    /// Wiederverwendbarer Scratch-Buffer fuer Instanz-Daten (verhindert per-Frame-Allokation)
    instance_scratch: Vec<MarkerInstance>,
    // Pin-Icon-Textur (muss gehalten werden, damit GPU-Ressourcen nicht freigegeben werden)
    _texture: wgpu::Texture,
    _sampler: wgpu::Sampler,
    /// Letzter angewendeter outline_width-Wert (fuer Change-Detection bei Textur-Rebuild)
    last_outline_width: f32,
    /// Fingerabdruck der letzten Render-Inputs fuer Buffer-Skip-Detection.
    last_fingerprint: Option<RenderFingerprint>,
    /// Instanzanzahl des letzten Render-Passes (fuer Draw-Call bei Skip).
    last_instance_count: u32,
}

/// Patcht die stroke-width im SVG-String auf den angegebenen Wert.
fn patch_svg_stroke_width(svg: &str, svg_stroke_width: f32) -> String {
    // SVG hat genau eine stroke-width-Angabe — direkte String-Manipulation
    if let Some(start_idx) = svg.find("stroke-width=\"") {
        let pos = start_idx + "stroke-width=\"".len();
        if let Some(end_offset) = svg[pos..].find('"') {
            let mut result = svg.to_string();
            result.replace_range(pos..pos + end_offset, &format!("{:.3}", svg_stroke_width));
            return result;
        }
    }
    svg.to_string()
}

/// Rasterisiert das Pin-Icon-SVG mit der angegebenen Strichdicke als DynamicImage (64×64 RGBA).
///
/// `outline_width` ist der Optionswert (0.01–0.3) und wird auf SVG-Koordinaten skaliert
/// (Faktor 10, viewBox 0 0 24 24 → stroke-width 0.1–3.0).
fn rasterize_svg(svg_str: &str, outline_width: f32) -> image::DynamicImage {
    use resvg::{tiny_skia, usvg};
    let svg_stroke_width = outline_width * 10.0;
    let patched = patch_svg_stroke_width(svg_str, svg_stroke_width);
    let options = usvg::Options::default();
    let tree =
        usvg::Tree::from_str(&patched, &options).expect("Marker-SVG konnte nicht geparst werden");

    const SIZE: u32 = 64;
    let mut pixmap =
        tiny_skia::Pixmap::new(SIZE, SIZE).expect("Pixmap konnte nicht erstellt werden");

    let svg_size = tree.size();
    let scale_x = SIZE as f32 / svg_size.width();
    let scale_y = SIZE as f32 / svg_size.height();
    let transform = tiny_skia::Transform::from_scale(scale_x, scale_y);
    resvg::render(&tree, transform, &mut pixmap.as_mut());

    // tiny-skia liefert prae-multipliziertes RGBA → in normales RGBA umrechnen
    let mut unpremul = pixmap.data().to_vec();
    for pixel in unpremul.chunks_mut(4) {
        let a = pixel[3];
        if a > 0 && a < 255 {
            pixel[0] = ((pixel[0] as u32 * 255 + a as u32 / 2) / a as u32).min(255) as u8;
            pixel[1] = ((pixel[1] as u32 * 255 + a as u32 / 2) / a as u32).min(255) as u8;
            pixel[2] = ((pixel[2] as u32 * 255 + a as u32 / 2) / a as u32).min(255) as u8;
        }
    }

    let rgba_image = image::RgbaImage::from_raw(SIZE, SIZE, unpremul)
        .expect("Marker-RGBA-Bild konnte nicht erstellt werden");
    image::DynamicImage::ImageRgba8(rgba_image)
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

        // Pin-Icon-SVG laden, rasterisieren und als wgpu-Textur erstellen
        let svg_str = include_str!("../../assets/icons/icon_map_pin.svg");
        let img = rasterize_svg(svg_str, MARKER_OUTLINE_WIDTH);
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
            bind_group_layout,
            bind_group,
            instance_buffer: None,
            instance_capacity: 0,
            instance_scratch: Vec::with_capacity(256),
            _texture: texture,
            _sampler: sampler,
            last_outline_width: MARKER_OUTLINE_WIDTH,
            last_fingerprint: None,
            last_instance_count: 0,
        }
    }

    /// Prueft ob `marker_outline_width` geaendert hat und rasterisiert das SVG neu.
    ///
    /// Erstellt neue Textur und BindGroup nur bei tatsaechlicher Aenderung
    /// (Change-Detection via Epsilon-Vergleich).
    fn rebuild_texture_if_needed(
        &mut self,
        device: &wgpu::Device,
        queue: &wgpu::Queue,
        outline_width: f32,
    ) {
        if (self.last_outline_width - outline_width).abs() < 1e-5 {
            return;
        }
        let svg_str = include_str!("../../assets/icons/icon_map_pin.svg");
        let img = rasterize_svg(svg_str, outline_width);
        let (texture, _) =
            super::texture::create_texture_from_image(device, queue, &img, "Marker Pin Texture");
        let texture_view = texture.create_view(&wgpu::TextureViewDescriptor::default());
        self.bind_group = device.create_bind_group(&wgpu::BindGroupDescriptor {
            label: Some("Marker Bind Group"),
            layout: &self.bind_group_layout,
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
                    resource: wgpu::BindingResource::Sampler(&self._sampler),
                },
            ],
        });
        self._texture = texture;
        self.last_outline_width = outline_width;
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
        render_map: &RenderMap,
        render_quality: RenderQuality,
    ) {
        if render_map.marker_count() == 0 {
            return;
        }

        // Fingerabdruck berechnen und mit dem letzten Frame vergleichen.
        // Bei Uebereinstimmung koennen Instanz-Aufbau und GPU-Upload uebersprungen werden.
        let new_fp = {
            let mut fp = RenderFingerprint::from_context(ctx, render_map);
            fp.quality = render_quality as u8;
            fp
        };

        let skip_rebuild = self.last_fingerprint.as_ref() == Some(&new_fp);
        if skip_rebuild {
            // Inputs unveraendert — Draw-Call mit gespeichertem Ergebnis wiederholen.
            if self.last_instance_count == 0 || self.instance_buffer.is_none() {
                return; // nichts zu zeichnen
            }
        } else {
            // Textur neu aufbauen wenn outline_width geaendert hat
            self.rebuild_texture_if_needed(ctx.device, ctx.queue, ctx.options.marker_outline_width);

            // Uniforms erstellen (View-Projection-Matrix + AA aus View-Einstellungen)
            let view_proj = super::types::build_view_projection(ctx.camera, ctx.viewport_size);
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
            ctx.queue
                .write_buffer(&self.uniform_buffer, 0, bytemuck::cast_slice(&[uniforms]));

            // Zoom-Kompensation und Mindestgroesse einmalig pro Frame berechnen.
            let compensation = ctx.options.zoom_compensation(ctx.camera.zoom);
            let wpp = ctx.camera.world_per_pixel(ctx.viewport_size[1]);
            let min_marker_world = ctx.options.min_marker_size_px * wpp;

            // Instanz-Daten vorbereiten (Scratch-Buffer wiederverwenden)
            self.instance_scratch.clear();
            self.instance_scratch
                .extend(render_map.markers().iter().map(|marker| {
                    let size = (ctx.options.marker_size_world * compensation).max(min_marker_world);
                    MarkerInstance::new(
                        [marker.position.x, marker.position.y],
                        ctx.options.marker_color,
                        ctx.options.marker_outline_color,
                        size,
                    )
                }));

            if self.instance_scratch.is_empty() {
                // Fingerabdruck speichern damit bei naechstem identischen Frame fruehzeitig
                // abgebrochen werden kann.
                self.last_fingerprint = Some(new_fp);
                self.last_instance_count = 0;
                return;
            }

            // Instanz-Buffer erstellen oder resizen
            let needed_capacity = self.instance_scratch.len();
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
                    .write_buffer(buffer, 0, bytemuck::cast_slice(&self.instance_scratch));
            }

            self.last_instance_count = self.instance_scratch.len() as u32;
            self.last_fingerprint = Some(new_fp);
        }

        // Draw-Call (laeuft immer — sowohl nach Rebuild als auch bei Skip)
        render_pass.set_pipeline(&self.pipeline);
        render_pass.set_bind_group(0, &self.bind_group, &[]);
        render_pass.set_vertex_buffer(0, self.vertex_buffer.slice(..));
        if let Some(buffer) = &self.instance_buffer {
            render_pass.set_vertex_buffer(1, buffer.slice(..));
        }
        render_pass.draw(0..6, 0..self.last_instance_count);
    }
}
