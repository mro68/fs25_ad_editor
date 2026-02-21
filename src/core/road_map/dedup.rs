//! Duplikat-Erkennung und -Bereinigung für Nodes mit identischer Position.

use super::RoadMap;
use crate::core::{Connection, ConnectionDirection};
use std::collections::HashMap;

impl RoadMap {
    /// Zählt duplizierte Nodes (gleiche Position innerhalb `epsilon`) ohne sie zu entfernen.
    ///
    /// Gibt die Anzahl der Duplikat-Nodes und der betroffenen Positionen zurück.
    pub fn count_duplicates(&self, epsilon: f32) -> (u32, u32) {
        let inv_epsilon = if epsilon > 0.0 { 1.0 / epsilon } else { 1000.0 };
        let mut grid: HashMap<(i64, i64), u32> = HashMap::new();
        for node in self.nodes.values() {
            let gx = (node.position.x * inv_epsilon).round() as i64;
            let gz = (node.position.y * inv_epsilon).round() as i64;
            *grid.entry((gx, gz)).or_default() += 1;
        }
        let mut dup_nodes = 0u32;
        let mut dup_groups = 0u32;
        for &count in grid.values() {
            if count > 1 {
                dup_nodes += count - 1;
                dup_groups += 1;
            }
        }
        (dup_nodes, dup_groups)
    }

    /// Erkennt und entfernt duplizierte Nodes (gleiche Position innerhalb `epsilon`).
    pub fn deduplicate_nodes(&mut self, epsilon: f32) -> DeduplicationResult {
        let inv_epsilon = if epsilon > 0.0 { 1.0 / epsilon } else { 1000.0 };

        let mut grid: HashMap<(i64, i64), Vec<u64>> = HashMap::new();
        for (&id, node) in &self.nodes {
            let gx = (node.position.x * inv_epsilon).round() as i64;
            let gz = (node.position.y * inv_epsilon).round() as i64;
            grid.entry((gx, gz)).or_default().push(id);
        }

        let mut remap: HashMap<u64, u64> = HashMap::new();
        let mut duplicate_ids: Vec<u64> = Vec::new();
        let mut groups_with_duplicates = 0u32;

        for (_cell, mut ids) in grid {
            if ids.len() <= 1 {
                continue;
            }
            ids.sort_unstable();
            let canonical = ids[0];
            groups_with_duplicates += 1;
            for &dup_id in &ids[1..] {
                remap.insert(dup_id, canonical);
                duplicate_ids.push(dup_id);
            }
        }

        if duplicate_ids.is_empty() {
            return DeduplicationResult::default();
        }

        let removed_nodes = duplicate_ids.len();

        let old_connections: Vec<Connection> = self.connections.drain().map(|(_, c)| c).collect();
        let mut remapped_connections = 0u32;
        let mut removed_self_connections = 0u32;

        for mut conn in old_connections {
            let orig_start = conn.start_id;
            let orig_end = conn.end_id;

            if let Some(&canonical) = remap.get(&conn.start_id) {
                conn.start_id = canonical;
            }
            if let Some(&canonical) = remap.get(&conn.end_id) {
                conn.end_id = canonical;
            }

            if conn.start_id == conn.end_id {
                removed_self_connections += 1;
                continue;
            }

            let was_remapped = conn.start_id != orig_start || conn.end_id != orig_end;
            if was_remapped {
                remapped_connections += 1;
                if let (Some(s), Some(e)) = (
                    self.nodes.get(&conn.start_id).map(|n| n.position),
                    self.nodes.get(&conn.end_id).map(|n| n.position),
                ) {
                    conn.update_geometry(s, e);
                }
            }

            let key = (conn.start_id, conn.end_id);
            if let Some(existing) = self.connections.get_mut(&key) {
                if conn.direction == ConnectionDirection::Dual {
                    existing.direction = ConnectionDirection::Dual;
                }
            } else {
                self.connections.insert(key, conn);
            }
        }

        for &dup_id in &duplicate_ids {
            self.nodes.remove(&dup_id);
        }

        let mut remapped_markers = 0u32;
        for marker in &mut self.map_markers {
            if let Some(&canonical) = remap.get(&marker.id) {
                marker.id = canonical;
                remapped_markers += 1;
            }
        }
        let mut seen_marker_ids = std::collections::HashSet::new();
        self.map_markers.retain(|m| seen_marker_ids.insert(m.id));

        self.rebuild_spatial_index();

        DeduplicationResult {
            removed_nodes: removed_nodes as u32,
            remapped_connections,
            removed_self_connections,
            remapped_markers,
            duplicate_groups: groups_with_duplicates,
        }
    }
}

/// Ergebnis einer Duplikat-Bereinigung.
#[derive(Debug, Clone, Default)]
pub struct DeduplicationResult {
    /// Anzahl entfernter Duplikat-Nodes
    pub removed_nodes: u32,
    /// Anzahl umgeleiteter Verbindungen
    pub remapped_connections: u32,
    /// Anzahl verworfener Selbst-Referenz-Verbindungen (nach Remap)
    pub removed_self_connections: u32,
    /// Anzahl umgeleiteter Map-Marker
    pub remapped_markers: u32,
    /// Anzahl der Positions-Gruppen mit Duplikaten
    pub duplicate_groups: u32,
}

impl DeduplicationResult {
    /// Prüft ob Duplikate gefunden und bereinigt wurden.
    pub fn had_duplicates(&self) -> bool {
        self.removed_nodes > 0
    }
}
