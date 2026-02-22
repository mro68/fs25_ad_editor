//! Use-Case: Selektierte Nodes löschen (inkl. aller betroffenen Connections).

use crate::app::AppState;
use std::sync::Arc;

/// Löscht alle selektierten Nodes und deren Connections.
pub fn delete_selected_nodes(state: &mut AppState) {
    if state.selection.selected_node_ids.is_empty() {
        log::debug!("Nichts zum Löschen selektiert");
        return;
    }

    if state.road_map.is_none() {
        return;
    }

    // Snapshot VOR Mutation
    state.record_undo_snapshot();

    let ids_to_delete: Vec<u64> = state.selection.selected_node_ids.iter().copied().collect();
    let Some(road_map_arc) = state.road_map.as_mut() else {
        log::warn!("Löschen abgebrochen: keine RoadMap geladen");
        return;
    };
    let road_map = Arc::make_mut(road_map_arc);

    // Sammle Nachbar-Nodes, deren Flags sich ändern könnten
    let mut affected_neighbors: Vec<u64> = Vec::new();
    for &del_id in &ids_to_delete {
        for conn in road_map.connections_iter() {
            if conn.start_id == del_id && !ids_to_delete.contains(&conn.end_id) {
                affected_neighbors.push(conn.end_id);
            }
            if conn.end_id == del_id && !ids_to_delete.contains(&conn.start_id) {
                affected_neighbors.push(conn.start_id);
            }
        }
    }

    // Marker der zu löschenden Nodes entfernen (Cascade Delete)
    let mut markers_removed = 0;
    for id in &ids_to_delete {
        if road_map.remove_marker(*id) {
            markers_removed += 1;
        }
    }
    if markers_removed > 0 {
        log::info!("{} Marker entfernt (Cascade Delete)", markers_removed);
    }

    for id in &ids_to_delete {
        road_map.remove_node(*id);
    }

    // Flags der verbleibenden Nachbar-Nodes neu berechnen
    if !affected_neighbors.is_empty() {
        road_map.recalculate_node_flags(&affected_neighbors);
    }

    // Spatial-Index einmalig nach allen Löschungen aktualisieren
    road_map.ensure_spatial_index();

    let count = ids_to_delete.len();
    state.selection.selected_node_ids.clear();
    state.selection.selection_anchor_node_id = None;

    log::info!("{} Node(s) gelöscht", count);
}
