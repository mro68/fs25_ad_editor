//! Node-Renderer mit GPU-Instancing.

use super::types::{NodeInstance, RenderContext, RenderQuality, Uniforms, Vertex};
use crate::{Camera2D, NodeFlag, RoadMap};
use eframe::{egui_wgpu, wgpu};
use glam::Vec2;
use std::collections::HashSet;
use wgpu::util::DeviceExt;

// HashSet-Import wird direkt in der Signatur genutzt (kein Re-collect mehr nötig)

/// Renderer für Nodes (Wegpunkte)
pub struct NodeRenderer {
    pipeline: wgpu::RenderPipeline,
    vertex_buffer: wgpu::Buffer,
    uniform_buffer: wgpu::Buffer,
    bind_group: wgpu::BindGroup,
    instance_buffer: Option<wgpu::Buffer>,
    instance_capacity: usize,
    /// Wiederverwendbarer Scratch-Buffer für Instanzdaten (vermeidet per-Frame-Allokation)
    instance_scratch: Vec<NodeInstance>,
}

impl NodeRenderer {
    /// Erstellt einen neuen Node-Renderer
    pub fn new(render_state: &egui_wgpu::RenderState, shader: &wgpu::ShaderModule) -> Self {
        let device = &render_state.device;

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

        // Vertex-Buffer für Quad (2 Dreiecke)
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
            instance_scratch: Vec::new(),
        }
    }

    /// Rendert alle sichtbaren Nodes der RoadMap per GPU-Instancing.
    ///
    /// Führt Viewport-Culling durch und schreibt Instanzdaten in den
    /// wiederverwendbaren Instance-Buffer.
    pub fn render(
        &mut self,
        ctx: &RenderContext,
        render_pass: &mut wgpu::RenderPass<'static>,
        road_map: &RoadMap,
        render_quality: RenderQuality,
        selected_node_ids: &HashSet<u64>,
    ) {
        log::debug!("NodeRenderer.render() called");
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

        let aspect = viewport_width / viewport_height;
        let zoom_scale = 1.0 / ctx.camera.zoom;
        let base_extent = Camera2D::BASE_WORLD_EXTENT;
        let half_height = base_extent * zoom_scale;
        let half_width = half_height * aspect;
        let padding = ctx.camera.world_per_pixel(viewport_height) * 8.0;
        let min = Vec2::new(
            ctx.camera.position.x - half_width - padding,
            ctx.camera.position.y - half_height - padding,
        );
        let max = Vec2::new(
            ctx.camera.position.x + half_width + padding,
            ctx.camera.position.y + half_height + padding,
        );

        // Instanzen aus RoadMap sammeln
        let mut instances = std::mem::take(&mut self.instance_scratch);
        instances.clear();

        for node_id in road_map.nodes_within_rect(min, max) {
            let Some(node) = road_map.nodes.get(&node_id) else {
                continue;
            };

            let is_selected = selected_set.contains(&node.id);
            // Basisfarbe entspricht dem Node-Flag (bleibt mittig sichtbar)
            let base_color = match node.flag {
                NodeFlag::SubPrio => ctx.options.node_color_subprio,
                NodeFlag::Warning => ctx.options.node_color_warning,
                _ => ctx.options.node_color_default,
            };
            // Rim/Markierungsfarbe außen — nur bei selektierten Nodes anders
            let rim_color = if is_selected {
                ctx.options.node_color_selected
            } else {
                base_color
            };

            let size = if is_selected {
                ctx.options.node_size_world * ctx.options.selection_size_factor
            } else {
                ctx.options.node_size_world
            };

            instances.push(NodeInstance::new(
                [node.position.x, node.position.y],
                base_color,
                rim_color,
                size,
            ));
        }

        if instances.is_empty() {
            log::warn!("No instances to render");
            self.instance_scratch = instances;
            return;
        }

        log::debug!(
            "Rendering {} instances, camera: ({:.1}, {:.1}), zoom: {:.2}",
            instances.len(),
            ctx.camera.position.x,
            ctx.camera.position.y,
            ctx.camera.zoom
        );

        // View-Projektion-Matrix berechnen (gemeinsame Funktion)
        let view_proj = super::types::build_view_projection(ctx.camera, ctx.viewport_size);
        let view_proj_array = view_proj.to_cols_array_2d();

        // Uniform-Buffer aktualisieren
        let aa_params = match render_quality {
            RenderQuality::Low => [0.0, 1.0, 0.0, 0.0],
            RenderQuality::Medium => [1.0, 0.0, 0.0, 0.0],
            RenderQuality::High => [1.8, 0.0, 0.0, 0.0],
        };

        let uniforms = Uniforms {
            view_proj: view_proj_array,
            aa_params,
        };
        ctx.queue
            .write_buffer(&self.uniform_buffer, 0, bytemuck::cast_slice(&[uniforms]));

        // Instance-Buffer erstellen/aktualisieren (Reuse)
        if self.instance_buffer.is_none() || instances.len() > self.instance_capacity {
            let instance_size = std::mem::size_of::<NodeInstance>() as u64;
            let buffer_size = (instances.len() as u64) * instance_size;
            self.instance_buffer = Some(ctx.device.create_buffer(&wgpu::BufferDescriptor {
                label: Some("Instance Buffer"),
                size: buffer_size,
                usage: wgpu::BufferUsages::VERTEX | wgpu::BufferUsages::COPY_DST,
                mapped_at_creation: false,
            }));
            self.instance_capacity = instances.len();
        }

        if let Some(instance_buffer) = &self.instance_buffer {
            ctx.queue
                .write_buffer(instance_buffer, 0, bytemuck::cast_slice(&instances));
        }

        // Rendern
        let Some(instance_buffer) = self.instance_buffer.as_ref() else {
            log::error!("NodeRenderer: missing instance buffer before draw call");
            self.instance_scratch = instances;
            return;
        };

        render_pass.set_pipeline(&self.pipeline);
        render_pass.set_bind_group(0, &self.bind_group, &[]);
        render_pass.set_vertex_buffer(0, self.vertex_buffer.slice(..));
        render_pass.set_vertex_buffer(1, instance_buffer.slice(..));
        render_pass.draw(0..6, 0..instances.len() as u32);
        self.instance_scratch = instances;
    }
}
