use fs25_auto_drive_editor::EditorTool;
use fs25_auto_drive_editor::{AppCommand, AppController, AppIntent, AppState};
use fs25_auto_drive_editor::{
    Connection, ConnectionDirection, ConnectionPriority, MapMarker, MapNode, NodeFlag, RoadMap,
};
use std::sync::Arc;

#[test]
fn test_save_requested_logs_save_command_without_panic() {
    let mut controller = AppController::new();
    let mut state = AppState::new();

    controller
        .handle_intent(&mut state, AppIntent::SaveRequested)
        .expect("SaveRequested sollte ohne Fehler durchlaufen");

    let last = state
        .command_log
        .entries()
        .last()
        .expect("Es sollte ein Command geloggt sein");

    match last {
        AppCommand::SaveFile { path } => assert!(path.is_empty()),
        other => panic!("Unerwarteter letzter Command: {other:?}"),
    }
}

#[test]
fn test_exit_requested_sets_exit_flag_and_logs_command() {
    let mut controller = AppController::new();
    let mut state = AppState::new();

    assert!(!state.should_exit);

    controller
        .handle_intent(&mut state, AppIntent::ExitRequested)
        .expect("ExitRequested sollte ohne Fehler durchlaufen");

    assert!(state.should_exit);

    let last = state
        .command_log
        .entries()
        .last()
        .expect("Es sollte ein Command geloggt sein");

    match last {
        AppCommand::RequestExit => {}
        other => panic!("Unerwarteter letzter Command: {other:?}"),
    }
}

#[test]
fn test_node_pick_requested_with_empty_map_clears_selection_and_logs_command() {
    let mut controller = AppController::new();
    let mut state = AppState::new();
    state.selection.selected_node_ids.insert(42);

    controller
        .handle_intent(
            &mut state,
            AppIntent::NodePickRequested {
                world_pos: glam::Vec2::new(0.0, 0.0),
                additive: false,
                extend_path: false,
            },
        )
        .expect("NodePickRequested sollte bei leerer Map robust sein");

    assert!(state.selection.selected_node_ids.is_empty());

    let last = state
        .command_log
        .entries()
        .last()
        .expect("Es sollte ein Command geloggt sein");

    match last {
        AppCommand::SelectNearestNode { .. } => {}
        other => panic!("Unerwarteter letzter Command: {other:?}"),
    }
}

#[test]
fn test_additive_node_pick_selects_multiple_nodes() {
    let mut controller = AppController::new();
    let mut state = AppState::new();

    let mut map = RoadMap::new(2);
    map.add_node(MapNode::new(
        1,
        glam::Vec2::new(0.0, 0.0),
        NodeFlag::Regular,
    ));
    map.add_node(MapNode::new(
        2,
        glam::Vec2::new(100.0, 0.0),
        NodeFlag::Regular,
    ));
    map.ensure_spatial_index();
    state.road_map = Some(Arc::new(map));
    state.view.viewport_size = [1280.0, 720.0];

    controller
        .handle_intent(
            &mut state,
            AppIntent::NodePickRequested {
                world_pos: glam::Vec2::new(0.1, 0.0),
                additive: false,
                extend_path: false,
            },
        )
        .expect("Erster Pick sollte funktionieren");

    controller
        .handle_intent(
            &mut state,
            AppIntent::NodePickRequested {
                world_pos: glam::Vec2::new(100.1, 0.0),
                additive: true,
                extend_path: false,
            },
        )
        .expect("Additiver Pick sollte funktionieren");

    assert!(state.selection.selected_node_ids.contains(&1));
    assert!(state.selection.selected_node_ids.contains(&2));
    assert_eq!(state.selection.selected_node_ids.len(), 2);
}

#[test]
fn test_click_window_larger_for_selected_nodes() {
    let mut controller = AppController::new();
    let mut state = AppState::new();

    let mut map = RoadMap::new(2);
    map.add_node(MapNode::new(
        1,
        glam::Vec2::new(0.0, 0.0),
        NodeFlag::Regular,
    ));
    map.ensure_spatial_index();
    state.road_map = Some(Arc::new(map));
    state.view.viewport_size = [1280.0, 720.0];

    let viewport_height = state.view.viewport_size[1].max(1.0);
    let base_max_distance = (fs25_auto_drive_editor::shared::options::SELECTION_PICK_RADIUS_PX
        * 2.0
        * fs25_auto_drive_editor::Camera2D::BASE_WORLD_EXTENT)
        / (fs25_auto_drive_editor::Camera2D::ZOOM_MAX * viewport_height);
    let increased_max_distance =
        base_max_distance * fs25_auto_drive_editor::shared::options::SELECTION_SIZE_FACTOR;

    // Wähle einen Punkt *zwischen* Basis- und erweitertem Radius.
    let between = (base_max_distance + increased_max_distance) / 2.0;

    // Ohne bestehende Selektion: Klick außerhalb Basis-Radius wählt nicht.
    controller
        .handle_intent(
            &mut state,
            AppIntent::NodePickRequested {
                world_pos: glam::Vec2::new(between, 0.0),
                additive: false,
                extend_path: false,
            },
        )
        .expect("NodePickRequested sollte ohne Fehler durchlaufen");

    assert!(state.selection.selected_node_ids.is_empty());

    // Wenn Node bereits selektiert ist, ist das Click-Fenster größer — Click trifft.
    state.selection.selected_node_ids.insert(1);

    controller
        .handle_intent(
            &mut state,
            AppIntent::NodePickRequested {
                world_pos: glam::Vec2::new(between, 0.0),
                additive: false,
                extend_path: false,
            },
        )
        .expect("NodePickRequested sollte nun den selektierten Node treffen");

    assert!(state.selection.selected_node_ids.contains(&1));
}

#[test]
fn test_move_selected_nodes_requested_moves_all_selected_nodes() {
    let mut controller = AppController::new();
    let mut state = AppState::new();

    let mut map = RoadMap::new(2);
    map.add_node(MapNode::new(
        1,
        glam::Vec2::new(0.0, 0.0),
        NodeFlag::Regular,
    ));
    map.add_node(MapNode::new(
        2,
        glam::Vec2::new(10.0, 5.0),
        NodeFlag::Regular,
    ));
    state.road_map = Some(Arc::new(map));
    state.selection.selected_node_ids.insert(1);
    state.selection.selected_node_ids.insert(2);

    controller
        .handle_intent(
            &mut state,
            AppIntent::MoveSelectedNodesRequested {
                delta_world: glam::Vec2::new(2.0, -1.0),
            },
        )
        .expect("MoveSelectedNodesRequested sollte funktionieren");

    let road_map = state.road_map.as_ref().expect("map vorhanden");
    let node1 = road_map.nodes.get(&1).expect("node 1 vorhanden");
    let node2 = road_map.nodes.get(&2).expect("node 2 vorhanden");

    assert_eq!(node1.position, glam::Vec2::new(2.0, -1.0));
    assert_eq!(node2.position, glam::Vec2::new(12.0, 4.0));

    let last = state
        .command_log
        .entries()
        .last()
        .expect("Es sollte ein Command geloggt sein");

    match last {
        AppCommand::MoveSelectedNodes { delta_world } => {
            assert_eq!(*delta_world, glam::Vec2::new(2.0, -1.0));
        }
        other => panic!("Unerwarteter letzter Command: {other:?}"),
    }
}

#[test]
fn test_undo_redo_moves_revert_and_restore_positions() {
    let mut controller = AppController::new();
    let mut state = AppState::new();

    let mut map = RoadMap::new(2);
    map.add_node(MapNode::new(
        1,
        glam::Vec2::new(0.0, 0.0),
        NodeFlag::Regular,
    ));
    map.add_node(MapNode::new(
        2,
        glam::Vec2::new(10.0, 5.0),
        NodeFlag::Regular,
    ));
    state.road_map = Some(Arc::new(map));
    state.selection.selected_node_ids.insert(1);
    state.selection.selected_node_ids.insert(2);

    // Begin move (snapshot should be recorded once)
    controller
        .handle_intent(&mut state, AppIntent::BeginMoveSelectedNodesRequested)
        .expect("BeginMoveSelectedNodesRequested sollte funktionieren");

    // Move once (during drag)
    controller
        .handle_intent(
            &mut state,
            AppIntent::MoveSelectedNodesRequested {
                delta_world: glam::Vec2::new(3.0, 1.0),
            },
        )
        .expect("MoveSelectedNodesRequested sollte funktionieren");

    // End move (mouse released)
    controller
        .handle_intent(&mut state, AppIntent::EndMoveSelectedNodesRequested)
        .expect("EndMoveSelectedNodesRequested sollte funktionieren");

    let after_move = state.road_map.as_ref().unwrap();
    assert_eq!(
        after_move.nodes.get(&1).unwrap().position,
        glam::Vec2::new(3.0, 1.0)
    );

    // Undo
    controller
        .handle_intent(&mut state, AppIntent::UndoRequested)
        .expect("UndoRequested sollte funktionieren");

    let after_undo = state.road_map.as_ref().unwrap();
    assert_eq!(
        after_undo.nodes.get(&1).unwrap().position,
        glam::Vec2::new(0.0, 0.0)
    );

    // Redo
    controller
        .handle_intent(&mut state, AppIntent::RedoRequested)
        .expect("RedoRequested sollte funktionieren");

    let after_redo = state.road_map.as_ref().unwrap();
    assert_eq!(
        after_redo.nodes.get(&1).unwrap().position,
        glam::Vec2::new(3.0, 1.0)
    );
}

#[test]
fn test_shift_second_pick_selects_nodes_between_two_nodes() {
    let mut controller = AppController::new();
    let mut state = AppState::new();

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
    map.add_connection(Connection::new(
        2,
        3,
        ConnectionDirection::Regular,
        ConnectionPriority::Regular,
        glam::Vec2::new(10.0, 0.0),
        glam::Vec2::new(20.0, 0.0),
    ));

    map.ensure_spatial_index();
    state.road_map = Some(Arc::new(map));
    state.view.viewport_size = [1280.0, 720.0];

    controller
        .handle_intent(
            &mut state,
            AppIntent::NodePickRequested {
                world_pos: glam::Vec2::new(0.1, 0.0),
                additive: false,
                extend_path: false,
            },
        )
        .expect("Erster Pick sollte funktionieren");

    controller
        .handle_intent(
            &mut state,
            AppIntent::NodePickRequested {
                world_pos: glam::Vec2::new(20.1, 0.0),
                additive: true,
                extend_path: true,
            },
        )
        .expect("Shift-Pick sollte Pfad selektieren");

    assert!(state.selection.selected_node_ids.contains(&1));
    assert!(state.selection.selected_node_ids.contains(&2));
    assert!(state.selection.selected_node_ids.contains(&3));
    assert_eq!(state.selection.selected_node_ids.len(), 3);
}

#[test]
fn test_select_nodes_in_rect_requested_selects_nodes_in_rectangle() {
    let mut controller = AppController::new();
    let mut state = AppState::new();

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
        glam::Vec2::new(30.0, 0.0),
        NodeFlag::Regular,
    ));
    map.ensure_spatial_index();
    state.road_map = Some(Arc::new(map));

    controller
        .handle_intent(
            &mut state,
            AppIntent::SelectNodesInRectRequested {
                min: glam::Vec2::new(-1.0, -1.0),
                max: glam::Vec2::new(15.0, 1.0),
                additive: false,
            },
        )
        .expect("Rechteckselektion sollte funktionieren");

    assert!(state.selection.selected_node_ids.contains(&1));
    assert!(state.selection.selected_node_ids.contains(&2));
    assert!(!state.selection.selected_node_ids.contains(&3));
}

#[test]
fn test_ctrl_additive_pick_does_not_select_intermediate_path_nodes() {
    let mut controller = AppController::new();
    let mut state = AppState::new();

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
    map.add_connection(Connection::new(
        2,
        3,
        ConnectionDirection::Regular,
        ConnectionPriority::Regular,
        glam::Vec2::new(10.0, 0.0),
        glam::Vec2::new(20.0, 0.0),
    ));

    map.ensure_spatial_index();
    state.road_map = Some(Arc::new(map));
    state.view.viewport_size = [1280.0, 720.0];

    controller
        .handle_intent(
            &mut state,
            AppIntent::NodePickRequested {
                world_pos: glam::Vec2::new(0.1, 0.0),
                additive: false,
                extend_path: false,
            },
        )
        .expect("Erster Pick sollte funktionieren");

    controller
        .handle_intent(
            &mut state,
            AppIntent::NodePickRequested {
                world_pos: glam::Vec2::new(20.1, 0.0),
                additive: true,
                extend_path: false,
            },
        )
        .expect("Ctrl-Pick sollte funktionieren");

    assert!(state.selection.selected_node_ids.contains(&1));
    assert!(state.selection.selected_node_ids.contains(&3));
    assert!(!state.selection.selected_node_ids.contains(&2));
}

#[test]
fn test_select_nodes_in_lasso_requested_selects_nodes_in_polygon() {
    let mut controller = AppController::new();
    let mut state = AppState::new();

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
        glam::Vec2::new(30.0, 0.0),
        NodeFlag::Regular,
    ));
    map.ensure_spatial_index();
    state.road_map = Some(Arc::new(map));

    controller
        .handle_intent(
            &mut state,
            AppIntent::SelectNodesInLassoRequested {
                polygon: vec![
                    glam::Vec2::new(-1.0, -1.0),
                    glam::Vec2::new(15.0, -1.0),
                    glam::Vec2::new(15.0, 1.0),
                    glam::Vec2::new(-1.0, 1.0),
                ],
                additive: false,
            },
        )
        .expect("Lasso-Selektion sollte funktionieren");

    assert!(state.selection.selected_node_ids.contains(&1));
    assert!(state.selection.selected_node_ids.contains(&2));
    assert!(!state.selection.selected_node_ids.contains(&3));
}

#[test]
fn test_node_segment_between_intersections_requested_selects_corridor() {
    let mut controller = AppController::new();
    let mut state = AppState::new();

    let mut map = RoadMap::new(3);
    map.add_node(MapNode::new(
        10,
        glam::Vec2::new(-20.0, 0.0),
        NodeFlag::Regular,
    ));
    map.add_node(MapNode::new(
        11,
        glam::Vec2::new(-10.0, 0.0),
        NodeFlag::Regular,
    ));
    map.add_node(MapNode::new(
        12,
        glam::Vec2::new(0.0, 0.0),
        NodeFlag::Regular,
    ));
    map.add_node(MapNode::new(
        13,
        glam::Vec2::new(10.0, 0.0),
        NodeFlag::Regular,
    ));
    map.add_node(MapNode::new(
        14,
        glam::Vec2::new(20.0, 0.0),
        NodeFlag::Regular,
    ));
    map.add_node(MapNode::new(
        20,
        glam::Vec2::new(-20.0, 10.0),
        NodeFlag::Regular,
    ));
    map.add_node(MapNode::new(
        21,
        glam::Vec2::new(20.0, 10.0),
        NodeFlag::Regular,
    ));
    map.add_node(MapNode::new(
        22,
        glam::Vec2::new(-20.0, -10.0),
        NodeFlag::Regular,
    ));
    map.add_node(MapNode::new(
        23,
        glam::Vec2::new(20.0, -10.0),
        NodeFlag::Regular,
    ));

    map.add_connection(Connection::new(
        10,
        11,
        ConnectionDirection::Regular,
        ConnectionPriority::Regular,
        glam::Vec2::new(-20.0, 0.0),
        glam::Vec2::new(-10.0, 0.0),
    ));
    map.add_connection(Connection::new(
        11,
        12,
        ConnectionDirection::Regular,
        ConnectionPriority::Regular,
        glam::Vec2::new(-10.0, 0.0),
        glam::Vec2::new(0.0, 0.0),
    ));
    map.add_connection(Connection::new(
        12,
        13,
        ConnectionDirection::Regular,
        ConnectionPriority::Regular,
        glam::Vec2::new(0.0, 0.0),
        glam::Vec2::new(10.0, 0.0),
    ));
    map.add_connection(Connection::new(
        13,
        14,
        ConnectionDirection::Regular,
        ConnectionPriority::Regular,
        glam::Vec2::new(10.0, 0.0),
        glam::Vec2::new(20.0, 0.0),
    ));
    map.add_connection(Connection::new(
        10,
        20,
        ConnectionDirection::Regular,
        ConnectionPriority::Regular,
        glam::Vec2::new(-20.0, 0.0),
        glam::Vec2::new(-20.0, 10.0),
    ));
    map.add_connection(Connection::new(
        14,
        21,
        ConnectionDirection::Regular,
        ConnectionPriority::Regular,
        glam::Vec2::new(20.0, 0.0),
        glam::Vec2::new(20.0, 10.0),
    ));
    map.add_connection(Connection::new(
        10,
        22,
        ConnectionDirection::Regular,
        ConnectionPriority::Regular,
        glam::Vec2::new(-20.0, 0.0),
        glam::Vec2::new(-20.0, -10.0),
    ));
    map.add_connection(Connection::new(
        14,
        23,
        ConnectionDirection::Regular,
        ConnectionPriority::Regular,
        glam::Vec2::new(20.0, 0.0),
        glam::Vec2::new(20.0, -10.0),
    ));

    map.ensure_spatial_index();
    state.road_map = Some(Arc::new(map));
    state.view.viewport_size = [1280.0, 720.0];

    controller
        .handle_intent(
            &mut state,
            AppIntent::NodeSegmentBetweenIntersectionsRequested {
                world_pos: glam::Vec2::new(0.2, 0.0),
                additive: false,
            },
        )
        .expect("Segment-Selektion sollte funktionieren");

    for node_id in [10_u64, 11, 12, 13, 14] {
        assert!(state.selection.selected_node_ids.contains(&node_id));
    }
    assert!(!state.selection.selected_node_ids.contains(&20));
    assert!(!state.selection.selected_node_ids.contains(&21));
    assert!(!state.selection.selected_node_ids.contains(&22));
    assert!(!state.selection.selected_node_ids.contains(&23));
}

// ═══════════════════════════════════════════════════════════════════
// Editing-Tests: Add Node, Delete, Connections, Direction, Undo/Redo
// ═══════════════════════════════════════════════════════════════════

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

    state.selection.selected_node_ids.insert(1);

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
    state.selection.selected_node_ids.insert(1);

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
    state.selection.selected_node_ids.clear();
    state.selection.selected_node_ids.insert(id_b);
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

#[test]
fn test_delete_node_with_marker_cascades_marker_removal() {
    let mut controller = AppController::new();
    let mut state = AppState::new();

    let mut map = RoadMap::new(2);
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
    map.add_map_marker(MapMarker::new(
        1,
        "Hof".to_string(),
        "Farmen".to_string(),
        1,
        false,
    ));
    map.ensure_spatial_index();
    state.road_map = Some(Arc::new(map));

    // Marker existiert
    assert!(state.road_map.as_ref().unwrap().has_marker(1));
    assert_eq!(state.road_map.as_ref().unwrap().marker_count(), 1);

    // Node 1 löschen → Marker muss mit entfernt werden
    state.selection.selected_node_ids.insert(1);
    controller
        .handle_intent(&mut state, AppIntent::DeleteSelectedRequested)
        .expect("Löschen mit Marker sollte funktionieren");

    assert!(!state.road_map.as_ref().unwrap().nodes.contains_key(&1));
    assert_eq!(state.road_map.as_ref().unwrap().marker_count(), 0);
}

#[test]
fn test_delete_node_with_marker_is_undoable() {
    let mut controller = AppController::new();
    let mut state = AppState::new();

    let mut map = RoadMap::new(1);
    map.add_node(MapNode::new(
        1,
        glam::Vec2::new(5.0, 5.0),
        NodeFlag::Regular,
    ));
    map.add_map_marker(MapMarker::new(
        1,
        "Laden".to_string(),
        "Shops".to_string(),
        1,
        false,
    ));
    map.ensure_spatial_index();
    state.road_map = Some(Arc::new(map));

    state.selection.selected_node_ids.insert(1);
    controller
        .handle_intent(&mut state, AppIntent::DeleteSelectedRequested)
        .unwrap();

    assert_eq!(state.road_map.as_ref().unwrap().node_count(), 0);
    assert_eq!(state.road_map.as_ref().unwrap().marker_count(), 0);

    // Undo → Node und Marker wiederhergestellt
    controller
        .handle_intent(&mut state, AppIntent::UndoRequested)
        .unwrap();

    assert_eq!(state.road_map.as_ref().unwrap().node_count(), 1);
    assert_eq!(state.road_map.as_ref().unwrap().marker_count(), 1);
    assert!(state.road_map.as_ref().unwrap().has_marker(1));
}

fn make_connected_map() -> AppState {
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
    map.add_connection(Connection::new(
        2,
        3,
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

#[test]
fn test_set_all_connections_direction_between_selected() {
    let mut controller = AppController::new();
    let mut state = make_connected_map();

    // Alle 3 Nodes selektieren
    state.selection.selected_node_ids.insert(1);
    state.selection.selected_node_ids.insert(2);
    state.selection.selected_node_ids.insert(3);

    controller
        .handle_intent(
            &mut state,
            AppIntent::SetAllConnectionsDirectionBetweenSelectedRequested {
                direction: ConnectionDirection::Dual,
            },
        )
        .expect("Bulk-Richtungsänderung sollte funktionieren");

    let rm = state.road_map.as_ref().unwrap();
    assert_eq!(
        rm.find_connection(1, 2).unwrap().direction,
        ConnectionDirection::Dual
    );
    assert_eq!(
        rm.find_connection(2, 3).unwrap().direction,
        ConnectionDirection::Dual
    );
}

#[test]
fn test_invert_all_connections_between_selected() {
    let mut controller = AppController::new();
    let mut state = make_connected_map();

    state.selection.selected_node_ids.insert(1);
    state.selection.selected_node_ids.insert(2);
    state.selection.selected_node_ids.insert(3);

    controller
        .handle_intent(
            &mut state,
            AppIntent::InvertAllConnectionsBetweenSelectedRequested,
        )
        .expect("Verbindungs-Invertierung sollte funktionieren");

    let rm = state.road_map.as_ref().unwrap();
    // Nach Invertierung: start/end getauscht → alte (1,2) ist jetzt (2,1)
    assert!(rm.find_connection(2, 1).is_some());
    assert!(rm.find_connection(3, 2).is_some());
    // Alte Richtung weg
    assert!(rm.find_connection(1, 2).is_none());
    assert!(rm.find_connection(2, 3).is_none());
}

#[test]
fn test_remove_all_connections_between_selected() {
    let mut controller = AppController::new();
    let mut state = make_connected_map();

    state.selection.selected_node_ids.insert(1);
    state.selection.selected_node_ids.insert(2);

    controller
        .handle_intent(
            &mut state,
            AppIntent::RemoveAllConnectionsBetweenSelectedRequested,
        )
        .expect("Bulk-Entfernung sollte funktionieren");

    let rm = state.road_map.as_ref().unwrap();
    // Verbindung 1→2 entfernt, 2→3 bleibt (3 nicht selektiert)
    assert!(rm.find_connection(1, 2).is_none());
    assert!(rm.find_connection(2, 3).is_some());
}

#[test]
fn test_connect_selected_nodes() {
    let mut controller = AppController::new();
    let mut state = AppState::new();

    let mut map = RoadMap::new(2);
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
    map.ensure_spatial_index();
    state.road_map = Some(Arc::new(map));
    state.view.viewport_size = [1280.0, 720.0];

    assert_eq!(state.road_map.as_ref().unwrap().connection_count(), 0);

    // Genau 2 Nodes selektieren → ConnectSelectedNodes verbindet sie
    state.selection.selected_node_ids.insert(1);
    state.selection.selected_node_ids.insert(2);

    controller
        .handle_intent(&mut state, AppIntent::ConnectSelectedNodesRequested)
        .expect("ConnectSelectedNodes sollte funktionieren");

    let rm = state.road_map.as_ref().unwrap();
    assert_eq!(rm.connection_count(), 1);
    // HashSet-Iteration ist nicht-deterministisch → beide Richtungen prüfen
    assert!(
        !rm.find_connections_between(1, 2).is_empty(),
        "Verbindung zwischen 1 und 2 erwartet (in beliebiger Richtung)"
    );
}
