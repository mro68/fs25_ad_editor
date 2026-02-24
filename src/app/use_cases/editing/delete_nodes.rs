//! Use-Case: Selektierte Nodes löschen (inkl. aller betroffenen Connections).

use super::delete_nodes_by_ids::delete_nodes_internal;
use crate::app::AppState;
use crate::core::{Connection, RoadMap};
use std::sync::Arc;

/// Sammelt Reconnect-Operationen für einen Node: wenn genau 1 Vorgänger und 1 Nachfolger
/// vorhanden sind (und die beiden nicht identisch), wird eine neue direkte Verbindung
/// zwischen ihnen vorgeschlagen.
fn collect_reconnect(
    road_map: &RoadMap,
    del_id: u64,
    id_set: &std::collections::HashSet<u64>,
) -> Option<(u64, u64)> {
    // Vorgänger: Nodes mit ausgehender Verbindung zu del_id (die nicht auch gelöscht werden)
    let predecessors: Vec<u64> = road_map
        .connections_iter()
        .filter(|c| c.end_id == del_id && !id_set.contains(&c.start_id))
        .map(|c| c.start_id)
        .collect();

    // Nachfolger: Nodes mit eingehender Verbindung von del_id (die nicht auch gelöscht werden)
    let successors: Vec<u64> = road_map
        .connections_iter()
        .filter(|c| c.start_id == del_id && !id_set.contains(&c.end_id))
        .map(|c| c.end_id)
        .collect();

    if predecessors.len() == 1 && successors.len() == 1 {
        let pred = predecessors[0];
        let succ = successors[0];
        if pred != succ && !road_map.has_connection(pred, succ) {
            return Some((pred, succ));
        }
    }
    None
}

/// Löscht alle selektierten Nodes und deren Connections.
///
/// Wenn `options.reconnect_on_delete` aktiviert ist, werden Nodes mit genau einem
/// Vorgänger und einem Nachfolger so gelöscht, dass Vorgänger und Nachfolger
/// direkt miteinander verbunden werden (Richtung/Priorität der Ausgangsverbindung).
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
    let id_set: std::collections::HashSet<u64> = ids_to_delete.iter().copied().collect();

    // Reconnect-Operationen vorbereiten (falls Option aktiv)
    let reconnect_ops: Vec<(u64, u64)> = if state.options.reconnect_on_delete {
        let road_map = state.road_map.as_ref().unwrap();
        ids_to_delete
            .iter()
            .filter_map(|&del_id| collect_reconnect(road_map, del_id, &id_set))
            .collect()
    } else {
        Vec::new()
    };

    let Some(road_map_arc) = state.road_map.as_mut() else {
        log::warn!("Löschen abgebrochen: keine RoadMap geladen");
        return;
    };
    let road_map = Arc::make_mut(road_map_arc);

    delete_nodes_internal(road_map, &ids_to_delete, true);

    // Reconnect: neue Verbindungen zwischen Vorgänger und Nachfolger erstellen
    if !reconnect_ops.is_empty() {
        for (pred, succ) in &reconnect_ops {
            if let (Some(p_node), Some(s_node)) =
                (road_map.nodes.get(pred), road_map.nodes.get(succ))
            {
                let p_pos = p_node.position;
                let s_pos = s_node.position;
                let conn = Connection::new(
                    *pred,
                    *succ,
                    state.editor.default_direction,
                    state.editor.default_priority,
                    p_pos,
                    s_pos,
                );
                road_map.add_connection(conn);
            }
        }
        let reconnect_ids: Vec<u64> = reconnect_ops.iter().flat_map(|(a, b)| [*a, *b]).collect();
        road_map.recalculate_node_flags(&reconnect_ids);
        road_map.ensure_spatial_index();
        log::info!("{} Reconnect-Verbindung(en) erstellt", reconnect_ops.len());
    }

    let count = ids_to_delete.len();

    // Segment-Registry: Records mit diesen Nodes invalidieren
    state
        .segment_registry
        .invalidate_by_node_ids(&ids_to_delete);

    state.selection.ids_mut().clear();
    state.selection.selection_anchor_node_id = None;

    log::info!("{} Node(s) gelöscht", count);
}
