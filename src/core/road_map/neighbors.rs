//! Nachbarschaftsabfragen auf `RoadMap`.

use super::{ConnectedNeighbor, RoadMap};

impl RoadMap {
    /// Gibt alle Nachbar-Nodes zurück, die über Verbindungen mit `node_id` verbunden sind.
    ///
    /// Iteriert über alle Connections — O(n), aber nur bei Snap-Events aufgerufen.
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
