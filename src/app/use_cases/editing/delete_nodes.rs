//! Use-Case: Selektierte Nodes löschen (inkl. aller betroffenen Connections).

use super::delete_nodes_by_ids::delete_nodes_internal;
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

    delete_nodes_internal(road_map, &ids_to_delete, true);

    let count = ids_to_delete.len();
    state.selection.ids_mut().clear();
    state.selection.selection_anchor_node_id = None;

    log::info!("{} Node(s) gelöscht", count);
}
