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

    // Keine neue Connection hinzugefügt
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
    // Source zurückgesetzt
    assert_eq!(state.editor.connect_source_node, None);
}

#[test]
fn test_full_editing_workflow() {
    let mut controller = AppController::new();
    let mut state = AppState::new();
    state.road_map = Some(Arc::new(RoadMap::new(3)));
    state.view.viewport_size = [1280.0, 720.0];

    // Node A hinzufügen
    controller
        .handle_intent(
            &mut state,
            AppIntent::AddNodeRequested {
                world_pos: glam::Vec2::new(0.0, 0.0),
            },
        )
        .unwrap();
    let id_a = *state.selection.selected_node_ids.iter().next().unwrap();

    // Node B hinzufügen
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

    // Richtung ändern auf Reverse
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

    // Node B löschen
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
