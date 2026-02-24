//! Integrationstests für die Editing-Use-Cases:
//! - AddNode mit split_connection_on_place
//! - DeleteSelected mit reconnect_on_delete
//! - ResamplePath (Distanzen-Feature)

use fs25_auto_drive_editor::{AppController, AppIntent, AppState};
use fs25_auto_drive_editor::{
    Connection, ConnectionDirection, ConnectionPriority, MapNode, NodeFlag, RoadMap,
};
use glam::Vec2;
use std::sync::Arc;

/// Erstellt eine RoadMap mit 3 Nodes in einer Linie (A → B → C).
fn map_a_b_c() -> RoadMap {
    let mut map = RoadMap::new(3);
    // A(1) bei x=0, B(2) bei x=100, C(3) bei x=200
    map.add_node(MapNode::new(1, Vec2::new(0.0, 0.0), NodeFlag::Regular));
    map.add_node(MapNode::new(2, Vec2::new(100.0, 0.0), NodeFlag::Regular));
    map.add_node(MapNode::new(3, Vec2::new(200.0, 0.0), NodeFlag::Regular));

    map.add_connection(Connection::new(
        1,
        2,
        ConnectionDirection::default(),
        ConnectionPriority::default(),
        Vec2::new(0.0, 0.0),
        Vec2::new(100.0, 0.0),
    ));
    map.add_connection(Connection::new(
        2,
        3,
        ConnectionDirection::default(),
        ConnectionPriority::default(),
        Vec2::new(100.0, 0.0),
        Vec2::new(200.0, 0.0),
    ));

    map.ensure_spatial_index();
    map
}

// ─── reconnect_on_delete ─────────────────────────────────────────────────────

#[test]
fn test_delete_middle_node_ohne_reconnect_hinterlaesst_keine_verbindung() {
    let mut controller = AppController::new();
    let mut state = AppState::new();
    state.options.reconnect_on_delete = false;
    state.road_map = Some(Arc::new(map_a_b_c()));
    state.view.viewport_size = [1280.0, 720.0];

    // Node 2 selektieren und löschen
    state.selection.ids_mut().insert(2);

    controller
        .handle_intent(&mut state, AppIntent::DeleteSelectedRequested)
        .expect("DeleteSelectedRequested darf nicht paniken");

    let rm = state.road_map.as_ref().unwrap();
    assert!(!rm.nodes.contains_key(&2), "Node 2 muss gelöscht sein");
    // Ohne Reconnect: keine direkte Verbindung A→C
    assert!(
        !rm.has_connection(1, 3),
        "Ohne reconnect_on_delete darf keine A→C Verbindung entstehen"
    );
}

#[test]
fn test_delete_middle_node_mit_reconnect_verbindet_a_und_c() {
    let mut controller = AppController::new();
    let mut state = AppState::new();
    state.options.reconnect_on_delete = true;
    state.road_map = Some(Arc::new(map_a_b_c()));
    state.view.viewport_size = [1280.0, 720.0];

    // Node 2 selektieren und löschen
    state.selection.ids_mut().insert(2);

    controller
        .handle_intent(&mut state, AppIntent::DeleteSelectedRequested)
        .expect("DeleteSelectedRequested darf nicht paniken");

    let rm = state.road_map.as_ref().unwrap();
    assert!(!rm.nodes.contains_key(&2), "Node 2 muss gelöscht sein");
    assert!(
        rm.has_connection(1, 3),
        "Mit reconnect_on_delete muss A→C verbunden sein"
    );
}

#[test]
fn test_delete_endnode_mit_reconnect_keine_neue_verbindung() {
    // A hat keinen Vorgänger → Reconnect nicht möglich
    let mut controller = AppController::new();
    let mut state = AppState::new();
    state.options.reconnect_on_delete = true;
    state.road_map = Some(Arc::new(map_a_b_c()));
    state.view.viewport_size = [1280.0, 720.0];

    state.selection.ids_mut().insert(1); // A löschen

    controller
        .handle_intent(&mut state, AppIntent::DeleteSelectedRequested)
        .expect("DeleteSelectedRequested darf nicht paniken");

    let rm = state.road_map.as_ref().unwrap();
    assert!(!rm.nodes.contains_key(&1), "Node 1 (A) muss gelöscht sein");
    // Node 2 und 3 bleiben, aber keine neue Verbindung (B hat keinen neuen Vorgänger)
    assert!(rm.nodes.contains_key(&2), "Node 2 muss erhalten bleiben");
}

// ─── split_connection_on_place ────────────────────────────────────────────────

#[test]
fn test_add_node_ohne_split_erstellt_keinen_split() {
    let mut controller = AppController::new();
    let mut state = AppState::new();
    state.options.split_connection_on_place = false;
    state.options.snap_radius = 20.0;

    let mut map = RoadMap::new(2);
    map.add_node(MapNode::new(1, Vec2::new(0.0, 0.0), NodeFlag::Regular));
    map.add_node(MapNode::new(2, Vec2::new(200.0, 0.0), NodeFlag::Regular));
    map.add_connection(Connection::new(
        1,
        2,
        ConnectionDirection::default(),
        ConnectionPriority::default(),
        Vec2::new(0.0, 0.0),
        Vec2::new(200.0, 0.0),
    ));
    map.ensure_spatial_index();
    state.road_map = Some(Arc::new(map));
    state.view.viewport_size = [1280.0, 720.0];

    // Klick mitten auf die Verbindung
    controller
        .handle_intent(
            &mut state,
            AppIntent::AddNodeRequested {
                world_pos: Vec2::new(100.0, 0.0),
            },
        )
        .expect("AddNodeRequested darf nicht paniken");

    let rm = state.road_map.as_ref().unwrap();
    // Verbindung 1→2 bleibt erhalten (kein Split)
    assert!(
        rm.has_connection(1, 2),
        "Ohne split_connection darf die Verbindung 1→2 nicht entfernt werden"
    );
}

#[test]
fn test_add_node_mit_split_teilt_verbindung() {
    let mut controller = AppController::new();
    let mut state = AppState::new();
    state.options.split_connection_on_place = true;
    state.options.snap_radius = 20.0;

    let mut map = RoadMap::new(2);
    map.add_node(MapNode::new(1, Vec2::new(0.0, 0.0), NodeFlag::Regular));
    map.add_node(MapNode::new(2, Vec2::new(200.0, 0.0), NodeFlag::Regular));
    map.add_connection(Connection::new(
        1,
        2,
        ConnectionDirection::default(),
        ConnectionPriority::default(),
        Vec2::new(0.0, 0.0),
        Vec2::new(200.0, 0.0),
    ));
    map.ensure_spatial_index();
    state.road_map = Some(Arc::new(map));
    state.view.viewport_size = [1280.0, 720.0];

    // Klick mitten auf die Verbindung (exakt auf Linie, Distanz = 0)
    controller
        .handle_intent(
            &mut state,
            AppIntent::AddNodeRequested {
                world_pos: Vec2::new(100.0, 0.0),
            },
        )
        .expect("AddNodeRequested darf nicht paniken");

    let rm = state.road_map.as_ref().unwrap();
    // Es muss ein neuer (dritter) Node entstanden sein
    assert_eq!(
        rm.nodes.len(),
        3,
        "Nach Split müssen 3 Nodes existieren (war: 2)"
    );
    // Die alte Verbindung 1→2 darf nicht mehr existieren
    assert!(
        !rm.has_connection(1, 2),
        "Nach Split darf die Originalverbindung 1→2 nicht mehr existieren"
    );
}

// ─── ResamplePath ─────────────────────────────────────────────────────────────

#[test]
fn test_resample_path_ohne_selektion_keine_aenderung() {
    let mut controller = AppController::new();
    let mut state = AppState::new();
    state.road_map = Some(Arc::new(map_a_b_c()));
    state.view.viewport_size = [1280.0, 720.0];
    // Keine Nodes selektiert

    let node_count_vorher = state.road_map.as_ref().unwrap().nodes.len();

    controller
        .handle_intent(&mut state, AppIntent::ResamplePathRequested)
        .expect("ResamplePathRequested darf ohne Selektion nicht paniken");

    let node_count_nachher = state.road_map.as_ref().unwrap().nodes.len();
    assert_eq!(
        node_count_vorher, node_count_nachher,
        "Ohne Selektion darf ResamplePath die Map nicht verändern"
    );
}

#[test]
fn test_resample_path_nach_distanz_erzeugt_neue_nodes() {
    let mut controller = AppController::new();
    let mut state = AppState::new();

    state.road_map = Some(Arc::new(map_a_b_c()));
    state.view.viewport_size = [1280.0, 720.0];

    // Alle 3 Nodes selektieren
    state.selection.ids_mut().extend([1, 2, 3]);

    // Distanz = 50.0 → bei Gesamtlänge 200.0 sollten ~4-5 Nodes entstehen
    state.ui.distanzen.by_count = false;
    state.ui.distanzen.distance = 50.0;

    controller
        .handle_intent(&mut state, AppIntent::ResamplePathRequested)
        .expect("ResamplePathRequested darf nicht paniken");

    let rm = state.road_map.as_ref().unwrap();
    // Mindestens 3 Nodes (keine Regression), aber mehr als vorher
    assert!(
        rm.nodes.len() >= 3,
        "Nach Resample müssen mindestens 3 Nodes vorhanden sein, haben: {}",
        rm.nodes.len()
    );
}

// ─── AddNode auf existierenden Node → Selektion ──────────────────────────────

#[test]
fn test_add_node_auf_existierenden_node_selektiert_statt_erstellt() {
    let mut controller = AppController::new();
    let mut state = AppState::new();
    state.options.snap_radius = 20.0;

    let mut map = RoadMap::new(1);
    map.add_node(MapNode::new(1, Vec2::new(50.0, 50.0), NodeFlag::Regular));
    map.ensure_spatial_index();
    state.road_map = Some(Arc::new(map));
    state.view.viewport_size = [1280.0, 720.0];

    // Klick auf Position nahe Node 1
    controller
        .handle_intent(
            &mut state,
            AppIntent::AddNodeRequested {
                world_pos: Vec2::new(50.0, 50.0),
            },
        )
        .expect("AddNodeRequested darf nicht paniken");

    let rm = state.road_map.as_ref().unwrap();
    // Kein neuer Node erstellt
    assert_eq!(rm.nodes.len(), 1, "Es darf kein neuer Node erstellt werden");
    // Node 1 ist selektiert
    assert!(
        state.selection.selected_node_ids.contains(&1),
        "Node 1 muss selektiert sein"
    );
}

#[test]
fn test_add_node_auf_leerer_flaeche_erstellt_neuen_node() {
    let mut controller = AppController::new();
    let mut state = AppState::new();
    state.options.snap_radius = 20.0;

    let mut map = RoadMap::new(1);
    map.add_node(MapNode::new(1, Vec2::new(50.0, 50.0), NodeFlag::Regular));
    map.ensure_spatial_index();
    state.road_map = Some(Arc::new(map));
    state.view.viewport_size = [1280.0, 720.0];

    // Klick weit weg von Node 1
    controller
        .handle_intent(
            &mut state,
            AppIntent::AddNodeRequested {
                world_pos: Vec2::new(500.0, 500.0),
            },
        )
        .expect("AddNodeRequested darf nicht paniken");

    let rm = state.road_map.as_ref().unwrap();
    assert_eq!(rm.nodes.len(), 2, "Ein neuer Node muss erstellt werden");
}
