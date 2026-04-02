//! Gemeinsame Hilfsfunktionen fuer Selektionslogik.

use crate::core::{ConnectionDirection, ConnectionPriority, NodeFlag, RoadMap};
use crate::AppState;
use std::collections::HashSet;

/// Loescht die aktuelle Selektion explizit.
pub fn clear_selection(state: &mut AppState) {
    state.selection.ids_mut().clear();
    state.selection.selection_anchor_node_id = None;
}

/// Berechnet das achsen-alignierte Bounding-Rect aus zwei Eckpunkten.
pub(super) fn rect_min_max(a: glam::Vec2, b: glam::Vec2) -> (glam::Vec2, glam::Vec2) {
    (
        glam::Vec2::new(a.x.min(b.x), a.y.min(b.y)),
        glam::Vec2::new(a.x.max(b.x), a.y.max(b.y)),
    )
}

/// Nachbar-Eintrag mit Winkelinformation fuer den Segment-Walk.
pub(super) struct AdjacencyNeighbor {
    /// ID des Nachbar-Nodes.
    pub node_id: u64,
    /// atan2-Winkel der Verbindung (Richtung: aktueller Node → Nachbar).
    pub angle: f32,
    /// Flag des Ziel-Nodes (Strassenart).
    pub target_flag: NodeFlag,
    /// Richtung der Verbindung (gerichtet vs. bidirektional).
    pub connection_direction: ConnectionDirection,
    /// Prioritaet der Verbindung (Haupt- vs. Nebenstrasse).
    pub connection_priority: ConnectionPriority,
}

/// Liest die Nachbarn fuer den Segment-Walk on demand aus der RoadMap.
///
/// Gegenlaeufige Kanten zum selben Nachbar-Node werden dedupliziert, damit
/// bidirektionale Strassen beim Grad-Test weiterhin als ein Ast zaehlen.
pub(super) fn neighbors_for_segment_walk(
    road_map: &RoadMap,
    node_id: u64,
) -> Vec<AdjacencyNeighbor> {
    let mut seen_neighbor_ids = HashSet::new();

    road_map
        .connected_neighbors(node_id)
        .into_iter()
        .filter_map(|neighbor| {
            let target_node = road_map.nodes.get(&neighbor.neighbor_id)?;
            let (start_id, end_id) = if neighbor.is_outgoing {
                (node_id, neighbor.neighbor_id)
            } else {
                (neighbor.neighbor_id, node_id)
            };
            let connection = road_map.find_connection(start_id, end_id)?;

            if !seen_neighbor_ids.insert(neighbor.neighbor_id) {
                return None;
            }

            Some(AdjacencyNeighbor {
                node_id: neighbor.neighbor_id,
                angle: neighbor.angle,
                target_flag: target_node.flag,
                connection_direction: connection.direction,
                connection_priority: connection.priority,
            })
        })
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::core::{Connection, MapNode};
    use glam::Vec2;

    #[test]
    fn neighbors_for_segment_walk_deduplicates_opposite_connections() {
        let mut map = RoadMap::new(3);
        map.add_node(MapNode::new(1, Vec2::ZERO, NodeFlag::Regular));
        map.add_node(MapNode::new(2, Vec2::new(10.0, 0.0), NodeFlag::Regular));
        map.add_connection(Connection::new(
            1,
            2,
            ConnectionDirection::Dual,
            ConnectionPriority::Regular,
            Vec2::ZERO,
            Vec2::new(10.0, 0.0),
        ));
        map.add_connection(Connection::new(
            2,
            1,
            ConnectionDirection::Dual,
            ConnectionPriority::Regular,
            Vec2::new(10.0, 0.0),
            Vec2::ZERO,
        ));

        let neighbors = neighbors_for_segment_walk(&map, 1);
        assert_eq!(neighbors.len(), 1);
        assert_eq!(neighbors[0].node_id, 2);
    }
}
