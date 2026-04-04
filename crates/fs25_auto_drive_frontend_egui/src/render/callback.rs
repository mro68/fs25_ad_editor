//! wgpu Custom Render Callback fuer egui-Integration.

use super::Renderer;
use crate::shared::RenderScene;
use std::sync::{Arc, Mutex};

/// Render-Daten fuer den wgpu Callback
pub struct WgpuRenderData {
    /// Die Render-Szene fuer diesen Frame
    pub scene: RenderScene,
}

/// Custom wgpu Render Callback – kapselt die Renderer-Interaktion fuer egui
pub struct WgpuRenderCallback {
    /// Geteilter Renderer-Zustand (thread-safe)
    pub renderer: Arc<Mutex<Renderer>>,
    /// Render-Daten fuer diesen Frame
    pub render_data: WgpuRenderData,
    /// wgpu Device fuer GPU-Ressourcen
    pub device: eframe::wgpu::Device,
    /// wgpu Queue fuer GPU-Befehle
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
        if let Ok(mut renderer) = self.renderer.lock() {
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
