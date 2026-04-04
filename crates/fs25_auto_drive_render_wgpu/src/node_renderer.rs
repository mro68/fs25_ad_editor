//! Node-Renderer mit GPU-Instancing.

use super::fingerprint::RenderFingerprint;
use super::types::{
    compute_visible_rect, NodeInstance, RenderContext, RenderQuality, Uniforms, Vertex,
};
use super::RendererTargetConfig;
use crate::shared::{RenderMap, RenderNodeKind, SelectionStyle};
use indexmap::IndexSet;
use std::collections::HashMap;
use wgpu::util::DeviceExt;

/// Renderer fuer Nodes (Wegpunkte)
pub struct NodeRenderer {
    pipeline: wgpu::RenderPipeline,
    vertex_buffer: wgpu::Buffer,
    uniform_buffer: wgpu::Buffer,
    bind_group: wgpu::BindGroup,
    instance_buffer: Option<wgpu::Buffer>,
    instance_capacity: usize,
    /// Wiederverwendbarer Scratch-Buffer fuer Instanzdaten (vermeidet per-Frame-Allokation)
    instance_scratch: Vec<NodeInstance>,
    /// Wiederverwendbarer Scratch-Buffer fuer sichtbare Node-IDs (KD-Query ohne pro-Frame-Vec)
    node_id_scratch: Vec<u64>,
    /// Wiederverwendbare Grid-Map fuer Node-Decimation (wird pro Frame per clear() geleert)
    decimation_grid: HashMap<(i32, i32), ()>,
    /// Fingerabdruck der letzten Render-Inputs fuer Buffer-Skip-Detection.
    last_fingerprint: Option<RenderFingerprint>,
    /// Instanzanzahl des letzten Render-Passes (fuer Draw-Call bei Skip).
    last_instance_count: u32,
}

impl NodeRenderer {
    /// Erstellt einen neuen Node-Renderer
    pub fn new(
        device: &wgpu::Device,
        shader: &wgpu::ShaderModule,
        target_config: RendererTargetConfig,
    ) -> Self {

        // Uniform-Buffer erstellen
        let uniform_buffer = device.create_buffer(&wgpu::BufferDescriptor {
            label: Some("Uniform Buffer"),
            size: std::mem::size_of::<Uniforms>() as u64,
            usage: wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST,
            mapped_at_creation: false,
        });

        // Bind-Group-Layout
        let bind_group_layout = device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
            label: Some("Bind Group Layout"),
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
            label: Some("Bind Group"),
            layout: &bind_group_layout,
            entries: &[wgpu::BindGroupEntry {
                binding: 0,
                resource: uniform_buffer.as_entire_binding(),
            }],
        });

        // Pipeline-Layout
        let pipeline_layout = device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
            label: Some("Pipeline Layout"),
            bind_group_layouts: &[&bind_group_layout],
            push_constant_ranges: &[],
        });

        // Render-Pipeline
        let pipeline = device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
            label: Some("Node Pipeline"),
            layout: Some(&pipeline_layout),
            vertex: wgpu::VertexState {
                module: shader,
                entry_point: Some("vs_main"),
                buffers: &[Vertex::desc(), NodeInstance::desc()],
                compilation_options: Default::default(),
            },
            fragment: Some(wgpu::FragmentState {
                module: shader,
                entry_point: Some("fs_main"),
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

        // Vertex-Buffer fuer Quad (2 Dreiecke)
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
            label: Some("Vertex Buffer"),
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
            // Reserve initial capacity to avoid tiny growths on first frames.
            instance_scratch: Vec::with_capacity(1024),
            node_id_scratch: Vec::with_capacity(1024),
            decimation_grid: HashMap::with_capacity(1024),
            last_fingerprint: None,
            last_instance_count: 0,
        }
    }

    /// Rendert alle sichtbaren Nodes der RoadMap per GPU-Instancing.
    ///
    /// Fuehrt Viewport-Culling durch und schreibt Instanzdaten in den
    /// wiederverwendbaren Instance-Buffer.
    pub fn render(
        &mut self,
        ctx: &RenderContext,
        render_pass: &mut wgpu::RenderPass<'_>,
        render_map: &RenderMap,
        render_quality: RenderQuality,
        selected_node_ids: &IndexSet<u64>,
    ) {
        let selected_set = selected_node_ids;
        let viewport_width = ctx.viewport_size[0];
        let viewport_height = ctx.viewport_size[1];
        if !viewport_width.is_finite()
            || !viewport_height.is_finite()
            || viewport_width <= 0.0
            || viewport_height <= 0.0
        {
            return;
        }

        // Fingerabdruck berechnen und mit dem letzten Frame vergleichen.
        // Bei Uebereinstimmung kann der gesamte CPU/GPU-Rebuild uebersprungen werden.
        let new_fp = {
            let mut fp = RenderFingerprint::from_context(ctx, render_map);
            fp.dimmed_ptr = ctx.dimmed_node_ids as *const IndexSet<u64> as usize;
            fp.dimmed_revision = ctx.dimmed_node_ids_revision;
            fp.selected_ptr = selected_node_ids as *const IndexSet<u64> as usize;
            fp.selected_revision = ctx.selected_node_ids_revision;
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
            let (min, max) = compute_visible_rect(ctx);

            // Instanzen aus RoadMap sammeln (Scratch-Buffer wiederverwenden)
            self.instance_scratch.clear();
            // node_id_scratch wird vom Spatial-Query befuellt; vorher leeren.
            self.node_id_scratch.clear();

            render_map.nodes_within_rect_into(min, max, &mut self.node_id_scratch);

            // Zoom-Kompensationsfaktor einmalig pro Frame berechnen (nicht pro Node).
            let compensation = ctx.options.zoom_compensation(ctx.camera.zoom);
            // Pixel -> Welteinheiten-Faktor fuer Mindestgroessen-Berechnung.
            let wpp = ctx.camera.world_per_pixel(viewport_height);
            let min_node_world = ctx.options.min_node_size_px * wpp;

            // --- Grid-Decimation: bei Zoomout einen Node pro Grid-Zelle behalten ---
            let cell_size = ctx.options.decimation_cell_size(wpp);
            if cell_size > 0.0 {
                self.decimation_grid.clear();
                let inv_cell = 1.0 / cell_size;
                // Separate Borrows auf zwei Felder, damit der Borrow-Checker den
                // gleichzeitigen &mut-Zugriff innerhalb der retain-Closure akzeptiert.
                let node_id_scratch = &mut self.node_id_scratch;
                let decimation_grid = &mut self.decimation_grid;
                node_id_scratch.retain(|&node_id| {
                    // Selektierte Nodes immer sichtbar lassen
                    if selected_set.contains(&node_id) {
                        return true;
                    }
                    let Some(node) = render_map.node(&node_id) else {
                        return false;
                    };
                    // Bogenpunkte immer sichtbar lassen (sonst erscheinen Boegen eckig bei Zoom-out)
                    if node.preserve_when_decimating {
                        return true;
                    }
                    let cell = (
                        (node.position.x * inv_cell).floor() as i32,
                        (node.position.y * inv_cell).floor() as i32,
                    );
                    // Nur einfuegen wenn Zelle noch leer — erster Node pro Zelle gewinnt
                    match decimation_grid.entry(cell) {
                        std::collections::hash_map::Entry::Vacant(e) => {
                            e.insert(());
                            true
                        }
                        std::collections::hash_map::Entry::Occupied(_) => false,
                    }
                });
            }

            // Reserve Platz fuer Instanzen entsprechend der Anzahl gefundener IDs,
            // um mehrfache Reallocs beim Push zu vermeiden.
            self.instance_scratch.reserve(
                self.node_id_scratch
                    .len()
                    .saturating_sub(self.instance_scratch.len()),
            );

            for node_id in self.node_id_scratch.iter() {
                if ctx.hidden_node_ids.contains(node_id) {
                    continue;
                }
                let Some(node) = render_map.node(node_id) else {
                    continue;
                };

                let is_selected = selected_set.contains(&node.id);
                // Basisfarbe entspricht dem Node-Flag (bleibt mittig sichtbar)
                let mut base_color = match node.kind {
                    RenderNodeKind::SubPrio => ctx.options.node_color_subprio,
                    RenderNodeKind::Warning => ctx.options.node_color_warning,
                    RenderNodeKind::Regular => ctx.options.node_color_default,
                };
                // Gedimmte Nodes des gleichen Segments auf 50% Opacity setzen
                if ctx.dimmed_node_ids.contains(&node.id) {
                    base_color[3] *= 0.5;
                }
                // Rim/Markierungsfarbe aussen — nur bei selektierten Nodes anders.
                // rim_color.a kodiert das Verhaeltnis Innendurchmesser/Aussendurchmesser fuer den Shader.
                let rim_color = if is_selected {
                    let mut c = ctx.options.node_color_selected;
                    c[3] = 1.0 / ctx.options.selection_size_multiplier();
                    c
                } else {
                    let mut c = base_color;
                    c[3] = 1.0;
                    c
                };

                let size = (if is_selected {
                    ctx.options.node_size_world * ctx.options.selection_size_multiplier()
                } else {
                    ctx.options.node_size_world
                } * compensation)
                    .max(min_node_world);

                self.instance_scratch.push(NodeInstance::new(
                    [node.position.x, node.position.y],
                    base_color,
                    rim_color,
                    size,
                ));
            }

            if self.instance_scratch.is_empty() {
                // Fingerabdruck speichern, damit bei naechstem identischen Frame fruehzeitig
                // abgebrochen werden kann (kein Rebuild, kein Draw).
                self.last_fingerprint = Some(new_fp);
                self.last_instance_count = 0;
                return;
            }

            // View-Projektion-Matrix berechnen (gemeinsame Funktion)
            let view_proj = super::types::build_view_projection(ctx.camera, ctx.viewport_size);
            let view_proj_array = view_proj.to_cols_array_2d();

            // Uniform-Buffer aktualisieren
            let selection_style_flag = match ctx.options.selection_style {
                SelectionStyle::Gradient => 0.0,
                SelectionStyle::Ring => 1.0,
            };
            let aa_params = match render_quality {
                RenderQuality::Low => [0.0, 1.0, selection_style_flag, 0.0],
                RenderQuality::Medium => [1.0, 0.0, selection_style_flag, 0.0],
                RenderQuality::High => [1.8, 0.0, selection_style_flag, 0.0],
            };

            let uniforms = Uniforms {
                view_proj: view_proj_array,
                aa_params,
            };
            ctx.queue
                .write_buffer(&self.uniform_buffer, 0, bytemuck::cast_slice(&[uniforms]));

            // Instance-Buffer erstellen/aktualisieren (Reuse)
            if self.instance_buffer.is_none()
                || self.instance_scratch.len() > self.instance_capacity
            {
                let instance_size = std::mem::size_of::<NodeInstance>() as u64;
                let new_capacity = self
                    .instance_scratch
                    .len()
                    .checked_next_power_of_two()
                    .unwrap_or(self.instance_scratch.len());
                let buffer_size = (new_capacity as u64) * instance_size;
                self.instance_buffer = Some(ctx.device.create_buffer(&wgpu::BufferDescriptor {
                    label: Some("Instance Buffer"),
                    size: buffer_size,
                    usage: wgpu::BufferUsages::VERTEX | wgpu::BufferUsages::COPY_DST,
                    mapped_at_creation: false,
                }));
                self.instance_capacity = new_capacity;
            }

            if let Some(instance_buffer) = &self.instance_buffer {
                ctx.queue.write_buffer(
                    instance_buffer,
                    0,
                    bytemuck::cast_slice(&self.instance_scratch),
                );
            }

            self.last_instance_count = self.instance_scratch.len() as u32;
            self.last_fingerprint = Some(new_fp);
        }

        // Draw-Call (laeuft immer — sowohl nach Rebuild als auch bei Skip)
        let Some(instance_buffer) = self.instance_buffer.as_ref() else {
            log::error!("NodeRenderer: missing instance buffer before draw call");
            return;
        };

        render_pass.set_pipeline(&self.pipeline);
        render_pass.set_bind_group(0, &self.bind_group, &[]);
        render_pass.set_vertex_buffer(0, self.vertex_buffer.slice(..));
        render_pass.set_vertex_buffer(1, instance_buffer.slice(..));
        render_pass.draw(0..6, 0..self.last_instance_count);
    }
}
