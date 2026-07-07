//! Haupt-App und Event-Loop-Integration fuer den Editor.

mod dialog_collector;
mod event_collection;
mod helpers;
mod overlays;
mod panel_collector;
/// Processor-Gegenstueck zu den Collector-Modulen: Event-Dispatch nach der
/// Collector-Phase (siehe `processor.rs`).
mod processor;
mod viewport_collector;

use crate::app::{use_cases, AppIntent};
use crate::{render, ui};
use eframe::egui;
use eframe::egui_wgpu;
use fs25_auto_drive_host_bridge::{HostBridgeSession, HostSessionAction};
use processor::{is_meaningful_event, map_intent_to_collected_event};

#[derive(Debug, Clone)]
pub(super) enum CollectedEvent {
    Intent(AppIntent),
    HostAction(HostSessionAction),
}

/// Haupt-Anwendungsstruktur.
pub(crate) struct EditorApp {
    session: HostBridgeSession,
    renderer: std::sync::Arc<std::sync::Mutex<render::Renderer>>,
    device: eframe::wgpu::Device,
    queue: eframe::wgpu::Queue,
    input: ui::InputState,
    /// Gecachte Cursor-Weltposition fuer Tool-Preview
    /// (bleibt erhalten wenn Maus den Viewport verlaesst).
    last_cursor_world: Option<glam::Vec2>,
    /// Letzte vom Host synchronisierte Background-Asset-Revision.
    last_background_asset_revision: u64,
    /// Letzte vom Host synchronisierte Background-Transform-Revision.
    last_background_transform_revision: u64,
    /// Render-Assets des im aktuellen egui-Frame aufgebauten gekoppelten RenderFrames.
    pending_render_assets: Option<crate::shared::RenderAssetsSnapshot>,
    /// Gecachte egui-Textur-Handles fuer Gruppen-Boundary-Icons (lazy initialisiert).
    group_boundary_icons: Option<ui::GroupBoundaryIcons>,
}

impl EditorApp {
    /// Erstellt die Editor-App mit geladenen Optionen und initialisiertem Renderer.
    pub(crate) fn new(render_state: &egui_wgpu::RenderState) -> Self {
        // Optionen aus TOML laden (oder Standardwerte)
        let editor_options = use_cases::options::load_editor_options();

        let mut session = HostBridgeSession::new();
        session
            .apply_action(HostSessionAction::ApplyOptions {
                options: Box::new(editor_options),
            })
            .expect("Editor-Optionen muessen beim Start in die Session geschrieben werden");

        Self {
            session,
            renderer: std::sync::Arc::new(std::sync::Mutex::new(render::Renderer::new(
                render_state,
            ))),
            device: render_state.device.clone(),
            queue: render_state.queue.clone(),
            input: ui::InputState::new(),
            last_cursor_world: None,
            last_background_asset_revision: 0,
            last_background_transform_revision: 0,
            pending_render_assets: None,
            group_boundary_icons: None,
        }
    }
}

impl eframe::App for EditorApp {
    fn ui(&mut self, ui: &mut egui::Ui, _frame: &mut eframe::Frame) {
        let ctx = ui.ctx().clone();

        if self.session.should_exit() {
            ctx.send_viewport_cmd(egui::ViewportCommand::Close);
            return;
        }

        self.pending_render_assets = None;

        let events = self.collect_ui_events(&ctx);

        let has_meaningful_events = events.iter().any(is_meaningful_event);

        self.process_events(&ctx, events);

        self.sync_background_upload();

        self.maybe_request_repaint(&ctx, has_meaningful_events);
    }
}
