//! Use-Case: Ergebnis eines Route-Tools anwenden (Nodes + Connections erstellen).

use crate::app::tools::ToolResult;
use crate::app::AppState;
use crate::core::{Connection, MapMarker, MapNode, RoadMap};
use std::collections::HashSet;
use std::sync::Arc;

/// Wendet ein `ToolResult` auf den AppState an.
///
/// Erstellt alle neuen Nodes und Verbindungen in einem Undo-Schritt. Wenn
/// `result.nodes_to_remove` gefuellt ist, werden diese Nodes vor dem Neuaufbau
/// ueber einen einzelnen Batch-Delete entfernt.
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
/// `result.nodes_to_remove` wird dabei ebenfalls vor dem Neuaufbau batchweise entfernt.
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
    let affected_neighbors = remove_nodes_to_replace(road_map, &result.nodes_to_remove);

    let new_ids = create_nodes_and_connections(road_map, &result, &affected_neighbors);

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

fn remove_nodes_to_replace(road_map: &mut RoadMap, node_ids: &[u64]) -> Vec<u64> {
    if node_ids.is_empty() {
        return Vec::new();
    }

    let id_set: HashSet<u64> = node_ids.iter().copied().collect();
    let mut affected_neighbors: Vec<u64> = node_ids
        .iter()
        .flat_map(|&node_id| {
            road_map
                .neighbors(node_id)
                .iter()
                .map(|&(neighbor_id, _)| neighbor_id)
        })
        .filter(|neighbor_id| !id_set.contains(neighbor_id))
        .collect();
    affected_neighbors.sort_unstable();
    affected_neighbors.dedup();

    road_map.remove_nodes_batch(&id_set);

    affected_neighbors
}

/// Erstellt Nodes und Connections aus einem `ToolResult` in der RoadMap.
fn create_nodes_and_connections(
    road_map: &mut RoadMap,
    result: &ToolResult,
    affected_neighbor_ids: &[u64],
) -> Vec<u64> {
    // Nodes erstellen und IDs merken
    let mut new_ids: Vec<u64> = Vec::with_capacity(result.new_nodes.len());
    for (pos, flag) in &result.new_nodes {
        let id = road_map.next_node_id();
        road_map.add_node(MapNode::new(id, *pos, *flag));
        new_ids.push(id);
    }

    // Interne Verbindungen (zwischen neuen Nodes)
    let mut affected_ids: HashSet<u64> = affected_neighbor_ids.iter().copied().collect();
    affected_ids.extend(new_ids.iter().copied());
    for &(from_idx, to_idx, direction, priority) in &result.internal_connections {
        let from_id = new_ids[from_idx];
        let to_id = new_ids[to_idx];
        let start_pos = road_map
            .node_position(from_id)
            .expect("Start-Node vorhanden");
        let end_pos = road_map.node_position(to_id).expect("End-Node vorhanden");
        let conn = Connection::new(from_id, to_id, direction, priority, start_pos, end_pos);
        road_map.add_connection(conn);
    }

    // Externe Verbindungen (Richtung ueber `existing_to_new` explizit vorgegeben)
    for &(new_idx, existing_id, existing_to_new, direction, priority) in
        &result.external_connections
    {
        let new_id = new_ids[new_idx];
        if !road_map.contains_node(existing_id) {
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
        let from_pos = road_map
            .node_position(from_id)
            .expect("Start-Node vorhanden");
        let to_pos = road_map.node_position(to_id).expect("End-Node vorhanden");
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
            let marker_index = road_map.next_marker_index();
            let marker = MapMarker::new(node_id, name.clone(), group.clone(), marker_index, false);
            road_map.add_map_marker(marker);
        }
    }

    // Spatial-Index einmalig nach allen Mutationen aktualisieren
    road_map.ensure_spatial_index();

    new_ids
}

#[cfg(test)]
mod tests {
    use super::{apply_tool_result_no_snapshot, ToolResult};
    use crate::app::AppState;
    use crate::core::{
        Connection, ConnectionDirection, ConnectionPriority, MapNode, NodeFlag, RoadMap,
    };
    use glam::Vec2;
    use std::sync::Arc;

    fn route_offset_like_map() -> RoadMap {
        let mut map = RoadMap::new(3);
        map.add_node(MapNode::new(10, Vec2::new(-50.0, 0.0), NodeFlag::Regular));
        map.add_node(MapNode::new(1, Vec2::new(0.0, 0.0), NodeFlag::Regular));
        map.add_node(MapNode::new(2, Vec2::new(50.0, 0.0), NodeFlag::Regular));
        map.add_node(MapNode::new(3, Vec2::new(100.0, 0.0), NodeFlag::Regular));
        map.add_node(MapNode::new(20, Vec2::new(150.0, 0.0), NodeFlag::Regular));

        for (start_id, end_id, start_pos, end_pos) in [
            (10u64, 1u64, Vec2::new(-50.0, 0.0), Vec2::new(0.0, 0.0)),
            (1, 2, Vec2::new(0.0, 0.0), Vec2::new(50.0, 0.0)),
            (2, 3, Vec2::new(50.0, 0.0), Vec2::new(100.0, 0.0)),
            (3, 20, Vec2::new(100.0, 0.0), Vec2::new(150.0, 0.0)),
        ] {
            map.add_connection(Connection::new(
                start_id,
                end_id,
                ConnectionDirection::Regular,
                ConnectionPriority::Regular,
                start_pos,
                end_pos,
            ));
        }

        map
    }

    #[test]
    fn apply_tool_result_replaces_nodes_to_remove_without_stale_ids_or_connections() {
        let mut state = AppState::new();
        state.road_map = Some(Arc::new(route_offset_like_map()));

        let result = ToolResult {
            new_nodes: vec![
                (Vec2::new(0.0, 10.0), NodeFlag::Regular),
                (Vec2::new(100.0, 10.0), NodeFlag::Regular),
            ],
            internal_connections: vec![(
                0,
                1,
                ConnectionDirection::Regular,
                ConnectionPriority::Regular,
            )],
            external_connections: vec![
                (
                    0,
                    10,
                    true,
                    ConnectionDirection::Regular,
                    ConnectionPriority::Regular,
                ),
                (
                    1,
                    20,
                    false,
                    ConnectionDirection::Regular,
                    ConnectionPriority::Regular,
                ),
            ],
            markers: Vec::new(),
            nodes_to_remove: vec![1, 2, 3],
        };

        let new_ids = apply_tool_result_no_snapshot(&mut state, result);
        let road_map = state
            .road_map
            .as_ref()
            .expect("RoadMap muss nach apply_tool_result erhalten bleiben");

        assert_eq!(new_ids.len(), 2);
        assert!(road_map.contains_node(10));
        assert!(road_map.contains_node(20));
        assert!(!road_map.contains_node(1));
        assert!(!road_map.contains_node(2));
        assert!(!road_map.contains_node(3));
        assert!(road_map.has_connection(10, new_ids[0]));
        assert!(road_map.has_connection(new_ids[0], new_ids[1]));
        assert!(road_map.has_connection(new_ids[1], 20));
        assert_eq!(road_map.connection_count(), 3);
        assert!(road_map.connections_iter().all(|connection| {
            !matches!(connection.start_id, 1..=3) && !matches!(connection.end_id, 1..=3)
        }));

        let selected_ids: Vec<u64> = state.selection.selected_node_ids.iter().copied().collect();
        assert_eq!(selected_ids, new_ids);
        assert_eq!(
            state.selection.selection_anchor_node_id,
            new_ids.last().copied()
        );
    }
}
