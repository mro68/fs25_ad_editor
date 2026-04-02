//! Hilfsmethoden fuer Floating-Menue, Background-Upload und Repaint.

use eframe::egui;
use eframe::egui_wgpu;
use fs25_auto_drive_editor::app::state::{FloatingMenuKind, FloatingMenuState};
use fs25_auto_drive_editor::render;

use super::EditorApp;

impl EditorApp {
    /// Zeichnet die wgpu-Render-Szene in den Viewport.
    pub(super) fn render_viewport(
        &mut self,
        ui: &egui::Ui,
        rect: egui::Rect,
        viewport_size: [f32; 2],
    ) {
        let render_data = render::WgpuRenderData {
            scene: self
                .controller
                .build_render_scene(&self.state, viewport_size),
        };

        let callback = egui_wgpu::Callback::new_paint_callback(
            rect,
            render::WgpuRenderCallback {
                renderer: self.renderer.clone(),
                render_data,
                device: self.device.clone(),
                queue: self.queue.clone(),
            },
        );

        ui.painter().add(callback);
    }

    pub(super) fn toggle_floating_menu(&mut self, ctx: &egui::Context, kind: FloatingMenuKind) {
        if let Some(existing) = self.state.ui.floating_menu {
            if existing.kind == kind {
                self.state.ui.floating_menu = None;
            } else {
                let pos = ctx.input(|i| i.pointer.hover_pos().or(i.pointer.latest_pos()));
                self.state.ui.floating_menu = pos.map(|p| FloatingMenuState { kind, pos: p });
            }
        } else {
            let pos = ctx.input(|i| i.pointer.hover_pos().or(i.pointer.latest_pos()));
            self.state.ui.floating_menu = pos.map(|p| FloatingMenuState { kind, pos: p });
        }
    }

    pub(super) fn sync_background_upload(&mut self) {
        if !self.state.view.background_dirty {
            return;
        }
        self.state.view.background_dirty = false;

        let Ok(mut renderer) = self.renderer.lock() else {
            log::error!("Renderer-Lock fehlgeschlagen (Mutex vergiftet)");
            return;
        };
        if let Some(bg_map) = self.state.view.background_map.as_deref() {
            let bounds = bg_map.world_bounds();
            renderer.set_background(
                &self.device,
                &self.queue,
                bg_map.image_data(),
                render::BackgroundWorldBounds {
                    min_x: bounds.min_x,
                    max_x: bounds.max_x,
                    min_y: bounds.min_z,
                    max_y: bounds.max_z,
                },
                self.state.view.background_scale,
            );
            log::info!("Background-Map in Renderer hochgeladen");
        } else {
            renderer.clear_background();
            log::info!("Background-Map aus Renderer entfernt");
        }
    }

    pub(super) fn maybe_request_repaint(&self, ctx: &egui::Context, has_meaningful_events: bool) {
        if has_meaningful_events
            || ctx.input(|i| i.pointer.is_moving())
            || self.state.ui.show_command_palette
            || self.state.ui.floating_menu.is_some()
            || self.state.ui.show_heightmap_warning
            || self.state.ui.marker_dialog.visible
            || self.state.show_options_dialog
        {
            ctx.request_repaint();
        }
    }
}
