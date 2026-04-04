//! Nachbarschaftsabfragen auf `RoadMap`.

use super::{ConnectedNeighbor, RoadMap};

impl RoadMap {
    /// Gibt alle Nachbar-Nodes zurueck, die ueber Verbindungen mit `node_id` verbunden sind.
    ///
    /// Nutzt den Adjacency-Index — O(degree) statt O(|connections|).
    pub fn connected_neighbors(&self, node_id: u64) -> Vec<ConnectedNeighbor> {
        self.neighbors(node_id)
            .iter()
            .filter_map(|&(nb_id, is_outgoing)| {
                let (start, end) = if is_outgoing {
                    (node_id, nb_id)
                } else {
                    (nb_id, node_id)
                };
                let conn = self.connections.get(&(start, end))?;
                let angle = if is_outgoing {
                    conn.angle
                } else {
                    conn.angle + std::f32::consts::PI
                };
                Some(ConnectedNeighbor {
                    neighbor_id: nb_id,
                    angle,
                    is_outgoing,
                })
            })
            .collect()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::core::{Connection, ConnectionDirection, ConnectionPriority, MapNode, NodeFlag};
    use glam::Vec2;

    /// Hilfsfunktion für einfache Verbindungen
    fn make_conn(s: u64, e: u64, sx: f32, sy: f32, ex: f32, ey: f32) -> Connection {
        Connection::new(
            s,
            e,
            ConnectionDirection::Regular,
            ConnectionPriority::Regular,
            Vec2::new(sx, sy),
            Vec2::new(ex, ey),
        )
    }

    #[test]
    fn test_connected_neighbors_returns_all() {
        // Node 1 mit 3 ausgehenden Verbindungen → 3 Nachbarn
        let mut map = RoadMap::new(3);
        map.add_node(MapNode::new(1, Vec2::new(0.0, 0.0), NodeFlag::Regular));
        map.add_node(MapNode::new(2, Vec2::new(10.0, 0.0), NodeFlag::Regular));
        map.add_node(MapNode::new(3, Vec2::new(0.0, 10.0), NodeFlag::Regular));
        map.add_node(MapNode::new(4, Vec2::new(-10.0, 0.0), NodeFlag::Regular));
        map.add_connection(make_conn(1, 2, 0.0, 0.0, 10.0, 0.0));
        map.add_connection(make_conn(1, 3, 0.0, 0.0, 0.0, 10.0));
        map.add_connection(make_conn(1, 4, 0.0, 0.0, -10.0, 0.0));

        let neighbors = map.connected_neighbors(1);
        assert_eq!(neighbors.len(), 3);
    }

    #[test]
    fn test_connected_neighbors_empty_for_isolated() {
        // Isolierter Node → leere Liste
        let mut map = RoadMap::new(3);
        map.add_node(MapNode::new(1, Vec2::new(0.0, 0.0), NodeFlag::Regular));

        assert!(map.connected_neighbors(1).is_empty());
    }

    #[test]
    fn test_connected_neighbors_bidirectional() {
        // A→B und B→A → Node 2 erscheint zweimal in Nachbarn von Node 1 (out + in)
        let mut map = RoadMap::new(3);
        map.add_node(MapNode::new(1, Vec2::new(0.0, 0.0), NodeFlag::Regular));
        map.add_node(MapNode::new(2, Vec2::new(10.0, 0.0), NodeFlag::Regular));
        map.add_connection(make_conn(1, 2, 0.0, 0.0, 10.0, 0.0));
        map.add_connection(make_conn(2, 1, 10.0, 0.0, 0.0, 0.0));

        let neighbors = map.connected_neighbors(1);
        let count_2 = neighbors.iter().filter(|n| n.neighbor_id == 2).count();
        // 1→2 (outgoing) + 2→1 (end_id==1, incoming) → 2 Einträge
        assert_eq!(count_2, 2);
    }
}
