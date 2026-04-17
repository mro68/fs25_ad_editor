use std::hint::black_box;
use std::path::PathBuf;
use std::time::Instant;

use fs25_auto_drive_engine::app::handlers;
use fs25_auto_drive_engine::app::tool_contract::RouteToolId;
use fs25_auto_drive_engine::app::{
    AppIntent, Connection, ConnectionDirection, ConnectionPriority, FloatingMenuKind,
    GroupEditState, GroupRecord, MapMarker, MapNode, NodeFlag, OverviewSourceContext, RoadMap,
    ZipBrowserState,
};
use fs25_auto_drive_engine::core::ZipImageEntry;
use fs25_auto_drive_engine::shared::{OverviewFieldDetectionSource, OverviewLayerOptions};
use glam::Vec2;
use std::sync::Arc;

use crate::dto::{
    HostFieldDetectionSource, HostOverviewLayersSnapshot, HostOverviewOptionsDialogSnapshot,
    HostOverviewSourceContext, HostResampleMode, HostRouteToolId,
};

use crate::dto::{
    EngineSessionAction, HostActiveTool, HostConnectionPairEntry, HostConnectionPairSnapshot,
    HostDefaultConnectionDirection, HostDefaultConnectionPriority, HostDialogRequestKind,
    HostDialogResult, HostInputModifiers, HostMarkerListSnapshot, HostNodeDetails,
    HostPointerButton, HostSessionAction, HostTapKind, HostViewportInputBatch,
    HostViewportInputEvent,
};

use super::{EngineRenderFrameSnapshot, FlutterBridgeSession, HostBridgeSession};

fn apply_test_intent(session: &mut HostBridgeSession, intent: AppIntent) {
    session
        .controller
        .handle_intent(&mut session.state, intent)
        .expect("Test-Intent muss verarbeitet werden");
    session.snapshot_dirty = true;
    session.drain_engine_requests();
    session.sync_chrome_from_engine();
}

fn viewport_test_map() -> RoadMap {
    let mut map = RoadMap::new(2);
    map.add_node(MapNode::new(1, Vec2::new(0.0, 0.0), NodeFlag::Regular));
    map.add_node(MapNode::new(2, Vec2::new(20.0, 0.0), NodeFlag::Regular));
    map.ensure_spatial_index();
    map
}

fn viewport_connected_path_map() -> RoadMap {
    let mut map = RoadMap::new(3);
    map.add_node(MapNode::new(1, Vec2::new(0.0, 0.0), NodeFlag::Regular));
    map.add_node(MapNode::new(2, Vec2::new(10.0, 0.0), NodeFlag::Regular));
    map.add_node(MapNode::new(3, Vec2::new(20.0, 0.0), NodeFlag::Regular));
    map.add_connection(Connection::new(
        1,
        2,
        ConnectionDirection::Regular,
        ConnectionPriority::Regular,
        Vec2::new(0.0, 0.0),
        Vec2::new(10.0, 0.0),
    ));
    map.add_connection(Connection::new(
        2,
        3,
        ConnectionDirection::Regular,
        ConnectionPriority::Regular,
        Vec2::new(10.0, 0.0),
        Vec2::new(20.0, 0.0),
    ));
    map.ensure_spatial_index();
    map
}

fn node_details_marker_test_map() -> RoadMap {
    let mut map = viewport_connected_path_map();
    map.add_map_marker(MapMarker::new(
        2,
        "Hof".to_string(),
        "All".to_string(),
        3,
        false,
    ));
    map
}

fn group_boundary_test_map() -> RoadMap {
    let mut map = RoadMap::new(3);
    map.add_node(MapNode::new(1, Vec2::new(0.0, 0.0), NodeFlag::Regular));
    map.add_node(MapNode::new(2, Vec2::new(10.0, 0.0), NodeFlag::Regular));
    map.add_node(MapNode::new(3, Vec2::new(20.0, 0.0), NodeFlag::Regular));
    map.add_node(MapNode::new(10, Vec2::new(-10.0, 0.0), NodeFlag::Regular));
    map.add_node(MapNode::new(11, Vec2::new(30.0, 0.0), NodeFlag::Regular));
    map.add_connection(Connection::new(
        1,
        2,
        ConnectionDirection::Regular,
        ConnectionPriority::Regular,
        Vec2::new(0.0, 0.0),
        Vec2::new(10.0, 0.0),
    ));
    map.add_connection(Connection::new(
        2,
        3,
        ConnectionDirection::Regular,
        ConnectionPriority::Regular,
        Vec2::new(10.0, 0.0),
        Vec2::new(20.0, 0.0),
    ));
    map.add_connection(Connection::new(
        10,
        1,
        ConnectionDirection::Regular,
        ConnectionPriority::Regular,
        Vec2::new(-10.0, 0.0),
        Vec2::new(0.0, 0.0),
    ));
    map.add_connection(Connection::new(
        3,
        11,
        ConnectionDirection::Regular,
        ConnectionPriority::Regular,
        Vec2::new(20.0, 0.0),
        Vec2::new(30.0, 0.0),
    ));
    map.ensure_spatial_index();
    map
}

fn make_group_record(record_id: u64, node_ids: &[u64], road_map: &RoadMap) -> GroupRecord {
    GroupRecord {
        id: record_id,
        node_ids: node_ids.to_vec(),
        original_positions: node_ids
            .iter()
            .filter_map(|node_id| road_map.node(*node_id).map(|node| node.position))
            .collect(),
        marker_node_ids: Vec::new(),
        locked: false,
        entry_node_id: Some(1),
        exit_node_id: Some(3),
    }
}

fn snapshot_measurement_session(selected_count: usize) -> HostBridgeSession {
    let mut session = HostBridgeSession::new();
    let mut map = RoadMap::new(3);

    for id in 1..=selected_count as u64 {
        let x = id as f32;
        map.add_node(MapNode::new(id, Vec2::new(x, x * 0.25), NodeFlag::Regular));
        session.state.selection.ids_mut().insert(id);
    }

    session.state.road_map = Some(Arc::new(map));
    session.state.view.viewport_size = [1280.0, 720.0];
    session.snapshot_dirty = true;
    let _ = session.snapshot();
    session
}

fn resize_event(size_px: [f32; 2]) -> HostViewportInputEvent {
    HostViewportInputEvent::Resize { size_px }
}

fn tap_event(screen_pos: [f32; 2]) -> HostViewportInputEvent {
    HostViewportInputEvent::Tap {
        button: HostPointerButton::Primary,
        tap_kind: HostTapKind::Single,
        screen_pos,
        modifiers: HostInputModifiers::default(),
    }
}

fn double_tap_event(screen_pos: [f32; 2], additive: bool) -> HostViewportInputEvent {
    HostViewportInputEvent::Tap {
        button: HostPointerButton::Primary,
        tap_kind: HostTapKind::Double,
        screen_pos,
        modifiers: HostInputModifiers {
            shift: false,
            alt: false,
            command: additive,
        },
    }
}

fn screen_for_world(session: &HostBridgeSession, world_pos: Vec2) -> [f32; 2] {
    let viewport = session.state.view.viewport_size;
    let screen = session
        .state
        .view
        .camera
        .world_to_screen(world_pos, Vec2::new(viewport[0], viewport[1]));
    [screen.x, screen.y]
}

#[test]
fn new_session_exposes_empty_snapshot() {
    let mut session = HostBridgeSession::new();
    let snapshot = session.snapshot();

    assert!(!snapshot.has_map);
    assert_eq!(snapshot.node_count, 0);
    assert_eq!(snapshot.connection_count, 0);
    assert_eq!(snapshot.active_tool, HostActiveTool::Select);
    assert!(!snapshot.can_undo);
    assert!(!snapshot.can_redo);
    assert_eq!(snapshot.pending_dialog_request_count, 0);
    assert!(snapshot.selection.selected_node_ids.is_empty());
}

#[test]
fn dispatch_updates_snapshot_state() {
    let mut session = HostBridgeSession::new();

    session
        .apply_action(HostSessionAction::ToggleCommandPalette)
        .expect("ToggleCommandPalette muss funktionieren");

    let snapshot = session.snapshot();
    assert!(snapshot.show_command_palette);
}

#[test]
fn active_tool_uses_explicit_stable_snapshot_identifier() {
    let mut session = HostBridgeSession::new();

    session
        .set_editor_tool(HostActiveTool::Route)
        .expect("SetEditorToolRequested muss funktionieren");

    let snapshot = session.snapshot();
    assert_eq!(snapshot.active_tool, HostActiveTool::Route);
}

#[test]
fn options_dialog_visibility_is_controlled_via_explicit_actions() {
    let mut session = HostBridgeSession::new();

    session
        .set_options_dialog_visible(true)
        .expect("OpenOptionsDialog muss funktionieren");
    assert!(session.snapshot().show_options_dialog);

    session
        .set_options_dialog_visible(false)
        .expect("CloseOptionsDialog muss funktionieren");
    assert!(!session.snapshot().show_options_dialog);
}

#[test]
fn undo_and_redo_actions_are_available_via_explicit_surface() {
    let mut session = HostBridgeSession::new();

    session.undo().expect("Undo muss verfuegbar sein");
    session.redo().expect("Redo muss verfuegbar sein");

    let snapshot = session.snapshot();
    assert!(!snapshot.can_undo);
    assert!(!snapshot.can_redo);
}

#[test]
fn take_dialog_requests_drains_pending_queue_for_host_polling() {
    let mut session = HostBridgeSession::new();

    apply_test_intent(&mut session, AppIntent::CurseplayImportRequested);
    apply_test_intent(&mut session, AppIntent::CurseplayExportRequested);

    assert_eq!(session.snapshot().pending_dialog_request_count, 2);

    let requests = session.take_dialog_requests();
    assert_eq!(requests.len(), 2);
    assert_eq!(requests[0].kind, HostDialogRequestKind::CurseplayImport);
    assert_eq!(requests[1].kind, HostDialogRequestKind::CurseplayExport);
    assert_eq!(session.snapshot().pending_dialog_request_count, 0);
}

#[test]
fn submit_dialog_result_roundtrips_heightmap_path_selected_into_state() {
    let mut session = HostBridgeSession::new();

    session
        .apply_action(HostSessionAction::RequestHeightmapSelection)
        .expect("RequestHeightmapSelection muss einen Host-Dialog anfordern");
    assert_eq!(session.snapshot().pending_dialog_request_count, 1);

    let requests = session.take_dialog_requests();
    assert_eq!(requests.len(), 1);
    assert_eq!(requests[0].kind, HostDialogRequestKind::Heightmap);
    assert_eq!(session.snapshot().pending_dialog_request_count, 0);

    let selected_path = "/tmp/test_heightmap.png".to_string();
    session
        .submit_dialog_result(HostDialogResult::PathSelected {
            kind: HostDialogRequestKind::Heightmap,
            path: selected_path.clone(),
        })
        .expect("PathSelected muss ueber die gemeinsame Dispatch-Seam verarbeitet werden");

    assert_eq!(session.state.ui.heightmap_path, Some(selected_path));
}

#[test]
fn render_accessors_expose_scene_and_assets_without_state_leaks() {
    let session = HostBridgeSession::new();

    let scene = session.build_render_scene([800.0, 600.0]);
    let assets = session.build_render_assets();
    let frame = session.build_render_frame([320.0, 240.0]);

    assert!(!scene.has_map());
    assert_eq!(assets.background_asset_revision(), 0);
    assert!(assets.background().is_none());
    assert_eq!(frame.scene.viewport_size(), [320.0, 240.0]);
    assert_eq!(frame.assets.background_transform_revision(), 0);
}

#[test]
fn host_ui_and_overlay_snapshots_are_available() {
    let mut session = HostBridgeSession::new();

    let host_ui = session.build_host_ui_snapshot();
    let chrome = session.build_host_chrome_snapshot();
    assert!(host_ui.command_palette_state().is_some());
    assert!(host_ui.options_panel_state().is_some());
    assert!(!chrome.has_map);
    assert_eq!(chrome.active_tool, HostActiveTool::Select);

    let overlay = session.build_viewport_overlay_snapshot(None);
    assert!(overlay.route_tool_preview.is_none());
    assert!(overlay.group_boundaries.is_empty());
}

#[test]
fn node_details_read_is_typed_and_side_effect_free() {
    let mut session = HostBridgeSession::new();
    session.state.road_map = Some(Arc::new(node_details_marker_test_map()));

    let details = session
        .node_details(2)
        .expect("Node-Details muessen fuer vorhandenen Node lesbar sein");

    assert_eq!(details.id, 2);
    assert_eq!(details.position, [10.0, 0.0]);
    assert_eq!(details.neighbors.len(), 2);
    assert!(details
        .neighbors
        .iter()
        .any(|neighbor| { neighbor.neighbor_id == 1 && !neighbor.is_outgoing }));
    assert!(details
        .neighbors
        .iter()
        .any(|neighbor| { neighbor.neighbor_id == 3 && neighbor.is_outgoing }));
    assert_eq!(
        details.marker,
        Some(crate::dto::HostNodeMarkerInfo {
            name: "Hof".to_string(),
            group: "All".to_string(),
        })
    );
    assert_eq!(session.inspected_node_id(), None);
}

#[test]
fn node_details_read_returns_none_for_unknown_node_id() {
    let mut session = HostBridgeSession::new();
    session.state.road_map = Some(Arc::new(node_details_marker_test_map()));

    assert_eq!(session.node_details(999), None);
}

#[test]
fn node_details_json_serializes_current_inspected_node_via_typed_read_seam() {
    let mut session = HostBridgeSession::new();
    session.state.road_map = Some(Arc::new(node_details_marker_test_map()));
    session.set_inspected_node_id(Some(2));

    let expected = session
        .node_details(2)
        .expect("Typed Node-Details muessen verfuegbar sein");
    let payload = session
        .node_details_json()
        .expect("JSON-Node-Details muessen fuer inspizierten Node serialisierbar sein");
    let parsed: HostNodeDetails = serde_json::from_str(&payload)
        .expect("Node-Details-JSON muss wieder in das DTO lesbar sein");

    assert_eq!(parsed, expected);
    assert_eq!(session.inspected_node_id(), Some(2));
}

#[test]
fn marker_list_typed_read_and_json_share_the_same_snapshot() {
    let mut session = HostBridgeSession::new();
    session.state.road_map = Some(Arc::new(node_details_marker_test_map()));

    let snapshot = session.marker_list();
    let parsed: HostMarkerListSnapshot = serde_json::from_str(&session.marker_list_json())
        .expect("Marker-List-JSON muss wieder in das DTO lesbar sein");

    assert_eq!(snapshot, parsed);
    assert_eq!(snapshot.groups, vec!["All".to_string()]);
    assert_eq!(snapshot.markers.len(), 1);
    assert_eq!(snapshot.markers[0].node_id, 2);
    assert_eq!(snapshot.markers[0].name, "Hof");
}

#[test]
fn marker_list_read_returns_empty_snapshot_for_empty_road_map() {
    let mut session = HostBridgeSession::new();
    session.state.road_map = Some(Arc::new(RoadMap::new(2)));

    let snapshot = session.marker_list();

    assert!(snapshot.markers.is_empty());
    assert!(snapshot.groups.is_empty());
}

#[test]
fn connection_pair_read_returns_bridge_snapshot_for_two_nodes() {
    let mut session = HostBridgeSession::new();
    let mut map = RoadMap::new(3);
    map.add_node(MapNode::new(1, Vec2::new(0.0, 0.0), NodeFlag::Regular));
    map.add_node(MapNode::new(2, Vec2::new(10.0, 0.0), NodeFlag::Regular));
    map.add_connection(Connection::new(
        1,
        2,
        ConnectionDirection::Dual,
        ConnectionPriority::Regular,
        Vec2::new(0.0, 0.0),
        Vec2::new(10.0, 0.0),
    ));
    map.add_connection(Connection::new(
        2,
        1,
        ConnectionDirection::Reverse,
        ConnectionPriority::SubPriority,
        Vec2::new(10.0, 0.0),
        Vec2::new(0.0, 0.0),
    ));
    session.state.road_map = Some(Arc::new(map));

    let snapshot = session.connection_pair(1, 2);

    assert_eq!(
        snapshot,
        HostConnectionPairSnapshot {
            node_a: 1,
            node_b: 2,
            connections: vec![
                HostConnectionPairEntry {
                    start_id: 1,
                    end_id: 2,
                    direction: HostDefaultConnectionDirection::Dual,
                    priority: HostDefaultConnectionPriority::Regular,
                },
                HostConnectionPairEntry {
                    start_id: 2,
                    end_id: 1,
                    direction: HostDefaultConnectionDirection::Reverse,
                    priority: HostDefaultConnectionPriority::SubPriority,
                },
            ],
        }
    );
}

#[test]
fn connection_pair_read_returns_empty_connections_for_unconnected_nodes() {
    let mut session = HostBridgeSession::new();
    let mut map = RoadMap::new(2);
    map.add_node(MapNode::new(1, Vec2::new(0.0, 0.0), NodeFlag::Regular));
    map.add_node(MapNode::new(2, Vec2::new(10.0, 0.0), NodeFlag::Regular));
    session.state.road_map = Some(Arc::new(map));

    let snapshot = session.connection_pair(1, 2);

    assert_eq!(
        snapshot,
        HostConnectionPairSnapshot {
            node_a: 1,
            node_b: 2,
            connections: Vec::new(),
        }
    );
}

#[test]
fn should_exit_surfaces_explicit_exit_seam() {
    let mut session = HostBridgeSession::new();

    assert!(!session.should_exit());

    session.state.should_exit = true;

    assert!(session.should_exit());
}

#[test]
fn session_dirty_state_surfaces_via_snapshot() {
    let mut session = HostBridgeSession::new();
    let sample_path = PathBuf::from(concat!(
        env!("CARGO_MANIFEST_DIR"),
        "/../../ad_sample_data/AutoDrive_config-test.xml"
    ));

    fs25_auto_drive_engine::app::use_cases::file_io::load_selected_file(
        &mut session.state,
        sample_path.to_string_lossy().into_owned(),
    )
    .expect("Beispiel-XML muss fuer Dirty-Tracking ladbar sein");

    session.snapshot_dirty = true;
    assert!(!session.is_dirty());
    assert!(!session.snapshot().is_dirty);

    Arc::make_mut(
        session
            .state
            .road_map
            .as_mut()
            .expect("RoadMap muss nach dem Laden vorhanden sein"),
    )
    .add_node(MapNode::new(
        999_999,
        Vec2::new(1.0, 1.0),
        NodeFlag::Regular,
    ));

    session.snapshot_dirty = true;
    assert!(session.is_dirty());
    assert!(session.snapshot().is_dirty);
}

#[test]
fn read_only_host_snapshots_do_not_mark_session_snapshot_dirty() {
    let session = snapshot_measurement_session(32);

    let _ = session.build_host_ui_snapshot();
    let _ = session.build_host_chrome_snapshot();
    let _ = session.editing_snapshot();
    let _ = session.build_route_tool_viewport_snapshot();
    let _ = session.build_viewport_geometry_snapshot([640.0, 480.0]);

    assert!(
        !session.snapshot_dirty,
        "Read-only Snapshot-Builder duerfen den Session-Cache nicht dirty markieren"
    );
}

#[test]
fn local_ui_seams_do_not_mark_snapshot_dirty_for_local_state_reads() {
    let mut session = snapshot_measurement_session(32);

    {
        let panel_state = session.panel_properties_state_mut();
        assert_eq!(panel_state.selected_node_ids.len(), 32);
    }
    assert!(!session.snapshot_dirty);

    {
        let dialog_state = session.dialog_ui_state_mut();
        assert!(!dialog_state.ui.show_options_dialog);
    }
    assert!(!session.snapshot_dirty);

    {
        let viewport_state = session.viewport_input_context_mut();
        assert_eq!(viewport_state.selected_node_ids.len(), 32);
    }
    assert!(!session.snapshot_dirty);
}

#[test]
fn explicit_snapshot_invalidation_keeps_local_ui_mutation_visible() {
    let mut session = snapshot_measurement_session(32);

    {
        // HostLocalDialogState-Mutation: show_command_palette setzen.
        // Diese Mutation liegt im host-lokalen Chrome-State, nicht im Engine-State.
        let dialog_state = session.dialog_ui_state_mut();
        dialog_state.ui.show_command_palette = true;
    }
    assert!(
        !session.snapshot_dirty,
        "Lokale Chrome-State-Muatationen invalidieren den Snapshot nicht implizit"
    );

    session.mark_snapshot_dirty();
    assert!(session.snapshot_dirty);

    // Nach mark_snapshot_dirty() wird der Snapshot bei naechstem Zugriff neu gebaut.
    let _snapshot = session.snapshot_owned();
    assert!(!session.snapshot_dirty);
}

#[test]
fn apply_intent_syncs_host_local_overview_options_before_generation() {
    let mut session = HostBridgeSession::new();
    let zip_path = "/tmp/host_bridge_overview_sync.zip".to_string();
    let expected_layers = OverviewLayerOptions {
        terrain: false,
        hillshade: false,
        farmlands: false,
        farmland_ids: true,
        pois: true,
        legend: true,
    };

    session
        .apply_intent(AppIntent::GenerateOverviewFromZip {
            path: zip_path.clone(),
        })
        .expect("Overview-Dialog muss geoeffnet werden");

    session.update_overview_options_dialog(HostOverviewOptionsDialogSnapshot {
        visible: true,
        zip_path: zip_path.clone(),
        layers: HostOverviewLayersSnapshot {
            terrain: expected_layers.terrain,
            hillshade: expected_layers.hillshade,
            farmlands: expected_layers.farmlands,
            farmland_ids: expected_layers.farmland_ids,
            pois: expected_layers.pois,
            legend: expected_layers.legend,
        },
        field_detection_source: HostFieldDetectionSource::GroundGdm,
        available_sources: vec![
            HostFieldDetectionSource::FromZip,
            HostFieldDetectionSource::GroundGdm,
        ],
    });

    assert!(session.chrome_state.chrome_dirty);
    assert_eq!(
        session
            .chrome_state
            .overview_options_dialog
            .field_detection_source,
        OverviewFieldDetectionSource::GroundGdm
    );
    assert!(
        session
            .app_state()
            .ui
            .overview_options_dialog
            .layers
            .terrain
    );

    let error = session
        .apply_intent(AppIntent::OverviewOptionsConfirmed)
        .expect_err("Fehlendes ZIP muss die Generierung scheitern lassen");

    assert!(
        error.to_string().contains(zip_path.as_str()),
        "Fehlermeldung soll den konfigurierten ZIP-Pfad referenzieren"
    );
    assert_eq!(session.app_state().options.overview_layers, expected_layers);
    assert_eq!(
        session.app_state().ui.overview_options_dialog.layers,
        expected_layers
    );
    assert_eq!(
        session.app_state().options.overview_field_detection_source,
        OverviewFieldDetectionSource::GroundGdm
    );
    assert_eq!(
        session
            .app_state()
            .ui
            .overview_options_dialog
            .field_detection_source,
        OverviewFieldDetectionSource::GroundGdm
    );
}

#[test]
fn apply_action_syncs_host_local_overview_options_before_confirm() {
    let mut session = HostBridgeSession::new();
    let zip_path = "/tmp/host_bridge_overview_sync_action.zip".to_string();
    let expected_layers = OverviewLayerOptions {
        terrain: false,
        hillshade: false,
        farmlands: false,
        farmland_ids: true,
        pois: true,
        legend: true,
    };

    session
        .apply_action(HostSessionAction::GenerateOverviewFromZip {
            path: zip_path.clone(),
        })
        .expect("Overview-Dialog muss ueber Host-Action geoeffnet werden");

    session.update_overview_options_dialog(HostOverviewOptionsDialogSnapshot {
        visible: true,
        zip_path: zip_path.clone(),
        layers: HostOverviewLayersSnapshot {
            terrain: expected_layers.terrain,
            hillshade: expected_layers.hillshade,
            farmlands: expected_layers.farmlands,
            farmland_ids: expected_layers.farmland_ids,
            pois: expected_layers.pois,
            legend: expected_layers.legend,
        },
        field_detection_source: HostFieldDetectionSource::GroundGdm,
        available_sources: vec![
            HostFieldDetectionSource::FromZip,
            HostFieldDetectionSource::GroundGdm,
        ],
    });

    assert!(session.chrome_state.chrome_dirty);
    assert_eq!(
        session
            .chrome_state
            .overview_options_dialog
            .field_detection_source,
        OverviewFieldDetectionSource::GroundGdm
    );
    assert!(
        session
            .app_state()
            .ui
            .overview_options_dialog
            .layers
            .terrain
    );

    let error = session
        .apply_action(HostSessionAction::ConfirmOverviewOptions)
        .expect_err("Fehlendes ZIP muss die Generierung ueber Action-Pfad scheitern lassen");

    assert!(
        error.to_string().contains(zip_path.as_str()),
        "Fehlermeldung soll den konfigurierten ZIP-Pfad referenzieren"
    );
    assert_eq!(session.app_state().options.overview_layers, expected_layers);
    assert_eq!(
        session.app_state().ui.overview_options_dialog.layers,
        expected_layers
    );
    assert_eq!(
        session.app_state().options.overview_field_detection_source,
        OverviewFieldDetectionSource::GroundGdm
    );
    assert_eq!(
        session
            .app_state()
            .ui
            .overview_options_dialog
            .field_detection_source,
        OverviewFieldDetectionSource::GroundGdm
    );
}

#[test]
fn generate_overview_from_zip_closes_source_dialog_and_opens_options_dialog() {
    let mut session = HostBridgeSession::new();
    let zip_path = "/tmp/host_bridge_overview_source.zip".to_string();

    session
        .apply_intent(AppIntent::GenerateOverviewRequested)
        .expect("Source-Dialog muss geoeffnet werden");

    assert!(session.chrome_state.post_load_dialog.visible);
    assert_eq!(
        session.chrome_state.post_load_dialog.context,
        OverviewSourceContext::ManualMenu
    );

    session
        .apply_intent(AppIntent::GenerateOverviewFromZip {
            path: zip_path.clone(),
        })
        .expect("ZIP-Auswahl muss den Options-Dialog oeffnen");

    assert!(!session.chrome_state.post_load_dialog.visible);
    assert!(!session.app_state().ui.post_load_dialog.visible);
    assert!(session.chrome_state.overview_options_dialog.visible);
    assert!(session.app_state().ui.overview_options_dialog.visible);
    assert_eq!(
        session.app_state().ui.overview_options_dialog.zip_path,
        zip_path
    );
}

#[test]
fn dialog_snapshot_reflects_host_local_dialog_state() {
    let mut session = HostBridgeSession::new();

    {
        let dialog_state = session.dialog_ui_state_mut();
        dialog_state.ui.show_heightmap_warning = true;
        dialog_state.ui.heightmap_warning_confirmed = true;
        dialog_state.ui.marker_dialog.visible = true;
        dialog_state.ui.marker_dialog.node_id = Some(17);
        dialog_state.ui.marker_dialog.name = "Hof".to_string();
        dialog_state.ui.marker_dialog.group = "All".to_string();
        dialog_state.ui.marker_dialog.is_new = false;
        dialog_state.ui.dedup_dialog.visible = true;
        dialog_state.ui.dedup_dialog.duplicate_count = 3;
        dialog_state.ui.dedup_dialog.group_count = 2;
        dialog_state.ui.zip_browser = Some(ZipBrowserState {
            zip_path: "/tmp/map.zip".to_string(),
            entries: vec![ZipImageEntry {
                name: "overview.png".to_string(),
                size: 4096,
            }],
            selected: Some(0),
            filter_overview: true,
        });
        dialog_state.ui.overview_options_dialog.visible = true;
        dialog_state.ui.overview_options_dialog.zip_path = "/tmp/map.zip".to_string();
        dialog_state.ui.overview_options_dialog.layers = OverviewLayerOptions {
            terrain: false,
            hillshade: false,
            farmlands: true,
            farmland_ids: true,
            pois: false,
            legend: true,
        };
        dialog_state
            .ui
            .overview_options_dialog
            .field_detection_source = OverviewFieldDetectionSource::ZipGroundGdm;
        dialog_state.ui.overview_options_dialog.available_sources = vec![
            OverviewFieldDetectionSource::FromZip,
            OverviewFieldDetectionSource::ZipGroundGdm,
        ];
        dialog_state.ui.post_load_dialog.visible = true;
        dialog_state.ui.post_load_dialog.context = OverviewSourceContext::PostLoadDetected;
        dialog_state.ui.post_load_dialog.heightmap_set = true;
        dialog_state.ui.post_load_dialog.heightmap_path = Some("/tmp/terrain.png".to_string());
        dialog_state.ui.post_load_dialog.overview_loaded = true;
        dialog_state.ui.post_load_dialog.matching_zips = vec![PathBuf::from("/mods/map.zip")];
        dialog_state.ui.post_load_dialog.selected_zip_index = 0;
        dialog_state.ui.post_load_dialog.map_name = "Elmcreek".to_string();
        dialog_state.ui.save_overview_dialog.visible = true;
        dialog_state.ui.save_overview_dialog.target_path = "/tmp/overview.png".to_string();
        dialog_state.ui.save_overview_dialog.is_overwrite = true;
        dialog_state.ui.trace_all_fields_dialog.visible = true;
        dialog_state.ui.trace_all_fields_dialog.spacing = 12.5;
        dialog_state.ui.trace_all_fields_dialog.offset = -1.0;
        dialog_state.ui.trace_all_fields_dialog.tolerance = 0.5;
        dialog_state
            .ui
            .trace_all_fields_dialog
            .corner_detection_enabled = true;
        dialog_state
            .ui
            .trace_all_fields_dialog
            .corner_angle_threshold_deg = 95.0;
        dialog_state
            .ui
            .trace_all_fields_dialog
            .corner_rounding_enabled = true;
        dialog_state
            .ui
            .trace_all_fields_dialog
            .corner_rounding_radius = 6.0;
        dialog_state
            .ui
            .trace_all_fields_dialog
            .corner_rounding_max_angle_deg = 18.0;
        dialog_state.ui.group_settings_popup.visible = true;
        dialog_state.ui.group_settings_popup.world_pos = Vec2::new(8.0, -4.0);
        dialog_state.ui.confirm_dissolve_group_id = Some(99);
        dialog_state.options.segment_stop_at_junction = true;
        dialog_state.options.segment_max_angle_deg = 42.5;
    }

    let snapshot = session.dialog_snapshot();

    assert!(snapshot.heightmap_warning.visible);
    assert!(snapshot.heightmap_warning.confirmed_for_current_save);
    assert_eq!(snapshot.marker_dialog.node_id, Some(17));
    assert_eq!(snapshot.marker_dialog.name, "Hof");
    assert_eq!(snapshot.dedup_dialog.duplicate_count, 3);
    assert!(snapshot.zip_browser.visible);
    assert_eq!(snapshot.zip_browser.entries.len(), 1);
    assert_eq!(snapshot.zip_browser.entries[0].name, "overview.png");
    assert!(!snapshot.overview_options_dialog.layers.terrain);
    assert_eq!(
        snapshot.overview_options_dialog.field_detection_source,
        HostFieldDetectionSource::ZipGroundGdm
    );
    assert_eq!(
        snapshot.post_load_dialog.context,
        HostOverviewSourceContext::PostLoadDetected
    );
    assert_eq!(
        snapshot.post_load_dialog.matching_zip_paths,
        vec!["/mods/map.zip".to_string()]
    );
    assert_eq!(snapshot.group_settings_popup.world_pos, [8.0, -4.0]);
    assert!(snapshot.group_settings_popup.segment_stop_at_junction);
    assert_eq!(snapshot.group_settings_popup.segment_max_angle_deg, 42.5);
    assert_eq!(snapshot.confirm_dissolve_group.segment_id, Some(99));
    assert!(snapshot.confirm_dissolve_group.visible);
}

#[test]
fn editing_snapshot_reports_resample_metrics_for_connected_chain() {
    let mut session = HostBridgeSession::new();
    session.state.road_map = Some(Arc::new(viewport_connected_path_map()));
    session.state.selection.ids_mut().insert(1);
    session.state.selection.ids_mut().insert(2);
    session.state.selection.ids_mut().insert(3);
    session.state.ui.distanzen.active = true;
    session.state.ui.distanzen.by_count = true;
    session.state.ui.distanzen.count = 5;
    session.state.ui.distanzen.distance = 4.0;
    session.state.ui.distanzen.hide_original = true;

    let snapshot = session.editing_snapshot();

    assert!(snapshot.resample.active);
    assert!(snapshot.resample.can_resample_current_selection);
    assert_eq!(snapshot.resample.selected_node_count, 3);
    assert_eq!(snapshot.resample.mode, HostResampleMode::Count);
    assert_eq!(snapshot.resample.count, 5);
    assert_eq!(snapshot.resample.preview_count, 5);
    assert!((snapshot.resample.path_length - 20.0).abs() < 0.01);
}

#[test]
fn editing_snapshot_reports_group_edit_boundary_candidates() {
    let mut session = HostBridgeSession::new();
    let road_map = group_boundary_test_map();
    let record_id = 42;
    let record = make_group_record(record_id, &[1, 2, 3], &road_map);

    session.state.road_map = Some(Arc::new(road_map));
    session.state.group_registry.register(record);
    session.state.group_editing = Some(GroupEditState {
        record_id,
        was_locked: true,
    });

    let snapshot = session.editing_snapshot();
    let group_edit = snapshot
        .group_edit
        .expect("Group-Edit-Snapshot muss vorhanden sein");

    assert_eq!(group_edit.record_id, record_id);
    assert!(!group_edit.locked);
    assert!(group_edit.was_locked_before_edit);
    assert_eq!(group_edit.entry_node_id, Some(1));
    assert_eq!(group_edit.exit_node_id, Some(3));
    assert_eq!(group_edit.boundary_candidates.len(), 3);

    let entry_candidate = group_edit
        .boundary_candidates
        .iter()
        .find(|candidate| candidate.node_id == 1)
        .expect("Entry-Kandidat muss enthalten sein");
    assert!(entry_candidate.has_external_incoming);
    assert!(!entry_candidate.has_external_outgoing);

    let middle_candidate = group_edit
        .boundary_candidates
        .iter()
        .find(|candidate| candidate.node_id == 2)
        .expect("Mittelknoten muss enthalten sein");
    assert!(!middle_candidate.has_external_incoming);
    assert!(!middle_candidate.has_external_outgoing);

    let exit_candidate = group_edit
        .boundary_candidates
        .iter()
        .find(|candidate| candidate.node_id == 3)
        .expect("Exit-Kandidat muss enthalten sein");
    assert!(!exit_candidate.has_external_incoming);
    assert!(exit_candidate.has_external_outgoing);
}

#[test]
fn editing_snapshot_reports_tool_editable_groups_for_persisted_route_tool() {
    let mut session = HostBridgeSession::new();
    let mut road_map = RoadMap::new(3);
    road_map.add_node(MapNode::new(1, Vec2::new(0.0, 0.0), NodeFlag::Regular));
    road_map.add_node(MapNode::new(2, Vec2::new(20.0, 0.0), NodeFlag::Regular));
    road_map.ensure_spatial_index();
    session.state.road_map = Some(Arc::new(road_map));

    handlers::route_tool::select_with_anchors(&mut session.state, RouteToolId::Straight, 1, 2);

    let record = session
        .state
        .group_registry
        .records()
        .next()
        .expect("Persistierter Straight-Record muss vorhanden sein")
        .clone();
    session.state.selection.ids_mut().clear();
    if let Some(&first_group_node) = record.node_ids.first() {
        session.state.selection.ids_mut().insert(first_group_node);
    }

    let snapshot = session.editing_snapshot();

    assert_eq!(snapshot.editable_groups.len(), 1);
    assert_eq!(snapshot.editable_groups[0].record_id, record.id);
    assert!(snapshot.editable_groups[0].has_tool_edit);
    assert_eq!(
        snapshot.editable_groups[0].tool_id,
        Some(HostRouteToolId::Straight)
    );
}

#[test]
fn snapshot_measurement_clean_poll_reports_zero_rebuild_candidates() {
    let mut session = snapshot_measurement_session(1024);
    let iterations = 256usize;
    let start = Instant::now();
    let mut rebuild_candidates = 0usize;

    for _ in 0..iterations {
        rebuild_candidates += usize::from(session.snapshot_dirty);
        black_box(session.snapshot_owned());
    }

    let elapsed_us_per_iter = start.elapsed().as_secs_f64() * 1_000_000.0 / iterations as f64;
    eprintln!(
            "snapshot_measurement_clean_poll selected_nodes=1024 iterations={iterations} rebuild_candidates={rebuild_candidates} elapsed_us_per_iter={elapsed_us_per_iter:.3}"
        );

    assert_eq!(rebuild_candidates, 0);
    assert!(!session.snapshot_dirty);
}

#[test]
fn snapshot_measurement_read_mostly_flow_reports_zero_rebuild_candidates() {
    let mut session = snapshot_measurement_session(1024);
    let iterations = 256usize;
    let start = Instant::now();
    let mut rebuild_candidates = 0usize;

    for _ in 0..iterations {
        black_box(session.build_host_ui_snapshot());
        black_box(session.build_host_chrome_snapshot());

        {
            let panel_state = session.panel_properties_state_mut();
            black_box(panel_state.selected_node_ids.len());
        }
        {
            let dialog_state = session.dialog_ui_state_mut();
            black_box(dialog_state.ui.show_options_dialog);
        }
        {
            let viewport_state = session.viewport_input_context_mut();
            black_box(viewport_state.selected_node_ids.len());
        }

        rebuild_candidates += usize::from(session.snapshot_dirty);
        black_box(session.snapshot_owned());
    }

    let elapsed_us_per_iter = start.elapsed().as_secs_f64() * 1_000_000.0 / iterations as f64;
    eprintln!(
            "snapshot_measurement_read_mostly selected_nodes=1024 iterations={iterations} rebuild_candidates={rebuild_candidates} elapsed_us_per_iter={elapsed_us_per_iter:.3}"
        );

    assert_eq!(rebuild_candidates, 0);
    assert!(!session.snapshot_dirty);
}

#[test]
fn flutter_session_alias_exposes_host_bridge_session_behavior() {
    let mut session = FlutterBridgeSession::new();

    session
        .apply_action(EngineSessionAction::ToggleCommandPalette)
        .expect("ToggleCommandPalette muss ueber den Alias funktionieren");

    assert!(session.snapshot().show_command_palette);
}

#[test]
fn engine_render_frame_snapshot_alias_keeps_render_contract() {
    let session = HostBridgeSession::new();

    let frame: EngineRenderFrameSnapshot = session.build_render_frame([512.0, 256.0]);

    assert_eq!(frame.scene.viewport_size(), [512.0, 256.0]);
    assert_eq!(frame.assets.background_asset_revision(), 0);
}

#[test]
fn viewport_input_resize_and_scroll_zoom_update_session_view() {
    let mut session = HostBridgeSession::new();

    session
        .apply_action(HostSessionAction::SubmitViewportInput {
            batch: HostViewportInputBatch {
                events: vec![
                    resize_event([640.0, 480.0]),
                    HostViewportInputEvent::Scroll {
                        screen_pos: Some([320.0, 240.0]),
                        smooth_delta_y: 1.0,
                        raw_delta_y: 0.0,
                        modifiers: HostInputModifiers::default(),
                    },
                ],
            },
        })
        .expect("Resize und Scroll-Zoom muessen ueber die Session funktionieren");

    assert_eq!(session.state.view.viewport_size, [640.0, 480.0]);
    assert!(session.state.view.camera.zoom > 1.0);
}

#[test]
fn viewport_input_tap_routes_to_add_node_and_connect_without_new_ffi_surface() {
    let mut session = HostBridgeSession::new();
    session.state.road_map = Some(Arc::new(viewport_test_map()));
    session.state.view.viewport_size = [800.0, 600.0];

    let add_node_screen = screen_for_world(&session, Vec2::new(200.0, 0.0));
    let node1_screen = screen_for_world(&session, Vec2::new(0.0, 0.0));
    let node2_screen = screen_for_world(&session, Vec2::new(20.0, 0.0));

    session
        .set_editor_tool(HostActiveTool::AddNode)
        .expect("AddNode-Tool muss gesetzt werden koennen");
    session
        .apply_action(HostSessionAction::SubmitViewportInput {
            batch: HostViewportInputBatch {
                events: vec![resize_event([800.0, 600.0]), tap_event(add_node_screen)],
            },
        })
        .expect("AddNode-Tap muss verarbeitet werden");

    assert_eq!(session.state.node_count(), 3);

    session
        .set_editor_tool(HostActiveTool::Connect)
        .expect("Connect-Tool muss gesetzt werden koennen");
    session
        .apply_action(HostSessionAction::SubmitViewportInput {
            batch: HostViewportInputBatch {
                events: vec![tap_event(node1_screen), tap_event(node2_screen)],
            },
        })
        .expect("Connect-Taps muessen verarbeitet werden");

    assert_eq!(session.state.connection_count(), 1);
}

#[test]
fn viewport_input_select_rect_and_move_drag_preserve_lifecycle() {
    let mut session = HostBridgeSession::new();
    session.state.road_map = Some(Arc::new(viewport_test_map()));
    session.state.view.viewport_size = [800.0, 600.0];

    let node1_screen = screen_for_world(&session, Vec2::new(0.0, 0.0));
    let rect_end = screen_for_world(&session, Vec2::new(5.0, 5.0));

    session
        .apply_action(HostSessionAction::SubmitViewportInput {
            batch: HostViewportInputBatch {
                events: vec![
                    resize_event([800.0, 600.0]),
                    HostViewportInputEvent::DragStart {
                        button: HostPointerButton::Primary,
                        screen_pos: node1_screen,
                        modifiers: HostInputModifiers {
                            shift: true,
                            alt: false,
                            command: false,
                        },
                    },
                    HostViewportInputEvent::DragUpdate {
                        button: HostPointerButton::Primary,
                        screen_pos: rect_end,
                        delta_px: [rect_end[0] - node1_screen[0], rect_end[1] - node1_screen[1]],
                    },
                    HostViewportInputEvent::DragEnd {
                        button: HostPointerButton::Primary,
                        screen_pos: Some(rect_end),
                    },
                ],
            },
        })
        .expect("Rect-Selektion muss verarbeitet werden");

    assert_eq!(session.state.selection.selected_node_ids.len(), 1);
    assert!(session.state.selection.selected_node_ids.contains(&1));

    let node_before = session
        .state
        .road_map
        .as_ref()
        .and_then(|map| map.node(1))
        .expect("Node 1 muss vorhanden sein")
        .position;

    session
        .apply_action(HostSessionAction::SubmitViewportInput {
            batch: HostViewportInputBatch {
                events: vec![
                    HostViewportInputEvent::DragStart {
                        button: HostPointerButton::Primary,
                        screen_pos: node1_screen,
                        modifiers: HostInputModifiers::default(),
                    },
                    HostViewportInputEvent::DragUpdate {
                        button: HostPointerButton::Primary,
                        screen_pos: [node1_screen[0] + 10.0, node1_screen[1]],
                        delta_px: [10.0, 0.0],
                    },
                    HostViewportInputEvent::DragEnd {
                        button: HostPointerButton::Primary,
                        screen_pos: Some([node1_screen[0] + 10.0, node1_screen[1]]),
                    },
                ],
            },
        })
        .expect("Move-Drag muss verarbeitet werden");

    let node_after = session
        .state
        .road_map
        .as_ref()
        .and_then(|map| map.node(1))
        .expect("Node 1 muss nach dem Drag vorhanden sein")
        .position;

    assert!(node_after.x > node_before.x);
    assert!(session.state.can_undo());
}

#[test]
fn viewport_input_alt_drag_selects_lasso_polygon_via_bridge_contract() {
    let mut session = HostBridgeSession::new();
    session.state.road_map = Some(Arc::new(viewport_connected_path_map()));
    session.state.view.viewport_size = [800.0, 600.0];

    let node1_screen = screen_for_world(&session, Vec2::new(0.0, 0.0));
    let node2_screen = screen_for_world(&session, Vec2::new(10.0, 0.0));
    let start = [node1_screen[0] - 20.0, node1_screen[1] - 20.0];
    let mid = [node2_screen[0] + 20.0, node1_screen[1] - 20.0];
    let end = [node2_screen[0] + 20.0, node1_screen[1] + 20.0];
    let close = [node1_screen[0] - 20.0, node1_screen[1] + 20.0];

    session
        .apply_action(HostSessionAction::SubmitViewportInput {
            batch: HostViewportInputBatch {
                events: vec![
                    resize_event([800.0, 600.0]),
                    HostViewportInputEvent::DragStart {
                        button: HostPointerButton::Primary,
                        screen_pos: start,
                        modifiers: HostInputModifiers {
                            shift: false,
                            alt: true,
                            command: false,
                        },
                    },
                    HostViewportInputEvent::DragUpdate {
                        button: HostPointerButton::Primary,
                        screen_pos: mid,
                        delta_px: [mid[0] - start[0], mid[1] - start[1]],
                    },
                    HostViewportInputEvent::DragUpdate {
                        button: HostPointerButton::Primary,
                        screen_pos: end,
                        delta_px: [end[0] - mid[0], end[1] - mid[1]],
                    },
                    HostViewportInputEvent::DragEnd {
                        button: HostPointerButton::Primary,
                        screen_pos: Some(close),
                    },
                ],
            },
        })
        .expect("Alt-Drag-Lasso muss ueber die Bridge verarbeitet werden");

    assert_eq!(session.state.selection.selected_node_ids.len(), 3);
    assert!(session.state.selection.selected_node_ids.contains(&1));
    assert!(session.state.selection.selected_node_ids.contains(&2));
    assert!(session.state.selection.selected_node_ids.contains(&3));
}

#[test]
fn viewport_input_double_tap_selects_segment_via_bridge_contract() {
    let mut session = HostBridgeSession::new();
    session.state.road_map = Some(Arc::new(viewport_connected_path_map()));
    session.state.view.viewport_size = [800.0, 600.0];

    let node2_screen = screen_for_world(&session, Vec2::new(10.0, 0.0));

    session
        .apply_action(HostSessionAction::SubmitViewportInput {
            batch: HostViewportInputBatch {
                events: vec![
                    resize_event([800.0, 600.0]),
                    double_tap_event(node2_screen, false),
                ],
            },
        })
        .expect("Double-Tap muss ueber die Bridge verarbeitet werden");

    assert_eq!(session.state.selection.selected_node_ids.len(), 3);
    assert!(session.state.selection.selected_node_ids.contains(&1));
    assert!(session.state.selection.selected_node_ids.contains(&2));
    assert!(session.state.selection.selected_node_ids.contains(&3));
}

#[test]
fn viewport_input_requires_resize_before_position_dependent_events() {
    let mut session = HostBridgeSession::new();

    let error = session
        .apply_action(HostSessionAction::SubmitViewportInput {
            batch: HostViewportInputBatch {
                events: vec![tap_event([10.0, 20.0])],
            },
        })
        .expect_err("Tap ohne Resize muss einen Integrationsfehler liefern");

    assert!(error
        .to_string()
        .contains("viewport input requires a positive finite viewport size"));
}

#[test]
fn floating_menu_seams_toggle_and_clear_without_full_state_escape() {
    let mut session = HostBridgeSession::new();

    session.toggle_floating_menu(FloatingMenuKind::Tools, Some(Vec2::new(10.0, 20.0)));
    let tools_menu = session
        .chrome_state()
        .floating_menu
        .expect("Tools-Menue muss geoeffnet sein");
    assert_eq!(tools_menu.kind, FloatingMenuKind::Tools);
    assert_eq!(tools_menu.pos, Vec2::new(10.0, 20.0));

    session.toggle_floating_menu(FloatingMenuKind::Tools, Some(Vec2::new(30.0, 40.0)));
    assert!(session.chrome_state().floating_menu.is_none());

    session.toggle_floating_menu(FloatingMenuKind::Zoom, None);
    assert!(session.chrome_state().floating_menu.is_none());

    session.toggle_floating_menu(FloatingMenuKind::Zoom, Some(Vec2::new(5.0, 6.0)));
    assert!(session.chrome_state().floating_menu.is_some());

    session.clear_floating_menu();
    assert!(session.chrome_state().floating_menu.is_none());
}

#[test]
fn viewport_geometry_snapshot_is_available_via_session_surface() {
    let session = HostBridgeSession::new();

    let geometry = session.build_viewport_geometry_snapshot([300.0, 200.0]);

    assert!(!geometry.has_map);
    assert!(geometry.nodes.is_empty());
    assert!(geometry.connections.is_empty());
    assert!(geometry.markers.is_empty());
    assert_eq!(geometry.viewport_size, [300.0, 200.0]);
}
