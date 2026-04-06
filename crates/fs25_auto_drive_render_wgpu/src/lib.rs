//! Host-neutraler wgpu-Renderer-Kern fuer den FS25 AutoDrive Editor.

mod background_renderer;
mod connection_renderer;
mod export_core;
mod fingerprint;
mod marker_renderer;
mod node_renderer;
mod shared_texture;
mod texture;
mod types;

pub use background_renderer::BackgroundWorldBounds;
pub use fs25_auto_drive_engine::shared;
pub use fs25_auto_drive_engine::shared::{RenderQuality, RenderScene};
pub use shared_texture::{
    SharedTextureAlphaMode, SharedTextureError, SharedTextureFrame, SharedTextureNativeHandle,
    SharedTexturePixelFormat, SharedTextureRuntime,
};

pub(crate) use background_renderer::BackgroundRenderer;
pub(crate) use connection_renderer::ConnectionRenderer;
use fs25_auto_drive_engine::shared::EditorOptions;
pub(crate) use marker_renderer::MarkerRenderer;
pub(crate) use node_renderer::NodeRenderer;
use types::RenderContext;

/// Zielkonfiguration des Render-Targets.
///
/// Der Host liefert damit das Farbformat und den gewuenschten MSAA-Count des
/// aktuellen Targets an den host-neutralen Render-Core weiter.
#[derive(Debug, Clone, Copy)]
pub struct RendererTargetConfig {
    /// Farbformat des Render-Targets.
    pub color_format: wgpu::TextureFormat,
    /// MSAA-Sample-Count des Render-Targets.
    pub sample_count: u32,
}

impl RendererTargetConfig {
    /// Erstellt eine neue Target-Konfiguration.
    pub fn new(color_format: wgpu::TextureFormat, sample_count: u32) -> Self {
        Self {
            color_format,
            sample_count,
        }
    }
}

impl Default for RendererTargetConfig {
    fn default() -> Self {
        Self {
            color_format: wgpu::TextureFormat::Bgra8UnormSrgb,
            sample_count: 4,
        }
    }
}

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
        // Kein Fade-Bereich — direkt Min-Zoom-Deckung
        opts.bg_opacity_at_min_zoom
    } else {
        let t = ((zoom - min_zoom) / (fade_start - min_zoom)).clamp(0.0, 1.0);
        opts.bg_opacity_at_min_zoom + t * (opts.bg_opacity - opts.bg_opacity_at_min_zoom)
    }
}

/// Haupt-Renderer fuer AutoDrive-Daten.
///
/// Dieser Renderer ist host-neutral und arbeitet nur mit raw `wgpu` plus
/// dem engine-seitigen Render-Vertrag `RenderScene`. Langlebige Assets wie der
/// Hintergrund werden ueber explizite Upload-Methoden separat synchronisiert.
pub struct Renderer {
    background_renderer: BackgroundRenderer,
    connection_renderer: ConnectionRenderer,
    node_renderer: NodeRenderer,
    marker_renderer: MarkerRenderer,
}

impl Renderer {
    /// Erstellt einen neuen Renderer.
    pub fn new(
        device: &wgpu::Device,
        queue: &wgpu::Queue,
        target_config: RendererTargetConfig,
    ) -> Self {
        // Shader einmalig laden — alle Sub-Renderer teilen dasselbe ShaderModule
        let shader = device.create_shader_module(wgpu::ShaderModuleDescriptor {
            label: Some("AutoDrive Shader"),
            source: wgpu::ShaderSource::Wgsl(include_str!("shaders.wgsl").into()),
        });

        let background_renderer = BackgroundRenderer::new(device, &shader, target_config);
        let connection_renderer = ConnectionRenderer::new(device, &shader, target_config);
        let node_renderer = NodeRenderer::new(device, &shader, target_config);
        let marker_renderer = MarkerRenderer::new(device, queue, &shader, target_config);

        Self {
            background_renderer,
            connection_renderer,
            node_renderer,
            marker_renderer,
        }
    }

    /// Rendert die komplette Szene.
    pub fn render_scene(
        &mut self,
        device: &wgpu::Device,
        queue: &wgpu::Queue,
        render_pass: &mut wgpu::RenderPass<'_>,
        scene: &RenderScene,
    ) {
        // Gemeinsamer Kontext fuer alle Sub-Renderer
        let ctx = RenderContext {
            device,
            queue,
            camera: scene.camera(),
            viewport_size: scene.viewport_size(),
            options: scene.options(),
            hidden_node_ids: scene.hidden_node_ids(),
            hidden_node_ids_revision: scene.hidden_node_ids_revision(),
            dimmed_node_ids: scene.dimmed_node_ids(),
            dimmed_node_ids_revision: scene.dimmed_node_ids_revision(),
            selected_node_ids_revision: scene.selected_node_ids_revision(),
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
            self.marker_renderer
                .render(&ctx, render_pass, render_map, scene.render_quality());

            // 3. Render Connections (darueber)
            self.connection_renderer
                .render(&ctx, render_pass, render_map);

            // 4. Render Nodes (zuoberst)
            self.node_renderer.render(
                &ctx,
                render_pass,
                render_map,
                scene.render_quality(),
                scene.selected_node_ids(),
            );
        }
    }

    /// Setzt das Hintergrundbild fuer den Renderer.
    ///
    /// `world_bounds` liegt im 2D-Koordinatensystem des Render-Core (`x/y`).
    /// Host-Adapter koennen Engine-seitige X/Z-Bounds vor dem Aufruf auf diese
    /// Achsen umlegen.
    pub fn set_background(
        &mut self,
        device: &wgpu::Device,
        queue: &wgpu::Queue,
        image: &image::DynamicImage,
        world_bounds: BackgroundWorldBounds,
        scale: f32,
    ) {
        self.background_renderer
            .set_background(device, queue, image, world_bounds, scale);
    }

    /// Entfernt die Background-Map.
    pub fn clear_background(&mut self) {
        self.background_renderer.clear_background();
    }
}
