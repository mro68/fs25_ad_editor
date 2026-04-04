//! GPU-Rendering mit wgpu.

mod background_renderer;
mod callback;
mod connection_renderer;
mod fingerprint;
mod marker_renderer;
mod node_renderer;
mod texture;
mod types;

pub use crate::shared::{RenderQuality, RenderScene};
pub(crate) use background_renderer::BackgroundRenderer;
pub use background_renderer::BackgroundWorldBounds;
pub use callback::{WgpuRenderCallback, WgpuRenderData};
pub(crate) use connection_renderer::ConnectionRenderer;
pub(crate) use marker_renderer::MarkerRenderer;
pub(crate) use node_renderer::NodeRenderer;
use types::RenderContext;

use crate::shared::EditorOptions;
use eframe::egui_wgpu;

/// Berechnet die Hintergrund-Opacity basierend auf Zoom-Level und Optionen.
///
/// Oberhalb von `fade_start_zoom`: Standard-Deckung.
/// Zwischen `fade_start_zoom` und Kamera-Minimum: linear interpoliert.
/// Bei/unter Kamera-Minimum: `opacity_at_min_zoom`.
fn compute_background_opacity(zoom: f32, opts: &EditorOptions) -> f32 {
    let fade_start = opts.bg_fade_start_zoom;
    let min_zoom = opts.camera_zoom_min;

    if zoom >= fade_start {
        opts.bg_opacity
    } else if fade_start <= min_zoom {
        // Kein Fade-Bereich — direkt Min-Zoom-Deckung
        opts.bg_opacity_at_min_zoom
    } else {
        let t = ((zoom - min_zoom) / (fade_start - min_zoom)).clamp(0.0, 1.0);
        opts.bg_opacity_at_min_zoom + t * (opts.bg_opacity - opts.bg_opacity_at_min_zoom)
    }
}

/// Haupt-Renderer fuer AutoDrive-Daten.
///
/// Dieser Renderer verwaltet seinen eigenen Zustand (GPU-Buffer, Pipeline)
/// und bietet eine saubere API: `new()` + `render_scene()` + `set_background()`.
pub struct Renderer {
    background_renderer: BackgroundRenderer,
    connection_renderer: ConnectionRenderer,
    node_renderer: NodeRenderer,
    marker_renderer: MarkerRenderer,
}

impl Renderer {
    /// Erstellt einen neuen Renderer
    pub fn new(render_state: &egui_wgpu::RenderState) -> Self {
        let device = &render_state.device;

        // Shader einmalig laden — alle Sub-Renderer teilen dasselbe ShaderModule
        let shader = device.create_shader_module(wgpu::ShaderModuleDescriptor {
            label: Some("AutoDrive Shader"),
            source: wgpu::ShaderSource::Wgsl(include_str!("shaders.wgsl").into()),
        });

        let background_renderer = BackgroundRenderer::new(render_state, &shader);
        let connection_renderer = ConnectionRenderer::new(render_state, &shader);
        let node_renderer = NodeRenderer::new(render_state, &shader);
        let marker_renderer = MarkerRenderer::new(render_state, &shader);

        Self {
            background_renderer,
            connection_renderer,
            node_renderer,
            marker_renderer,
        }
    }

    /// Rendert die komplette Szene
    ///
    /// Diese Methode nimmt nur Referenzen - keine Daten werden kopiert!
    pub fn render_scene(
        &mut self,
        device: &eframe::wgpu::Device,
        queue: &eframe::wgpu::Queue,
        render_pass: &mut eframe::wgpu::RenderPass<'static>,
        scene: &RenderScene,
    ) {
        log::debug!(
            "Renderer.render_scene() called, has_map: {}",
            scene.has_map()
        );

        // Gemeinsamer Kontext fuer alle Sub-Renderer
        let ctx = RenderContext {
            device,
            queue,
            camera: scene.camera(),
            viewport_size: scene.viewport_size(),
            options: scene.options(),
            hidden_node_ids: scene.hidden_node_ids(),
            dimmed_node_ids: scene.dimmed_node_ids(),
        };

        // 1. Render Background zuerst (falls vorhanden)
        if scene.has_background() {
            let opacity = compute_background_opacity(scene.camera().zoom, scene.options());
            self.background_renderer.render(
                queue,
                render_pass,
                scene.camera(),
                scene.viewport_size(),
                scene.background_visible(),
                opacity,
            );
        }

        // 2. Render Markers (hinter Connections und Nodes)
        if let Some(render_map) = scene.map() {
            if render_map.marker_count() > 0 {
                log::debug!("Rendering {} markers", render_map.marker_count());
                self.marker_renderer
                    .render(&ctx, render_pass, render_map, scene.render_quality());
            }

            // 3. Render Connections (darueber)
            self.connection_renderer
                .render(&ctx, render_pass, render_map);

            // 4. Render Nodes (zuoberst)
            log::debug!(
                "Delegating to node_renderer, {} nodes",
                render_map.node_count()
            );
            self.node_renderer.render(
                &ctx,
                render_pass,
                render_map,
                scene.render_quality(),
                scene.selected_node_ids(),
            );
        } else {
            log::debug!("No render_map to render");
        }
    }

    /// Setzt das Hintergrundbild fuer den Renderer.
    pub fn set_background(
        &mut self,
        device: &eframe::wgpu::Device,
        queue: &eframe::wgpu::Queue,
        image: &image::DynamicImage,
        world_bounds: BackgroundWorldBounds,
        scale: f32,
    ) {
        self.background_renderer
            .set_background(device, queue, image, world_bounds, scale);
    }

    /// Entfernt die Background-Map
    pub fn clear_background(&mut self) {
        self.background_renderer.clear_background();
    }
}
