//! Die zentrale RoadMap-Datenstruktur mit Nodes, Connections und Spatial-Index.

use super::{
    AutoDriveMeta, Connection, ConnectionDirection, ConnectionPriority, MapMarker, MapNode,
};
use super::{SpatialIndex, SpatialMatch};
use glam::Vec2;
use std::collections::HashMap;

/// Ein Nachbar-Node, der über eine Verbindung erreichbar ist.
#[derive(Debug, Clone, Copy)]
pub struct ConnectedNeighbor {
    /// ID des Nachbar-Nodes
    pub neighbor_id: u64,
    /// Winkel der Verbindung (Radiant, atan2) — zeigt vom Quell-Node zum Nachbar
    pub angle: f32,
    /// true = Verbindung geht vom Quell-Node zum Nachbar (outgoing)
    pub is_outgoing: bool,
}

mod dedup;
pub use dedup::DeduplicationResult;

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
    /// Signalisiert, dass der Spatial-Index veraltet ist und rebuild benötigt
    spatial_dirty: bool,
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
            spatial_dirty: false,
        }
    }

    /// Fügt einen Node hinzu
    pub fn add_node(&mut self, node: MapNode) {
        self.nodes.insert(node.id, node);
        self.spatial_dirty = true;
    }

    /// Entfernt einen Node inklusive aller betroffenen Verbindungen
    pub fn remove_node(&mut self, node_id: u64) -> Option<MapNode> {
        let removed = self.nodes.remove(&node_id);
        if removed.is_some() {
            self.connections
                .retain(|(s, e), _| *s != node_id && *e != node_id);
            self.spatial_dirty = true;
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
        self.spatial_dirty = true;
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

    /// Gibt alle Connections zurück, deren Start- und End-Ids in der gegebenen Menge liegen.
    ///
    /// Verwendet zum Filtern von Connections zwischen selektierten Nodes.
    /// O(n) über alle Connections, aber nur bei Use-Cases aufgerufen (nicht per-Frame).
    pub fn connections_between_ids<'a>(
        &'a self,
        ids: &'a std::collections::HashSet<u64>,
    ) -> Box<dyn Iterator<Item = &'a Connection> + 'a> {
        Box::new(
            self.connections
                .values()
                .filter(move |c| ids.contains(&c.start_id) && ids.contains(&c.end_id)),
        )
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
    ///
    /// **Bulk-Optimierung:** Statt für jeden Node alle Verbindungen zu scannen
    /// (O(n·m)), wird ein temporärer Adjacency-Cache aufgebaut (O(m)), der dann
    /// in O(n) abgefragt wird. Netto O(n+m) statt O(n·m).
    pub fn recalculate_node_flags(&mut self, node_ids: &[u64]) {
        use super::NodeFlag;

        if node_ids.is_empty() {
            return;
        }

        // Adjacency-Cache aufbauen: node_id → (has_any, has_regular)
        // Nur für die übergebenen Node-IDs relevant — alle anderen überspringen
        let node_set: std::collections::HashSet<u64> = node_ids.iter().copied().collect();

        let mut has_any: std::collections::HashMap<u64, bool> =
            node_set.iter().map(|&id| (id, false)).collect();
        let mut has_regular: std::collections::HashMap<u64, bool> =
            node_set.iter().map(|&id| (id, false)).collect();

        for conn in self.connections.values() {
            for &nid in &[conn.start_id, conn.end_id] {
                if node_set.contains(&nid) {
                    has_any.insert(nid, true);
                    if conn.priority == ConnectionPriority::Regular {
                        has_regular.insert(nid, true);
                    }
                }
            }
        }

        for &nid in node_ids {
            let Some(node) = self.nodes.get(&nid) else {
                continue;
            };

            // Warning/Reserved nicht anfassen
            match node.flag {
                NodeFlag::Warning | NodeFlag::Reserved => continue,
                _ => {}
            }

            let any = has_any.get(&nid).copied().unwrap_or(false);
            let regular = has_regular.get(&nid).copied().unwrap_or(false);

            let new_flag = if !any || regular {
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

    /// Baut den persistenten Spatial-Index neu auf, falls er veraltet ist.
    ///
    /// Muss nach einer Serie von Mutationen (add_node, remove_node, etc.)
    /// aufgerufen werden, bevor der `Arc<RoadMap>` wieder geteilt wird.
    pub fn ensure_spatial_index(&mut self) {
        if self.spatial_dirty {
            self.spatial_index = SpatialIndex::from_nodes(&self.nodes);
            self.spatial_dirty = false;
        }
    }

    /// Baut den persistenten Spatial-Index aus den aktuellen Nodes neu auf.
    pub fn rebuild_spatial_index(&mut self) {
        self.spatial_index = SpatialIndex::from_nodes(&self.nodes);
        self.spatial_dirty = false;
    }

    /// Findet den nächstgelegenen Node zur Weltposition.
    pub fn nearest_node(&self, query: Vec2) -> Option<SpatialMatch> {
        debug_assert!(
            !self.spatial_dirty,
            "Spatial-Index ist veraltet — ensure_spatial_index() fehlt"
        );
        self.spatial_index.nearest(query)
    }

    /// Findet alle Nodes innerhalb eines Radius.
    pub fn nodes_within_radius(&self, query: Vec2, radius: f32) -> Vec<SpatialMatch> {
        debug_assert!(
            !self.spatial_dirty,
            "Spatial-Index ist veraltet — ensure_spatial_index() fehlt"
        );
        self.spatial_index.within_radius(query, radius)
    }

    /// Findet alle Nodes innerhalb eines Rechtecks.
    pub fn nodes_within_rect(&self, min: Vec2, max: Vec2) -> Vec<u64> {
        debug_assert!(
            !self.spatial_dirty,
            "Spatial-Index ist veraltet — ensure_spatial_index() fehlt"
        );
        self.spatial_index.within_rect(min, max)
    }
}

impl Default for RoadMap {
    fn default() -> Self {
        Self::new(3) // FS25 als Default
    }
}

#[cfg(test)]
mod tests;
