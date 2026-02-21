//! Use-Case: Verbindungen erstellen (direkt und über Connect-Tool).

use crate::app::AppState;
use crate::core::{Connection, ConnectionDirection, ConnectionPriority};
use std::sync::Arc;

/// Erstellt eine Verbindung zwischen zwei Nodes.
///
/// Validiert gegen Self-Loops und Duplikate.
pub fn add_connection(
    state: &mut AppState,
    from_id: u64,
    to_id: u64,
    direction: ConnectionDirection,
    priority: ConnectionPriority,
) {
    if from_id == to_id {
        log::warn!("Self-Loop nicht erlaubt (Node {})", from_id);
        return;
    }

    let Some(road_map_arc) = state.road_map.as_ref() else {
        return;
    };

    // Prüfe ob beide Nodes existieren
    if !road_map_arc.nodes.contains_key(&from_id) || !road_map_arc.nodes.contains_key(&to_id) {
        log::warn!(
            "Verbindung nicht möglich: Node {} oder {} existiert nicht",
            from_id,
            to_id
        );
        return;
    }

    // Duplikat-Check: exaktes Match auf start_id + end_id
    if road_map_arc.has_connection(from_id, to_id) {
        log::warn!("Verbindung {}→{} existiert bereits", from_id, to_id);
        return;
    }

    // Snapshot VOR Mutation
    state.record_undo_snapshot();

    let Some(road_map_arc) = state.road_map.as_mut() else {
        log::warn!("Verbindung nicht möglich: keine RoadMap geladen");
        return;
    };
    let road_map = Arc::make_mut(road_map_arc);

    let start_pos = road_map.nodes[&from_id].position;
    let end_pos = road_map.nodes[&to_id].position;

    let conn = Connection::new(from_id, to_id, direction, priority, start_pos, end_pos);
    road_map.add_connection(conn);
    // Flags der betroffenen Nodes neu berechnen
    road_map.recalculate_node_flags(&[from_id, to_id]);

    log::info!(
        "Verbindung {}→{} ({:?}) erstellt",
        from_id,
        to_id,
        direction
    );
}

/// Connect-Tool: Nächsten Node an Weltposition picken.
///
/// Beim ersten Klick wird der Source-Node gesetzt.
/// Beim zweiten Klick wird die Verbindung erstellt und der Source zurückgesetzt.
pub fn connect_tool_pick_node(state: &mut AppState, world_pos: glam::Vec2, max_distance: f32) {
    let Some(road_map) = state.road_map.as_deref() else {
        return;
    };

    let hit = road_map
        .nearest_node(world_pos)
        .filter(|hit| hit.distance <= max_distance)
        .map(|hit| hit.node_id);

    let Some(node_id) = hit else {
        // Kein Node getroffen — Source zurücksetzen
        state.editor.connect_source_node = None;
        log::debug!("Connect-Tool: kein Node gefunden, Source zurückgesetzt");
        return;
    };

    if let Some(source_id) = state.editor.connect_source_node.take() {
        // Zweiter Klick: Verbindung erstellen
        let direction = state.editor.default_direction;
        let priority = state.editor.default_priority;
        add_connection(state, source_id, node_id, direction, priority);
    } else {
        // Erster Klick: Source setzen
        state.editor.connect_source_node = Some(node_id);
        // Source-Node selektieren als visuelles Feedback
        state.selection.selected_node_ids.clear();
        state.selection.selected_node_ids.insert(node_id);
        state.selection.selection_anchor_node_id = Some(node_id);
        log::info!("Connect-Tool: Startknoten {} gewählt", node_id);
    }
}
