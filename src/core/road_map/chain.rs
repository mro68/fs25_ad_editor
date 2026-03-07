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
        let start = node_ids
            .iter()
            .find(|&&id| {
                !self
                    .connections
                    .values()
                    .any(|c| c.end_id == id && node_ids.contains(&c.start_id))
            })
            .copied()
            .or_else(|| node_ids.iter().next().copied())?;

        let mut path = Vec::with_capacity(node_ids.len());
        let mut visited = HashSet::new();
        let mut current = start;

        loop {
            path.push(current);
            visited.insert(current);

            let next = self
                .connections
                .values()
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
            let degree: usize = self
                .connections
                .values()
                .filter(|c| {
                    (c.start_id == nid && node_ids.contains(&c.end_id))
                        || (c.end_id == nid && node_ids.contains(&c.start_id))
                })
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
}
