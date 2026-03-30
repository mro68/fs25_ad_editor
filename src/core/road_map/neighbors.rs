//! Nachbarschaftsabfragen auf `RoadMap`.

use super::{ConnectedNeighbor, RoadMap};

impl RoadMap {
    /// Gibt alle Nachbar-Nodes zurueck, die ueber Verbindungen mit `node_id` verbunden sind.
    ///
    /// Iteriert ueber alle Connections — O(n), aber nur bei Snap-Events aufgerufen.
    pub fn connected_neighbors(&self, node_id: u64) -> Vec<ConnectedNeighbor> {
        let mut neighbors = Vec::new();
        for conn in self.connections.values() {
            if conn.start_id == node_id {
                neighbors.push(ConnectedNeighbor {
                    neighbor_id: conn.end_id,
                    angle: conn.angle,
                    is_outgoing: true,
                });
            } else if conn.end_id == node_id {
                neighbors.push(ConnectedNeighbor {
                    neighbor_id: conn.start_id,
                    angle: conn.angle + std::f32::consts::PI,
                    is_outgoing: false,
                });
            }
        }
        neighbors
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
