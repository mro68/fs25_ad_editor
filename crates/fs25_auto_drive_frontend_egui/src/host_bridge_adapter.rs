//! Egui-spezifische Adapter-Helfer fuer die gemeinsame Host-Bridge.
//!
//! Dieses Modul mappt stabile, niederfrequente `AppIntent`s auf die explizite
//! Action-Surface der kanonischen Host-Bridge.
//! Nicht gemappte Intents bleiben im bisherigen egui-Flow und koennen weiterhin
//! direkt ueber `AppController` verarbeitet werden.

use anyhow::Result;
use fs25_auto_drive_host_bridge::{apply_host_action, HostActiveTool, HostSessionAction};

use crate::app::{AppController, AppIntent, AppState, EditorTool};

fn map_active_tool(tool: EditorTool) -> HostActiveTool {
    match tool {
        EditorTool::Select => HostActiveTool::Select,
        EditorTool::Connect => HostActiveTool::Connect,
        EditorTool::AddNode => HostActiveTool::AddNode,
        EditorTool::Route => HostActiveTool::Route,
    }
}

/// Mappt einen egui-`AppIntent` auf eine explizite Host-Bridge-Action.
///
/// Rueckgabewert `None` bedeutet, dass der Intent aktuell ausserhalb der
/// schlanken Adapter-Surface liegt und weiterhin ueber den bestehenden
/// Controller-Flow verarbeitet werden soll.
pub fn map_intent_to_host_action(intent: &AppIntent) -> Option<HostSessionAction> {
    match intent {
        AppIntent::OpenFileRequested => Some(HostSessionAction::OpenFile),
        AppIntent::SaveRequested => Some(HostSessionAction::Save),
        AppIntent::SaveAsRequested => Some(HostSessionAction::SaveAs),
        AppIntent::HeightmapSelectionRequested => {
            Some(HostSessionAction::RequestHeightmapSelection)
        }
        AppIntent::BackgroundMapSelectionRequested => {
            Some(HostSessionAction::RequestBackgroundMapSelection)
        }
        AppIntent::GenerateOverviewRequested => Some(HostSessionAction::GenerateOverview),
        AppIntent::CurseplayImportRequested => Some(HostSessionAction::CurseplayImport),
        AppIntent::CurseplayExportRequested => Some(HostSessionAction::CurseplayExport),
        AppIntent::ResetCameraRequested => Some(HostSessionAction::ResetCamera),
        AppIntent::ZoomToFitRequested => Some(HostSessionAction::ZoomToFit),
        AppIntent::ZoomToSelectionBoundsRequested => Some(HostSessionAction::ZoomToSelectionBounds),
        AppIntent::ExitRequested => Some(HostSessionAction::Exit),
        AppIntent::CommandPaletteToggled => Some(HostSessionAction::ToggleCommandPalette),
        AppIntent::SetEditorToolRequested { tool } => Some(HostSessionAction::SetEditorTool {
            tool: map_active_tool(*tool),
        }),
        AppIntent::OpenOptionsDialogRequested => Some(HostSessionAction::OpenOptionsDialog),
        AppIntent::CloseOptionsDialogRequested => Some(HostSessionAction::CloseOptionsDialog),
        AppIntent::UndoRequested => Some(HostSessionAction::Undo),
        AppIntent::RedoRequested => Some(HostSessionAction::Redo),
        _ => None,
    }
}

/// Wendet einen gemappten Intent auf Controller und State an.
///
/// Gibt `Ok(true)` zurueck, wenn der Intent gemappt und angewendet wurde.
/// `Ok(false)` bedeutet, dass kein Mapping existiert.
pub fn apply_mapped_intent(
    controller: &mut AppController,
    state: &mut AppState,
    intent: &AppIntent,
) -> Result<bool> {
    let Some(action) = map_intent_to_host_action(intent) else {
        return Ok(false);
    };

    apply_host_action(controller, state, action)
}

#[cfg(test)]
mod tests {
    use fs25_auto_drive_host_bridge::{HostActiveTool, HostSessionAction};
    use glam::Vec2;

    use crate::app::{AppController, AppIntent, AppState, EditorTool};

    use super::{apply_mapped_intent, map_intent_to_host_action};

    #[test]
    fn maps_stable_host_intents_to_expected_actions() {
        let cases = vec![
            (AppIntent::OpenFileRequested, HostSessionAction::OpenFile),
            (AppIntent::SaveRequested, HostSessionAction::Save),
            (AppIntent::SaveAsRequested, HostSessionAction::SaveAs),
            (
                AppIntent::HeightmapSelectionRequested,
                HostSessionAction::RequestHeightmapSelection,
            ),
            (
                AppIntent::BackgroundMapSelectionRequested,
                HostSessionAction::RequestBackgroundMapSelection,
            ),
            (
                AppIntent::GenerateOverviewRequested,
                HostSessionAction::GenerateOverview,
            ),
            (
                AppIntent::CurseplayImportRequested,
                HostSessionAction::CurseplayImport,
            ),
            (
                AppIntent::CurseplayExportRequested,
                HostSessionAction::CurseplayExport,
            ),
            (
                AppIntent::ResetCameraRequested,
                HostSessionAction::ResetCamera,
            ),
            (AppIntent::ZoomToFitRequested, HostSessionAction::ZoomToFit),
            (
                AppIntent::ZoomToSelectionBoundsRequested,
                HostSessionAction::ZoomToSelectionBounds,
            ),
            (AppIntent::ExitRequested, HostSessionAction::Exit),
            (
                AppIntent::CommandPaletteToggled,
                HostSessionAction::ToggleCommandPalette,
            ),
            (
                AppIntent::SetEditorToolRequested {
                    tool: EditorTool::Route,
                },
                HostSessionAction::SetEditorTool {
                    tool: HostActiveTool::Route,
                },
            ),
            (
                AppIntent::OpenOptionsDialogRequested,
                HostSessionAction::OpenOptionsDialog,
            ),
            (
                AppIntent::CloseOptionsDialogRequested,
                HostSessionAction::CloseOptionsDialog,
            ),
            (AppIntent::UndoRequested, HostSessionAction::Undo),
            (AppIntent::RedoRequested, HostSessionAction::Redo),
        ];

        for (intent, expected_action) in cases {
            assert_eq!(map_intent_to_host_action(&intent), Some(expected_action));
        }
    }

    #[test]
    fn leaves_high_frequency_and_tool_intents_unmapped() {
        let unmapped = vec![
            AppIntent::ViewportResized {
                size: [1920.0, 1080.0],
            },
            AppIntent::CameraPan {
                delta: Vec2::new(2.0, -1.0),
            },
            AppIntent::CameraZoom {
                factor: 1.1,
                focus_world: Some(Vec2::ZERO),
            },
            AppIntent::NodePickRequested {
                world_pos: Vec2::new(5.0, 10.0),
                additive: false,
                extend_path: false,
            },
            AppIntent::RouteToolClicked {
                world_pos: Vec2::new(3.0, 4.0),
                ctrl: false,
            },
            AppIntent::MoveSelectedNodesRequested {
                delta_world: Vec2::new(1.0, 1.0),
            },
        ];

        for intent in unmapped {
            assert!(map_intent_to_host_action(&intent).is_none());
        }
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
    fn apply_mapped_intent_supports_set_editor_tool_path() {
        let mut controller = AppController::new();
        let mut state = AppState::new();

        let handled = apply_mapped_intent(
            &mut controller,
            &mut state,
            &AppIntent::SetEditorToolRequested {
                tool: EditorTool::Route,
            },
        )
        .expect("SetEditorToolRequested muss ueber die Bridge-Seam verarbeitet werden");

        assert!(handled);
        assert_eq!(state.editor.active_tool, EditorTool::Route);
    }

    #[test]
    fn apply_mapped_intent_supports_options_dialog_visibility_path() {
        let mut controller = AppController::new();
        let mut state = AppState::new();

        let opened = apply_mapped_intent(
            &mut controller,
            &mut state,
            &AppIntent::OpenOptionsDialogRequested,
        )
        .expect("OpenOptionsDialogRequested muss ueber die Bridge-Seam verarbeitet werden");
        assert!(opened);
        assert!(state.ui.show_options_dialog);

        let closed = apply_mapped_intent(
            &mut controller,
            &mut state,
            &AppIntent::CloseOptionsDialogRequested,
        )
        .expect("CloseOptionsDialogRequested muss ueber die Bridge-Seam verarbeitet werden");
        assert!(closed);
        assert!(!state.ui.show_options_dialog);
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
        assert!(state.ui.dialog_requests.is_empty());
    }

    #[test]
    fn apply_mapped_intent_supports_exit_path() {
        let mut controller = AppController::new();
        let mut state = AppState::new();

        let handled = apply_mapped_intent(&mut controller, &mut state, &AppIntent::ExitRequested)
            .expect("ExitRequested muss ueber die Bridge-Seam verarbeitet werden");

        assert!(handled);
        assert!(state.should_exit);
    }
}
