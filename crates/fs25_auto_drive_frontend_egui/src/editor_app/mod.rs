//! Haupt-App und Event-Loop-Integration fuer den Editor.

mod event_collection;
mod helpers;
mod overlays;
mod panel_collector;

use crate::app::{use_cases, AppIntent};
use crate::{render, ui};
use eframe::egui;
use eframe::egui_wgpu;
use fs25_auto_drive_host_bridge::{
    map_intent_to_host_action, HostBridgeSession, HostSessionAction, HostViewportInputEvent,
};

fn map_intent_to_collected_event(intent: AppIntent) -> CollectedEvent {
    if let Some(action) = map_intent_to_host_action(&intent) {
        CollectedEvent::HostAction(action)
    } else {
        CollectedEvent::Intent(intent)
    }
}

#[derive(Debug, Clone)]
pub(super) enum CollectedEvent {
    Intent(AppIntent),
    HostAction(HostSessionAction),
}

fn is_meaningful_event(event: &CollectedEvent) -> bool {
    match event {
        CollectedEvent::Intent(intent) => !matches!(intent, AppIntent::ViewportResized { .. }),
        CollectedEvent::HostAction(HostSessionAction::SubmitViewportInput { batch }) => batch
            .events
            .iter()
            .any(|event| !matches!(event, HostViewportInputEvent::Resize { .. })),
        CollectedEvent::HostAction(_) => true,
    }
}

fn intent_requires_canonical_host_action(intent: &AppIntent) -> bool {
    matches!(
        intent,
        AppIntent::CommandPaletteToggled
            | AppIntent::SetEditorToolRequested { .. }
            | AppIntent::SetDefaultDirectionRequested { .. }
            | AppIntent::SetDefaultPriorityRequested { .. }
            | AppIntent::OptionsChanged { .. }
            | AppIntent::ResetOptionsRequested
            | AppIntent::OpenOptionsDialogRequested
            | AppIntent::CloseOptionsDialogRequested
            | AppIntent::UndoRequested
            | AppIntent::RedoRequested
            | AppIntent::SelectRouteToolRequested { .. }
            | AppIntent::RouteToolWithAnchorsRequested { .. }
            | AppIntent::RouteToolPanelActionRequested { .. }
            | AppIntent::RouteToolExecuteRequested
            | AppIntent::RouteToolCancelled
            | AppIntent::RouteToolConfigChanged
            | AppIntent::RouteToolRecreateRequested
            | AppIntent::RouteToolTangentSelected { .. }
            | AppIntent::RouteToolClicked { .. }
            | AppIntent::RouteToolLassoCompleted { .. }
            | AppIntent::RouteToolDragStarted { .. }
            | AppIntent::RouteToolDragUpdated { .. }
            | AppIntent::RouteToolDragEnded
            | AppIntent::RouteToolScrollRotated { .. }
            | AppIntent::IncreaseRouteToolNodeCount
            | AppIntent::DecreaseRouteToolNodeCount
            | AppIntent::IncreaseRouteToolSegmentLength
            | AppIntent::DecreaseRouteToolSegmentLength
    )
}

fn dispatch_intent_via_session(
    session: &mut HostBridgeSession,
    intent: AppIntent,
) -> anyhow::Result<()> {
    if let Some(action) = map_intent_to_host_action(&intent) {
        session.apply_action(action)?;
    } else if intent_requires_canonical_host_action(&intent) {
        anyhow::bail!(
            "Intent muss ueber die kanonische HostAction-Seam laufen und darf nicht in den lokalen Fallback fallen: {:?}",
            intent
        );
    } else {
        session.apply_intent(intent)?;
    }
    Ok(())
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

        if self.session.app_state().should_exit {
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

impl EditorApp {
    fn process_events(&mut self, ctx: &egui::Context, events: Vec<CollectedEvent>) {
        for event in events {
            match event {
                CollectedEvent::HostAction(action) => {
                    if let Err(e) = self.session.apply_action(action) {
                        self.session
                            .set_status_message(Some(format!("Aktion fehlgeschlagen: {}", e)));
                        log::error!("Host action handling failed: {:#}", e);
                    }
                }
                CollectedEvent::Intent(AppIntent::ToggleFloatingMenu { kind }) => {
                    self.toggle_floating_menu(ctx, kind);
                }
                CollectedEvent::Intent(intent) => {
                    if let Err(e) = dispatch_intent_via_session(&mut self.session, intent) {
                        self.session
                            .set_status_message(Some(format!("Aktion fehlgeschlagen: {}", e)));
                        log::error!("Event handling failed: {:#}", e);
                    }
                }
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use crate::app::AppIntent;
    use fs25_auto_drive_host_bridge::HostBridgeSession;

    use super::{dispatch_intent_via_session, intent_requires_canonical_host_action};

    #[test]
    fn dispatch_via_session_routes_mapped_intents_over_host_actions() {
        let mut session = HostBridgeSession::new();

        dispatch_intent_via_session(&mut session, AppIntent::OpenFileRequested)
            .expect("OpenFileRequested muss ueber die Bridge-Seam laufen");

        assert_eq!(session.snapshot().pending_dialog_request_count, 1);
    }

    #[test]
    fn dispatch_via_session_keeps_unmapped_intents_funktional() {
        let mut session = HostBridgeSession::new();

        dispatch_intent_via_session(
            &mut session,
            AppIntent::ViewportResized {
                size: [640.0, 480.0],
            },
        )
        .expect("Unmapped Intent muss ueber den lokalen Fallback verarbeitet werden");

        assert_eq!(session.app_state().view.viewport_size, [640.0, 480.0]);
        assert!(session.app_state().ui.dialog_requests.is_empty());
    }

    #[test]
    fn canonical_route_tool_and_chrome_intents_are_guarded_against_fallback() {
        assert!(intent_requires_canonical_host_action(
            &AppIntent::RouteToolClicked {
                world_pos: glam::Vec2::new(1.0, 2.0),
                ctrl: false,
            }
        ));
        assert!(intent_requires_canonical_host_action(
            &AppIntent::SetDefaultPriorityRequested {
                priority: crate::app::ConnectionPriority::SubPriority,
            }
        ));
        assert!(!intent_requires_canonical_host_action(
            &AppIntent::ViewportResized {
                size: [320.0, 200.0],
            }
        ));
    }
}
