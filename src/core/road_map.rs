//! Die zentrale RoadMap-Datenstruktur mit Nodes, Connections und Spatial-Index.

use super::{
    AutoDriveMeta, Connection, ConnectionDirection, ConnectionPriority, MapMarker, MapNode,
};
use super::{SpatialIndex, SpatialMatch};
use glam::Vec2;
/// Container für das gesamte AutoDrive-Straßennetzwerk
use std::collections::HashMap;

/// Vollständige AutoDrive-Konfiguration
#[derive(Debug, Clone)]
pub struct RoadMap {
    /// Alle Wegpunkte, indexiert nach ihrer ID
    pub nodes: HashMap<u64, MapNode>,
    /// Alle Verbindungen, indexiert nach (start_id, end_id) für O(1)-Zugriff
    connections: HashMap<(u64, u64), Connection>,
    /// Alle Map-Marker
    pub map_markers: Vec<MapMarker>,
    /// Zusaetzliche Metadaten aus der XML
    pub meta: AutoDriveMeta,
    /// Version der Config (3 = FS25, Legacy: 1 = FS19, 2 = FS22)
    pub version: u32,
    /// Name der Map (optional)
    pub map_name: Option<String>,
    /// Persistenter Spatial-Index fuer schnelle Node-Abfragen
    spatial_index: SpatialIndex,
}

impl RoadMap {
    /// Erstellt eine neue leere RoadMap
    pub fn new(version: u32) -> Self {
        Self {
            nodes: HashMap::new(),
            connections: HashMap::new(),
            map_markers: Vec::new(),
            meta: AutoDriveMeta::default(),
            version,
            map_name: None,
            spatial_index: SpatialIndex::empty(),
        }
    }

    /// Fügt einen Node hinzu
    pub fn add_node(&mut self, node: MapNode) {
        self.nodes.insert(node.id, node);
        self.rebuild_spatial_index();
    }

    /// Entfernt einen Node inklusive aller betroffenen Verbindungen
    pub fn remove_node(&mut self, node_id: u64) -> Option<MapNode> {
        let removed = self.nodes.remove(&node_id);
        if removed.is_some() {
            self.connections
                .retain(|(s, e), _| *s != node_id && *e != node_id);
            self.rebuild_spatial_index();
        }
        removed
    }

    /// Aktualisiert die Position eines Nodes und baut bei Bedarf Geometrie/Index neu auf
    pub fn update_node_position(&mut self, node_id: u64, new_position: Vec2) -> bool {
        let Some(node) = self.nodes.get_mut(&node_id) else {
            return false;
        };

        if node.position == new_position {
            return true;
        }

        node.position = new_position;
        self.rebuild_connection_geometry();
        self.rebuild_spatial_index();
        true
    }

    /// Fügt eine Verbindung hinzu
    pub fn add_connection(&mut self, connection: Connection) {
        self.connections
            .insert((connection.start_id, connection.end_id), connection);
    }

    /// Prüft ob eine Verbindung existiert (exaktes Match auf start_id + end_id) — O(1)
    pub fn has_connection(&self, start_id: u64, end_id: u64) -> bool {
        self.connections.contains_key(&(start_id, end_id))
    }

    /// Findet eine Verbindung (exaktes Match) — O(1)
    pub fn find_connection(&self, start_id: u64, end_id: u64) -> Option<&Connection> {
        self.connections.get(&(start_id, end_id))
    }

    /// Findet alle Verbindungen zwischen zwei Nodes (in beiden Richtungen) — O(1)
    pub fn find_connections_between(&self, node_a: u64, node_b: u64) -> Vec<&Connection> {
        let mut result = Vec::with_capacity(2);
        if let Some(c) = self.connections.get(&(node_a, node_b)) {
            result.push(c);
        }
        if let Some(c) = self.connections.get(&(node_b, node_a)) {
            result.push(c);
        }
        result
    }

    /// Entfernt eine spezifische Verbindung (exaktes Match) — O(1)
    pub fn remove_connection(&mut self, start_id: u64, end_id: u64) -> bool {
        self.connections.remove(&(start_id, end_id)).is_some()
    }

    /// Entfernt alle Verbindungen zwischen zwei Nodes (in beiden Richtungen) — O(1)
    pub fn remove_connections_between(&mut self, node_a: u64, node_b: u64) -> usize {
        let mut removed = 0;
        if self.connections.remove(&(node_a, node_b)).is_some() {
            removed += 1;
        }
        if self.connections.remove(&(node_b, node_a)).is_some() {
            removed += 1;
        }
        removed
    }

    /// Ändert die Richtung einer bestehenden Verbindung — O(1)
    pub fn set_connection_direction(
        &mut self,
        start_id: u64,
        end_id: u64,
        direction: ConnectionDirection,
    ) -> bool {
        if let Some(conn) = self.connections.get_mut(&(start_id, end_id)) {
            conn.direction = direction;
            true
        } else {
            false
        }
    }

    /// Ändert die Priorität einer bestehenden Verbindung — O(1)
    pub fn set_connection_priority(
        &mut self,
        start_id: u64,
        end_id: u64,
        priority: ConnectionPriority,
    ) -> bool {
        if let Some(conn) = self.connections.get_mut(&(start_id, end_id)) {
            conn.priority = priority;
            true
        } else {
            false
        }
    }

    /// Invertiert eine Verbindung (start ⇔ end) und aktualisiert die Geometrie — O(1)
    pub fn invert_connection(&mut self, start_id: u64, end_id: u64) -> bool {
        if let Some(mut conn) = self.connections.remove(&(start_id, end_id)) {
            conn.start_id = end_id;
            conn.end_id = start_id;
            let new_start = self.nodes.get(&end_id).map(|n| n.position);
            let new_end = self.nodes.get(&start_id).map(|n| n.position);
            if let (Some(s), Some(e)) = (new_start, new_end) {
                conn.update_geometry(s, e);
            }
            self.connections.insert((end_id, start_id), conn);
            true
        } else {
            false
        }
    }

    /// Iterator über alle Verbindungen (read-only).
    pub fn connections_iter(&self) -> impl Iterator<Item = &Connection> {
        self.connections.values()
    }

    /// Berechnet die nächste freie Node-ID
    pub fn next_node_id(&self) -> u64 {
        self.nodes.keys().max().copied().unwrap_or(0) + 1
    }

    /// Fügt einen Map-Marker hinzu
    pub fn add_map_marker(&mut self, marker: MapMarker) {
        self.map_markers.push(marker);
    }

    /// Prüft ob ein Node einen Marker hat
    pub fn has_marker(&self, node_id: u64) -> bool {
        self.map_markers.iter().any(|m| m.id == node_id)
    }

    /// Findet Marker für einen Node
    pub fn find_marker_by_node_id(&self, node_id: u64) -> Option<&MapMarker> {
        self.map_markers.iter().find(|m| m.id == node_id)
    }

    /// Entfernt Marker für einen Node (gibt true zurück falls gefunden)
    pub fn remove_marker(&mut self, node_id: u64) -> bool {
        let before = self.map_markers.len();
        self.map_markers.retain(|m| m.id != node_id);
        self.map_markers.len() < before
    }

    /// Aktualisiert die Geometrie aller Verbindungen
    pub fn rebuild_connection_geometry(&mut self) {
        // Positionen zuerst einsammeln, um Borrow-Konflikt zu vermeiden
        let updates: Vec<((u64, u64), Vec2, Vec2)> = self
            .connections
            .keys()
            .filter_map(|&(s, e)| {
                let start_pos = self.nodes.get(&s)?.position;
                let end_pos = self.nodes.get(&e)?.position;
                Some(((s, e), start_pos, end_pos))
            })
            .collect();
        for ((s, e), start_pos, end_pos) in updates {
            if let Some(conn) = self.connections.get_mut(&(s, e)) {
                conn.update_geometry(start_pos, end_pos);
            }
        }
    }

    /// Gibt die Anzahl der Nodes zurück
    pub fn node_count(&self) -> usize {
        self.nodes.len()
    }

    /// Gibt die Anzahl der Verbindungen zurück
    pub fn connection_count(&self) -> usize {
        self.connections.len()
    }

    /// Gibt die Anzahl der Map-Marker zurück
    pub fn marker_count(&self) -> usize {
        self.map_markers.len()
    }

    /// Berechnet die NodeFlags (Regular/SubPrio) für die angegebenen Nodes neu.
    ///
    /// Logik (entspricht AutoDrive FLAG_REGULAR / FLAG_SUBPRIO):
    /// - Mindestens eine Verbindung mit `ConnectionPriority::Regular` → `Regular`
    /// - Nur Verbindungen mit `ConnectionPriority::SubPriority` → `SubPrio`
    /// - Keine Verbindungen → `Regular` (Default)
    /// - Nodes mit Warning/Reserved-Flag werden nicht verändert.
    pub fn recalculate_node_flags(&mut self, node_ids: &[u64]) {
        use super::NodeFlag;

        for &nid in node_ids {
            let Some(node) = self.nodes.get(&nid) else {
                continue;
            };

            // Warning/Reserved nicht anfassen
            match node.flag {
                NodeFlag::Warning | NodeFlag::Reserved => continue,
                _ => {}
            }

            let mut has_any_connection = false;
            let mut has_regular_priority = false;

            for conn in self.connections.values() {
                if conn.start_id == nid || conn.end_id == nid {
                    has_any_connection = true;
                    if conn.priority == ConnectionPriority::Regular {
                        has_regular_priority = true;
                        break; // Ein Regular reicht
                    }
                }
            }

            let new_flag = if !has_any_connection || has_regular_priority {
                NodeFlag::Regular
            } else {
                NodeFlag::SubPrio
            };

            if let Some(node) = self.nodes.get_mut(&nid) {
                node.flag = new_flag;
            }
        }
    }

    /// Baut einen read-only Spatial-Index aus allen Nodes.
    pub fn build_spatial_index(&self) -> SpatialIndex {
        self.spatial_index.clone()
    }

    /// Baut den persistenten Spatial-Index aus den aktuellen Nodes neu auf.
    pub fn rebuild_spatial_index(&mut self) {
        self.spatial_index = SpatialIndex::from_nodes(&self.nodes);
    }

    /// Findet den nächstgelegenen Node zur Weltposition.
    pub fn nearest_node(&self, query: Vec2) -> Option<SpatialMatch> {
        self.spatial_index.nearest(query)
    }

    /// Findet alle Nodes innerhalb eines Radius.
    pub fn nodes_within_radius(&self, query: Vec2, radius: f32) -> Vec<SpatialMatch> {
        self.spatial_index.within_radius(query, radius)
    }

    /// Findet alle Nodes innerhalb eines Rechtecks.
    pub fn nodes_within_rect(&self, min: Vec2, max: Vec2) -> Vec<u64> {
        self.spatial_index.within_rect(min, max)
    }

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
    ///
    /// AutoDrive erzeugt bei der automatischen Weggenerierung oft das gesamte Netzwerk
    /// mehrfach. Diese Methode:
    /// 1. Gruppiert Nodes nach quantisierter Position (Grid-Zellen der Größe `epsilon`)
    /// 2. Behält pro Gruppe den Node mit der niedrigsten ID (= Original)
    /// 3. Leitet Verbindungen der Duplikate auf die kanonische ID um
    /// 4. Vereinigt Verbindungen (keine gehen verloren)
    /// 5. Aktualisiert Map-Marker auf kanonische IDs
    ///
    /// Gibt ein `DeduplicationResult` mit Statistiken zurück.
    pub fn deduplicate_nodes(&mut self, epsilon: f32) -> DeduplicationResult {
        let inv_epsilon = if epsilon > 0.0 { 1.0 / epsilon } else { 1000.0 };

        // Schritt 1: Nodes nach quantisierter Position gruppieren
        let mut grid: HashMap<(i64, i64), Vec<u64>> = HashMap::new();
        for (&id, node) in &self.nodes {
            let gx = (node.position.x * inv_epsilon).round() as i64;
            let gz = (node.position.y * inv_epsilon).round() as i64;
            grid.entry((gx, gz)).or_default().push(id);
        }

        // Schritt 2: Mapping von Duplikat-ID → kanonische ID (niedrigste ID pro Gruppe)
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
            return DeduplicationResult {
                removed_nodes: 0,
                remapped_connections: 0,
                removed_self_connections: 0,
                remapped_markers: 0,
                duplicate_groups: 0,
            };
        }

        let removed_nodes = duplicate_ids.len();

        // Schritt 3: Verbindungen umleiten
        // Alle Verbindungen einsammeln, remappen und de-duplizieren
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

            // Selbst-Referenz nach Remap → verwerfen
            if conn.start_id == conn.end_id {
                removed_self_connections += 1;
                continue;
            }

            let was_remapped = conn.start_id != orig_start || conn.end_id != orig_end;
            if was_remapped {
                remapped_connections += 1;
                // Geometrie mit neuen Positionen aktualisieren
                if let (Some(s), Some(e)) = (
                    self.nodes.get(&conn.start_id).map(|n| n.position),
                    self.nodes.get(&conn.end_id).map(|n| n.position),
                ) {
                    conn.update_geometry(s, e);
                }
            }

            let key = (conn.start_id, conn.end_id);
            // Beim Merge: Dual hat Vorrang vor Regular/Reverse
            if let Some(existing) = self.connections.get_mut(&key) {
                if conn.direction == ConnectionDirection::Dual {
                    existing.direction = ConnectionDirection::Dual;
                }
            } else {
                self.connections.insert(key, conn);
            }
        }

        // Schritt 4: Duplikat-Nodes entfernen
        for &dup_id in &duplicate_ids {
            self.nodes.remove(&dup_id);
        }

        // Schritt 5: Map-Marker umleiten
        let mut remapped_markers = 0u32;
        for marker in &mut self.map_markers {
            if let Some(&canonical) = remap.get(&marker.id) {
                marker.id = canonical;
                remapped_markers += 1;
            }
        }
        // Duplikat-Marker entfernen (gleiche kanonische ID)
        let mut seen_marker_ids = std::collections::HashSet::new();
        self.map_markers.retain(|m| seen_marker_ids.insert(m.id));

        // Spatial-Index neu aufbauen
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

impl Default for RoadMap {
    fn default() -> Self {
        Self::new(3) // FS25 als Default
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::core::{ConnectionDirection, ConnectionPriority, NodeFlag};
    use glam::Vec2;

    #[test]
    fn test_roadmap_creation() {
        let mut map = RoadMap::new(3);

        let node = MapNode::new(1, Vec2::new(100.0, 300.0), NodeFlag::Regular);
        map.add_node(node);

        assert_eq!(map.node_count(), 1);
        assert_eq!(map.connection_count(), 0);
        assert_eq!(map.marker_count(), 0);
    }

    #[test]
    fn test_rebuild_connection_geometry() {
        let mut map = RoadMap::new(3);

        let node_a = MapNode::new(1, Vec2::new(0.0, 0.0), NodeFlag::Regular);
        let node_b = MapNode::new(2, Vec2::new(10.0, 0.0), NodeFlag::Regular);
        map.add_node(node_a);
        map.add_node(node_b);

        let connection = Connection::new(
            1,
            2,
            ConnectionDirection::Regular,
            ConnectionPriority::Regular,
            Vec2::new(0.0, 0.0),
            Vec2::new(10.0, 0.0),
        );
        map.add_connection(connection);

        map.rebuild_connection_geometry();

        let connection = map.connections_iter().next().expect("Verbindung erwartet");
        assert_eq!(connection.midpoint, Vec2::new(5.0, 0.0));
        assert_eq!(connection.angle, 0.0);
    }

    #[test]
    fn test_spatial_queries() {
        let mut map = RoadMap::new(3);
        map.add_node(MapNode::new(1, Vec2::new(0.0, 0.0), NodeFlag::Regular));
        map.add_node(MapNode::new(2, Vec2::new(10.0, 0.0), NodeFlag::Regular));
        map.add_node(MapNode::new(3, Vec2::new(5.0, 5.0), NodeFlag::Regular));

        let nearest = map
            .nearest_node(Vec2::new(5.2, 5.1))
            .expect("Treffer erwartet");
        assert_eq!(nearest.node_id, 3);

        let mut in_rect = map.nodes_within_rect(Vec2::new(-1.0, -1.0), Vec2::new(6.0, 6.0));
        in_rect.sort_unstable();
        assert_eq!(in_rect, vec![1, 3]);
    }

    #[test]
    fn test_spatial_index_consistency_on_remove_and_update() {
        let mut map = RoadMap::new(3);
        map.add_node(MapNode::new(1, Vec2::new(0.0, 0.0), NodeFlag::Regular));
        map.add_node(MapNode::new(2, Vec2::new(10.0, 0.0), NodeFlag::Regular));

        assert_eq!(
            map.nearest_node(Vec2::new(9.8, 0.1)).map(|m| m.node_id),
            Some(2)
        );

        assert!(map.update_node_position(2, Vec2::new(2.0, 0.0)));
        assert_eq!(
            map.nearest_node(Vec2::new(2.1, 0.0)).map(|m| m.node_id),
            Some(2)
        );

        let removed = map.remove_node(2);
        assert!(removed.is_some());
        assert_eq!(
            map.nearest_node(Vec2::new(2.1, 0.0)).map(|m| m.node_id),
            Some(1)
        );

        let mut ids = map.nodes_within_rect(Vec2::new(-1.0, -1.0), Vec2::new(3.0, 1.0));
        ids.sort_unstable();
        assert_eq!(ids, vec![1]);
    }

    #[test]
    fn test_recalculate_node_flags_subprio_only() {
        let mut map = RoadMap::new(3);
        map.add_node(MapNode::new(1, Vec2::ZERO, NodeFlag::Regular));
        map.add_node(MapNode::new(2, Vec2::new(10.0, 0.0), NodeFlag::Regular));

        // Nur SubPriority-Verbindung → beide werden SubPrio
        let conn = Connection::new(
            1,
            2,
            ConnectionDirection::Regular,
            ConnectionPriority::SubPriority,
            Vec2::ZERO,
            Vec2::new(10.0, 0.0),
        );
        map.add_connection(conn);
        map.recalculate_node_flags(&[1, 2]);

        assert_eq!(map.nodes[&1].flag, NodeFlag::SubPrio);
        assert_eq!(map.nodes[&2].flag, NodeFlag::SubPrio);
    }

    #[test]
    fn test_recalculate_node_flags_mixed_priority() {
        let mut map = RoadMap::new(3);
        map.add_node(MapNode::new(1, Vec2::ZERO, NodeFlag::Regular));
        map.add_node(MapNode::new(2, Vec2::new(10.0, 0.0), NodeFlag::Regular));
        map.add_node(MapNode::new(3, Vec2::new(20.0, 0.0), NodeFlag::Regular));

        // Node 2 hat eine Regular- und eine SubPriority-Verbindung → bleibt Regular
        let c1 = Connection::new(
            1,
            2,
            ConnectionDirection::Regular,
            ConnectionPriority::Regular,
            Vec2::ZERO,
            Vec2::new(10.0, 0.0),
        );
        let c2 = Connection::new(
            2,
            3,
            ConnectionDirection::Regular,
            ConnectionPriority::SubPriority,
            Vec2::new(10.0, 0.0),
            Vec2::new(20.0, 0.0),
        );
        map.add_connection(c1);
        map.add_connection(c2);
        map.recalculate_node_flags(&[1, 2, 3]);

        assert_eq!(map.nodes[&1].flag, NodeFlag::Regular);
        assert_eq!(map.nodes[&2].flag, NodeFlag::Regular);
        assert_eq!(map.nodes[&3].flag, NodeFlag::SubPrio);
    }

    #[test]
    fn test_recalculate_node_flags_preserves_warning() {
        let mut map = RoadMap::new(3);
        map.add_node(MapNode::new(1, Vec2::ZERO, NodeFlag::Warning));
        map.add_node(MapNode::new(2, Vec2::new(10.0, 0.0), NodeFlag::Regular));

        let conn = Connection::new(
            1,
            2,
            ConnectionDirection::Regular,
            ConnectionPriority::SubPriority,
            Vec2::ZERO,
            Vec2::new(10.0, 0.0),
        );
        map.add_connection(conn);
        map.recalculate_node_flags(&[1, 2]);

        // Warning darf nicht verändert werden
        assert_eq!(map.nodes[&1].flag, NodeFlag::Warning);
        assert_eq!(map.nodes[&2].flag, NodeFlag::SubPrio);
    }

    #[test]
    fn test_recalculate_node_flags_no_connections() {
        let mut map = RoadMap::new(3);
        map.add_node(MapNode::new(1, Vec2::ZERO, NodeFlag::SubPrio));

        // Node ohne Verbindungen → wird Regular
        map.recalculate_node_flags(&[1]);
        assert_eq!(map.nodes[&1].flag, NodeFlag::Regular);
    }

    // ── Deduplizierungs-Tests ──────────────────────────────────

    #[test]
    fn test_deduplicate_no_duplicates() {
        let mut map = RoadMap::new(3);
        map.add_node(MapNode::new(1, Vec2::new(0.0, 0.0), NodeFlag::Regular));
        map.add_node(MapNode::new(2, Vec2::new(10.0, 0.0), NodeFlag::Regular));

        let result = map.deduplicate_nodes(0.01);
        assert!(!result.had_duplicates());
        assert_eq!(map.node_count(), 2);
    }

    #[test]
    fn test_deduplicate_removes_exact_duplicates() {
        let mut map = RoadMap::new(3);
        // Zwei Nodes an exakt gleicher Position
        map.add_node(MapNode::new(1, Vec2::new(100.0, 200.0), NodeFlag::Regular));
        map.add_node(MapNode::new(2, Vec2::new(100.0, 200.0), NodeFlag::Regular));
        map.add_node(MapNode::new(3, Vec2::new(50.0, 50.0), NodeFlag::Regular));

        // Verbindungen: 1→3 und 2→3 (werden zu 1→3 gemergt)
        let c1 = Connection::new(
            1, 3,
            ConnectionDirection::Regular, ConnectionPriority::Regular,
            Vec2::new(100.0, 200.0), Vec2::new(50.0, 50.0),
        );
        let c2 = Connection::new(
            2, 3,
            ConnectionDirection::Regular, ConnectionPriority::Regular,
            Vec2::new(100.0, 200.0), Vec2::new(50.0, 50.0),
        );
        map.add_connection(c1);
        map.add_connection(c2);

        let result = map.deduplicate_nodes(0.01);
        assert!(result.had_duplicates());
        assert_eq!(result.removed_nodes, 1);
        assert_eq!(result.duplicate_groups, 1);
        assert_eq!(map.node_count(), 2); // 1 und 3 bleiben
        assert!(map.nodes.contains_key(&1));
        assert!(!map.nodes.contains_key(&2)); // Duplikat entfernt
        assert!(map.has_connection(1, 3));
    }

    #[test]
    fn test_deduplicate_keeps_lowest_id() {
        let mut map = RoadMap::new(3);
        map.add_node(MapNode::new(5, Vec2::new(10.0, 10.0), NodeFlag::Regular));
        map.add_node(MapNode::new(2, Vec2::new(10.0, 10.0), NodeFlag::Regular));
        map.add_node(MapNode::new(8, Vec2::new(10.0, 10.0), NodeFlag::Regular));

        let result = map.deduplicate_nodes(0.01);
        assert_eq!(result.removed_nodes, 2);
        assert_eq!(map.node_count(), 1);
        assert!(map.nodes.contains_key(&2)); // Niedrigste ID bleibt
    }

    #[test]
    fn test_deduplicate_remaps_connections() {
        let mut map = RoadMap::new(3);
        // A(1) und A'(10) an gleicher Position, B(2) und B'(20) an gleicher Position
        map.add_node(MapNode::new(1, Vec2::new(0.0, 0.0), NodeFlag::Regular));
        map.add_node(MapNode::new(10, Vec2::new(0.0, 0.0), NodeFlag::Regular));
        map.add_node(MapNode::new(2, Vec2::new(10.0, 0.0), NodeFlag::Regular));
        map.add_node(MapNode::new(20, Vec2::new(10.0, 0.0), NodeFlag::Regular));

        // Original-Netzwerk: 1→2, Duplikat-Netzwerk: 10→20
        let c1 = Connection::new(
            1, 2,
            ConnectionDirection::Dual, ConnectionPriority::Regular,
            Vec2::new(0.0, 0.0), Vec2::new(10.0, 0.0),
        );
        let c2 = Connection::new(
            10, 20,
            ConnectionDirection::Regular, ConnectionPriority::Regular,
            Vec2::new(0.0, 0.0), Vec2::new(10.0, 0.0),
        );
        map.add_connection(c1);
        map.add_connection(c2);

        let result = map.deduplicate_nodes(0.01);
        assert_eq!(result.removed_nodes, 2);
        assert_eq!(map.node_count(), 2);
        assert_eq!(map.connection_count(), 1);
        // Die überlebende Verbindung sollte Dual sein (hat Vorrang)
        let conn = map.find_connection(1, 2).expect("Verbindung 1→2 erwartet");
        assert_eq!(conn.direction, ConnectionDirection::Dual);
    }

    #[test]
    fn test_deduplicate_removes_self_connections() {
        let mut map = RoadMap::new(3);
        map.add_node(MapNode::new(1, Vec2::new(0.0, 0.0), NodeFlag::Regular));
        map.add_node(MapNode::new(2, Vec2::new(0.0, 0.0), NodeFlag::Regular));

        // Verbindung 1→2 wird nach Remap zu 1→1 = Selbstreferenz
        let conn = Connection::new(
            1, 2,
            ConnectionDirection::Regular, ConnectionPriority::Regular,
            Vec2::new(0.0, 0.0), Vec2::new(0.0, 0.0),
        );
        map.add_connection(conn);

        let result = map.deduplicate_nodes(0.01);
        assert_eq!(result.removed_nodes, 1);
        assert_eq!(result.removed_self_connections, 1);
        assert_eq!(map.connection_count(), 0);
    }

    #[test]
    fn test_deduplicate_updates_markers() {
        let mut map = RoadMap::new(3);
        map.add_node(MapNode::new(1, Vec2::new(0.0, 0.0), NodeFlag::Regular));
        map.add_node(MapNode::new(5, Vec2::new(0.0, 0.0), NodeFlag::Regular));

        use crate::core::MapMarker;
        map.map_markers.push(MapMarker::new(5, "TestMarker".to_string(), "All".to_string(), 1, false));

        let result = map.deduplicate_nodes(0.01);
        assert_eq!(result.remapped_markers, 1);
        assert_eq!(map.map_markers.len(), 1);
        assert_eq!(map.map_markers[0].id, 1); // Umgeleitet auf kanonische ID
    }

    #[test]
    fn test_deduplicate_within_epsilon_tolerance() {
        let mut map = RoadMap::new(3);
        // Zwei Nodes innerhalb derselben Grid-Zelle (epsilon=0.01)
        // 100.004 und 100.004 runden beide auf Grid-Zelle 10000
        map.add_node(MapNode::new(1, Vec2::new(100.004, 200.004), NodeFlag::Regular));
        map.add_node(MapNode::new(2, Vec2::new(100.004, 200.004), NodeFlag::Regular));

        let result = map.deduplicate_nodes(0.01);
        assert!(result.had_duplicates());
        assert_eq!(map.node_count(), 1);
    }

    #[test]
    fn test_deduplicate_outside_epsilon_no_merge() {
        let mut map = RoadMap::new(3);
        // Zwei Nodes außerhalb epsilon=0.01 Abstand
        map.add_node(MapNode::new(1, Vec2::new(100.0, 200.0), NodeFlag::Regular));
        map.add_node(MapNode::new(2, Vec2::new(100.02, 200.0), NodeFlag::Regular));

        let result = map.deduplicate_nodes(0.01);
        assert!(!result.had_duplicates());
        assert_eq!(map.node_count(), 2);
    }
}
