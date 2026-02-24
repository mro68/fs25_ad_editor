use fs25_auto_drive_editor::{AppController, AppIntent, AppState};
use fs25_auto_drive_editor::{
    Connection, ConnectionDirection, ConnectionPriority, MapNode, NodeFlag, RoadMap,
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

    assert!(
        last.contains("SaveFile") && last.contains("path: None"),
        "Unerwarteter letzter Command: {last}"
    );
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

    assert!(
        last.contains("RequestExit"),
        "Unerwarteter letzter Command: {last}"
    );
}

#[test]
fn test_node_pick_requested_with_empty_map_clears_selection_and_logs_command() {
    let mut controller = AppController::new();
    let mut state = AppState::new();
    state.selection.ids_mut().insert(42);

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

    assert!(
        last.contains("SelectNearestNode"),
        "Unerwarteter letzter Command: {last}"
    );
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

    let base_max_distance = state.options.hitbox_radius();
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
    state.selection.ids_mut().insert(1);

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
    state.selection.ids_mut().insert(1);
    state.selection.ids_mut().insert(2);

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

    assert!(
        last.contains("MoveSelectedNodes"),
        "Unerwarteter letzter Command: {last}"
    );
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
    state.selection.ids_mut().insert(1);
    state.selection.ids_mut().insert(2);

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
