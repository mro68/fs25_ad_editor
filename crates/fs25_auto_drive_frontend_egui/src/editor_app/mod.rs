//! Haupt-App und Event-Loop-Integration fuer den Editor.

mod event_collection;
mod helpers;
mod overlays;

use crate::app::{use_cases, AppController, AppIntent, AppState};
use crate::{render, ui};
use eframe::egui;
use eframe::egui_wgpu;
use fs25_auto_drive_host_bridge::apply_mapped_intent;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum IntentDispatchRoute {
    Bridge,
    LocalFallback,
}

fn dispatch_intent_with_bridge_fallback(
    controller: &mut AppController,
    state: &mut AppState,
    intent: AppIntent,
) -> anyhow::Result<IntentDispatchRoute> {
    if apply_mapped_intent(controller, state, &intent)? {
        return Ok(IntentDispatchRoute::Bridge);
    }

    controller.handle_intent(state, intent)?;
    Ok(IntentDispatchRoute::LocalFallback)
}

/// Haupt-Anwendungsstruktur.
pub(crate) struct EditorApp {
    state: AppState,
    controller: AppController,
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

        let mut state = AppState::new();
        state.set_options(editor_options);

        Self {
            state,
            controller: AppController::new(),
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

        if self.state.should_exit {
            ctx.send_viewport_cmd(egui::ViewportCommand::Close);
            return;
        }

        self.pending_render_assets = None;

        let events = self.collect_ui_events(&ctx);

        let has_meaningful_events = events
            .iter()
            .any(|e| !matches!(e, AppIntent::ViewportResized { .. }));

        self.process_events(&ctx, events);

        self.sync_background_upload();

        self.maybe_request_repaint(&ctx, has_meaningful_events);
    }
}

impl EditorApp {
    fn process_events(&mut self, ctx: &egui::Context, events: Vec<AppIntent>) {
        for event in events {
            match event {
                AppIntent::ToggleFloatingMenu { kind } => {
                    self.toggle_floating_menu(ctx, kind);
                }
                intent => {
                    if let Err(e) = dispatch_intent_with_bridge_fallback(
                        &mut self.controller,
                        &mut self.state,
                        intent,
                    ) {
                        self.state.ui.status_message =
                            Some(format!("Aktion fehlgeschlagen: {}", e));
                        log::error!("Event handling failed: {:#}", e);
                    }
                }
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use crate::app::{AppController, AppIntent, AppState};

    use super::{dispatch_intent_with_bridge_fallback, IntentDispatchRoute};

    #[test]
    fn dispatch_prefers_bridge_for_mapped_intents() {
        let mut controller = AppController::new();
        let mut state = AppState::new();

        let route = dispatch_intent_with_bridge_fallback(
            &mut controller,
            &mut state,
            AppIntent::OpenFileRequested,
        )
        .expect("OpenFileRequested muss ueber die Bridge-Seam laufen");

        assert_eq!(route, IntentDispatchRoute::Bridge);
        assert_eq!(state.ui.dialog_requests.len(), 1);
    }

    #[test]
    fn dispatch_falls_back_to_local_controller_for_unmapped_intents() {
        let mut controller = AppController::new();
        let mut state = AppState::new();

        let route = dispatch_intent_with_bridge_fallback(
            &mut controller,
            &mut state,
            AppIntent::ViewportResized {
                size: [640.0, 480.0],
            },
        )
        .expect("Unmapped Intent muss ueber den lokalen Fallback verarbeitet werden");

        assert_eq!(route, IntentDispatchRoute::LocalFallback);
        assert_eq!(state.view.viewport_size, [640.0, 480.0]);
        assert!(state.ui.dialog_requests.is_empty());
    }
}
