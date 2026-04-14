//! Hilfsmethoden fuer Floating-Menue, Background-Upload und Repaint.

use crate::app::state::FloatingMenuKind;
use crate::render;
use eframe::egui;
use eframe::egui_wgpu;
use fs25_auto_drive_host_bridge::{
    HostChromeSnapshot, HostDialogSnapshot, HostRenderFrameSnapshot,
};

use super::EditorApp;

fn split_render_frame_for_egui(
    frame: HostRenderFrameSnapshot,
) -> (render::WgpuRenderData, crate::shared::RenderAssetsSnapshot) {
    (render::WgpuRenderData { scene: frame.scene }, frame.assets)
}

fn should_request_repaint(
    has_meaningful_events: bool,
    pointer_is_moving: bool,
    floating_menu_open: bool,
    chrome_snapshot: &HostChromeSnapshot,
    dialog_snapshot: &HostDialogSnapshot,
) -> bool {
    has_meaningful_events
        || pointer_is_moving
        || chrome_snapshot.show_command_palette
        || floating_menu_open
        || dialog_snapshot.heightmap_warning.visible
        || dialog_snapshot.marker_dialog.visible
        || chrome_snapshot.show_options_dialog
}

impl EditorApp {
    /// Zeichnet die wgpu-Render-Szene in den Viewport.
    pub(super) fn render_viewport(
        &mut self,
        ui: &egui::Ui,
        rect: egui::Rect,
        viewport_size: [f32; 2],
    ) {
        let frame = self.session.build_render_frame(viewport_size);
        let (render_data, assets) = split_render_frame_for_egui(frame);
        self.pending_render_assets = Some(assets);

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
        let pointer_pos = ctx
            .input(|i| i.pointer.hover_pos().or(i.pointer.latest_pos()))
            .map(|p| glam::vec2(p.x, p.y));
        self.session.toggle_floating_menu(kind, pointer_pos);
    }

    pub(super) fn sync_background_upload(&mut self) {
        let Some(assets) = self.pending_render_assets.as_ref() else {
            return;
        };
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
        let chrome_snapshot = self.session.build_host_chrome_snapshot();
        let dialog_snapshot = self.session.dialog_snapshot();
        let floating_menu_open = self.session.chrome_state().floating_menu.is_some();
        let pointer_is_moving = ctx.input(|i| i.pointer.is_moving());

        if should_request_repaint(
            has_meaningful_events,
            pointer_is_moving,
            floating_menu_open,
            &chrome_snapshot,
            &dialog_snapshot,
        ) {
            ctx.request_repaint();
        }
    }
}

#[cfg(test)]
mod tests {
    use crate::app::AppState;
    use crate::core::{
        Connection, ConnectionDirection, ConnectionPriority, MapNode, NodeFlag, RoadMap,
    };
    use fs25_auto_drive_host_bridge::{build_render_frame, HostBridgeSession};
    use glam::Vec2;
    use std::sync::Arc;

    use super::{should_request_repaint, split_render_frame_for_egui};

    fn regression_test_map() -> RoadMap {
        let mut map = RoadMap::new(2);
        map.add_node(MapNode::new(1, Vec2::new(0.0, 0.0), NodeFlag::Regular));
        map.add_node(MapNode::new(2, Vec2::new(20.0, 10.0), NodeFlag::SubPrio));
        map.add_connection(Connection::new(
            1,
            2,
            ConnectionDirection::Dual,
            ConnectionPriority::SubPriority,
            Vec2::new(0.0, 0.0),
            Vec2::new(20.0, 10.0),
        ));
        map.ensure_spatial_index();
        map
    }

    #[test]
    fn split_render_frame_for_egui_keeps_background_sync_assets_coupled_to_scene() {
        let mut state = AppState::new();
        state.road_map = Some(Arc::new(regression_test_map()));
        state.view.background_asset_revision = 7;
        state.view.background_transform_revision = 11;

        let frame = build_render_frame(&state, [512.0, 256.0]);
        let expected_asset_revision = frame.assets.background_asset_revision();
        let expected_transform_revision = frame.assets.background_transform_revision();

        // Simuliert eine spaetere State-Aenderung nach dem Frame-Build.
        state.view.background_asset_revision = 101;
        state.view.background_transform_revision = 151;

        let (render_data, pending_assets) = split_render_frame_for_egui(frame);

        assert!(render_data.scene.has_map());
        assert_eq!(render_data.scene.viewport_size(), [512.0, 256.0]);
        assert_eq!(
            pending_assets.background_asset_revision(),
            expected_asset_revision
        );
        assert_eq!(
            pending_assets.background_transform_revision(),
            expected_transform_revision
        );
        assert_ne!(
            pending_assets.background_asset_revision(),
            state.view.background_asset_revision
        );
        assert_ne!(
            pending_assets.background_transform_revision(),
            state.view.background_transform_revision
        );
    }

    #[test]
    fn should_request_repaint_stays_idle_without_activity() {
        let session = HostBridgeSession::new();
        let chrome_snapshot = session.build_host_chrome_snapshot();
        let dialog_snapshot = session.dialog_snapshot();

        assert!(!should_request_repaint(
            false,
            false,
            false,
            &chrome_snapshot,
            &dialog_snapshot,
        ));
    }

    #[test]
    fn should_request_repaint_uses_dialog_snapshot_visibilities() {
        let mut session = HostBridgeSession::new();
        {
            let chrome_state = session.chrome_state_mut();
            chrome_state.show_heightmap_warning = true;
        }

        let chrome_snapshot = session.build_host_chrome_snapshot();
        let dialog_snapshot = session.dialog_snapshot();
        assert!(should_request_repaint(
            false,
            false,
            false,
            &chrome_snapshot,
            &dialog_snapshot,
        ));

        {
            let chrome_state = session.chrome_state_mut();
            chrome_state.show_heightmap_warning = false;
            chrome_state.marker_dialog.visible = true;
        }

        let chrome_snapshot = session.build_host_chrome_snapshot();
        let dialog_snapshot = session.dialog_snapshot();
        assert!(should_request_repaint(
            false,
            false,
            false,
            &chrome_snapshot,
            &dialog_snapshot,
        ));
    }
}
