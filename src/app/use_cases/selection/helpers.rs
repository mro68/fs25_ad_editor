//! Gemeinsame Hilfsfunktionen für Selektionslogik.

use crate::core::RoadMap;
use crate::AppState;
use std::collections::HashMap;

/// Löscht die aktuelle Selektion explizit.
pub fn clear_selection(state: &mut AppState) {
    state.selection.selected_node_ids.clear();
    state.selection.selection_anchor_node_id = None;
}

/// Berechnet das achsen-alignierte Bounding-Rect aus zwei Eckpunkten.
pub(super) fn rect_min_max(a: glam::Vec2, b: glam::Vec2) -> (glam::Vec2, glam::Vec2) {
    (
        glam::Vec2::new(a.x.min(b.x), a.y.min(b.y)),
        glam::Vec2::new(a.x.max(b.x), a.y.max(b.y)),
    )
}

/// Baut eine ungerichtete Adjazenzliste aus den Connections der RoadMap.
///
/// Duplikate werden entfernt, damit bidirektionale Verbindungen (A→B + B→A)
/// den Grad eines Nodes nicht künstlich verdoppeln.
pub(super) fn build_undirected_adjacency(road_map: &RoadMap) -> HashMap<u64, Vec<u64>> {
    use std::collections::HashSet;
    let mut adjacency_set: HashMap<u64, HashSet<u64>> = HashMap::new();

    for connection in road_map.connections_iter() {
        if road_map.nodes.contains_key(&connection.start_id)
            && road_map.nodes.contains_key(&connection.end_id)
        {
            adjacency_set
                .entry(connection.start_id)
                .or_default()
                .insert(connection.end_id);
            adjacency_set
                .entry(connection.end_id)
                .or_default()
                .insert(connection.start_id);
        }
    }

    adjacency_set
        .into_iter()
        .map(|(k, v)| (k, v.into_iter().collect()))
        .collect()
}
