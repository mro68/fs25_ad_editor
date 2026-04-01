use fs25_auto_drive_editor::app::handlers;
use fs25_auto_drive_editor::app::tool_contract::TangentSource;
use fs25_auto_drive_editor::app::tools::RouteToolId;
use fs25_auto_drive_editor::app::{GroupBase, GroupKind, GroupRecord, ToolAnchor};
use fs25_auto_drive_editor::EditorTool;
use fs25_auto_drive_editor::{AppController, AppIntent, AppState};
use fs25_auto_drive_editor::{
    Connection, ConnectionDirection, ConnectionPriority, MapNode, NodeFlag, RoadMap,
};
use std::sync::Arc;

fn make_test_map() -> AppState {
    let mut map = RoadMap::new(3);
    map.add_node(MapNode::new(
        1,
        glam::Vec2::new(0.0, 0.0),
        NodeFlag::Regular,
    ));
    map.add_node(MapNode::new(
        2,
        glam::Vec2::new(10.0, 0.0),
        NodeFlag::Regular,
    ));
    map.add_node(MapNode::new(
        3,
        glam::Vec2::new(20.0, 0.0),
        NodeFlag::Regular,
    ));
    map.add_connection(Connection::new(
        1,
        2,
        ConnectionDirection::Regular,
        ConnectionPriority::Regular,
        glam::Vec2::new(0.0, 0.0),
        glam::Vec2::new(10.0, 0.0),
    ));
    map.ensure_spatial_index();
    let mut state = AppState::new();
    state.road_map = Some(Arc::new(map));
    state.view.viewport_size = [1280.0, 720.0];
    state
}

fn make_curve_anchor_map() -> AppState {
    let mut map = RoadMap::new(3);

    // Start/End-Anker
    map.add_node(MapNode::new(
        1,
        glam::Vec2::new(0.0, 0.0),
        NodeFlag::Regular,
    ));
    map.add_node(MapNode::new(
        2,
        glam::Vec2::new(10.0, 0.0),
        NodeFlag::Regular,
    ));

    // Nachbar fuer Start (incoming in Richtung Start)
    map.add_node(MapNode::new(
        10,
        glam::Vec2::new(-10.0, 0.0),
        NodeFlag::Regular,
    ));
    map.add_connection(Connection::new(
        10,
        1,
        ConnectionDirection::Regular,
        ConnectionPriority::Regular,
        glam::Vec2::new(-10.0, 0.0),
        glam::Vec2::new(0.0, 0.0),
    ));

    // Nachbar fuer Ende (outgoing in Richtung weg vom Start)
    map.add_node(MapNode::new(
        20,
        glam::Vec2::new(20.0, 0.0),
        NodeFlag::Regular,
    ));
    map.add_connection(Connection::new(
        2,
        20,
        ConnectionDirection::Regular,
        ConnectionPriority::Regular,
        glam::Vec2::new(10.0, 0.0),
        glam::Vec2::new(20.0, 0.0),
    ));

    map.ensure_spatial_index();

    let mut state = AppState::new();
    state.road_map = Some(Arc::new(map));
    state.view.viewport_size = [1280.0, 720.0];
    state
}

fn current_cubic_tangents(state: &AppState) -> (TangentSource, TangentSource) {
    let tool = state
        .editor
        .tool_manager
        .active_tool()
        .expect("Route-Tool muss aktiv sein");
    let menu = tool
        .tangent_menu_data()
        .expect("Tangenten-Menue muss in Control-Phase verfuegbar sein");
    (menu.current_start, menu.current_end)
}

fn make_manual_group_record(id: u64, tool_id: Option<RouteToolId>) -> GroupRecord {
    GroupRecord {
        id,
        tool_id,
        node_ids: vec![1, 2],
        start_anchor: ToolAnchor::ExistingNode(1, glam::Vec2::new(0.0, 0.0)),
        end_anchor: ToolAnchor::ExistingNode(2, glam::Vec2::new(10.0, 0.0)),
        kind: GroupKind::Manual {
            base: GroupBase {
                direction: ConnectionDirection::Regular,
                priority: ConnectionPriority::Regular,
                max_segment_length: 10.0,
            },
        },
        original_positions: vec![glam::Vec2::new(0.0, 0.0), glam::Vec2::new(10.0, 0.0)],
        marker_node_ids: Vec::new(),
        locked: false,
        entry_node_id: None,
        exit_node_id: None,
    }
}

#[test]
fn test_cubic_with_anchors_matches_manual_tangent_defaults() {
    // Flow A: via RouteToolWithAnchors (2 selektierte Nodes)
    let mut state_with_anchors = make_curve_anchor_map();
    handlers::route_tool::select_with_anchors(
        &mut state_with_anchors,
        RouteToolId::CurveCubic,
        1,
        2,
    );
    let tangents_with_anchors = current_cubic_tangents(&state_with_anchors);

    // Flow B: manuell (Tool waehlen -> Start klicken -> Ende klicken)
    let mut state_manual = make_curve_anchor_map();
    handlers::route_tool::select(&mut state_manual, RouteToolId::CurveCubic);
    handlers::route_tool::click(&mut state_manual, glam::Vec2::new(0.0, 0.0), false);
    handlers::route_tool::click(&mut state_manual, glam::Vec2::new(10.0, 0.0), false);
    let tangents_manual = current_cubic_tangents(&state_manual);

    assert_eq!(
        tangents_with_anchors, tangents_manual,
        "Cubic-Tangenten sollten im Anchor-Flow und manuellen Flow identisch vorbelegt werden"
    );

    assert!(
        matches!(tangents_with_anchors.0, TangentSource::Connection { .. }),
        "Start-Tangente sollte als Verbindungs-Tangente vorbelegt sein"
    );
    assert!(
        matches!(tangents_with_anchors.1, TangentSource::Connection { .. }),
        "End-Tangente sollte als Verbindungs-Tangente vorbelegt sein"
    );
}

#[test]
fn select_route_tool_requested_merkt_analysis_tool_per_tool_id() {
    let mut controller = AppController::new();
    let mut state = make_test_map();

    controller
        .handle_intent(
            &mut state,
            AppIntent::SelectRouteToolRequested {
                tool_id: RouteToolId::ColorPath,
            },
        )
        .expect("SelectRouteToolRequested sollte ueber RouteToolId funktionieren");

    assert_eq!(state.editor.active_tool, EditorTool::Route);
    assert_eq!(
        state.editor.tool_manager.active_id(),
        Some(RouteToolId::ColorPath)
    );
    assert_eq!(
        state.editor.route_tool_memory.analysis,
        RouteToolId::ColorPath
    );
}

#[test]
fn group_edit_tool_requested_bricht_fuer_nicht_editierbare_tools_ohne_nebeneffekte_ab() {
    let mut controller = AppController::new();
    let mut state = make_test_map();
    let record_id = state.group_registry.next_id();
    state.group_registry.register(make_manual_group_record(
        record_id,
        Some(RouteToolId::ColorPath),
    ));

    controller
        .handle_intent(&mut state, AppIntent::GroupEditStartRequested { record_id })
        .expect("GroupEditStartRequested sollte den Gruppen-Edit starten");

    let before_node_count = state.road_map.as_ref().unwrap().node_count();
    let before_connection_count = state.road_map.as_ref().unwrap().connection_count();
    let before_active_tool = state.editor.active_tool;
    let before_active_route_id = state.editor.tool_manager.active_id();

    controller
        .handle_intent(&mut state, AppIntent::GroupEditToolRequested { record_id })
        .expect("Nicht editierbare Tools duerfen den Flow nicht crashen");

    assert!(state.group_editing.is_none());
    assert!(state.group_registry.get(record_id).is_some());
    assert_eq!(
        state.road_map.as_ref().unwrap().node_count(),
        before_node_count
    );
    assert_eq!(
        state.road_map.as_ref().unwrap().connection_count(),
        before_connection_count
    );
    assert_eq!(state.editor.active_tool, before_active_tool);
    assert_eq!(
        state.editor.tool_manager.active_id(),
        before_active_route_id
    );
    assert!(state.selection.selected_node_ids.is_empty());
    assert_eq!(state.tool_editing_record_id, None);
    assert!(state.tool_editing_record_backup.is_none());
}

#[test]
fn test_add_node_at_position() {
    let mut controller = AppController::new();
    let mut state = make_test_map();

    let before_count = state.road_map.as_ref().unwrap().node_count();

    controller
        .handle_intent(
            &mut state,
            AppIntent::AddNodeRequested {
                world_pos: glam::Vec2::new(50.0, 50.0),
            },
        )
        .expect("AddNodeRequested sollte funktionieren");

    let rm = state.road_map.as_ref().unwrap();
    assert_eq!(rm.node_count(), before_count + 1);

    // Neuer Node sollte selektiert sein
    assert_eq!(state.selection.selected_node_ids.len(), 1);
    let new_id = *state.selection.selected_node_ids.iter().next().unwrap();
    let node = rm.nodes.get(&new_id).expect("Neuer Node existiert");
    assert_eq!(node.position, glam::Vec2::new(50.0, 50.0));
}

#[test]
fn test_add_node_is_undoable() {
    let mut controller = AppController::new();
    let mut state = make_test_map();
    let before_count = state.road_map.as_ref().unwrap().node_count();

    controller
        .handle_intent(
            &mut state,
            AppIntent::AddNodeRequested {
                world_pos: glam::Vec2::new(50.0, 50.0),
            },
        )
        .unwrap();

    assert_eq!(
        state.road_map.as_ref().unwrap().node_count(),
        before_count + 1
    );

    controller
        .handle_intent(&mut state, AppIntent::UndoRequested)
        .unwrap();

    assert_eq!(state.road_map.as_ref().unwrap().node_count(), before_count);
}

#[test]
fn test_delete_selected_nodes() {
    let mut controller = AppController::new();
    let mut state = make_test_map();

    state.selection.ids_mut().insert(1);

    controller
        .handle_intent(&mut state, AppIntent::DeleteSelectedRequested)
        .expect("DeleteSelectedRequested sollte funktionieren");

    let rm = state.road_map.as_ref().unwrap();
    assert_eq!(rm.node_count(), 2);
    assert!(!rm.nodes.contains_key(&1));
    // Verbindung 1→2 sollte auch entfernt sein
    assert_eq!(rm.connection_count(), 0);
    // Selektion leer
    assert!(state.selection.selected_node_ids.is_empty());
}

#[test]
fn test_delete_is_undoable() {
    let mut controller = AppController::new();
    let mut state = make_test_map();
    state.selection.ids_mut().insert(1);

    controller
        .handle_intent(&mut state, AppIntent::DeleteSelectedRequested)
        .unwrap();
    assert_eq!(state.road_map.as_ref().unwrap().node_count(), 2);

    controller
        .handle_intent(&mut state, AppIntent::UndoRequested)
        .unwrap();
    assert_eq!(state.road_map.as_ref().unwrap().node_count(), 3);
    assert!(state.road_map.as_ref().unwrap().nodes.contains_key(&1));
    assert_eq!(state.road_map.as_ref().unwrap().connection_count(), 1);
}

#[test]
fn test_add_connection_regular() {
    let mut controller = AppController::new();
    let mut state = make_test_map();
    let before_conns = state.road_map.as_ref().unwrap().connection_count();

    controller
        .handle_intent(
            &mut state,
            AppIntent::AddConnectionRequested {
                from_id: 2,
                to_id: 3,
                direction: ConnectionDirection::Regular,
                priority: ConnectionPriority::Regular,
            },
        )
        .expect("AddConnectionRequested sollte funktionieren");

    let rm = state.road_map.as_ref().unwrap();
    assert_eq!(rm.connection_count(), before_conns + 1);
    let conn = rm.find_connection(2, 3).expect("Verbindung 2→3 existiert");
    assert_eq!(conn.direction, ConnectionDirection::Regular);
}

#[test]
fn test_add_connection_dual() {
    let mut controller = AppController::new();
    let mut state = make_test_map();

    controller
        .handle_intent(
            &mut state,
            AppIntent::AddConnectionRequested {
                from_id: 2,
                to_id: 3,
                direction: ConnectionDirection::Dual,
                priority: ConnectionPriority::Regular,
            },
        )
        .unwrap();

    let rm = state.road_map.as_ref().unwrap();
    let conn = rm.find_connection(2, 3).unwrap();
    assert_eq!(conn.direction, ConnectionDirection::Dual);
}

#[test]
fn test_add_connection_reverse() {
    let mut controller = AppController::new();
    let mut state = make_test_map();

    controller
        .handle_intent(
            &mut state,
            AppIntent::AddConnectionRequested {
                from_id: 2,
                to_id: 3,
                direction: ConnectionDirection::Reverse,
                priority: ConnectionPriority::Regular,
            },
        )
        .unwrap();

    let rm = state.road_map.as_ref().unwrap();
    let conn = rm.find_connection(2, 3).unwrap();
    assert_eq!(conn.direction, ConnectionDirection::Reverse);
}

#[test]
fn test_add_connection_rejects_duplicate() {
    let mut controller = AppController::new();
    let mut state = make_test_map();
    let before_conns = state.road_map.as_ref().unwrap().connection_count();

    // Verbindung 1→2 existiert bereits
    controller
        .handle_intent(
            &mut state,
            AppIntent::AddConnectionRequested {
                from_id: 1,
                to_id: 2,
                direction: ConnectionDirection::Regular,
                priority: ConnectionPriority::Regular,
            },
        )
        .unwrap();

    // Keine neue Connection hinzugefuegt
    assert_eq!(
        state.road_map.as_ref().unwrap().connection_count(),
        before_conns
    );
}

#[test]
fn test_add_connection_rejects_self_loop() {
    let mut controller = AppController::new();
    let mut state = make_test_map();
    let before_conns = state.road_map.as_ref().unwrap().connection_count();

    controller
        .handle_intent(
            &mut state,
            AppIntent::AddConnectionRequested {
                from_id: 1,
                to_id: 1,
                direction: ConnectionDirection::Regular,
                priority: ConnectionPriority::Regular,
            },
        )
        .unwrap();

    assert_eq!(
        state.road_map.as_ref().unwrap().connection_count(),
        before_conns
    );
}

#[test]
fn test_remove_connection_between() {
    let mut controller = AppController::new();
    let mut state = make_test_map();

    controller
        .handle_intent(
            &mut state,
            AppIntent::RemoveConnectionBetweenRequested {
                node_a: 1,
                node_b: 2,
            },
        )
        .expect("RemoveConnectionBetweenRequested sollte funktionieren");

    assert_eq!(state.road_map.as_ref().unwrap().connection_count(), 0);
}

#[test]
fn test_remove_connection_is_undoable() {
    let mut controller = AppController::new();
    let mut state = make_test_map();

    controller
        .handle_intent(
            &mut state,
            AppIntent::RemoveConnectionBetweenRequested {
                node_a: 1,
                node_b: 2,
            },
        )
        .unwrap();
    assert_eq!(state.road_map.as_ref().unwrap().connection_count(), 0);

    controller
        .handle_intent(&mut state, AppIntent::UndoRequested)
        .unwrap();
    assert_eq!(state.road_map.as_ref().unwrap().connection_count(), 1);
}

#[test]
fn test_set_connection_direction() {
    let mut controller = AppController::new();
    let mut state = make_test_map();

    controller
        .handle_intent(
            &mut state,
            AppIntent::SetConnectionDirectionRequested {
                start_id: 1,
                end_id: 2,
                direction: ConnectionDirection::Dual,
            },
        )
        .expect("SetConnectionDirectionRequested sollte funktionieren");

    let rm = state.road_map.as_ref().unwrap();
    let conn = rm.find_connection(1, 2).unwrap();
    assert_eq!(conn.direction, ConnectionDirection::Dual);
}

#[test]
fn test_set_connection_direction_is_undoable() {
    let mut controller = AppController::new();
    let mut state = make_test_map();

    controller
        .handle_intent(
            &mut state,
            AppIntent::SetConnectionDirectionRequested {
                start_id: 1,
                end_id: 2,
                direction: ConnectionDirection::Reverse,
            },
        )
        .unwrap();

    assert_eq!(
        state
            .road_map
            .as_ref()
            .unwrap()
            .find_connection(1, 2)
            .unwrap()
            .direction,
        ConnectionDirection::Reverse
    );

    controller
        .handle_intent(&mut state, AppIntent::UndoRequested)
        .unwrap();

    assert_eq!(
        state
            .road_map
            .as_ref()
            .unwrap()
            .find_connection(1, 2)
            .unwrap()
            .direction,
        ConnectionDirection::Regular
    );
}

#[test]
fn test_set_editor_tool() {
    let mut controller = AppController::new();
    let mut state = make_test_map();

    assert_eq!(state.editor.active_tool, EditorTool::Select);

    controller
        .handle_intent(
            &mut state,
            AppIntent::SetEditorToolRequested {
                tool: EditorTool::Connect,
            },
        )
        .unwrap();

    assert_eq!(state.editor.active_tool, EditorTool::Connect);

    controller
        .handle_intent(
            &mut state,
            AppIntent::SetEditorToolRequested {
                tool: EditorTool::AddNode,
            },
        )
        .unwrap();

    assert_eq!(state.editor.active_tool, EditorTool::AddNode);
}

#[test]
fn test_set_default_direction() {
    let mut controller = AppController::new();
    let mut state = make_test_map();

    assert_eq!(state.editor.default_direction, ConnectionDirection::Regular);

    controller
        .handle_intent(
            &mut state,
            AppIntent::SetDefaultDirectionRequested {
                direction: ConnectionDirection::Dual,
            },
        )
        .unwrap();

    assert_eq!(state.editor.default_direction, ConnectionDirection::Dual);
}

#[test]
fn test_connect_tool_flow() {
    let mut controller = AppController::new();
    let mut state = make_test_map();
    let before_conns = state.road_map.as_ref().unwrap().connection_count();

    // Setze Connect-Tool
    controller
        .handle_intent(
            &mut state,
            AppIntent::SetEditorToolRequested {
                tool: EditorTool::Connect,
            },
        )
        .unwrap();

    // Erster Klick: nahe Node 2
    controller
        .handle_intent(
            &mut state,
            AppIntent::ConnectToolNodeClicked {
                world_pos: glam::Vec2::new(10.1, 0.0),
            },
        )
        .unwrap();

    assert_eq!(state.editor.connect_source_node, Some(2));

    // Zweiter Klick: nahe Node 3
    controller
        .handle_intent(
            &mut state,
            AppIntent::ConnectToolNodeClicked {
                world_pos: glam::Vec2::new(20.1, 0.0),
            },
        )
        .unwrap();

    assert_eq!(
        state.road_map.as_ref().unwrap().connection_count(),
        before_conns + 1
    );
    assert!(state
        .road_map
        .as_ref()
        .unwrap()
        .find_connection(2, 3)
        .is_some());
    // Source zurueckgesetzt
    assert_eq!(state.editor.connect_source_node, None);
}

#[test]
fn test_full_editing_workflow() {
    let mut controller = AppController::new();
    let mut state = AppState::new();
    state.road_map = Some(Arc::new(RoadMap::new(3)));
    state.view.viewport_size = [1280.0, 720.0];

    // Node A hinzufuegen
    controller
        .handle_intent(
            &mut state,
            AppIntent::AddNodeRequested {
                world_pos: glam::Vec2::new(0.0, 0.0),
            },
        )
        .unwrap();
    let id_a = *state.selection.selected_node_ids.iter().next().unwrap();

    // Node B hinzufuegen
    controller
        .handle_intent(
            &mut state,
            AppIntent::AddNodeRequested {
                world_pos: glam::Vec2::new(10.0, 0.0),
            },
        )
        .unwrap();
    let id_b = *state.selection.selected_node_ids.iter().next().unwrap();

    assert_eq!(state.road_map.as_ref().unwrap().node_count(), 2);

    // Verbinden als Dual
    controller
        .handle_intent(
            &mut state,
            AppIntent::AddConnectionRequested {
                from_id: id_a,
                to_id: id_b,
                direction: ConnectionDirection::Dual,
                priority: ConnectionPriority::Regular,
            },
        )
        .unwrap();

    assert_eq!(state.road_map.as_ref().unwrap().connection_count(), 1);

    // Richtung aendern auf Reverse
    controller
        .handle_intent(
            &mut state,
            AppIntent::SetConnectionDirectionRequested {
                start_id: id_a,
                end_id: id_b,
                direction: ConnectionDirection::Reverse,
            },
        )
        .unwrap();

    assert_eq!(
        state
            .road_map
            .as_ref()
            .unwrap()
            .find_connection(id_a, id_b)
            .unwrap()
            .direction,
        ConnectionDirection::Reverse
    );

    // Verbindung entfernen
    controller
        .handle_intent(
            &mut state,
            AppIntent::RemoveConnectionBetweenRequested {
                node_a: id_a,
                node_b: id_b,
            },
        )
        .unwrap();

    assert_eq!(state.road_map.as_ref().unwrap().connection_count(), 0);

    // Node B loeschen
    state.selection.ids_mut().clear();
    state.selection.ids_mut().insert(id_b);
    controller
        .handle_intent(&mut state, AppIntent::DeleteSelectedRequested)
        .unwrap();

    assert_eq!(state.road_map.as_ref().unwrap().node_count(), 1);

    // Alles undo bis leer
    controller
        .handle_intent(&mut state, AppIntent::UndoRequested)
        .unwrap(); // undo delete
    assert_eq!(state.road_map.as_ref().unwrap().node_count(), 2);
    controller
        .handle_intent(&mut state, AppIntent::UndoRequested)
        .unwrap(); // undo remove conn
    assert_eq!(state.road_map.as_ref().unwrap().connection_count(), 1);
}

// ═══════════════════════════════════════════════════════════════════
// DRY-Extraktion Tests: Marker-Kaskade & Bulk-Connection-Operationen
// ═══════════════════════════════════════════════════════════════════
