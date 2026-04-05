//! Lock- und Edit-Guard-Methoden der [`GroupRegistry`].

use super::GroupRegistry;

impl GroupRegistry {
    /// Wechselt den Lock-Zustand des Segments mit der angegebenen ID.
    ///
    /// Tut nichts, wenn kein Segment mit dieser ID existiert.
    pub fn toggle_lock(&mut self, segment_id: u64) {
        if let Some(record) = self.get_mut(segment_id) {
            record.locked = !record.locked;
        }
    }

    /// Setzt den Lock-Zustand des Segments explizit.
    ///
    /// Tut nichts, wenn kein Segment mit dieser ID existiert.
    pub fn set_locked(&mut self, segment_id: u64, locked: bool) {
        if let Some(record) = self.get_mut(segment_id) {
            record.locked = locked;
        }
    }

    /// Sammelt alle Node-IDs von locked Segments, die mindestens einen der
    /// gegebenen Nodes enthalten. Gibt eine deduplizierte Menge zurueck.
    ///
    /// Wird bei jedem Drag-Update aufgerufen — intern HashSet fuer O(1)-Lookup.
    pub fn expand_locked_selection(&self, selected_nodes: &[u64]) -> Vec<u64> {
        use std::collections::HashSet;
        let mut expanded: HashSet<u64> = HashSet::new();
        let mut processed_segments: HashSet<u64> = HashSet::new();
        for &nid in selected_nodes {
            if let Some(record_ids) = self.node_to_records.get(&nid) {
                for &rid in record_ids {
                    if processed_segments.insert(rid)
                        && let Some(record) = self.records.get(&rid)
                        && record.locked
                    {
                        expanded.extend(record.node_ids.iter().copied());
                    }
                }
            }
        }
        expanded.into_iter().collect()
    }

    /// Setzt die Record-ID, die von automatischer Invalidierung ausgenommen werden soll.
    ///
    /// `None` = kein Guard aktiv (Normal-Modus).
    pub fn set_edit_guard(&mut self, record_id: Option<u64>) {
        self.edit_guard_id = record_id;
    }

    /// Gibt den Lock-Zustand des Segments zurueck.
    ///
    /// Gibt `false` zurueck wenn das Segment nicht existiert.
    pub fn is_locked(&self, segment_id: u64) -> bool {
        self.records
            .get(&segment_id)
            .map(|r| r.locked)
            .unwrap_or(false)
    }
}
