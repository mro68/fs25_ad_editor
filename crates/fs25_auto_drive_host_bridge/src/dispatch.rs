use anyhow::Result;
use fs25_auto_drive_engine::app::ui_contract::{
    dialog_result_to_intent, DialogRequest, DialogRequestKind, DialogResult, HostUiSnapshot,
    ViewportOverlaySnapshot,
};
use fs25_auto_drive_engine::app::{AppController, AppIntent, AppState, EditorTool};
use fs25_auto_drive_engine::shared::{RenderAssetsSnapshot, RenderScene};
use glam::Vec2;

use crate::dto::{
    HostActiveTool, HostDialogRequest, HostDialogRequestKind, HostDialogResult, HostSessionAction,
};

fn map_host_active_tool(tool: HostActiveTool) -> EditorTool {
    match tool {
        HostActiveTool::Select => EditorTool::Select,
        HostActiveTool::Connect => EditorTool::Connect,
        HostActiveTool::AddNode => EditorTool::AddNode,
        HostActiveTool::Route => EditorTool::Route,
    }
}

fn map_editor_tool(tool: EditorTool) -> HostActiveTool {
    match tool {
        EditorTool::Select => HostActiveTool::Select,
        EditorTool::Connect => HostActiveTool::Connect,
        EditorTool::AddNode => HostActiveTool::AddNode,
        EditorTool::Route => HostActiveTool::Route,
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

/// Mappt einen stabilen Engine-Intent auf eine explizite Host-Action.
///
/// Rueckgabewert `None` bedeutet, dass der Intent nicht zur stabilen,
/// niederfrequenten Host-Action-Surface gehoert.
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
            tool: map_editor_tool(*tool),
        }),
        AppIntent::OpenOptionsDialogRequested => Some(HostSessionAction::OpenOptionsDialog),
        AppIntent::CloseOptionsDialogRequested => Some(HostSessionAction::CloseOptionsDialog),
        AppIntent::UndoRequested => Some(HostSessionAction::Undo),
        AppIntent::RedoRequested => Some(HostSessionAction::Redo),
        _ => None,
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
            tool: map_host_active_tool(tool),
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

/// Wendet einen stabil gemappten Engine-Intent ueber die Host-Bridge-Seam an.
///
/// Rueckgabe:
/// - `Ok(true)`: Der Intent wurde auf eine Host-Action gemappt und verarbeitet.
/// - `Ok(false)`: Der Intent gehoert nicht zur stabilen Host-Action-Surface.
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

/// Baut den host-neutralen Panel-Snapshot fuer Hosts mit lokalem Controller/State.
pub fn build_host_ui_snapshot(controller: &AppController, state: &AppState) -> HostUiSnapshot {
    controller.build_host_ui_snapshot(state)
}

/// Baut den host-neutralen Viewport-Overlay-Snapshot fuer lokale Host-Adapter.
///
/// Die mutable State-Referenz bleibt noetig, weil beim Aufbau Caches im
/// `AppState` aufgewaermt werden koennen.
pub fn build_viewport_overlay_snapshot(
    controller: &AppController,
    state: &mut AppState,
    cursor_world: Option<Vec2>,
) -> ViewportOverlaySnapshot {
    controller.build_viewport_overlay_snapshot(state, cursor_world)
}

/// Baut den per-frame Render-Vertrag fuer lokale Host-Adapter.
pub fn build_render_scene(
    controller: &AppController,
    state: &AppState,
    viewport_size: [f32; 2],
) -> RenderScene {
    controller.build_render_scene(state, viewport_size)
}

/// Baut den langlebigen Render-Asset-Snapshot fuer lokale Host-Adapter.
pub fn build_render_assets(controller: &AppController, state: &AppState) -> RenderAssetsSnapshot {
    controller.build_render_assets(state)
}

#[cfg(test)]
mod tests {
    use fs25_auto_drive_engine::app::{AppController, AppIntent, AppState};
    use glam::Vec2;

    use crate::dto::{HostActiveTool, HostDialogRequestKind, HostDialogResult, HostSessionAction};

    use super::{
        apply_host_action, apply_mapped_intent, build_host_ui_snapshot, build_render_assets,
        build_render_scene, build_viewport_overlay_snapshot, map_host_action_to_intent,
        map_intent_to_host_action, take_host_dialog_requests,
    };

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
    fn take_host_dialog_requests_covers_save_background_and_curseplay_export_requests() {
        let mut controller = AppController::new();
        let mut state = AppState::new();

        controller
            .handle_intent(&mut state, AppIntent::SaveAsRequested)
            .expect("SaveAsRequested muss Dialog-Anforderung erzeugen");
        controller
            .handle_intent(&mut state, AppIntent::BackgroundMapSelectionRequested)
            .expect("BackgroundMapSelectionRequested muss Dialog-Anforderung erzeugen");
        controller
            .handle_intent(&mut state, AppIntent::CurseplayExportRequested)
            .expect("CurseplayExportRequested muss Dialog-Anforderung erzeugen");

        let requests = take_host_dialog_requests(&controller, &mut state);
        assert_eq!(requests.len(), 3);
        assert_eq!(requests[0].kind, HostDialogRequestKind::SaveFile);
        assert_eq!(requests[1].kind, HostDialogRequestKind::BackgroundMap);
        assert_eq!(requests[2].kind, HostDialogRequestKind::CurseplayExport);
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

    #[test]
    fn map_host_action_to_intent_covers_new_dialog_result_branches() {
        let save_intent = map_host_action_to_intent(HostSessionAction::SubmitDialogResult {
            result: HostDialogResult::PathSelected {
                kind: HostDialogRequestKind::SaveFile,
                path: "/tmp/savegame.xml".to_string(),
            },
        });
        let background_zip_intent =
            map_host_action_to_intent(HostSessionAction::SubmitDialogResult {
                result: HostDialogResult::PathSelected {
                    kind: HostDialogRequestKind::BackgroundMap,
                    path: "/tmp/map_overview.zip".to_string(),
                },
            });
        let curseplay_export_intent =
            map_host_action_to_intent(HostSessionAction::SubmitDialogResult {
                result: HostDialogResult::PathSelected {
                    kind: HostDialogRequestKind::CurseplayExport,
                    path: "/tmp/customField.xml".to_string(),
                },
            });

        assert!(matches!(
            save_intent,
            Some(AppIntent::SaveFilePathSelected { path }) if path == "/tmp/savegame.xml"
        ));
        assert!(matches!(
            background_zip_intent,
            Some(AppIntent::ZipBackgroundBrowseRequested { path }) if path == "/tmp/map_overview.zip"
        ));
        assert!(matches!(
            curseplay_export_intent,
            Some(AppIntent::CurseplayExportPathSelected { path }) if path == "/tmp/customField.xml"
        ));
    }

    #[test]
    fn map_intent_to_host_action_covers_stable_bridge_intents() {
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
                    tool: fs25_auto_drive_engine::app::EditorTool::Route,
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
    fn map_intent_to_host_action_keeps_high_frequency_intents_unmapped() {
        let cases = vec![
            AppIntent::ViewportResized {
                size: [1920.0, 1080.0],
            },
            AppIntent::CameraPan {
                delta: Vec2::new(3.0, -2.0),
            },
            AppIntent::CameraZoom {
                factor: 1.1,
                focus_world: Some(Vec2::ZERO),
            },
            AppIntent::NodePickRequested {
                world_pos: Vec2::new(5.0, 6.0),
                additive: false,
                extend_path: false,
            },
            AppIntent::RouteToolClicked {
                world_pos: Vec2::new(1.0, 2.0),
                ctrl: false,
            },
        ];

        for intent in cases {
            assert!(map_intent_to_host_action(&intent).is_none());
        }
    }

    #[test]
    fn apply_mapped_intent_dispatches_open_file_request() {
        let mut controller = AppController::new();
        let mut state = AppState::new();

        let handled =
            apply_mapped_intent(&mut controller, &mut state, &AppIntent::OpenFileRequested)
                .expect("OpenFileRequested muss ueber die Bridge-Seam verarbeitet werden");

        assert!(handled);
        assert_eq!(state.ui.dialog_requests.len(), 1);
    }

    #[test]
    fn apply_mapped_intent_returns_false_for_unmapped_intents() {
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

    #[test]
    fn read_helpers_delegate_to_controller_read_seams() {
        let controller = AppController::new();
        let mut state = AppState::new();

        let host_ui = build_host_ui_snapshot(&controller, &state);
        let overlay = build_viewport_overlay_snapshot(&controller, &mut state, None);
        let scene = build_render_scene(&controller, &state, [640.0, 480.0]);
        let assets = build_render_assets(&controller, &state);

        assert!(host_ui.command_palette_state().is_some());
        assert!(overlay.route_tool_preview.is_none());
        assert_eq!(scene.viewport_size(), [640.0, 480.0]);
        assert_eq!(assets.background_asset_revision(), 0);
    }
}
