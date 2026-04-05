use anyhow::Result;
use fs25_auto_drive_engine::app::ui_contract::{dialog_result_to_intent, DialogRequestKind, DialogResult};
use fs25_auto_drive_engine::app::{AppController, AppIntent, AppState, EditorTool};

use crate::dto::{HostActiveTool, HostDialogRequestKind, HostDialogResult, HostSessionAction};

fn map_editor_tool(tool: HostActiveTool) -> EditorTool {
    match tool {
        HostActiveTool::Select => EditorTool::Select,
        HostActiveTool::Connect => EditorTool::Connect,
        HostActiveTool::AddNode => EditorTool::AddNode,
        HostActiveTool::Route => EditorTool::Route,
    }
}

fn map_host_dialog_request_kind(kind: HostDialogRequestKind) -> DialogRequestKind {
    match kind {
        HostDialogRequestKind::OpenFile => DialogRequestKind::OpenFile,
        HostDialogRequestKind::SaveFile => DialogRequestKind::SaveFile,
        HostDialogRequestKind::Heightmap => DialogRequestKind::Heightmap,
        HostDialogRequestKind::BackgroundMap => DialogRequestKind::BackgroundMap,
        HostDialogRequestKind::OverviewZip => DialogRequestKind::OverviewZip,
        HostDialogRequestKind::CurseplayImport => DialogRequestKind::CurseplayImport,
        HostDialogRequestKind::CurseplayExport => DialogRequestKind::CurseplayExport,
    }
}

fn map_dialog_result(result: HostDialogResult) -> DialogResult {
    match result {
        HostDialogResult::Cancelled { kind } => DialogResult::Cancelled {
            kind: map_host_dialog_request_kind(kind),
        },
        HostDialogResult::PathSelected { kind, path } => DialogResult::PathSelected {
            kind: map_host_dialog_request_kind(kind),
            path,
        },
    }
}

/// Uebersetzt eine explizite Host-Action in einen stabilen Engine-Intent.
///
/// Gibt `None` zurueck, wenn die Action keinen direkten Intent erzeugt
/// (z. B. ein abgebrochenes Dialog-Ergebnis).
pub fn map_host_action_to_intent(action: HostSessionAction) -> Option<AppIntent> {
    match action {
        HostSessionAction::OpenFile => Some(AppIntent::OpenFileRequested),
        HostSessionAction::Save => Some(AppIntent::SaveRequested),
        HostSessionAction::SaveAs => Some(AppIntent::SaveAsRequested),
        HostSessionAction::RequestHeightmapSelection => {
            Some(AppIntent::HeightmapSelectionRequested)
        }
        HostSessionAction::RequestBackgroundMapSelection => {
            Some(AppIntent::BackgroundMapSelectionRequested)
        }
        HostSessionAction::GenerateOverview => Some(AppIntent::GenerateOverviewRequested),
        HostSessionAction::CurseplayImport => Some(AppIntent::CurseplayImportRequested),
        HostSessionAction::CurseplayExport => Some(AppIntent::CurseplayExportRequested),
        HostSessionAction::ResetCamera => Some(AppIntent::ResetCameraRequested),
        HostSessionAction::ZoomToFit => Some(AppIntent::ZoomToFitRequested),
        HostSessionAction::ZoomToSelectionBounds => Some(AppIntent::ZoomToSelectionBoundsRequested),
        HostSessionAction::Exit => Some(AppIntent::ExitRequested),
        HostSessionAction::ToggleCommandPalette => Some(AppIntent::CommandPaletteToggled),
        HostSessionAction::SetEditorTool { tool } => Some(AppIntent::SetEditorToolRequested {
            tool: map_editor_tool(tool),
        }),
        HostSessionAction::OpenOptionsDialog => Some(AppIntent::OpenOptionsDialogRequested),
        HostSessionAction::CloseOptionsDialog => Some(AppIntent::CloseOptionsDialogRequested),
        HostSessionAction::Undo => Some(AppIntent::UndoRequested),
        HostSessionAction::Redo => Some(AppIntent::RedoRequested),
        HostSessionAction::SubmitDialogResult { result } => {
            dialog_result_to_intent(map_dialog_result(result))
        }
    }
}

/// Wendet eine Host-Action auf den uebergebenen Controller/State an.
///
/// Rueckgabe:
/// - `Ok(true)`: Es wurde ein Intent erzeugt und erfolgreich verarbeitet.
/// - `Ok(false)`: Die Action war semantisch ein No-Op ohne Intent.
pub fn apply_host_action(
    controller: &mut AppController,
    state: &mut AppState,
    action: HostSessionAction,
) -> Result<bool> {
    let Some(intent) = map_host_action_to_intent(action) else {
        return Ok(false);
    };

    controller.handle_intent(state, intent)?;
    Ok(true)
}

#[cfg(test)]
mod tests {
    use fs25_auto_drive_engine::app::{AppController, AppState};

    use crate::dto::{HostDialogRequestKind, HostDialogResult, HostSessionAction};

    use super::apply_host_action;

    #[test]
    fn apply_host_action_dispatches_mapped_action() {
        let mut controller = AppController::new();
        let mut state = AppState::new();

        let handled = apply_host_action(
            &mut controller,
            &mut state,
            HostSessionAction::ToggleCommandPalette,
        )
        .expect("ToggleCommandPalette muss verarbeitet werden");

        assert!(handled);
        assert!(state.ui.show_command_palette);
    }

    #[test]
    fn apply_host_action_returns_false_for_dialog_cancel_without_intent() {
        let mut controller = AppController::new();
        let mut state = AppState::new();

        let handled = apply_host_action(
            &mut controller,
            &mut state,
            HostSessionAction::SubmitDialogResult {
                result: HostDialogResult::Cancelled {
                    kind: HostDialogRequestKind::OpenFile,
                },
            },
        )
        .expect("Abgebrochene Dialoge duerfen keinen Fehler ausloesen");

        assert!(!handled);
        assert!(state.ui.dialog_requests.is_empty());
    }
}
