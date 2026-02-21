//! wgpu Custom Render Callback für egui-Integration.

use super::Renderer;
use crate::shared::RenderScene;
use std::sync::{Arc, Mutex};

/// Render-Daten für den wgpu Callback
pub struct WgpuRenderData {
    /// Die Render-Szene für diesen Frame
    pub scene: RenderScene,
}

/// Custom wgpu Render Callback – kapselt die Renderer-Interaktion für egui
pub struct WgpuRenderCallback {
    /// Geteilter Renderer-Zustand (thread-safe)
    pub renderer: Arc<Mutex<Renderer>>,
    /// Render-Daten für diesen Frame
    pub render_data: WgpuRenderData,
    /// wgpu Device für GPU-Ressourcen
    pub device: eframe::wgpu::Device,
    /// wgpu Queue für GPU-Befehle
    pub queue: eframe::wgpu::Queue,
}

impl eframe::egui_wgpu::CallbackTrait for WgpuRenderCallback {
    fn prepare(
        &self,
        _device: &eframe::wgpu::Device,
        _queue: &eframe::wgpu::Queue,
        _screen_descriptor: &eframe::egui_wgpu::ScreenDescriptor,
        _egui_encoder: &mut eframe::wgpu::CommandEncoder,
        _callback_resources: &mut eframe::egui_wgpu::CallbackResources,
    ) -> Vec<eframe::wgpu::CommandBuffer> {
        Vec::new()
    }

    fn paint<'b>(
        &'b self,
        _info: egui::PaintCallbackInfo,
        render_pass: &mut eframe::wgpu::RenderPass<'static>,
        _callback_resources: &'b eframe::egui_wgpu::CallbackResources,
    ) {
        log::debug!("paint() called");
        if let Ok(mut renderer) = self.renderer.lock() {
            let has_map = self.render_data.scene.has_map();
            log::debug!("Calling renderer.render_scene(), has_map: {}", has_map);
            renderer.render_scene(
                &self.device,
                &self.queue,
                render_pass,
                &self.render_data.scene,
            );
        } else {
            log::error!("Failed to lock renderer");
        }
    }
}
