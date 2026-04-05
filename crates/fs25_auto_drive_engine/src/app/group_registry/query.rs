//! Lookup- und Query-Methoden der [`GroupRegistry`].

use super::{GroupRecord, GroupRegistry};
use crate::core::RoadMap;
use glam::Vec2;

impl GroupRegistry {
    /// Gibt alle Records zurueck, die mindestens einen der angegebenen Node-IDs enthalten.
    pub fn find_by_node_ids(&self, node_ids: &indexmap::IndexSet<u64>) -> Vec<&GroupRecord> {
        let mut seen_ids = std::collections::HashSet::new();
        let mut result = Vec::new();
        for &nid in node_ids {
            if let Some(record_ids) = self.node_to_records.get(&nid) {
                for &rid in record_ids {
                    if seen_ids.insert(rid)
                        && let Some(record) = self.records.get(&rid)
                    {
                        result.push(record);
                    }
                }
            }
        }
        result
    }

    /// Findet den ersten Record, der den angegebenen Node enthaelt.
    pub fn find_first_by_node_id(&self, node_id: u64) -> Option<&GroupRecord> {
        self.node_to_records
            .get(&node_id)
            .and_then(|ids| ids.first())
            .and_then(|id| self.records.get(id))
    }

    /// Findet alle Segment-IDs, zu denen ein Node gehoert.
    pub fn groups_for_node(&self, node_id: u64) -> Vec<u64> {
        self.node_to_records
            .get(&node_id)
            .cloned()
            .unwrap_or_default()
    }

    /// Ermittelt die Boundary-Nodes eines Segments (Nodes mit externen Verbindungen).
    ///
    /// Gibt `None` zurueck wenn das Segment nicht existiert.
    pub fn open_nodes(
        &self,
        record_id: u64,
        road_map: &RoadMap,
    ) -> Option<Vec<crate::core::BoundaryNode>> {
        use indexmap::IndexSet;
        let record = self.get(record_id)?;
        let group_ids: IndexSet<u64> = record.node_ids.iter().copied().collect();
        Some(road_map.boundary_nodes(&group_ids))
    }

    /// Berechnet die Achsen-ausgerichtete Bounding-Box (AABB) des Segments.
    ///
    /// Gibt `(min, max)` in Weltkoordinaten zurueck, oder `None` wenn das
    /// Segment nicht existiert oder keine Nodes hat.
    pub fn segment_bounding_box(
        &self,
        segment_id: u64,
        road_map: &RoadMap,
    ) -> Option<(Vec2, Vec2)> {
        let record = self.records.get(&segment_id)?;
        if record.node_ids.is_empty() {
            return None;
        }
        let mut min = Vec2::splat(f32::MAX);
        let mut max = Vec2::splat(f32::MIN);
        let mut found = false;
        for &node_id in &record.node_ids {
            if let Some(node) = road_map.node(node_id) {
                min = min.min(node.position);
                max = max.max(node.position);
                found = true;
            }
        }
        if found {
            Some((min, max))
        } else {
            None
        }
    }
}
