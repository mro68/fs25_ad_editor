//! Use-Case: Ergebnis eines Route-Tools anwenden (Nodes + Connections erstellen).

use crate::app::tools::ToolResult;
use crate::app::AppState;
use crate::core::{Connection, MapNode};
use std::sync::Arc;

/// Wendet ein `ToolResult` auf den AppState an.
///
/// Erstellt alle neuen Nodes und Verbindungen in einem Undo-Schritt.
/// Selektion wird auf die neuen Nodes gesetzt.
/// Gibt die IDs der erstellten Nodes zurück.
pub fn apply_tool_result(state: &mut AppState, result: ToolResult) -> Vec<u64> {
    if state.road_map.is_none() {
        log::warn!("Route-Tool-Ergebnis kann nicht angewendet werden: keine RoadMap geladen");
        return Vec::new();
    }

    // Snapshot VOR Mutation
    state.record_undo_snapshot();

    let Some(road_map_arc) = state.road_map.as_mut() else {
        return Vec::new();
    };
    let road_map = Arc::make_mut(road_map_arc);

    // Nodes erstellen und IDs merken
    let mut new_ids: Vec<u64> = Vec::with_capacity(result.new_nodes.len());
    for (pos, flag) in &result.new_nodes {
        let id = road_map.next_node_id();
        road_map.add_node(MapNode::new(id, *pos, *flag));
        new_ids.push(id);
    }

    // Interne Verbindungen (zwischen neuen Nodes)
    let mut affected_ids: Vec<u64> = new_ids.clone();
    for &(from_idx, to_idx, direction, priority) in &result.internal_connections {
        let from_id = new_ids[from_idx];
        let to_id = new_ids[to_idx];
        let start_pos = road_map.nodes[&from_id].position;
        let end_pos = road_map.nodes[&to_id].position;
        let conn = Connection::new(from_id, to_id, direction, priority, start_pos, end_pos);
        road_map.add_connection(conn);
    }

    // Externe Verbindungen (neue Nodes → existierende Nodes)
    for &(new_idx, existing_id, direction, priority) in &result.external_connections {
        let new_id = new_ids[new_idx];
        if !road_map.nodes.contains_key(&existing_id) {
            log::warn!(
                "Externer Node {} existiert nicht — Verbindung übersprungen",
                existing_id
            );
            continue;
        }
        let new_pos = road_map.nodes[&new_id].position;
        let existing_pos = road_map.nodes[&existing_id].position;
        let conn = Connection::new(
            new_id,
            existing_id,
            direction,
            priority,
            new_pos,
            existing_pos,
        );
        road_map.add_connection(conn);
        if !affected_ids.contains(&existing_id) {
            affected_ids.push(existing_id);
        }
    }

    // Flags der betroffenen Nodes neu berechnen
    road_map.recalculate_node_flags(&affected_ids);

    // Selektion auf neue Nodes setzen
    state.selection.selected_node_ids.clear();
    for &id in &new_ids {
        state.selection.selected_node_ids.insert(id);
    }
    state.selection.selection_anchor_node_id = new_ids.last().copied();

    log::info!(
        "Route-Tool: {} Nodes und {} Verbindungen erstellt",
        new_ids.len(),
        result.internal_connections.len() + result.external_connections.len()
    );

    new_ids
}

/// Wie `apply_tool_result`, aber OHNE Undo-Snapshot.
/// Für Neuberechnung, wenn der Caller bereits einen Snapshot erstellt hat.
pub fn apply_tool_result_no_snapshot(state: &mut AppState, result: ToolResult) -> Vec<u64> {
    let Some(road_map_arc) = state.road_map.as_mut() else {
        return Vec::new();
    };
    let road_map = Arc::make_mut(road_map_arc);

    let mut new_ids: Vec<u64> = Vec::with_capacity(result.new_nodes.len());
    for (pos, flag) in &result.new_nodes {
        let id = road_map.next_node_id();
        road_map.add_node(MapNode::new(id, *pos, *flag));
        new_ids.push(id);
    }

    let mut affected_ids: Vec<u64> = new_ids.clone();
    for &(from_idx, to_idx, direction, priority) in &result.internal_connections {
        let from_id = new_ids[from_idx];
        let to_id = new_ids[to_idx];
        let start_pos = road_map.nodes[&from_id].position;
        let end_pos = road_map.nodes[&to_id].position;
        let conn = Connection::new(from_id, to_id, direction, priority, start_pos, end_pos);
        road_map.add_connection(conn);
    }

    for &(new_idx, existing_id, direction, priority) in &result.external_connections {
        let new_id = new_ids[new_idx];
        if !road_map.nodes.contains_key(&existing_id) {
            continue;
        }
        let new_pos = road_map.nodes[&new_id].position;
        let existing_pos = road_map.nodes[&existing_id].position;
        let conn = Connection::new(
            new_id,
            existing_id,
            direction,
            priority,
            new_pos,
            existing_pos,
        );
        road_map.add_connection(conn);
        if !affected_ids.contains(&existing_id) {
            affected_ids.push(existing_id);
        }
    }

    road_map.recalculate_node_flags(&affected_ids);

    state.selection.selected_node_ids.clear();
    for &id in &new_ids {
        state.selection.selected_node_ids.insert(id);
    }
    state.selection.selection_anchor_node_id = new_ids.last().copied();

    new_ids
}
