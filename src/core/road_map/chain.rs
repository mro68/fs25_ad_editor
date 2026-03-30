//! Ketten- und Resample-bezogene Abfragen auf `RoadMap`.

use super::RoadMap;
use indexmap::IndexSet;
use std::collections::HashSet;

impl RoadMap {
    /// Findet den Startknoten (kein Vorgaenger in Selektion) und baut die Pfad-Reihenfolge auf.
    ///
    /// Interner Helfer fuer `is_resampleable_chain` und `ordered_chain_nodes`.
    /// Gibt `None` zurueck wenn keine Node in `node_ids` ist.
    fn collect_ordered_path(&self, node_ids: &IndexSet<u64>) -> Option<Vec<u64>> {
        // Startknoten: kein eingehender Nachbar aus der Selektion — O(nodes * degree)
        let start = node_ids
            .iter()
            .find(|&&id| {
                !self
                    .neighbors(id)
                    .iter()
                    .any(|&(nb, out)| !out && node_ids.contains(&nb))
            })
            .copied()
            .or_else(|| node_ids.iter().next().copied())?;

        let mut path = Vec::with_capacity(node_ids.len());
        let mut visited = HashSet::new();
        let mut current = start;

        loop {
            path.push(current);
            visited.insert(current);

            // Naechster: ausgehender Nachbar in der Selektion, noch nicht besucht
            let next = self
                .outgoing_neighbors(current)
                .find(|&nb| node_ids.contains(&nb) && !visited.contains(&nb));

            match next {
                Some(n) => current = n,
                None => break,
            }
        }

        Some(path)
    }

    /// Prueft ob die selektierten Nodes eine zusammenhaengende Kette bilden,
    /// bei der Kreuzungen (Grad ≥ 3 innerhalb der Selektion) nur an den
    /// Endpunkten vorkommen. Mindestens 2 Nodes erforderlich.
    pub fn is_resampleable_chain(&self, node_ids: &IndexSet<u64>) -> bool {
        if node_ids.len() < 2 {
            return false;
        }

        let Some(path) = self.collect_ordered_path(node_ids) else {
            return false;
        };

        // Alle Nodes muessen in der Kette sein
        if path.len() != node_ids.len() {
            return false;
        }

        // Innere Nodes duerfen keine Kreuzungen sein (Grad innerhalb Selektion ≤ 2)
        for &nid in &path[1..path.len() - 1] {
            let degree = self
                .neighbors(nid)
                .iter()
                .filter(|&&(nb, _)| node_ids.contains(&nb))
                .count();
            if degree > 2 {
                return false;
            }
        }

        true
    }

    /// Gibt die Nodes in Ketten-Reihenfolge zurueck (folgt Verbindungen).
    ///
    /// Voraussetzung: `is_resampleable_chain` ergibt `true`.
    /// Gibt `None` zurueck wenn die Nodes keine vollstaendige lineare Kette bilden.
    pub fn ordered_chain_nodes(&self, node_ids: &IndexSet<u64>) -> Option<Vec<u64>> {
        if node_ids.len() < 2 {
            return None;
        }

        let path = self.collect_ordered_path(node_ids)?;

        if path.len() == node_ids.len() {
            Some(path)
        } else {
            None
        }
    }

    /// Prueft ob die selektierten Nodes einen zusammenhaengenden Subgraphen bilden.
    ///
    /// Mindestens 2 Nodes erforderlich. Im Gegensatz zu `is_resampleable_chain`
    /// duerfen die Nodes beliebige Verbindungstopologien haben (Kreuzungen,
    /// Verzweigungen, Schleifen). Es muss lediglich jeder Node ueber Verbindungen
    /// (direkt oder transitiv) von jedem anderen Node erreichbar sein.
    pub fn is_connected_subgraph(&self, node_ids: &IndexSet<u64>) -> bool {
        if node_ids.len() < 2 {
            return false;
        }

        let start = match node_ids.iter().next().copied() {
            Some(id) => id,
            None => return false,
        };

        let mut visited = HashSet::with_capacity(node_ids.len());
        let mut stack = vec![start];

        while let Some(current) = stack.pop() {
            if !visited.insert(current) {
                continue;
            }
            for &(nb, _) in self.neighbors(current) {
                if node_ids.contains(&nb) && !visited.contains(&nb) {
                    stack.push(nb);
                }
            }
        }

        visited.len() == node_ids.len()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{Connection, ConnectionDirection, ConnectionPriority, MapNode, NodeFlag, RoadMap};

    fn make_map_with_star() -> RoadMap {
        // Stern-Topologie: Node 1 in der Mitte, verbunden mit 2, 3, 4
        //     2
        //     |
        // 3 - 1 - 4
        let mut map = RoadMap::new(3);
        for id in 1..=4 {
            map.add_node(MapNode::new(
                id,
                glam::Vec2::new(id as f32 * 10.0, 0.0),
                NodeFlag::Regular,
            ));
        }
        for &end in &[2, 3, 4] {
            let start_pos = glam::Vec2::new(10.0, 0.0);
            let end_pos = glam::Vec2::new(end as f32 * 10.0, 0.0);
            map.add_connection(Connection::new(
                1,
                end,
                ConnectionDirection::Regular,
                ConnectionPriority::Regular,
                start_pos,
                end_pos,
            ));
        }
        map
    }

    #[test]
    fn connected_subgraph_stern() {
        let map = make_map_with_star();
        let ids: IndexSet<u64> = [1, 2, 3, 4].into_iter().collect();
        assert!(map.is_connected_subgraph(&ids));
    }

    #[test]
    fn connected_subgraph_teilmenge_verbunden() {
        let map = make_map_with_star();
        let ids: IndexSet<u64> = [1, 2].into_iter().collect();
        assert!(map.is_connected_subgraph(&ids));
    }

    #[test]
    fn connected_subgraph_teilmenge_unverbunden() {
        let map = make_map_with_star();
        // 2 und 3 sind nicht direkt verbunden (nur ueber 1)
        let ids: IndexSet<u64> = [2, 3].into_iter().collect();
        assert!(!map.is_connected_subgraph(&ids));
    }

    #[test]
    fn connected_subgraph_einzelner_node() {
        let map = make_map_with_star();
        let ids: IndexSet<u64> = [1].into_iter().collect();
        assert!(!map.is_connected_subgraph(&ids));
    }

    #[test]
    fn connected_subgraph_leere_menge() {
        let map = make_map_with_star();
        let ids: IndexSet<u64> = IndexSet::new();
        assert!(!map.is_connected_subgraph(&ids));
    }

    #[test]
    fn connected_subgraph_kette() {
        // Lineare Kette: 1 → 2 → 3 → 4
        let mut map = RoadMap::new(3);
        for id in 1..=4 {
            map.add_node(MapNode::new(
                id,
                glam::Vec2::new(id as f32, 0.0),
                NodeFlag::Regular,
            ));
        }
        for start in 1..=3 {
            let start_pos = glam::Vec2::new(start as f32, 0.0);
            let end_pos = glam::Vec2::new((start + 1) as f32, 0.0);
            map.add_connection(Connection::new(
                start,
                start + 1,
                ConnectionDirection::Regular,
                ConnectionPriority::Regular,
                start_pos,
                end_pos,
            ));
        }
        let ids: IndexSet<u64> = [1, 2, 3, 4].into_iter().collect();
        assert!(map.is_connected_subgraph(&ids));
    }
}
