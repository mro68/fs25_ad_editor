//! Use-Case: Selektierte Nodes-Kette per Catmull-Rom-Spline neu verteilen (Distanzen-Feature).

use super::delete_nodes_by_ids::delete_nodes_internal;
use crate::app::tools::spline::geometry::{
    catmull_rom_chain_with_tangents, polyline_length, resample_by_distance,
};
use crate::app::AppState;
use crate::core::{Connection, MapNode, NodeFlag};
use glam::Vec2;
use std::collections::HashSet;
use std::sync::Arc;

/// Dichte der Catmull-Rom-Interpolation (Punkte je Segment).
const SAMPLES_PER_SEGMENT: usize = 16;

/// Ordnet die selektierten Nodes zu einer linearen Kette anhand der Verbindungen.
///
/// Sucht einen Startpunkt (keine eingehenden Verbindungen von selektierten Nodes)
/// und folgt dann den Verbindungen. Gibt `None` zurück wenn die Nodes keine
/// vollständige lineare Kette bilden.
fn order_chain(node_ids: &HashSet<u64>, road_map: &crate::core::RoadMap) -> Option<Vec<u64>> {
    // Startpunkt: Node ohne eingehende Verbindungen von selektierten Nodes
    let start = node_ids
        .iter()
        .find(|&&id| {
            road_map
                .connections_iter()
                .filter(|c| c.end_id == id && node_ids.contains(&c.start_id))
                .count()
                == 0
        })
        .copied()
        .or_else(|| node_ids.iter().next().copied())?;

    let mut path = Vec::with_capacity(node_ids.len());
    let mut visited = HashSet::new();
    let mut current = start;

    loop {
        path.push(current);
        visited.insert(current);

        // Nächster Node: ausgehende Verbindung zu einem unbesuchten selektierten Node
        let next = road_map
            .connections_iter()
            .find(|c| {
                c.start_id == current
                    && node_ids.contains(&c.end_id)
                    && !visited.contains(&c.end_id)
            })
            .map(|c| c.end_id);

        match next {
            Some(n) => current = n,
            None => break,
        }
    }

    if path.len() == node_ids.len() {
        Some(path)
    } else {
        None
    }
}

/// Verteilt die selektierten Nodes gleichmäßig entlang eines Catmull-Rom-Splines.
///
/// Liest die Konfiguration aus `state.ui.distanzen`:
/// - `by_count = true`: exakt `count` Waypoints (gleichmäßig verteilt)
/// - `by_count = false`: Waypoints mit maximalem Abstand `distance` Welteinheiten
///
/// Gibt eine Warnung aus (und bricht ab) wenn die selektierten Nodes keine
/// lineare Kette bilden.
pub fn resample_selected_path(state: &mut AppState) {
    let Some(road_map_ref) = state.road_map.as_ref() else {
        log::warn!("Distanzen: keine RoadMap geladen");
        return;
    };

    let n_selected = state.selection.selected_node_ids.len();
    if n_selected < 2 {
        log::warn!("Distanzen: mindestens 2 Nodes selektieren");
        return;
    }

    // Kette ordnen
    let selected = state.selection.selected_node_ids.clone();
    let Some(ordered) = order_chain(&selected, road_map_ref) else {
        log::warn!("Distanzen: selektierte Nodes bilden keine vollständige lineare Kette");
        return;
    };

    // Positionen und Verbindungsparameter
    let positions: Vec<Vec2> = ordered
        .iter()
        .filter_map(|id| road_map_ref.nodes.get(id).map(|n| n.position))
        .collect();

    if positions.len() < 2 {
        return;
    }

    let (direction, priority) = {
        let first_id = ordered[0];
        let second_id = ordered[1];
        road_map_ref
            .find_connection(first_id, second_id)
            .map(|c| (c.direction, c.priority))
            .unwrap_or((
                state.editor.default_direction,
                state.editor.default_priority,
            ))
    };

    // Catmull-Rom-Spline berechnen
    let dense = catmull_rom_chain_with_tangents(&positions, SAMPLES_PER_SEGMENT, None, None);

    // Resample nach Konfiguration
    let new_positions = if state.ui.distanzen.by_count {
        let n = state.ui.distanzen.count.max(2) as usize;
        let total = polyline_length(&dense);
        let step = total / (n - 1) as f32;
        resample_by_distance(&dense, step)
    } else {
        let d = state.ui.distanzen.distance.max(0.1);
        resample_by_distance(&dense, d)
    };

    if new_positions.len() < 2 {
        log::warn!("Distanzen: Resample-Ergebnis enthält zu wenige Punkte");
        return;
    }

    // Snapshot VOR Mutation
    state.record_undo_snapshot();

    let ids_to_delete: Vec<u64> = ordered;

    let Some(road_map_arc) = state.road_map.as_mut() else {
        return;
    };
    let road_map = Arc::make_mut(road_map_arc);

    // Alte Nodes entfernen
    delete_nodes_internal(road_map, &ids_to_delete, false);

    // Neue Nodes und Verbindungen erstellen
    let mut prev_id: Option<u64> = None;
    let mut new_ids = Vec::with_capacity(new_positions.len());

    for &pos in &new_positions {
        let id = road_map.next_node_id();
        let node = MapNode::new(id, pos, NodeFlag::Regular);
        road_map.add_node(node);

        if let Some(p_id) = prev_id {
            if let Some(p_node) = road_map.nodes.get(&p_id) {
                let p_pos = p_node.position;
                let conn = Connection::new(p_id, id, direction, priority, p_pos, pos);
                road_map.add_connection(conn);
            }
        }

        prev_id = Some(id);
        new_ids.push(id);
    }

    road_map.recalculate_node_flags(&new_ids);
    road_map.ensure_spatial_index();

    // Segment-Registry: Records mit alten Nodes invalidieren
    state
        .segment_registry
        .invalidate_by_node_ids(&ids_to_delete);

    // Neue Nodes selektieren
    state.selection.ids_mut().clear();
    for &id in &new_ids {
        state.selection.ids_mut().insert(id);
    }
    state.selection.selection_anchor_node_id = new_ids.first().copied();

    let avg_dist = if new_ids.len() > 1 {
        polyline_length(&new_positions) / (new_ids.len() - 1) as f32
    } else {
        0.0
    };
    log::info!(
        "Distanzen: {} → {} Nodes, Ø-Abstand {:.1}m",
        ids_to_delete.len(),
        new_ids.len(),
        avg_dist,
    );
}
