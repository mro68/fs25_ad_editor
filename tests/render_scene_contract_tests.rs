//! Contract-Test zwischen Engine-Projektionen und Render-Core.
//!
//! Prueft strukturell/typbasiert, dass `RenderScene` (der explizite
//! Uebergabevertrag zwischen App-Layer und Renderer, siehe
//! `crates/fs25_auto_drive_engine/src/shared/render_scene.rs`) konsistente,
//! fuer den Renderer verwertbare Daten liefert. Bewusst ohne echte
//! wgpu-Geraeteerstellung/-Rendering: CI hat i. d. R. keine GPU, daher bleibt
//! dieser Test rein struktureller Natur (Node-/Connection-Zaehlungen,
//! referenzielle Integritaet, endliche Koordinaten) statt eines Hard-Require
//! auf einen wgpu-Adapter.

use fs25_auto_drive_editor::app::{self, AppState};
use fs25_auto_drive_editor::core::{
    Connection, ConnectionDirection, ConnectionPriority, MapNode, NodeFlag, RoadMap,
};
use glam::Vec2;
use std::sync::Arc;

fn sample_state() -> AppState {
    let mut map = RoadMap::new(3);
    map.add_node(MapNode::new(1, Vec2::new(0.0, 0.0), NodeFlag::Regular));
    map.add_node(MapNode::new(2, Vec2::new(10.0, 0.0), NodeFlag::Regular));
    map.add_node(MapNode::new(3, Vec2::new(10.0, 10.0), NodeFlag::Regular));
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
        ConnectionDirection::Dual,
        ConnectionPriority::SubPriority,
        Vec2::new(10.0, 0.0),
        Vec2::new(10.0, 10.0),
    ));
    map.ensure_spatial_index();

    let mut state = AppState::new();
    state.road_map = Some(Arc::new(map));
    state
}

#[test]
fn render_scene_reflects_node_and_connection_counts_from_engine_road_map() {
    let state = sample_state();
    let viewport_size = [1024.0, 768.0];

    let scene = app::build_render_scene(&state, viewport_size);

    assert!(
        scene.has_map(),
        "RenderScene muss eine Map exponieren, wenn eine RoadMap geladen ist"
    );
    let render_map = scene
        .map()
        .expect("RenderScene::map() muss Some sein, wenn has_map() true ist");

    assert_eq!(
        render_map.node_count(),
        3,
        "Node-Anzahl muss mit der Engine-RoadMap uebereinstimmen"
    );
    assert_eq!(
        render_map.connection_count(),
        2,
        "Connection-Anzahl muss mit der Engine-RoadMap uebereinstimmen"
    );
    assert_eq!(scene.viewport_size(), viewport_size);
}

#[test]
fn render_scene_connections_reference_only_existing_node_ids() {
    let state = sample_state();
    let scene = app::build_render_scene(&state, [800.0, 600.0]);
    let render_map = scene.map().expect("RenderMap muss vorhanden sein");

    let known_ids: Vec<u64> = render_map.nodes().map(|node| node.id).collect();

    for connection in render_map.connections() {
        assert!(
            known_ids.contains(&connection.start_id),
            "Connection.start_id {} referenziert keinen bekannten Node",
            connection.start_id
        );
        assert!(
            known_ids.contains(&connection.end_id),
            "Connection.end_id {} referenziert keinen bekannten Node",
            connection.end_id
        );
    }
}

#[test]
fn render_scene_node_and_connection_geometry_has_no_nan_or_infinite_coordinates() {
    let state = sample_state();
    let scene = app::build_render_scene(&state, [800.0, 600.0]);
    let render_map = scene.map().expect("RenderMap muss vorhanden sein");

    for node in render_map.nodes() {
        assert!(
            node.position.is_finite(),
            "Node-Position muss endlich sein (kein NaN/Infinite): {:?}",
            node.position
        );
    }

    for connection in render_map.connections() {
        assert!(
            connection.start_pos.is_finite() && connection.end_pos.is_finite(),
            "Connection-Geometrie muss endlich sein (kein NaN/Infinite)"
        );
    }
}

#[test]
fn render_scene_without_loaded_map_reports_no_map() {
    let state = AppState::new();
    let scene = app::build_render_scene(&state, [640.0, 480.0]);

    assert!(
        !scene.has_map(),
        "Ohne geladene RoadMap darf RenderScene keine Map exponieren"
    );
    assert!(scene.map().is_none());
}
