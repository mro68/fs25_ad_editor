//! Processor-Gruppe der Editor-App-Integrationsschale: Event-Dispatch nach der
//! Collector-Phase. Pendant zu `panel_collector.rs`/`dialog_collector.rs`/
//! `viewport_collector.rs` (Collector-Seite): waehrend die Collector-Module
//! Panel-, Dialog- und Viewport-Events einsammeln, verarbeitet dieses Modul die
//! gesammelten `CollectedEvent`s und routet sie an die `HostBridgeSession`.
//! `editor_app::mod` bleibt dadurch reiner Orchestrator (Collector-Aufruf →
//! Processor-Aufruf → Background-Sync → Repaint).

use super::{CollectedEvent, EditorApp};
use crate::app::AppIntent;
use eframe::egui;
use fs25_auto_drive_host_bridge::{
    map_intent_to_host_action, HostBridgeSession, HostSessionAction, HostViewportInputEvent,
};

/// Ordnet einen `AppIntent` einem `CollectedEvent` zu.
///
/// Intents mit kanonischer `HostSessionAction`-Entsprechung werden direkt als
/// `HostAction` markiert, damit sie in `process_events` ohne weiteren
/// Mapping-Umweg an die Session gehen.
pub(super) fn map_intent_to_collected_event(intent: AppIntent) -> CollectedEvent {
    if let Some(action) = map_intent_to_host_action(&intent) {
        CollectedEvent::HostAction(action)
    } else {
        CollectedEvent::Intent(intent)
    }
}

/// Filtert Events heraus, die fuer Repaint-Entscheidungen irrelevant sind
/// (z. B. reine Resize-Events ohne sichtbare Zustandsaenderung).
pub(super) fn is_meaningful_event(event: &CollectedEvent) -> bool {
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
    intent.requires_bridge_action()
}

/// Intent-Routing-Guard: schuetzt kanonische Bridge-Intents vor dem lokalen
/// `apply_intent`-Fallback und stellt sicher, dass sie ausschliesslich ueber
/// `HostSessionAction` laufen.
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

impl EditorApp {
    /// Verarbeitet die in der Collector-Phase gesammelten Events: `HostSessionAction`
    /// direkt ueber die Session, `AppIntent` ueber `dispatch_intent_via_session(...)`.
    pub(super) fn process_events(&mut self, ctx: &egui::Context, events: Vec<CollectedEvent>) {
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
    fn canonical_route_tool_and_parity_intents_are_guarded_against_fallback() {
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
        assert!(intent_requires_canonical_host_action(
            &AppIntent::AddConnectionRequested {
                from_id: 1,
                to_id: 2,
                direction: crate::app::ConnectionDirection::Dual,
                priority: crate::app::ConnectionPriority::Regular,
            }
        ));
        assert!(intent_requires_canonical_host_action(
            &AppIntent::RemoveAllConnectionsBetweenSelectedRequested
        ));
        assert!(intent_requires_canonical_host_action(
            &AppIntent::CenterOnNodeRequested { node_id: 42 }
        ));
        assert!(intent_requires_canonical_host_action(
            &AppIntent::RenderQualityChanged {
                quality: fs25_auto_drive_engine::shared::RenderQuality::Medium,
            }
        ));
        assert!(intent_requires_canonical_host_action(
            &AppIntent::GroupEditStartRequested { record_id: 7 }
        ));
        assert!(intent_requires_canonical_host_action(
            &AppIntent::OpenTraceAllFieldsDialogRequested
        ));
        assert!(!intent_requires_canonical_host_action(
            &AppIntent::ViewportResized {
                size: [320.0, 200.0],
            }
        ));
    }
}
