//! Die zentrale RoadMap-Datenstruktur mit Nodes, Connections und Spatial-Index.

use super::SpatialIndex;
use super::{
    AutoDriveMeta, Connection, ConnectionDirection, ConnectionPriority, MapMarker, MapNode,
};
use glam::Vec2;
use std::collections::HashMap;

/// Ein Nachbar-Node, der ueber eine Verbindung erreichbar ist.
#[derive(Debug, Clone, Copy)]
pub struct ConnectedNeighbor {
    /// ID des Nachbar-Nodes
    pub neighbor_id: u64,
    /// Winkel der Verbindung (Radiant, atan2) — zeigt vom Quell-Node zum Nachbar
    pub angle: f32,
    /// true = Verbindung geht vom Quell-Node zum Nachbar (outgoing)
    pub is_outgoing: bool,
}

/// Ein Node, der Verbindungen ausserhalb einer Gruppe hat (Ein-/Ausfahrt).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct BoundaryNode {
    /// ID des Boundary-Nodes
    pub node_id: u64,
    /// true = Node hat mindestens eine eingehende Verbindung von ausserhalb
    pub has_external_incoming: bool,
    /// true = Node hat mindestens eine ausgehende Verbindung nach ausserhalb
    pub has_external_outgoing: bool,
}

mod boundary;
mod chain;
mod dedup;
mod neighbors;
mod query;
pub use dedup::DeduplicationResult;

/// Vollstaendige AutoDrive-Konfiguration
#[derive(Debug, Clone)]
pub struct RoadMap {
    /// Alle Wegpunkte, indexiert nach ihrer ID
    pub nodes: HashMap<u64, MapNode>,
    /// Alle Verbindungen, indexiert nach (start_id, end_id) fuer O(1)-Zugriff
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
    /// Signalisiert, dass der Spatial-Index veraltet ist und rebuild benoetigt
    spatial_dirty: bool,
    /// Adjacency-Index: Node-ID → Liste von (Nachbar-ID, ist_ausgehend).
    /// Wird bei jeder Connection-Mutation synchron gepflegt.
    adjacency: HashMap<u64, Vec<(u64, bool)>>,
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
            adjacency: HashMap::new(),
        }
    }

    /// Fuegt einen Node hinzu
    pub fn add_node(&mut self, node: MapNode) {
        self.adjacency.entry(node.id).or_default();
        self.nodes.insert(node.id, node);
        self.spatial_dirty = true;
    }

    /// Entfernt einen Node inklusive aller betroffenen Verbindungen
    pub fn remove_node(&mut self, node_id: u64) -> Option<MapNode> {
        let removed = self.nodes.remove(&node_id);
        if removed.is_some() {
            // Adjacency-Eintraege der Nachbarn bereinigen — muss vor connections.retain() geschehen
            let neighbors: Vec<u64> = self
                .adjacency
                .get(&node_id)
                .map(|v| v.iter().map(|&(nb, _)| nb).collect())
                .unwrap_or_default();
            for nb in neighbors {
                if let Some(adj) = self.adjacency.get_mut(&nb) {
                    adj.retain(|&(id, _)| id != node_id);
                }
            }
            self.adjacency.remove(&node_id);

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

    /// Setzt das Flag eines Nodes — O(1)
    pub fn set_node_flag(&mut self, node_id: u64, flag: super::NodeFlag) -> bool {
        if let Some(node) = self.nodes.get_mut(&node_id) {
            node.flag = flag;
            true
        } else {
            false
        }
    }

    /// Fuegt eine Verbindung hinzu
    pub fn add_connection(&mut self, connection: Connection) {
        let s = connection.start_id;
        let e = connection.end_id;
        self.adjacency.entry(s).or_default().push((e, true));
        self.adjacency.entry(e).or_default().push((s, false));
        self.connections.insert((s, e), connection);
    }

    /// Prueft ob eine Verbindung existiert (exaktes Match auf start_id + end_id) — O(1)
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
        if self.connections.remove(&(start_id, end_id)).is_some() {
            if let Some(adj) = self.adjacency.get_mut(&start_id) {
                adj.retain(|&(nb, out)| nb != end_id || !out);
            }
            if let Some(adj) = self.adjacency.get_mut(&end_id) {
                adj.retain(|&(nb, out)| nb != start_id || out);
            }
            true
        } else {
            false
        }
    }

    /// Entfernt alle Verbindungen zwischen zwei Nodes (in beiden Richtungen) — O(1)
    pub fn remove_connections_between(&mut self, node_a: u64, node_b: u64) -> usize {
        let mut removed = 0;
        if self.connections.remove(&(node_a, node_b)).is_some() {
            // A→B: adj[A] verliert (B, true), adj[B] verliert (A, false)
            if let Some(adj) = self.adjacency.get_mut(&node_a) {
                adj.retain(|&(nb, out)| nb != node_b || !out);
            }
            if let Some(adj) = self.adjacency.get_mut(&node_b) {
                adj.retain(|&(nb, out)| nb != node_a || out);
            }
            removed += 1;
        }
        if self.connections.remove(&(node_b, node_a)).is_some() {
            // B→A: adj[B] verliert (A, true), adj[A] verliert (B, false)
            if let Some(adj) = self.adjacency.get_mut(&node_b) {
                adj.retain(|&(nb, out)| nb != node_a || !out);
            }
            if let Some(adj) = self.adjacency.get_mut(&node_a) {
                adj.retain(|&(nb, out)| nb != node_b || out);
            }
            removed += 1;
        }
        removed
    }

    /// Aendert die Richtung einer bestehenden Verbindung — O(1)
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

    /// Aendert die Prioritaet einer bestehenden Verbindung — O(1)
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

            // Adjacency: in adj[start_id] (end_id, true) → (end_id, false)
            if let Some(adj) = self.adjacency.get_mut(&start_id) {
                for entry in adj.iter_mut() {
                    if entry.0 == end_id && entry.1 {
                        entry.1 = false;
                        break;
                    }
                }
            }
            // Adjacency: in adj[end_id] (start_id, false) → (start_id, true)
            if let Some(adj) = self.adjacency.get_mut(&end_id) {
                for entry in adj.iter_mut() {
                    if entry.0 == start_id && !entry.1 {
                        entry.1 = true;
                        break;
                    }
                }
            }
            true
        } else {
            false
        }
    }

    /// Iterator ueber alle Verbindungen (read-only).
    pub fn connections_iter(&self) -> impl Iterator<Item = &Connection> {
        self.connections.values()
    }

    /// Berechnet die naechste freie Node-ID
    pub fn next_node_id(&self) -> u64 {
        self.nodes.keys().max().copied().unwrap_or(0) + 1
    }

    /// Fuegt einen Map-Marker hinzu
    pub fn add_map_marker(&mut self, marker: MapMarker) {
        self.map_markers.push(marker);
    }

    /// Prueft ob ein Node einen Marker hat
    pub fn has_marker(&self, node_id: u64) -> bool {
        self.map_markers.iter().any(|m| m.id == node_id)
    }

    /// Findet Marker fuer einen Node
    pub fn find_marker_by_node_id(&self, node_id: u64) -> Option<&MapMarker> {
        self.map_markers.iter().find(|m| m.id == node_id)
    }

    /// Entfernt Marker fuer einen Node (gibt true zurueck falls gefunden)
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

    /// Gibt die Anzahl der Nodes zurueck
    pub fn node_count(&self) -> usize {
        self.nodes.len()
    }

    /// Gibt die Anzahl der Verbindungen zurueck
    pub fn connection_count(&self) -> usize {
        self.connections.len()
    }

    /// Gibt die Anzahl der Map-Marker zurueck
    pub fn marker_count(&self) -> usize {
        self.map_markers.len()
    }

    /// Berechnet die NodeFlags (Regular/SubPrio) fuer die angegebenen Nodes neu.
    ///
    /// Logik (entspricht AutoDrive FLAG_REGULAR / FLAG_SUBPRIO):
    /// - Mindestens eine Verbindung mit `ConnectionPriority::Regular` → `Regular`
    /// - Nur Verbindungen mit `ConnectionPriority::SubPriority` → `SubPrio`
    /// - Keine Verbindungen → `Regular` (Default)
    /// - Nodes mit Warning/Reserved-Flag werden nicht veraendert.
    ///
    /// **Bulk-Optimierung:** Statt fuer jeden Node alle Verbindungen zu scannen
    /// (O(n·m)), wird ein temporaerer Adjacency-Cache aufgebaut (O(m)), der dann
    /// in O(n) abgefragt wird. Netto O(n+m) statt O(n·m).
    pub fn recalculate_node_flags(&mut self, node_ids: &[u64]) {
        use super::NodeFlag;

        if node_ids.is_empty() {
            return;
        }

        // Adjacency-Cache aufbauen: node_id → (has_any, has_regular)
        // Nur fuer die uebergebenen Node-IDs relevant — alle anderen ueberspringen
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

            // Warning/Reserved/RoundedCorner nicht anfassen
            match node.flag {
                NodeFlag::Warning | NodeFlag::Reserved | NodeFlag::RoundedCorner => continue,
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

    /// Baut den Adjacency-Index vollstaendig aus den aktuellen Nodes und Connections neu auf.
    ///
    /// Internale Methode — aufrufen nach Bulk-Ladungen oder Dedup-Operationen.
    fn rebuild_adjacency(&mut self) {
        self.adjacency.clear();
        for &node_id in self.nodes.keys() {
            self.adjacency.entry(node_id).or_default();
        }
        for conn in self.connections.values() {
            self.adjacency
                .entry(conn.start_id)
                .or_default()
                .push((conn.end_id, true));
            self.adjacency
                .entry(conn.end_id)
                .or_default()
                .push((conn.start_id, false));
        }
    }

    /// Baut den Adjacency-Index neu auf — oeffentlich fuer den XML-Parser und Bulk-Operationen.
    ///
    /// Nach Mass-Inserts (z. B. XML-Laden, `deduplicate_nodes`) aufrufen,
    /// damit isolierte Nodes einen leeren Eintrag erhalten und der Index konsistent ist.
    pub fn rebuild_adjacency_index(&mut self) {
        self.rebuild_adjacency();
    }

    /// Gibt alle Nachbar-Eintraege eines Nodes zurueck — O(1) Lookup.
    ///
    /// Jeder Eintrag ist `(nachbar_id, ist_ausgehend)`:
    /// - `true`  = Verbindung von `node_id` zum Nachbar (outgoing)
    /// - `false` = Verbindung vom Nachbar zu `node_id` (incoming)
    ///
    /// Gibt einen leeren Slice zurueck, wenn der Node unbekannt ist.
    pub fn neighbors(&self, node_id: u64) -> &[(u64, bool)] {
        self.adjacency
            .get(&node_id)
            .map(Vec::as_slice)
            .unwrap_or(&[])
    }

    /// Iterator ueber alle ausgehenden Nachbar-IDs eines Nodes.
    pub fn outgoing_neighbors(&self, node_id: u64) -> impl Iterator<Item = u64> + '_ {
        self.adjacency
            .get(&node_id)
            .into_iter()
            .flat_map(|v| v.iter())
            .filter_map(|&(nb, out)| if out { Some(nb) } else { None })
    }

    /// Iterator ueber alle eingehenden Nachbar-IDs eines Nodes.
    pub fn incoming_neighbors(&self, node_id: u64) -> impl Iterator<Item = u64> + '_ {
        self.adjacency
            .get(&node_id)
            .into_iter()
            .flat_map(|v| v.iter())
            .filter_map(|&(nb, out)| if !out { Some(nb) } else { None })
    }

    /// Gibt den Grad (Anzahl aller Verbindungen, ein- und ausgehend) eines Nodes zurueck — O(1).
    pub fn degree(&self, node_id: u64) -> usize {
        self.adjacency.get(&node_id).map(Vec::len).unwrap_or(0)
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
}

impl Default for RoadMap {
    fn default() -> Self {
        Self::new(3) // FS25 als Default
    }
}

#[cfg(test)]
mod tests;
