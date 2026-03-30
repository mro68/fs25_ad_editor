//! Use-Case: Bestimmte Nodes per ID loeschen (fuer Route-Tool-Neuberechnung).
//!
//! Enthaelt auch die gemeinsame interne Loeschlogik `delete_nodes_internal`,
//! die sowohl von `delete_selected_nodes` als auch von `delete_nodes_by_ids` genutzt wird.

use crate::app::AppState;
use crate::core::RoadMap;
use std::sync::Arc;

/// Gemeinsame interne Loeschlogik fuer Nodes.
///
/// Fuehrt die Kern-Schritte aus:
/// 1. Nachbar-Nodes sammeln, deren Flags sich aendern koennten
/// 2. Optional: Marker der zu loeschenden Nodes entfernen (Cascade Delete)
/// 3. Nodes entfernen
/// 4. Flags der verbleibenden Nachbarn neu berechnen
/// 5. Spatial-Index aktualisieren
pub(crate) fn delete_nodes_internal(road_map: &mut RoadMap, ids: &[u64], remove_markers: bool) {
    // HashSet fuer O(1)-Lookup statt O(n) Vec::contains
    let id_set: std::collections::HashSet<u64> = ids.iter().copied().collect();

    // Nachbar-Nodes sammeln, deren Flags sich aendern koennten
    let mut affected_neighbors: Vec<u64> = Vec::new();
    for &id in ids {
        for &(nb, _) in road_map.neighbors(id) {
            if !id_set.contains(&nb) {
                affected_neighbors.push(nb);
            }
        }
    }

    // Marker der zu loeschenden Nodes entfernen (Cascade Delete)
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

    // Spatial-Index einmalig nach allen Loeschungen aktualisieren
    road_map.ensure_spatial_index();
}

/// Loescht die angegebenen Nodes und deren Connections.
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

    // Geloeschte Nodes aus Selektion entfernen
    for &id in ids {
        state.selection.ids_mut().shift_remove(&id);
    }

    // Segment-Registry: Records mit diesen Nodes invalidieren
    state.group_registry.invalidate_by_node_ids(ids);

    log::debug!("{} Nodes geloescht (Route-Tool-Neuberechnung)", ids.len());
}
