//! Egui-spezifischer Host-Adapter ueber dem host-neutralen wgpu-Renderkern.

mod callback;

pub use callback::{WgpuRenderCallback, WgpuRenderData};
pub use fs25_auto_drive_render_wgpu::BackgroundWorldBounds;
pub use fs25_auto_drive_render_wgpu::RenderQuality;
pub use fs25_auto_drive_render_wgpu::RenderScene;
pub use fs25_auto_drive_render_wgpu::RendererTargetConfig;

/// Egui-Host-Adapter fuer den host-neutralen Renderer-Kern.
pub struct Renderer {
    core: fs25_auto_drive_render_wgpu::Renderer,
}

impl Renderer {
    /// Erstellt einen neuen Renderer auf Basis des egui/wgpu-Hostzustands.
    pub fn new(render_state: &eframe::egui_wgpu::RenderState) -> Self {
        let target_config = RendererTargetConfig::new(render_state.target_format, 4);
        let core = fs25_auto_drive_render_wgpu::Renderer::new(
            &render_state.device,
            &render_state.queue,
            target_config,
        );
        Self { core }
    }

    /// Rendert die aktuelle Szene ueber den host-neutralen Kern.
    pub fn render_scene(
        &mut self,
        device: &eframe::wgpu::Device,
        queue: &eframe::wgpu::Queue,
        render_pass: &mut eframe::wgpu::RenderPass<'_>,
        scene: &RenderScene,
    ) {
        self.core.render_scene(device, queue, render_pass, scene);
    }

    /// Setzt oder aktualisiert das Background-Asset.
    pub fn set_background(
        &mut self,
        device: &eframe::wgpu::Device,
        queue: &eframe::wgpu::Queue,
        image: &image::DynamicImage,
        world_bounds: BackgroundWorldBounds,
        scale: f32,
    ) {
        self.core
            .set_background(device, queue, image, world_bounds, scale);
    }

    /// Entfernt das Background-Asset.
    pub fn clear_background(&mut self) {
        self.core.clear_background();
    }
}
