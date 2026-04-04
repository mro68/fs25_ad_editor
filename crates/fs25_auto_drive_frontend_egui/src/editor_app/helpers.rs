//! Hilfsmethoden fuer Floating-Menue, Background-Upload und Repaint.

use crate::app::state::{FloatingMenuKind, FloatingMenuState};
use crate::render;
use eframe::egui;
use eframe::egui_wgpu;

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
                self.state.ui.floating_menu = pos.map(|p| FloatingMenuState {
                    kind,
                    pos: glam::vec2(p.x, p.y),
                });
            }
        } else {
            let pos = ctx.input(|i| i.pointer.hover_pos().or(i.pointer.latest_pos()));
            self.state.ui.floating_menu = pos.map(|p| FloatingMenuState {
                kind,
                pos: glam::vec2(p.x, p.y),
            });
        }
    }

    pub(super) fn sync_background_upload(&mut self) {
        let assets = self.controller.build_render_assets(&self.state);
        let asset_revision = assets.background_asset_revision();
        let transform_revision = assets.background_transform_revision();

        if asset_revision == self.last_background_asset_revision
            && transform_revision == self.last_background_transform_revision
        {
            return;
        }

        let Ok(mut renderer) = self.renderer.lock() else {
            log::error!("Renderer-Lock fehlgeschlagen (Mutex vergiftet)");
            return;
        };

        if let Some(background) = assets.background() {
            renderer.set_background(
                &self.device,
                &self.queue,
                background.image.as_ref(),
                render::BackgroundWorldBounds {
                    min_x: background.world_bounds.min_x,
                    max_x: background.world_bounds.max_x,
                    min_y: background.world_bounds.min_z,
                    max_y: background.world_bounds.max_z,
                },
                background.scale,
            );
            log::info!(
                "Background-Map in Renderer synchronisiert (asset_rev={}, transform_rev={})",
                background.asset_revision,
                background.transform_revision
            );
        } else {
            renderer.clear_background();
            log::info!(
                "Background-Map aus Renderer entfernt (asset_rev={}, transform_rev={})",
                asset_revision,
                transform_revision
            );
        }

        self.last_background_asset_revision = asset_revision;
        self.last_background_transform_revision = transform_revision;
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
