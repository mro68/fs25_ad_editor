//! Use-Case: Ergebnis eines Route-Tools anwenden (Nodes + Connections erstellen).

use crate::app::tools::ToolResult;
use crate::app::AppState;
use crate::core::{Connection, MapMarker, MapNode, RoadMap};
use std::collections::HashSet;
use std::sync::Arc;

/// Wendet ein `ToolResult` auf den AppState an.
///
/// Erstellt alle neuen Nodes und Verbindungen in einem Undo-Schritt.
/// Selektion wird auf die neuen Nodes gesetzt.
/// Gibt die IDs der erstellten Nodes zurueck.
pub fn apply_tool_result(state: &mut AppState, result: ToolResult) -> Vec<u64> {
    if state.road_map.is_none() {
        log::warn!("Route-Tool-Ergebnis kann nicht angewendet werden: keine RoadMap geladen");
        return Vec::new();
    }

    // Snapshot VOR Mutation
    state.record_undo_snapshot();

    apply_result_inner(state, result)
}

/// Wie `apply_tool_result`, aber OHNE Undo-Snapshot.
/// Fuer Neuberechnung, wenn der Caller bereits einen Snapshot erstellt hat.
pub fn apply_tool_result_no_snapshot(state: &mut AppState, result: ToolResult) -> Vec<u64> {
    apply_result_inner(state, result)
}

/// Gemeinsame Implementierung: Nodes + Connections erstellen, Selektion setzen.
fn apply_result_inner(state: &mut AppState, result: ToolResult) -> Vec<u64> {
    let Some(road_map_arc) = state.road_map.as_mut() else {
        return Vec::new();
    };
    let road_map = Arc::make_mut(road_map_arc);

    // Nodes entfernen (z.B. Original-Kette bei RouteOffsetTool mit "Original entfernen")
    // Muss VOR der Erstellung neuer Nodes erfolgen damit Undo alles in einem Snapshot abdeckt.
    for &node_id in &result.nodes_to_remove {
        road_map.remove_node(node_id);
    }

    let new_ids = create_nodes_and_connections(road_map, &result);

    // Selektion auf neue Nodes setzen
    state.selection.ids_mut().clear();
    for &id in &new_ids {
        state.selection.ids_mut().insert(id);
    }
    state.selection.selection_anchor_node_id = new_ids.last().copied();

    log::info!(
        "Route-Tool: {} Nodes und {} Verbindungen erstellt",
        new_ids.len(),
        result.internal_connections.len() + result.external_connections.len()
    );

    new_ids
}

/// Erstellt Nodes und Connections aus einem `ToolResult` in der RoadMap.
fn create_nodes_and_connections(road_map: &mut RoadMap, result: &ToolResult) -> Vec<u64> {
    // Nodes erstellen und IDs merken
    let mut new_ids: Vec<u64> = Vec::with_capacity(result.new_nodes.len());
    for (pos, flag) in &result.new_nodes {
        let id = road_map.next_node_id();
        road_map.add_node(MapNode::new(id, *pos, *flag));
        new_ids.push(id);
    }

    // Interne Verbindungen (zwischen neuen Nodes)
    let mut affected_ids: HashSet<u64> = new_ids.iter().copied().collect();
    for &(from_idx, to_idx, direction, priority) in &result.internal_connections {
        let from_id = new_ids[from_idx];
        let to_id = new_ids[to_idx];
        let start_pos = road_map.nodes[&from_id].position;
        let end_pos = road_map.nodes[&to_id].position;
        let conn = Connection::new(from_id, to_id, direction, priority, start_pos, end_pos);
        road_map.add_connection(conn);
    }

    // Externe Verbindungen (Richtung ueber `existing_to_new` explizit vorgegeben)
    for &(new_idx, existing_id, existing_to_new, direction, priority) in
        &result.external_connections
    {
        let new_id = new_ids[new_idx];
        if !road_map.nodes.contains_key(&existing_id) {
            log::warn!(
                "Externer Node {} existiert nicht — Verbindung uebersprungen",
                existing_id
            );
            continue;
        }
        let (from_id, to_id) = if existing_to_new {
            (existing_id, new_id)
        } else {
            (new_id, existing_id)
        };
        let from_pos = road_map.nodes[&from_id].position;
        let to_pos = road_map.nodes[&to_id].position;
        let conn = Connection::new(from_id, to_id, direction, priority, from_pos, to_pos);
        road_map.add_connection(conn);
        affected_ids.insert(existing_id);
    }

    // Flags der betroffenen Nodes neu berechnen
    let affected_vec: Vec<u64> = affected_ids.into_iter().collect();
    road_map.recalculate_node_flags(&affected_vec);

    // Map-Marker erzeugen (z.B. ParkingTool)
    for (new_idx, name, group) in &result.markers {
        if let Some(&node_id) = new_ids.get(*new_idx) {
            let marker_index = road_map.map_markers.len() as u32 + 1;
            let marker = MapMarker::new(node_id, name.clone(), group.clone(), marker_index, false);
            road_map.add_map_marker(marker);
        }
    }

    // Spatial-Index einmalig nach allen Mutationen aktualisieren
    road_map.ensure_spatial_index();

    new_ids
}
