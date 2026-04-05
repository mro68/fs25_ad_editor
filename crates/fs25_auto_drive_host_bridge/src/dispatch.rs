use anyhow::Result;
use fs25_auto_drive_engine::app::ui_contract::{
    dialog_result_to_intent, DialogRequest, DialogRequestKind, DialogResult,
};
use fs25_auto_drive_engine::app::{AppController, AppIntent, AppState, EditorTool};

use crate::dto::{
    HostActiveTool, HostDialogRequest, HostDialogRequestKind, HostDialogResult, HostSessionAction,
};

fn map_editor_tool(tool: HostActiveTool) -> EditorTool {
    match tool {
        HostActiveTool::Select => EditorTool::Select,
        HostActiveTool::Connect => EditorTool::Connect,
        HostActiveTool::AddNode => EditorTool::AddNode,
        HostActiveTool::Route => EditorTool::Route,
    }
}

fn map_engine_dialog_request_kind(kind: DialogRequestKind) -> HostDialogRequestKind {
    match kind {
        DialogRequestKind::OpenFile => HostDialogRequestKind::OpenFile,
        DialogRequestKind::SaveFile => HostDialogRequestKind::SaveFile,
        DialogRequestKind::Heightmap => HostDialogRequestKind::Heightmap,
        DialogRequestKind::BackgroundMap => HostDialogRequestKind::BackgroundMap,
        DialogRequestKind::OverviewZip => HostDialogRequestKind::OverviewZip,
        DialogRequestKind::CurseplayImport => HostDialogRequestKind::CurseplayImport,
        DialogRequestKind::CurseplayExport => HostDialogRequestKind::CurseplayExport,
    }
}

fn map_engine_dialog_request(request: DialogRequest) -> HostDialogRequest {
    HostDialogRequest {
        kind: map_engine_dialog_request_kind(request.kind()),
        suggested_file_name: request.suggested_file_name().map(str::to_owned),
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

/// Entnimmt ausstehende Dialog-Anforderungen als Host-Bridge-DTOs.
///
/// Diese Funktion ist fuer Host-Adapter gedacht, die weiterhin einen eigenen
/// `AppController`/`AppState` besitzen, den Dialog-Lifecycle aber bereits ueber
/// die kanonischen `HostDialogRequest`-DTOs konsolidieren wollen.
pub fn take_host_dialog_requests(
    controller: &AppController,
    state: &mut AppState,
) -> Vec<HostDialogRequest> {
    controller
        .take_dialog_requests(state)
        .into_iter()
        .map(map_engine_dialog_request)
        .collect()
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

/// Wendet die gemeinsame Rust-Host-Dispatch-Seam auf Controller und State an.
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
    use fs25_auto_drive_engine::app::{AppController, AppIntent, AppState};

    use crate::dto::{HostDialogRequestKind, HostDialogResult, HostSessionAction};

    use super::{apply_host_action, take_host_dialog_requests};

    #[test]
    fn take_host_dialog_requests_maps_and_drains_engine_queue() {
        let mut controller = AppController::new();
        let mut state = AppState::new();

        controller
            .handle_intent(&mut state, AppIntent::OpenFileRequested)
            .expect("OpenFileRequested muss Dialog-Anforderung erzeugen");
        controller
            .handle_intent(&mut state, AppIntent::HeightmapSelectionRequested)
            .expect("HeightmapSelectionRequested muss Dialog-Anforderung erzeugen");

        let requests = take_host_dialog_requests(&controller, &mut state);
        assert_eq!(requests.len(), 2);
        assert_eq!(requests[0].kind, HostDialogRequestKind::OpenFile);
        assert_eq!(requests[1].kind, HostDialogRequestKind::Heightmap);

        let drained = take_host_dialog_requests(&controller, &mut state);
        assert!(drained.is_empty());
    }

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

    #[test]
    fn apply_host_action_dispatches_dialog_path_selected_into_state() {
        let mut controller = AppController::new();
        let mut state = AppState::new();
        let selected_path = "/tmp/test_heightmap.png".to_string();

        let handled = apply_host_action(
            &mut controller,
            &mut state,
            HostSessionAction::SubmitDialogResult {
                result: HostDialogResult::PathSelected {
                    kind: HostDialogRequestKind::Heightmap,
                    path: selected_path.clone(),
                },
            },
        )
        .expect("PathSelected muss einen Intent erzeugen und verarbeitet werden");

        assert!(handled);
        assert_eq!(state.ui.heightmap_path, Some(selected_path));
    }
}
