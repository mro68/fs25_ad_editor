//! Boundary-Erkennung: Welche Nodes einer Gruppe haben externe Verbindungen?

use std::collections::HashMap;

use indexmap::IndexSet;

use super::{BoundaryNode, RoadMap};

impl RoadMap {
    /// Ermittelt alle Nodes in `group_ids`, die Verbindungen nach ausserhalb haben.
    ///
    /// Iteriert ueber alle Connections — O(|connections|).
    /// Nur bei Gruppen-Aenderungen aufrufen, nicht pro Frame.
    pub fn boundary_nodes(&self, group_ids: &IndexSet<u64>) -> Vec<BoundaryNode> {
        // (has_incoming, has_outgoing)
        let mut result_map: HashMap<u64, (bool, bool)> = HashMap::new();

        for conn in self.connections.values() {
            let start_in = group_ids.contains(&conn.start_id);
            let end_in = group_ids.contains(&conn.end_id);

            if start_in && !end_in {
                // start_id ist in der Gruppe, end_id draussen → outgoing
                result_map.entry(conn.start_id).or_insert((false, false)).1 = true;
            }
            if end_in && !start_in {
                // end_id ist in der Gruppe, start_id draussen → incoming
                result_map.entry(conn.end_id).or_insert((false, false)).0 = true;
            }
        }

        result_map
            .into_iter()
            .map(|(id, (inc, out))| BoundaryNode {
                node_id: id,
                has_external_incoming: inc,
                has_external_outgoing: out,
            })
            .collect()
    }
}

#[cfg(test)]
mod tests {
    use glam::Vec2;
    use indexmap::IndexSet;

    use crate::core::{
        connection::{ConnectionDirection, ConnectionPriority},
        node::NodeFlag,
        Connection, MapNode, RoadMap,
    };

    /// Erstellt eine minimale RoadMap mit Nodes und Connections fuer Tests.
    fn make_map(node_ids: &[u64], connections: &[(u64, u64)]) -> RoadMap {
        let mut map = RoadMap::new(3);
        for &id in node_ids {
            let node = MapNode {
                id,
                position: Vec2::new(id as f32, 0.0),
                flag: NodeFlag::Regular,
            };
            map.nodes.insert(id, node);
        }
        for &(start, end) in connections {
            let start_pos = map.nodes[&start].position;
            let end_pos = map.nodes[&end].position;
            let conn = Connection {
                start_id: start,
                end_id: end,
                direction: ConnectionDirection::Regular,
                priority: ConnectionPriority::Regular,
                midpoint: (start_pos + end_pos) * 0.5,
                angle: (end_pos - start_pos).y.atan2((end_pos - start_pos).x),
            };
            map.connections.insert((start, end), conn);
        }
        map
    }

    /// Kette A→B→C, Gruppe={A,B} → B ist Boundary mit has_external_outgoing=true
    #[test]
    fn test_chain_abc_group_ab() {
        let map = make_map(&[1, 2, 3], &[(1, 2), (2, 3)]);
        let group: IndexSet<u64> = [1, 2].into_iter().collect();
        let mut boundaries = map.boundary_nodes(&group);
        // Nur Node 2 hat eine externe Verbindung (nach 3)
        assert_eq!(boundaries.len(), 1);
        boundaries.sort_by_key(|b| b.node_id);
        let b = &boundaries[0];
        assert_eq!(b.node_id, 2);
        assert!(!b.has_external_incoming);
        assert!(b.has_external_outgoing);
    }

    /// Node mit eingehender UND ausgehender externer Verbindung → beide Flags gesetzt
    #[test]
    fn test_dual_boundary_node() {
        // Externe Nodes: 0,3 — Gruppe: {1,2}
        // Verbindungen: 0→1, 1→2, 2→3 und 3→1
        let map = make_map(&[0, 1, 2, 3], &[(0, 1), (1, 2), (2, 3), (3, 1)]);
        let group: IndexSet<u64> = [1, 2].into_iter().collect();
        let mut boundaries = map.boundary_nodes(&group);
        boundaries.sort_by_key(|b| b.node_id);
        // Node 2: outgoing nach 3
        // Node 1: incoming von 0 UND incoming von 3
        let b1 = boundaries
            .iter()
            .find(|b| b.node_id == 1)
            .expect("Node 1 erwartet");
        assert!(b1.has_external_incoming);
        assert!(!b1.has_external_outgoing);
        let b2 = boundaries
            .iter()
            .find(|b| b.node_id == 2)
            .expect("Node 2 erwartet");
        assert!(!b2.has_external_incoming);
        assert!(b2.has_external_outgoing);
    }

    /// Alle Nodes in Gruppe → keine Boundary-Nodes
    #[test]
    fn test_all_in_group_no_boundary() {
        let map = make_map(&[1, 2, 3], &[(1, 2), (2, 3)]);
        let group: IndexSet<u64> = [1, 2, 3].into_iter().collect();
        let boundaries = map.boundary_nodes(&group);
        assert!(boundaries.is_empty());
    }

    /// Leere Gruppe → keine Boundary-Nodes
    #[test]
    fn test_empty_group() {
        let map = make_map(&[1, 2], &[(1, 2)]);
        let group: IndexSet<u64> = IndexSet::new();
        let boundaries = map.boundary_nodes(&group);
        assert!(boundaries.is_empty());
    }
}
