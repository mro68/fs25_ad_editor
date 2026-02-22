//! Use-Case: Bestimmte Nodes per ID löschen (für Route-Tool-Neuberechnung).
//!
//! Enthält auch die gemeinsame interne Löschlogik `delete_nodes_internal`,
//! die sowohl von `delete_selected_nodes` als auch von `delete_nodes_by_ids` genutzt wird.

use crate::app::AppState;
use crate::core::RoadMap;
use std::sync::Arc;

/// Gemeinsame interne Löschlogik für Nodes.
///
/// Führt die Kern-Schritte aus:
/// 1. Nachbar-Nodes sammeln, deren Flags sich ändern könnten
/// 2. Optional: Marker der zu löschenden Nodes entfernen (Cascade Delete)
/// 3. Nodes entfernen
/// 4. Flags der verbleibenden Nachbarn neu berechnen
/// 5. Spatial-Index aktualisieren
pub(crate) fn delete_nodes_internal(road_map: &mut RoadMap, ids: &[u64], remove_markers: bool) {
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

    // Marker der zu löschenden Nodes entfernen (Cascade Delete)
    if remove_markers {
        let mut markers_removed = 0;
        for &id in ids {
            if road_map.remove_marker(id) {
                markers_removed += 1;
            }
        }
        if markers_removed > 0 {
            log::info!("{} Marker entfernt (Cascade Delete)", markers_removed);
        }
    }

    for &id in ids {
        road_map.remove_node(id);
    }

    // Flags der verbleibenden Nachbar-Nodes neu berechnen
    if !affected_neighbors.is_empty() {
        road_map.recalculate_node_flags(&affected_neighbors);
    }

    // Spatial-Index einmalig nach allen Löschungen aktualisieren
    road_map.ensure_spatial_index();
}

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

    delete_nodes_internal(road_map, ids, false);

    // Gelöschte Nodes aus Selektion entfernen
    for &id in ids {
        state.selection.selected_node_ids.remove(&id);
    }

    log::debug!("{} Nodes gelöscht (Route-Tool-Neuberechnung)", ids.len());
}
