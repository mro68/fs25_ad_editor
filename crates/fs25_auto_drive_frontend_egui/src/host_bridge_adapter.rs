//! Egui-spezifische Kompatibilitaets-Reexports fuer die gemeinsame Host-Bridge.
//!
//! Das kanonische Mapping (`AppIntent` -> `HostSessionAction`) und der
//! zugehoerige Dispatch leben in `fs25_auto_drive_host_bridge::dispatch`.
//! Dieses Modul bleibt nur als stabile Import-Surface fuer bestehende egui-
//! interne Aufrufer erhalten.

pub use fs25_auto_drive_host_bridge::{apply_mapped_intent, map_intent_to_host_action};

#[cfg(test)]
mod tests {
    use fs25_auto_drive_host_bridge::HostSessionAction;

    use crate::app::{AppController, AppIntent, AppState};

    use super::{apply_mapped_intent, map_intent_to_host_action};

    #[test]
    fn forwards_mapping_to_canonical_host_bridge_source_of_truth() {
        assert_eq!(
            map_intent_to_host_action(&AppIntent::OpenFileRequested),
            Some(HostSessionAction::OpenFile)
        );
    }

    #[test]
    fn apply_mapped_intent_dispatches_action_through_shared_host_seam() {
        let mut controller = AppController::new();
        let mut state = AppState::new();

        let handled =
            apply_mapped_intent(&mut controller, &mut state, &AppIntent::OpenFileRequested)
                .expect("OpenFileRequested muss ueber die Bridge-Seam verarbeitet werden");

        assert!(handled);
        assert_eq!(state.ui.dialog_requests.len(), 1);
    }

    #[test]
    fn apply_mapped_intent_returns_false_for_unmapped_intent() {
        let mut controller = AppController::new();
        let mut state = AppState::new();

        let handled = apply_mapped_intent(
            &mut controller,
            &mut state,
            &AppIntent::ViewportResized {
                size: [640.0, 480.0],
            },
        )
        .expect("Unmapped Intent darf keinen Fehler ausloesen");

        assert!(!handled);
    }
}
