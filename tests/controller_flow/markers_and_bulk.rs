use fs25_auto_drive_editor::{AppController, AppIntent, AppState};
use fs25_auto_drive_editor::{
    Connection, ConnectionDirection, ConnectionPriority, MapMarker, MapNode, NodeFlag, RoadMap,
};
use std::sync::Arc;

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
    state.selection.ids_mut().insert(1);
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

    state.selection.ids_mut().insert(1);
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
    state.selection.ids_mut().insert(1);
    state.selection.ids_mut().insert(2);
    state.selection.ids_mut().insert(3);

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

    state.selection.ids_mut().insert(1);
    state.selection.ids_mut().insert(2);
    state.selection.ids_mut().insert(3);

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

    state.selection.ids_mut().insert(1);
    state.selection.ids_mut().insert(2);

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
    state.selection.ids_mut().insert(1);
    state.selection.ids_mut().insert(2);

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
