use fs25_auto_drive_engine::app::ui_contract::{BypassPanelAction, RouteToolPanelAction};
use fs25_auto_drive_engine::app::use_cases::background_layers::{
    discover_background_layer_files, load_background_layer_catalog,
};
use fs25_auto_drive_engine::app::{AppController, AppIntent, AppState};
use fs25_auto_drive_engine::core::{
    Connection, ConnectionDirection, ConnectionPriority, MapMarker, MapNode, NodeFlag, RoadMap,
};
use fs25_auto_drive_engine::shared::{BackgroundLayerKind, OverviewLayerOptions, RenderQuality};
use glam::Vec2;
use std::fs;
use std::path::PathBuf;
use std::sync::Arc;
use std::time::{SystemTime, UNIX_EPOCH};

use crate::dto::{
    HostActiveTool, HostBackgroundLayerKind, HostDefaultConnectionDirection,
    HostDefaultConnectionPriority, HostDialogRequestKind, HostDialogResult, HostRouteToolAction,
    HostRouteToolDisabledReason, HostRouteToolGroup, HostRouteToolIconKey, HostRouteToolId,
    HostRouteToolSurface, HostSessionAction, HostTangentSource, HostViewportConnectionDirection,
    HostViewportConnectionPriority, HostViewportNodeKind,
};

use super::{
    apply_host_action, apply_mapped_intent, build_host_chrome_snapshot, build_host_ui_snapshot,
    build_render_assets, build_render_frame, build_render_scene,
    build_route_tool_viewport_snapshot, build_viewport_geometry_snapshot,
    build_viewport_overlay_snapshot, map_host_action_to_intent, map_intent_to_host_action,
    take_host_dialog_requests,
};

fn geometry_test_map() -> RoadMap {
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
    map.add_map_marker(MapMarker::new(
        1,
        "Hof".to_string(),
        "Farmen".to_string(),
        1,
        false,
    ));
    map.ensure_spatial_index();
    map
}

fn geometry_sorting_test_map() -> RoadMap {
    let mut map = RoadMap::new(3);
    map.add_node(MapNode::new(3, Vec2::new(20.0, 10.0), NodeFlag::Regular));
    map.add_node(MapNode::new(1, Vec2::new(0.0, 0.0), NodeFlag::Warning));
    map.add_node(MapNode::new(2, Vec2::new(10.0, 5.0), NodeFlag::SubPrio));
    map.add_connection(Connection::new(
        2,
        1,
        ConnectionDirection::Regular,
        ConnectionPriority::Regular,
        Vec2::new(10.0, 5.0),
        Vec2::new(0.0, 0.0),
    ));
    map.add_connection(Connection::new(
        1,
        3,
        ConnectionDirection::Reverse,
        ConnectionPriority::SubPriority,
        Vec2::new(0.0, 0.0),
        Vec2::new(20.0, 10.0),
    ));
    map.add_map_marker(MapMarker::new(
        2,
        "M2".to_string(),
        "M2".to_string(),
        3,
        false,
    ));
    map.add_map_marker(MapMarker::new(
        3,
        "M3".to_string(),
        "M3".to_string(),
        1,
        false,
    ));
    map.add_map_marker(MapMarker::new(
        1,
        "M1".to_string(),
        "M1".to_string(),
        2,
        false,
    ));
    map.ensure_spatial_index();
    map
}

struct TempDirGuard {
    path: PathBuf,
}

impl TempDirGuard {
    fn new(prefix: &str) -> Self {
        let timestamp = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .expect("Systemzeit muss nach der Unix-Epoche liegen")
            .as_nanos();
        let path = std::env::temp_dir().join(format!(
            "{}_{}_{}",
            prefix,
            std::process::id(),
            timestamp
        ));
        fs::create_dir_all(&path).expect("Temp-Verzeichnis fuer Test muss erstellt werden");
        Self { path }
    }

    fn path(&self) -> &std::path::Path {
        &self.path
    }
}

impl Drop for TempDirGuard {
    fn drop(&mut self) {
        let _ = fs::remove_dir_all(&self.path);
    }
}

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
    assert!(
        state.ui.dialog_requests.iter().any(|r| matches!(
            r,
            fs25_auto_drive_engine::app::ui_contract::DialogRequest::ToggleCommandPalette
        )),
        "ToggleCommandPalette muss in dialog_requests stehen"
    );
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
fn build_viewport_geometry_snapshot_exposes_full_geometry_transport() {
    let mut state = AppState::new();
    state.road_map = Some(Arc::new(geometry_test_map()));
    state.view.camera.position = Vec2::new(3.0, -4.0);
    state.view.camera.zoom = 2.0;
    state.selection.ids_mut().insert(2);

    let snapshot = build_viewport_geometry_snapshot(&state, [640.0, 320.0]);

    assert!(snapshot.has_map);
    assert_eq!(snapshot.viewport_size, [640.0, 320.0]);
    assert_eq!(snapshot.camera_position, [3.0, -4.0]);
    assert_eq!(snapshot.zoom, 2.0);
    assert_eq!(snapshot.nodes.len(), 2);
    assert_eq!(snapshot.connections.len(), 1);
    assert_eq!(snapshot.markers.len(), 1);
    assert_eq!(snapshot.nodes[0].id, 1);
    assert_eq!(snapshot.nodes[0].kind, HostViewportNodeKind::Regular);
    assert!(!snapshot.nodes[0].selected);
    assert_eq!(snapshot.nodes[1].id, 2);
    assert_eq!(snapshot.nodes[1].kind, HostViewportNodeKind::SubPrio);
    assert!(snapshot.nodes[1].selected);
    assert_eq!(
        snapshot.connections[0].direction,
        HostViewportConnectionDirection::Dual
    );
    assert_eq!(
        snapshot.connections[0].priority,
        HostViewportConnectionPriority::SubPriority
    );
    assert_eq!(snapshot.markers[0].position, [0.0, 0.0]);
    assert!(snapshot.world_per_pixel.is_finite());
    assert!(snapshot.world_per_pixel > 0.0);
}

#[test]
fn build_viewport_geometry_snapshot_sorts_and_keeps_full_geometry_lists() {
    let mut state = AppState::new();
    state.road_map = Some(Arc::new(geometry_sorting_test_map()));
    state.view.camera.position = Vec2::new(10.0, 5.0);
    state.view.camera.zoom = 2.0;

    let snapshot = build_viewport_geometry_snapshot(&state, [800.0, 600.0]);

    assert_eq!(snapshot.nodes.len(), 3);
    assert_eq!(snapshot.connections.len(), 2);
    assert_eq!(snapshot.markers.len(), 3);

    assert_eq!(
        snapshot
            .nodes
            .iter()
            .map(|node| node.id)
            .collect::<Vec<_>>(),
        [1, 2, 3]
    );
    assert_eq!(
        snapshot
            .connections
            .iter()
            .map(|connection| (connection.start_id, connection.end_id))
            .collect::<Vec<_>>(),
        [(1, 3), (2, 1)]
    );
    assert_eq!(
        snapshot
            .markers
            .iter()
            .map(|marker| marker.position)
            .collect::<Vec<_>>(),
        [[0.0, 0.0], [10.0, 5.0], [20.0, 10.0]]
    );
}

#[test]
fn build_render_frame_couples_scene_and_assets_for_local_hosts() {
    let mut state = AppState::new();
    state.road_map = Some(Arc::new(geometry_test_map()));

    let frame = build_render_frame(&state, [640.0, 320.0]);

    assert!(frame.scene.has_map());
    assert_eq!(frame.scene.viewport_size(), [640.0, 320.0]);
    assert_eq!(frame.assets.background_asset_revision(), 0);
    assert_eq!(frame.assets.background_transform_revision(), 0);
    assert!(frame.assets.background().is_none());
}

#[test]
fn map_host_action_to_intent_covers_new_dialog_result_branches() {
    let save_intent = map_host_action_to_intent(HostSessionAction::SubmitDialogResult {
        result: HostDialogResult::PathSelected {
            kind: HostDialogRequestKind::SaveFile,
            path: "/tmp/savegame.xml".to_string(),
        },
    });
    let background_zip_intent = map_host_action_to_intent(HostSessionAction::SubmitDialogResult {
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
fn map_host_action_to_intent_covers_route_tool_and_chrome_writes() {
    let route_intent = map_host_action_to_intent(HostSessionAction::RouteTool {
        action: HostRouteToolAction::ScrollRotate { delta: -1.0 },
    });
    let default_direction_intent =
        map_host_action_to_intent(HostSessionAction::SetDefaultDirection {
            direction: HostDefaultConnectionDirection::Reverse,
        });
    let default_priority_intent =
        map_host_action_to_intent(HostSessionAction::SetDefaultPriority {
            priority: HostDefaultConnectionPriority::SubPriority,
        });

    assert!(matches!(
        route_intent,
        Some(AppIntent::RouteToolScrollRotated { delta }) if (delta + 1.0).abs() < f32::EPSILON
    ));
    assert!(matches!(
        default_direction_intent,
        Some(AppIntent::SetDefaultDirectionRequested {
            direction: ConnectionDirection::Reverse
        })
    ));
    assert!(matches!(
        default_priority_intent,
        Some(AppIntent::SetDefaultPriorityRequested {
            priority: ConnectionPriority::SubPriority
        })
    ));
}

#[test]
fn map_host_action_to_intent_covers_selection_and_clipboard_writes() {
    assert!(matches!(
        map_host_action_to_intent(HostSessionAction::DeleteSelected),
        Some(AppIntent::DeleteSelectedRequested)
    ));
    assert!(matches!(
        map_host_action_to_intent(HostSessionAction::SelectAll),
        Some(AppIntent::SelectAllRequested)
    ));
    assert!(matches!(
        map_host_action_to_intent(HostSessionAction::ClearSelection),
        Some(AppIntent::ClearSelectionRequested)
    ));
    assert!(matches!(
        map_host_action_to_intent(HostSessionAction::CopySelection),
        Some(AppIntent::CopySelectionRequested)
    ));
    assert!(matches!(
        map_host_action_to_intent(HostSessionAction::PasteStart),
        Some(AppIntent::PasteStartRequested)
    ));
    assert!(matches!(
        map_host_action_to_intent(HostSessionAction::PasteConfirm),
        Some(AppIntent::PasteConfirmRequested)
    ));
    assert!(matches!(
        map_host_action_to_intent(HostSessionAction::PasteCancel),
        Some(AppIntent::PasteCancelled)
    ));
}

#[test]
fn map_host_action_to_intent_covers_egui_parity_gap_actions() {
    assert!(matches!(
        map_host_action_to_intent(HostSessionAction::ClearHeightmap),
        Some(AppIntent::HeightmapCleared)
    ));
    assert!(matches!(
        map_host_action_to_intent(HostSessionAction::BrowseOverviewZip),
        Some(AppIntent::OverviewZipBrowseRequested)
    ));
    assert!(matches!(
        map_host_action_to_intent(HostSessionAction::GenerateOverviewFromZip {
            path: "/tmp/overview.zip".to_string(),
        }),
        Some(AppIntent::GenerateOverviewFromZip { path }) if path == "/tmp/overview.zip"
    ));
    assert!(matches!(
        map_host_action_to_intent(HostSessionAction::CenterOnNode { node_id: 42 }),
        Some(AppIntent::CenterOnNodeRequested { node_id: 42 })
    ));
    assert!(matches!(
        map_host_action_to_intent(HostSessionAction::SetRenderQuality {
            quality: RenderQuality::Medium,
        }),
        Some(AppIntent::RenderQualityChanged {
            quality: RenderQuality::Medium,
        })
    ));
    assert!(matches!(
        map_host_action_to_intent(HostSessionAction::OpenCreateMarkerDialog { node_id: 7 }),
        Some(AppIntent::CreateMarkerRequested { node_id: 7 })
    ));
    assert!(matches!(
        map_host_action_to_intent(HostSessionAction::CancelMarkerDialog),
        Some(AppIntent::MarkerDialogCancelled)
    ));
    assert!(matches!(
        map_host_action_to_intent(HostSessionAction::StartResampleSelection),
        Some(AppIntent::StreckenteilungAktivieren)
    ));
    assert!(matches!(
        map_host_action_to_intent(HostSessionAction::ApplyCurrentResample),
        Some(AppIntent::ResamplePathRequested)
    ));
    assert!(matches!(
        map_host_action_to_intent(HostSessionAction::StartGroupEdit { record_id: 9 }),
        Some(AppIntent::GroupEditStartRequested { record_id: 9 })
    ));
    assert!(matches!(
        map_host_action_to_intent(HostSessionAction::SetGroupBoundaryNodes {
            record_id: 5,
            entry_node_id: Some(11),
            exit_node_id: None,
        }),
        Some(AppIntent::SetGroupBoundaryNodes {
            record_id: 5,
            entry_node_id: Some(11),
            exit_node_id: None,
        })
    ));
    assert!(matches!(
        map_host_action_to_intent(HostSessionAction::RecomputeNodeSegmentSelection {
            world_pos: [1.0, -2.0],
            additive: true,
        }),
        Some(AppIntent::NodeSegmentBetweenIntersectionsRequested { world_pos, additive })
            if world_pos == Vec2::new(1.0, -2.0) && additive
    ));
    assert!(matches!(
        map_host_action_to_intent(HostSessionAction::ConfirmDissolveGroup { segment_id: 13 }),
        Some(AppIntent::DissolveGroupConfirmed { segment_id: 13 })
    ));
    assert!(matches!(
        map_host_action_to_intent(HostSessionAction::ConfirmTraceAllFields {
            spacing: 4.0,
            offset: 0.5,
            tolerance: 1.0,
            corner_angle: Some(30.0),
            corner_rounding_radius: Some(2.0),
            corner_rounding_max_angle_deg: None,
        }),
        Some(AppIntent::TraceAllFieldsConfirmed {
            spacing: 4.0,
            offset: 0.5,
            tolerance: 1.0,
            corner_angle: Some(30.0),
            corner_rounding_radius: Some(2.0),
            corner_rounding_max_angle_deg: None,
        })
    ));
    assert!(matches!(
        map_host_action_to_intent(HostSessionAction::ConfirmDeduplication),
        Some(AppIntent::DeduplicateConfirmed)
    ));
}

#[test]
fn map_connection_actions_and_intents_roundtrip_bidirectionally() {
    let add_intent = AppIntent::AddConnectionRequested {
        from_id: 11,
        to_id: 12,
        direction: ConnectionDirection::Dual,
        priority: ConnectionPriority::SubPriority,
    };
    let remove_intent = AppIntent::RemoveConnectionBetweenRequested {
        node_a: 21,
        node_b: 22,
    };

    let add_action = map_intent_to_host_action(&add_intent)
        .expect("AddConnectionRequested muss auf HostSessionAction gemappt werden");
    let remove_action = map_intent_to_host_action(&remove_intent)
        .expect("RemoveConnectionBetweenRequested muss auf HostSessionAction gemappt werden");

    assert_eq!(
        add_action,
        HostSessionAction::AddConnection {
            from_id: 11,
            to_id: 12,
            direction: HostDefaultConnectionDirection::Dual,
            priority: HostDefaultConnectionPriority::SubPriority,
        }
    );
    assert_eq!(
        remove_action,
        HostSessionAction::RemoveConnectionBetween {
            node_a: 21,
            node_b: 22,
        }
    );
    assert!(matches!(
        map_host_action_to_intent(add_action),
        Some(AppIntent::AddConnectionRequested {
            from_id: 11,
            to_id: 12,
            direction: ConnectionDirection::Dual,
            priority: ConnectionPriority::SubPriority,
        })
    ));
    assert!(matches!(
        map_host_action_to_intent(remove_action),
        Some(AppIntent::RemoveConnectionBetweenRequested {
            node_a: 21,
            node_b: 22,
        })
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
            AppIntent::SetDefaultDirectionRequested {
                direction: ConnectionDirection::Reverse,
            },
            HostSessionAction::SetDefaultDirection {
                direction: HostDefaultConnectionDirection::Reverse,
            },
        ),
        (
            AppIntent::SetDefaultPriorityRequested {
                priority: ConnectionPriority::SubPriority,
            },
            HostSessionAction::SetDefaultPriority {
                priority: HostDefaultConnectionPriority::SubPriority,
            },
        ),
        (
            AppIntent::AddConnectionRequested {
                from_id: 1,
                to_id: 2,
                direction: ConnectionDirection::Dual,
                priority: ConnectionPriority::SubPriority,
            },
            HostSessionAction::AddConnection {
                from_id: 1,
                to_id: 2,
                direction: HostDefaultConnectionDirection::Dual,
                priority: HostDefaultConnectionPriority::SubPriority,
            },
        ),
        (
            AppIntent::RemoveConnectionBetweenRequested {
                node_a: 2,
                node_b: 3,
            },
            HostSessionAction::RemoveConnectionBetween {
                node_a: 2,
                node_b: 3,
            },
        ),
        (
            AppIntent::SetConnectionDirectionRequested {
                start_id: 3,
                end_id: 4,
                direction: ConnectionDirection::Reverse,
            },
            HostSessionAction::SetConnectionDirection {
                start_id: 3,
                end_id: 4,
                direction: HostDefaultConnectionDirection::Reverse,
            },
        ),
        (
            AppIntent::SetConnectionPriorityRequested {
                start_id: 4,
                end_id: 5,
                priority: ConnectionPriority::Regular,
            },
            HostSessionAction::SetConnectionPriority {
                start_id: 4,
                end_id: 5,
                priority: HostDefaultConnectionPriority::Regular,
            },
        ),
        (
            AppIntent::SetAllConnectionsDirectionBetweenSelectedRequested {
                direction: ConnectionDirection::Dual,
            },
            HostSessionAction::SetAllConnectionsDirectionBetweenSelected {
                direction: HostDefaultConnectionDirection::Dual,
            },
        ),
        (
            AppIntent::RemoveAllConnectionsBetweenSelectedRequested,
            HostSessionAction::RemoveAllConnectionsBetweenSelected,
        ),
        (
            AppIntent::InvertAllConnectionsBetweenSelectedRequested,
            HostSessionAction::InvertAllConnectionsBetweenSelected,
        ),
        (
            AppIntent::SetAllConnectionsPriorityBetweenSelectedRequested {
                priority: ConnectionPriority::SubPriority,
            },
            HostSessionAction::SetAllConnectionsPriorityBetweenSelected {
                priority: HostDefaultConnectionPriority::SubPriority,
            },
        ),
        (
            AppIntent::ConnectSelectedNodesRequested,
            HostSessionAction::ConnectSelectedNodes,
        ),
        (
            AppIntent::OptionsChanged {
                options: Box::new(fs25_auto_drive_engine::shared::EditorOptions::default()),
            },
            HostSessionAction::ApplyOptions {
                options: Box::new(fs25_auto_drive_engine::shared::EditorOptions::default()),
            },
        ),
        (
            AppIntent::ResetOptionsRequested,
            HostSessionAction::ResetOptions,
        ),
        (
            AppIntent::SelectRouteToolRequested {
                tool_id: fs25_auto_drive_engine::app::tool_contract::RouteToolId::CurveCubic,
            },
            HostSessionAction::RouteTool {
                action: HostRouteToolAction::SelectTool {
                    tool: HostRouteToolId::CurveCubic,
                },
            },
        ),
        (
            AppIntent::RouteToolPanelActionRequested {
                action: RouteToolPanelAction::Bypass(BypassPanelAction::SetOffset(2.5)),
            },
            HostSessionAction::RouteTool {
                action: HostRouteToolAction::PanelAction {
                    action: RouteToolPanelAction::Bypass(BypassPanelAction::SetOffset(2.5)),
                },
            },
        ),
        (
            AppIntent::RouteToolTangentSelected {
                start: fs25_auto_drive_engine::app::tool_contract::TangentSource::Connection {
                    neighbor_id: 7,
                    angle: 0.5,
                },
                end: fs25_auto_drive_engine::app::tool_contract::TangentSource::None,
            },
            HostSessionAction::RouteTool {
                action: HostRouteToolAction::ApplyTangent {
                    start: HostTangentSource::Connection {
                        neighbor_id: 7,
                        angle: 0.5,
                    },
                    end: HostTangentSource::None,
                },
            },
        ),
        (
            AppIntent::RouteToolScrollRotated { delta: 1.0 },
            HostSessionAction::RouteTool {
                action: HostRouteToolAction::ScrollRotate { delta: 1.0 },
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
        (
            AppIntent::DeleteSelectedRequested,
            HostSessionAction::DeleteSelected,
        ),
        (AppIntent::SelectAllRequested, HostSessionAction::SelectAll),
        (
            AppIntent::ClearSelectionRequested,
            HostSessionAction::ClearSelection,
        ),
        (
            AppIntent::CopySelectionRequested,
            HostSessionAction::CopySelection,
        ),
        (
            AppIntent::PasteStartRequested,
            HostSessionAction::PasteStart,
        ),
        (
            AppIntent::PasteConfirmRequested,
            HostSessionAction::PasteConfirm,
        ),
        (AppIntent::PasteCancelled, HostSessionAction::PasteCancel),
    ];

    for (intent, expected_action) in cases {
        assert_eq!(map_intent_to_host_action(&intent), Some(expected_action));
    }
}

#[test]
fn map_intent_to_host_action_covers_missing_egui_parity_intents() {
    let cases = vec![
        (
            AppIntent::HeightmapCleared,
            HostSessionAction::ClearHeightmap,
        ),
        (
            AppIntent::HeightmapWarningConfirmed,
            HostSessionAction::ConfirmHeightmapWarning,
        ),
        (
            AppIntent::HeightmapWarningCancelled,
            HostSessionAction::CancelHeightmapWarning,
        ),
        (
            AppIntent::OverviewZipBrowseRequested,
            HostSessionAction::BrowseOverviewZip,
        ),
        (
            AppIntent::GenerateOverviewFromZip {
                path: "/tmp/overview.zip".to_string(),
            },
            HostSessionAction::GenerateOverviewFromZip {
                path: "/tmp/overview.zip".to_string(),
            },
        ),
        (
            AppIntent::ZipBackgroundFileSelected {
                zip_path: "/tmp/background.zip".to_string(),
                entry_name: "overview.png".to_string(),
            },
            HostSessionAction::SelectZipBackgroundFile {
                zip_path: "/tmp/background.zip".to_string(),
                entry_name: "overview.png".to_string(),
            },
        ),
        (
            AppIntent::ZipBrowserCancelled,
            HostSessionAction::CancelZipBrowser,
        ),
        (
            AppIntent::OverviewOptionsConfirmed,
            HostSessionAction::ConfirmOverviewOptions,
        ),
        (
            AppIntent::OverviewOptionsCancelled,
            HostSessionAction::CancelOverviewOptions,
        ),
        (
            AppIntent::PostLoadDialogDismissed,
            HostSessionAction::DismissPostLoadDialog,
        ),
        (
            AppIntent::SaveBackgroundAsOverviewConfirmed,
            HostSessionAction::ConfirmSaveBackgroundAsOverview,
        ),
        (
            AppIntent::SaveBackgroundAsOverviewDismissed,
            HostSessionAction::DismissSaveBackgroundAsOverview,
        ),
        (
            AppIntent::DeduplicateConfirmed,
            HostSessionAction::ConfirmDeduplication,
        ),
        (
            AppIntent::DeduplicateCancelled,
            HostSessionAction::CancelDeduplication,
        ),
        (AppIntent::ZoomInRequested, HostSessionAction::ZoomIn),
        (AppIntent::ZoomOutRequested, HostSessionAction::ZoomOut),
        (
            AppIntent::CenterOnNodeRequested { node_id: 17 },
            HostSessionAction::CenterOnNode { node_id: 17 },
        ),
        (
            AppIntent::RenderQualityChanged {
                quality: RenderQuality::Low,
            },
            HostSessionAction::SetRenderQuality {
                quality: RenderQuality::Low,
            },
        ),
        (
            AppIntent::ToggleBackgroundVisibility,
            HostSessionAction::ToggleBackgroundVisibility,
        ),
        (
            AppIntent::SetBackgroundLayerVisibility {
                layer: BackgroundLayerKind::Legend,
                visible: false,
            },
            HostSessionAction::SetBackgroundLayerVisibility {
                layer: HostBackgroundLayerKind::Legend,
                visible: false,
            },
        ),
        (
            AppIntent::ScaleBackground { factor: 0.5 },
            HostSessionAction::ScaleBackground { factor: 0.5 },
        ),
        (
            AppIntent::CreateMarkerRequested { node_id: 21 },
            HostSessionAction::OpenCreateMarkerDialog { node_id: 21 },
        ),
        (
            AppIntent::EditMarkerRequested { node_id: 22 },
            HostSessionAction::OpenEditMarkerDialog { node_id: 22 },
        ),
        (
            AppIntent::MarkerDialogCancelled,
            HostSessionAction::CancelMarkerDialog,
        ),
        (
            AppIntent::InvertSelectionRequested,
            HostSessionAction::InvertSelection,
        ),
        (
            AppIntent::StreckenteilungAktivieren,
            HostSessionAction::StartResampleSelection,
        ),
        (
            AppIntent::ResamplePathRequested,
            HostSessionAction::ApplyCurrentResample,
        ),
        (
            AppIntent::GroupEditStartRequested { record_id: 3 },
            HostSessionAction::StartGroupEdit { record_id: 3 },
        ),
        (
            AppIntent::GroupEditApplyRequested,
            HostSessionAction::ApplyGroupEdit,
        ),
        (
            AppIntent::GroupEditCancelRequested,
            HostSessionAction::CancelGroupEdit,
        ),
        (
            AppIntent::GroupEditToolRequested { record_id: 4 },
            HostSessionAction::OpenGroupEditTool { record_id: 4 },
        ),
        (
            AppIntent::GroupSelectionAsGroupRequested,
            HostSessionAction::GroupSelectionAsGroup,
        ),
        (
            AppIntent::RemoveSelectedNodesFromGroupRequested,
            HostSessionAction::RemoveSelectedNodesFromGroup,
        ),
        (
            AppIntent::SetGroupBoundaryNodes {
                record_id: 5,
                entry_node_id: Some(10),
                exit_node_id: None,
            },
            HostSessionAction::SetGroupBoundaryNodes {
                record_id: 5,
                entry_node_id: Some(10),
                exit_node_id: None,
            },
        ),
        (
            AppIntent::NodeSegmentBetweenIntersectionsRequested {
                world_pos: Vec2::new(3.0, 4.0),
                additive: true,
            },
            HostSessionAction::RecomputeNodeSegmentSelection {
                world_pos: [3.0, 4.0],
                additive: true,
            },
        ),
        (
            AppIntent::ToggleGroupLockRequested { segment_id: 6 },
            HostSessionAction::ToggleGroupLock { segment_id: 6 },
        ),
        (
            AppIntent::DissolveGroupRequested { segment_id: 7 },
            HostSessionAction::DissolveGroup { segment_id: 7 },
        ),
        (
            AppIntent::DissolveGroupConfirmed { segment_id: 8 },
            HostSessionAction::ConfirmDissolveGroup { segment_id: 8 },
        ),
        (
            AppIntent::OpenTraceAllFieldsDialogRequested,
            HostSessionAction::OpenTraceAllFieldsDialog,
        ),
        (
            AppIntent::TraceAllFieldsConfirmed {
                spacing: 6.0,
                offset: 0.5,
                tolerance: 1.5,
                corner_angle: Some(25.0),
                corner_rounding_radius: None,
                corner_rounding_max_angle_deg: Some(45.0),
            },
            HostSessionAction::ConfirmTraceAllFields {
                spacing: 6.0,
                offset: 0.5,
                tolerance: 1.5,
                corner_angle: Some(25.0),
                corner_rounding_radius: None,
                corner_rounding_max_angle_deg: Some(45.0),
            },
        ),
        (
            AppIntent::TraceAllFieldsCancelled,
            HostSessionAction::CancelTraceAllFields,
        ),
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
        AppIntent::AddNodeRequested {
            world_pos: Vec2::new(9.0, 1.0),
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

    let handled = apply_mapped_intent(&mut controller, &mut state, &AppIntent::OpenFileRequested)
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
    let mut state = AppState::new();

    let host_ui = build_host_ui_snapshot(&state);
    let overlay = build_viewport_overlay_snapshot(&mut state, None);
    let scene = build_render_scene(&state, [640.0, 480.0]);
    let assets = build_render_assets(&state);

    assert!(host_ui.command_palette_state().is_some());
    assert!(overlay.route_tool_preview.is_none());
    assert_eq!(scene.viewport_size(), [640.0, 480.0]);
    assert_eq!(assets.background_asset_revision(), 0);
}

#[test]
fn build_route_tool_viewport_snapshot_exposes_straight_tool_flags() {
    let road_map = RoadMap::default();
    let mut state = AppState::new();

    state.editor.active_tool = fs25_auto_drive_engine::app::EditorTool::Route;
    state
        .editor
        .tool_manager
        .set_active_by_id(fs25_auto_drive_engine::app::tool_contract::RouteToolId::Straight);
    state
        .editor
        .tool_manager
        .active_tool_mut()
        .expect("Straight-Tool muss fuer den Snapshot-Test aktiv sein")
        .on_click(Vec2::new(0.0, 0.0), &road_map, false);

    let snapshot = build_route_tool_viewport_snapshot(&state);

    assert!(snapshot.has_pending_input);
    assert!(snapshot.drag_targets.is_empty());
    assert!(snapshot.segment_shortcuts_active);
    assert!(snapshot.tangent_menu_data.is_none());
    assert!(!snapshot.needs_lasso_input);
}

#[test]
fn build_host_chrome_snapshot_exposes_status_defaults_and_route_tool_entries() {
    let mut state = AppState::new();
    state.ui.status_message = Some("bereit".to_string());
    state.editor.active_tool = fs25_auto_drive_engine::app::EditorTool::Route;
    state
        .editor
        .tool_manager
        .set_active_by_id(fs25_auto_drive_engine::app::tool_contract::RouteToolId::CurveCubic);
    state.editor.default_direction = ConnectionDirection::Dual;
    state.editor.default_priority = ConnectionPriority::SubPriority;

    let chrome = build_host_chrome_snapshot(&state);

    assert_eq!(chrome.status_message.as_deref(), Some("bereit"));
    // show_command_palette stammt aus HostLocalDialogState (Session-Schicht), die freie Funktion liefert immer false
    assert!(!chrome.show_command_palette);
    assert_eq!(chrome.active_tool, HostActiveTool::Route);
    assert_eq!(chrome.active_route_tool, Some(HostRouteToolId::CurveCubic));
    assert_eq!(
        chrome.default_direction,
        HostDefaultConnectionDirection::Dual
    );
    assert_eq!(
        chrome.default_priority,
        HostDefaultConnectionPriority::SubPriority
    );

    let defaults_entry = chrome
        .route_tool_entries
        .iter()
        .find(|entry| {
            entry.surface == HostRouteToolSurface::DefaultsPanel
                && entry.group == HostRouteToolGroup::Basics
                && entry.tool == HostRouteToolId::CurveCubic
        })
        .expect("Defaults-Panel muss Cubic-Tool-Eintrag enthalten");
    assert!(defaults_entry.enabled);
    assert_eq!(defaults_entry.icon_key, HostRouteToolIconKey::CurveCubic);
    assert!(defaults_entry.disabled_reason.is_none());

    let disabled_analysis_entry = chrome
        .route_tool_entries
        .iter()
        .find(|entry| {
            entry.surface == HostRouteToolSurface::MainMenu
                && entry.group == HostRouteToolGroup::Analysis
                && entry.tool == HostRouteToolId::FieldBoundary
        })
        .expect("MainMenu muss Analysis-Eintrag enthalten");
    assert!(!disabled_analysis_entry.enabled);
    assert_eq!(
        disabled_analysis_entry.disabled_reason,
        Some(HostRouteToolDisabledReason::MissingFarmland)
    );
    assert!(!chrome.background_layers_available);
    assert!(chrome.background_layer_entries.is_empty());
}

#[test]
fn build_host_chrome_snapshot_exposes_background_layer_entries() {
    // 1x1 RGBA PNG (valide Testdatei), damit der Test ohne externe Fixtures laeuft.
    const TEST_PNG_1X1_RGBA: &[u8] = &[
        0x89, 0x50, 0x4E, 0x47, 0x0D, 0x0A, 0x1A, 0x0A, 0x00, 0x00, 0x00, 0x0D, 0x49, 0x48,
        0x44, 0x52, 0x00, 0x00, 0x00, 0x01, 0x00, 0x00, 0x00, 0x01, 0x08, 0x06, 0x00, 0x00,
        0x00, 0x1F, 0x15, 0xC4, 0x89, 0x00, 0x00, 0x00, 0x0A, 0x49, 0x44, 0x41, 0x54, 0x78,
        0x9C, 0x63, 0x60, 0x00, 0x00, 0x00, 0x02, 0x00, 0x01, 0xE5, 0x27, 0xD4, 0xA2, 0x00,
        0x00, 0x00, 0x00, 0x49, 0x45, 0x4E, 0x44, 0xAE, 0x42, 0x60, 0x82,
    ];

    let temp_dir = TempDirGuard::new("fs25_host_bridge_background_layers");
    fs::write(temp_dir.path().join("overview_terrain.png"), TEST_PNG_1X1_RGBA)
        .expect("Terrain-PNG fuer Layer-Test muss erzeugt werden");
    fs::write(temp_dir.path().join("overview_hillshade.png"), TEST_PNG_1X1_RGBA)
        .expect("Hillshade-PNG fuer Layer-Test muss erzeugt werden");

    let mut state = AppState::new();
    let visible = OverviewLayerOptions {
        terrain: true,
        hillshade: false,
        farmlands: false,
        farmland_ids: false,
        pois: false,
        legend: false,
    };
    let files = discover_background_layer_files(temp_dir.path());
    state.background_layers = Some(
        load_background_layer_catalog(files, &visible)
            .expect("Layer-Katalog fuer Snapshot-Test muss ladbar sein"),
    );

    let chrome = build_host_chrome_snapshot(&state);

    assert!(chrome.background_layers_available);
    assert_eq!(
        chrome
            .background_layer_entries
            .iter()
            .map(|entry| (entry.kind, entry.visible))
            .collect::<Vec<_>>(),
        vec![
            (HostBackgroundLayerKind::Terrain, true),
            (HostBackgroundLayerKind::Hillshade, false),
        ]
    );

}

/// Prüft, dass build_host_chrome_snapshot bei unveraendertem State identische
/// Schlüsselfelder liefert (Grundlage für zukünftiges Frame-Caching).
#[test]
fn build_host_chrome_snapshot_is_idempotent_for_unchanged_state() {
    let mut state = AppState::new();
    state.ui.status_message = Some("bereit".to_string());
    state.editor.default_direction = ConnectionDirection::Dual;

    let first = build_host_chrome_snapshot(&state);
    let second = build_host_chrome_snapshot(&state);

    assert_eq!(
        first.status_message, second.status_message,
        "status_message muss stabil sein"
    );
    assert_eq!(
        first.active_tool, second.active_tool,
        "active_tool muss stabil sein"
    );
    assert_eq!(
        first.default_direction, second.default_direction,
        "default_direction muss stabil sein"
    );
    assert_eq!(
        first.route_tool_entries.len(),
        second.route_tool_entries.len(),
        "route_tool_entries-Laenge muss stabil sein"
    );
    assert_eq!(
        first.options, second.options,
        "EditorOptions müssen stabil sein"
    );
}

/// Prüft, dass build_host_chrome_snapshot die aktuellen EditorOptions korrekt
/// überträgt — insbesondere angepasste Werte statt Default-Werte.
#[test]
fn build_host_chrome_snapshot_propagates_non_default_options() {
    let mut state = AppState::new();
    state.options.node_size_world = 99.0;
    state.options.snap_scale_percent = 42.0;

    let chrome = build_host_chrome_snapshot(&state);

    assert!(
        (chrome.options.node_size_world - 99.0).abs() < f32::EPSILON,
        "node_size_world muss korrekt übertragen werden, war: {}",
        chrome.options.node_size_world
    );
    assert!(
        (chrome.options.snap_scale_percent - 42.0).abs() < f32::EPSILON,
        "snap_scale_percent muss korrekt übertragen werden, war: {}",
        chrome.options.snap_scale_percent
    );
}
