//! Use-Case: Bestimmte Nodes per ID löschen (für Route-Tool-Neuberechnung).

use crate::app::AppState;
use std::sync::Arc;

/// Löscht die angegebenen Nodes und deren Connections.
/// Erstellt KEINEN Undo-Snapshot (Caller verantwortlich).
pub fn delete_nodes_by_ids(state: &mut AppState, ids: &[u64]) {
    if ids.is_empty() {
        return;
    }

    let Some(road_map_arc) = state.road_map.as_mut() else {
        return;
    };
    let road_map = Arc::make_mut(road_map_arc);

    // Nachbar-Nodes sammeln, deren Flags sich ändern könnten
    let mut affected_neighbors: Vec<u64> = Vec::new();
    for &del_id in ids {
        for conn in road_map.connections_iter() {
            if conn.start_id == del_id && !ids.contains(&conn.end_id) {
                affected_neighbors.push(conn.end_id);
            }
            if conn.end_id == del_id && !ids.contains(&conn.start_id) {
                affected_neighbors.push(conn.start_id);
            }
        }
    }

    for &id in ids {
        road_map.remove_node(id);
    }

    if !affected_neighbors.is_empty() {
        road_map.recalculate_node_flags(&affected_neighbors);
    }

    // Gelöschte Nodes aus Selektion entfernen
    for &id in ids {
        state.selection.selected_node_ids.remove(&id);
    }

    log::debug!("{} Nodes gelöscht (Route-Tool-Neuberechnung)", ids.len());
}
