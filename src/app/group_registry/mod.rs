//! In-Session-Registry aller erstellten Segmente (zum nachtraeglichen Bearbeiten).
//! NOTE: In der UI als "Gruppe" bezeichnet — intern bleibt "Segment" als historischer Begriff.
//!
//! Wird **nicht** in den Undo/Redo-Snapshot aufgenommen — die Registry ist
//! transient und gilt nur fuer die aktuelle Session. Beim Laden einer Datei
//! ist sie leer.
//!
//! Beim Bearbeiten eines Segments werden die zugehoerigen Nodes aus der
//! RoadMap geloescht und das passende Tool mit den gespeicherten Parametern
//! neu geladen.
//!
//! # Modulstruktur
//! - [`types`]: Datentypes (`GroupBase`, `GroupKind`, `GroupRecord`) und Tool-Index-Konstanten
//! - Dieses Modul: [`GroupRegistry`] mit allen CRUD-Operationen

mod types;
pub use types::*;

use crate::core::RoadMap;
use glam::Vec2;
use std::collections::HashMap;

/// In-Session-Registry aller erstellten Segmente.
///
/// Ermoeglicht das nachtraegliche Bearbeiten von Segmenten, indem die
/// Tool-Parameter beim Erstellen gespeichert und beim Bearbeiten
/// wiederhergestellt werden.
///
/// Interne Speicherung als `HashMap<u64, GroupRecord>` fuer O(1)-Zugriffe.
/// Ein Reverse-Index `node_to_records` ermoeglicht effiziente Node→Segment-Abfragen.
#[derive(Debug, Clone, Default)]
pub struct GroupRegistry {
    /// Primaere Speicherstruktur: Record-ID → GroupRecord.
    records: HashMap<u64, GroupRecord>,
    next_id: u64,
    /// Record-ID, die von automatischer Invalidierung ausgenommen ist (aktiver Group-Edit).
    edit_guard_id: Option<u64>,
    /// Reverse-Index: Node-ID → Liste der zugehoerigen Record-IDs.
    node_to_records: HashMap<u64, Vec<u64>>,
    /// Cache fuer gecachte Boundary-Infos pro Record (Key = record_id).
    /// Wird bei register/remove/update_record invalidiert;
    /// bei neuem RoadMap-Pointer komplett geleert.
    boundary_cache: HashMap<u64, Vec<BoundaryInfo>>,
    /// Adresse der zuletzt verwendeten RoadMap fuer PTR-basierten Cache-Reset.
    last_roadmap_ptr: usize,
}

impl GroupRegistry {
    /// Erstellt eine leere Registry.
    pub fn new() -> Self {
        Self::default()
    }

    /// Fuegt alle Node-IDs eines Records zum Reverse-Index hinzu.
    fn add_to_index(&mut self, record_id: u64, node_ids: &[u64]) {
        for &nid in node_ids {
            self.node_to_records.entry(nid).or_default().push(record_id);
        }
    }

    /// Entfernt alle Node-IDs eines Records aus dem Reverse-Index.
    fn remove_from_index(&mut self, record_id: u64, node_ids: &[u64]) {
        for &nid in node_ids {
            if let Some(vec) = self.node_to_records.get_mut(&nid) {
                vec.retain(|&id| id != record_id);
                if vec.is_empty() {
                    self.node_to_records.remove(&nid);
                }
            }
        }
    }

    /// Registriert ein neues Segment und gibt die vergebene ID zurueck.
    pub fn register(&mut self, record: GroupRecord) -> u64 {
        let id = record.id;
        for &nid in &record.node_ids {
            self.node_to_records.entry(nid).or_default().push(id);
        }
        self.records.insert(id, record);
        // Cache fuer diesen Record invalidieren (neue Gruppe → neues Boundary-Bild noetig)
        self.boundary_cache.remove(&id);
        id
    }

    /// Erstellt eine neue Record-ID (auto-increment).
    pub fn next_id(&mut self) -> u64 {
        let id = self.next_id;
        self.next_id += 1;
        id
    }

    /// Gibt den Record mit der angegebenen ID zurueck (falls vorhanden).
    pub fn get(&self, record_id: u64) -> Option<&GroupRecord> {
        self.records.get(&record_id)
    }

    /// Entfernt den Record mit der angegebenen ID.
    pub fn remove(&mut self, record_id: u64) {
        if let Some(record) = self.records.remove(&record_id) {
            self.remove_from_index(record_id, &record.node_ids);
        }
        self.boundary_cache.remove(&record_id);
    }

    /// Gibt alle Records zurueck, die mindestens einen der angegebenen Node-IDs enthalten.
    pub fn find_by_node_ids(&self, node_ids: &indexmap::IndexSet<u64>) -> Vec<&GroupRecord> {
        let mut seen_ids = std::collections::HashSet::new();
        let mut result = Vec::new();
        for &nid in node_ids {
            if let Some(record_ids) = self.node_to_records.get(&nid) {
                for &rid in record_ids {
                    if seen_ids.insert(rid) {
                        if let Some(record) = self.records.get(&rid) {
                            result.push(record);
                        }
                    }
                }
            }
        }
        result
    }

    /// Entfernt alle Records, die mindestens einen der angegebenen Node-IDs enthalten.
    ///
    /// Wird aufgerufen wenn Nodes manuell geloescht werden (z.B. Delete-Taste).
    /// Records, deren ID dem aktiven `edit_guard_id` entspricht, werden nie invalidiert.
    pub fn invalidate_by_node_ids(&mut self, node_ids: &[u64]) {
        // IDs der zu entfernenden Records sammeln (ohne edit_guard)
        let mut to_remove: std::collections::HashSet<u64> = std::collections::HashSet::new();
        for &nid in node_ids {
            if let Some(record_ids) = self.node_to_records.get(&nid) {
                for &rid in record_ids {
                    if Some(rid) != self.edit_guard_id {
                        to_remove.insert(rid);
                    }
                }
            }
        }
        for rid in to_remove {
            self.remove(rid);
        }
    }

    /// Findet den ersten Record, der den angegebenen Node enthaelt.
    pub fn find_first_by_node_id(&self, node_id: u64) -> Option<&GroupRecord> {
        self.node_to_records
            .get(&node_id)
            .and_then(|ids| ids.first())
            .and_then(|id| self.records.get(id))
    }

    /// Prueft ob ein Segment noch gueltig ist (Nodes existieren und Positionen unveraendert).
    pub fn is_group_valid(&self, record: &GroupRecord, road_map: &RoadMap) -> bool {
        if record.original_positions.len() != record.node_ids.len() {
            return false;
        }
        record
            .node_ids
            .iter()
            .zip(record.original_positions.iter())
            .all(|(id, orig_pos)| {
                road_map
                    .nodes
                    .get(id)
                    .map(|node| node.position.distance(*orig_pos) < 0.01)
                    .unwrap_or(false)
            })
    }

    /// Gibt alle Records als Iterator zurueck.
    pub fn records(&self) -> impl Iterator<Item = &GroupRecord> {
        self.records.values()
    }

    /// Gibt eine veraenderliche Referenz auf alle Records als Iterator zurueck.
    pub fn records_mut(&mut self) -> impl Iterator<Item = &mut GroupRecord> {
        self.records.values_mut()
    }

    /// Gibt eine Referenz auf die interne HashMap zurueck.
    pub fn records_map(&self) -> &HashMap<u64, GroupRecord> {
        &self.records
    }

    /// Findet alle Segment-IDs, zu denen ein Node gehoert.
    pub fn groups_for_node(&self, node_id: u64) -> Vec<u64> {
        self.node_to_records
            .get(&node_id)
            .cloned()
            .unwrap_or_default()
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
                    if processed_segments.insert(rid) {
                        if let Some(record) = self.records.get(&rid) {
                            if record.locked {
                                expanded.extend(record.node_ids.iter().copied());
                            }
                        }
                    }
                }
            }
        }
        expanded.into_iter().collect()
    }

    /// Aktualisiert die original_positions eines Segments nach einem Locked-Move.
    ///
    /// Liest die aktuellen Node-Positionen aus der RoadMap und ueberschreibt
    /// `original_positions`. Muss nach jedem Locked-Move aufgerufen werden,
    /// damit `is_group_valid()` weiterhin `true` zurueckgibt.
    pub fn update_original_positions(&mut self, segment_id: u64, road_map: &RoadMap) {
        if let Some(record) = self.get_mut(segment_id) {
            record.original_positions = record
                .node_ids
                .iter()
                .filter_map(|id| road_map.nodes.get(id).map(|n| n.position))
                .collect();
        }
    }

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

    /// Gibt den Lock-Zustand des Segments zurueck.
    ///
    /// Gibt `false` zurueck wenn das Segment nicht existiert.
    pub fn is_locked(&self, segment_id: u64) -> bool {
        self.records
            .get(&segment_id)
            .map(|r| r.locked)
            .unwrap_or(false)
    }

    /// Setzt die Record-ID, die von automatischer Invalidierung ausgenommen werden soll.
    ///
    /// `None` = kein Guard aktiv (Normal-Modus).
    pub fn set_edit_guard(&mut self, record_id: Option<u64>) {
        self.edit_guard_id = record_id;
    }

    /// Aktualisiert einen bestehenden Record in-place (ID und locked-Status bleiben erhalten).
    ///
    /// Passt den Reverse-Index an die neuen Node-IDs an.
    /// Gibt `false` zurueck wenn kein Record mit dieser ID existiert.
    pub fn update_record(
        &mut self,
        record_id: u64,
        node_ids: Vec<u64>,
        original_positions: Vec<glam::Vec2>,
    ) -> bool {
        // Alte node_ids klonen bevor der Borrow beginnt
        let old_ids = match self.records.get(&record_id) {
            Some(r) => r.node_ids.clone(),
            None => return false,
        };
        // Reverse-Index aktualisieren (vor Record-Mutation, damit node_ids per Move geht)
        self.remove_from_index(record_id, &old_ids);
        self.add_to_index(record_id, &node_ids);
        // Record aktualisieren (Move statt Clone)
        if let Some(record) = self.records.get_mut(&record_id) {
            record.node_ids = node_ids;
            record.original_positions = original_positions;
            record.marker_node_ids.clear();
        }
        // Cache-Eintrag invalidieren (Boundary-Bild veraltet nach Node-Aenderung)
        self.boundary_cache.remove(&record_id);
        true
    }

    /// Entfernt die angegebenen Nodes aus einem Record.
    ///
    /// Aktualisiert `node_ids`, `original_positions` und den Reverse-Index.
    /// Wenn nach der Entfernung weniger als 2 Nodes verbleiben, wird der
    /// gesamte Record aufgeloest (automatisches Dissolve).
    ///
    /// Gibt `true` zurueck wenn der Record noch existiert, `false` wenn aufgeloest.
    pub fn remove_nodes_from_record(&mut self, record_id: u64, nodes_to_remove: &[u64]) -> bool {
        // Benoetigte Daten zuerst klonen, damit der immutable Borrow endet
        // bevor die mutierbaren Methoden aufgerufen werden.
        let (old_node_ids, old_orig_pos) = match self.records.get(&record_id) {
            Some(r) => (r.node_ids.clone(), r.original_positions.clone()),
            None => return false,
        };

        let remove_set: std::collections::HashSet<u64> = nodes_to_remove.iter().copied().collect();

        // Tatsaechlich vorhandene Nodes aus remove_set bestimmen + Reverse-Index bereinigen
        let actually_removed: Vec<u64> = old_node_ids
            .iter()
            .copied()
            .filter(|id| remove_set.contains(id))
            .collect();
        self.remove_from_index(record_id, &actually_removed);

        // node_ids und original_positions synchron filtern
        let pairs: Vec<(u64, Vec2)> = old_node_ids
            .into_iter()
            .zip(old_orig_pos)
            .filter(|(id, _)| !remove_set.contains(id))
            .collect();

        let remaining = pairs.len();

        // Weniger als 2 Nodes uebrig → Record aufloesen
        if remaining < 2 {
            // Verbleibende Nodes aus Reverse-Index entfernen
            let leftover: Vec<u64> = pairs.iter().map(|(id, _)| *id).collect();
            self.remove_from_index(record_id, &leftover);
            self.records.remove(&record_id);
            self.boundary_cache.remove(&record_id);
            return false;
        }

        // Record aktualisieren
        if let Some(record) = self.records.get_mut(&record_id) {
            record.node_ids = pairs.iter().map(|(id, _)| *id).collect();
            record.original_positions = pairs.iter().map(|(_, pos)| *pos).collect();
            record.marker_node_ids.retain(|id| !remove_set.contains(id));
        }
        self.boundary_cache.remove(&record_id);
        true
    }

    /// Gibt eine mutable Referenz auf den Record mit der angegebenen ID zurueck.
    fn get_mut(&mut self, record_id: u64) -> Option<&mut GroupRecord> {
        self.records.get_mut(&record_id)
    }

    /// Waermt den Boundary-Cache fuer alle nicht-gecachten Records auf.
    ///
    /// Muss einmal pro Frame VOR dem Rendern des Boundary-Overlays aufgerufen werden.
    /// Invalidiert den kompletten Cache wenn sich der RoadMap-Pointer aendert
    /// (d.h. eine neue Datei geladen wurde).
    ///
    /// Kosten: O(|Records ohne Cache-Eintrag| * |connections|) — typisch nur fuer
    /// neue Records teuer; bereits gecachte Records werden uebersprungen.
    pub fn warm_boundary_cache(&mut self, road_map: &RoadMap) {
        use std::collections::HashSet;

        let current_ptr = road_map as *const RoadMap as usize;
        if current_ptr != self.last_roadmap_ptr {
            self.boundary_cache.clear();
            self.last_roadmap_ptr = current_ptr;
        }

        let missing_ids: Vec<u64> = self
            .records
            .keys()
            .filter(|id| !self.boundary_cache.contains_key(*id))
            .copied()
            .collect();

        if missing_ids.is_empty() {
            return;
        }

        // Union aller gruppierten Nodes fuer "truly external"-Pruefung:
        // hat der externe Nachbar eine eigene Gruppe, oder ist er komplett ungrouped?
        let all_grouped_ids: HashSet<u64> = self
            .records
            .values()
            .flat_map(|r| r.node_ids.iter().copied())
            .collect();

        for rid in missing_ids {
            let Some(record) = self.records.get(&rid) else {
                continue;
            };
            let group_set: indexmap::IndexSet<u64> = record.node_ids.iter().copied().collect();

            // (has_incoming, has_outgoing, has_truly_external)
            let mut node_info: HashMap<u64, (bool, bool, bool)> = HashMap::new();
            // Winkel interner Verbindungen pro Node (Richtung aus Sicht des Nodes)
            let mut internal_angles: HashMap<u64, Vec<f32>> = HashMap::new();
            // Winkel externer Verbindungen pro Node (Richtung aus Sicht des Nodes)
            let mut external_angles: HashMap<u64, Vec<f32>> = HashMap::new();

            for conn in road_map.connections_iter() {
                let start_in = group_set.contains(&conn.start_id);
                let end_in = group_set.contains(&conn.end_id);

                if start_in && end_in {
                    // Interne Verbindung: Winkel aus Sicht beider Nodes sammeln
                    internal_angles
                        .entry(conn.start_id)
                        .or_default()
                        .push(conn.angle);
                    internal_angles
                        .entry(conn.end_id)
                        .or_default()
                        .push(conn.angle + std::f32::consts::PI);
                }

                if start_in && !end_in {
                    let entry = node_info
                        .entry(conn.start_id)
                        .or_insert((false, false, false));
                    entry.1 = true; // has_outgoing
                    if !all_grouped_ids.contains(&conn.end_id) {
                        entry.2 = true; // Nachbar ausserhalb jeder Gruppe
                    }
                    // Ext. Winkel: aus Sicht des start_node Richtung end_node
                    external_angles
                        .entry(conn.start_id)
                        .or_default()
                        .push(conn.angle);
                }
                if end_in && !start_in {
                    let entry = node_info
                        .entry(conn.end_id)
                        .or_insert((false, false, false));
                    entry.0 = true; // has_incoming
                    if !all_grouped_ids.contains(&conn.start_id) {
                        entry.2 = true; // Nachbar ausserhalb jeder Gruppe
                    }
                    // Ext. Winkel: aus Sicht des end_node Richtung start_node (Gegenrichtung)
                    external_angles
                        .entry(conn.end_id)
                        .or_default()
                        .push(conn.angle + std::f32::consts::PI);
                }
            }

            let infos: Vec<BoundaryInfo> = node_info
                .into_iter()
                .map(|(id, (inc, out, ext))| {
                    let direction = match (inc, out) {
                        (true, true) => BoundaryDirection::Bidirectional,
                        (true, false) => BoundaryDirection::Entry,
                        (false, true) => BoundaryDirection::Exit,
                        _ => BoundaryDirection::Entry,
                    };

                    // Maximale Winkelabweichung: interner Durchschnittswinkel vs. externe Winkel
                    let max_dev = {
                        let int_angles = internal_angles.get(&id).map(Vec::as_slice).unwrap_or(&[]);
                        let ext_angles = external_angles.get(&id).map(Vec::as_slice).unwrap_or(&[]);
                        if !int_angles.is_empty() && !ext_angles.is_empty() {
                            // Zirkulaerer Durchschnitt der internen Winkel via Einheitsvektoren
                            let (sin_sum, cos_sum) = int_angles
                                .iter()
                                .fold((0.0f32, 0.0f32), |(s, c), &a| (s + a.sin(), c + a.cos()));
                            let avg_internal = sin_sum.atan2(cos_sum);
                            let max = ext_angles
                                .iter()
                                .map(|&ea| crate::shared::angle_deviation(avg_internal, ea))
                                .fold(0.0f32, f32::max);
                            Some(max)
                        } else {
                            None
                        }
                    };

                    BoundaryInfo {
                        node_id: id,
                        has_external_connection: ext,
                        direction,
                        max_external_angle_deviation: max_dev,
                    }
                })
                .collect();

            self.boundary_cache.insert(rid, infos);
        }
    }

    /// Gibt die gecachten Boundary-Infos fuer den angegebenen Record zurueck.
    ///
    /// Gibt `None` zurueck wenn kein Cache-Eintrag existiert
    /// (d.h. `warm_boundary_cache()` wurde noch nicht fuer diesen Record aufgerufen).
    pub fn boundary_cache_for(&self, record_id: u64) -> Option<&[BoundaryInfo]> {
        self.boundary_cache.get(&record_id).map(Vec::as_slice)
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
            if let Some(node) = road_map.nodes.get(&node_id) {
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

    /// Gibt die Anzahl der gespeicherten Records zurueck.
    pub fn len(&self) -> usize {
        self.records.len()
    }

    /// Gibt zurueck ob die Registry leer ist.
    pub fn is_empty(&self) -> bool {
        self.records.is_empty()
    }
}

#[cfg(test)]
mod registry_tests {
    use super::*;
    use crate::app::tools::ToolAnchor;
    use crate::{ConnectionDirection, ConnectionPriority, MapNode, NodeFlag, RoadMap};

    fn make_test_record(
        id: u64,
        node_ids: Vec<u64>,
        positions: Vec<Vec2>,
        locked: bool,
    ) -> GroupRecord {
        GroupRecord {
            id,
            node_ids,
            start_anchor: ToolAnchor::NewPosition(Vec2::ZERO),
            end_anchor: ToolAnchor::NewPosition(Vec2::ZERO),
            kind: GroupKind::Straight {
                base: GroupBase {
                    direction: ConnectionDirection::Regular,
                    priority: ConnectionPriority::Regular,
                    max_segment_length: 10.0,
                },
            },
            original_positions: positions,
            marker_node_ids: Vec::new(),
            locked,
        }
    }

    #[test]
    fn groups_for_node_findet_alle_zugehoerigen_segmente() {
        let mut registry = GroupRegistry::new();
        registry.register(make_test_record(0, vec![1, 2, 3], vec![], true));
        registry.register(make_test_record(1, vec![3, 4, 5], vec![], false));
        registry.register(make_test_record(2, vec![6, 7], vec![], true));

        let result = registry.groups_for_node(3);
        assert_eq!(result.len(), 2, "Node 3 gehoert zu Segmenten 0 und 1");
        assert!(result.contains(&0));
        assert!(result.contains(&1));

        let result_solo = registry.groups_for_node(7);
        assert_eq!(result_solo, vec![2]);

        let result_none = registry.groups_for_node(99);
        assert!(result_none.is_empty());
    }

    #[test]
    fn expand_locked_selection_gibt_alle_nodes_locked_segmente() {
        let mut registry = GroupRegistry::new();
        // Locked: Nodes 1, 2, 3
        registry.register(make_test_record(0, vec![1, 2, 3], vec![], true));
        // Unlocked: Nodes 4, 5
        registry.register(make_test_record(1, vec![4, 5], vec![], false));
        // Locked: Nodes 6, 7
        registry.register(make_test_record(2, vec![6, 7], vec![], true));

        // Selektion: nur Node 1 (gehoert zu Segment 0, locked)
        let mut extra = registry.expand_locked_selection(&[1]);
        extra.sort();
        assert_eq!(extra, vec![1, 2, 3]);

        // Selektion: Node 4 (gehoert zu Segment 1, UNlocked) → kein Expand
        let extra_unlocked = registry.expand_locked_selection(&[4]);
        assert!(extra_unlocked.is_empty());

        // Selektion: Node 1 + Node 6 → beide locked Segmente expandieren
        let mut extra_multi = registry.expand_locked_selection(&[1, 6]);
        extra_multi.sort();
        assert_eq!(extra_multi, vec![1, 2, 3, 6, 7]);
    }

    #[test]
    fn update_original_positions_aktualisiert_korrekt() {
        let mut map = RoadMap::new(3);
        map.add_node(MapNode::new(10, Vec2::new(5.0, 0.0), NodeFlag::Regular));
        map.add_node(MapNode::new(11, Vec2::new(15.0, 0.0), NodeFlag::Regular));

        let mut registry = GroupRegistry::new();
        // original_positions absichtlich falsch (alt)
        registry.register(make_test_record(
            0,
            vec![10, 11],
            vec![Vec2::new(0.0, 0.0), Vec2::new(10.0, 0.0)],
            true,
        ));

        registry.update_original_positions(0, &map);

        let record = registry.get(0).expect("Record vorhanden");
        assert_eq!(record.original_positions[0], Vec2::new(5.0, 0.0));
        assert_eq!(record.original_positions[1], Vec2::new(15.0, 0.0));
    }

    /// Prüft, dass der Reverse-Index nach register/remove konsistent bleibt.
    #[test]
    fn reverse_index_konsistent_nach_register_und_remove() {
        let mut registry = GroupRegistry::new();
        registry.register(make_test_record(0, vec![1, 2, 3], vec![], false));
        registry.register(make_test_record(1, vec![3, 4], vec![], false));

        // Node 3 gehoert zu beiden Records
        assert_eq!(registry.groups_for_node(3).len(), 2);

        // Record 0 entfernen → Node 3 nur noch in Record 1
        registry.remove(0);
        let segs = registry.groups_for_node(3);
        assert_eq!(segs, vec![1], "Node 3 sollte nur noch Record 1 haben");

        // Nodes 1, 2 sollten keine Zuordnung mehr haben
        assert!(registry.groups_for_node(1).is_empty());
        assert!(registry.groups_for_node(2).is_empty());

        // Node 4 weiterhin Record 1
        assert_eq!(registry.groups_for_node(4), vec![1]);
    }

    /// Prüft, dass update_record den Reverse-Index korrekt aktualisiert.
    #[test]
    fn reverse_index_konsistent_nach_update_record() {
        let mut registry = GroupRegistry::new();
        registry.register(make_test_record(0, vec![1, 2, 3], vec![], false));

        // Alte Nodes: 1,2,3 → Neue Nodes: 2,4,5
        let success =
            registry.update_record(0, vec![2, 4, 5], vec![Vec2::ZERO, Vec2::ZERO, Vec2::ZERO]);
        assert!(success, "update_record sollte true zurueckgeben");

        // Node 1, 3 sollten nicht mehr zugeordnet sein
        assert!(registry.groups_for_node(1).is_empty());
        assert!(registry.groups_for_node(3).is_empty());

        // Node 2 sollte weiterhin, 4 und 5 neu zugeordnet sein
        assert_eq!(registry.groups_for_node(2), vec![0]);
        assert_eq!(registry.groups_for_node(4), vec![0]);
        assert_eq!(registry.groups_for_node(5), vec![0]);
    }

    /// Prüft, dass update_record false zurückgibt bei nicht-existierender ID.
    #[test]
    fn update_record_nicht_existierend_gibt_false() {
        let mut registry = GroupRegistry::new();
        let result = registry.update_record(99, vec![1], vec![Vec2::ZERO]);
        assert!(!result, "Nicht-existierende ID sollte false ergeben");
    }

    /// Prüft, dass invalidate_by_node_ids den edit_guard respektiert.
    #[test]
    fn invalidate_respektiert_edit_guard() {
        let mut registry = GroupRegistry::new();
        registry.register(make_test_record(0, vec![1, 2], vec![], false));
        registry.register(make_test_record(1, vec![2, 3], vec![], false));

        // Record 1 als edit_guard setzen
        registry.set_edit_guard(Some(1));

        // Node 2 invalidieren → sollte nur Record 0 entfernen
        registry.invalidate_by_node_ids(&[2]);

        assert!(registry.get(0).is_none(), "Record 0 sollte entfernt sein");
        assert!(
            registry.get(1).is_some(),
            "Record 1 sollte durch Guard geschuetzt sein"
        );
    }

    /// Prüft, dass find_by_node_ids korrekte Records findet.
    #[test]
    fn find_by_node_ids_findet_betroffene_records() {
        let mut registry = GroupRegistry::new();
        registry.register(make_test_record(0, vec![1, 2], vec![], false));
        registry.register(make_test_record(1, vec![3, 4], vec![], false));
        registry.register(make_test_record(2, vec![2, 5], vec![], false));

        let query: indexmap::IndexSet<u64> = [2, 3].into_iter().collect();
        let found = registry.find_by_node_ids(&query);
        let mut found_ids: Vec<u64> = found.iter().map(|r| r.id).collect();
        found_ids.sort();
        assert_eq!(found_ids, vec![0, 1, 2], "Alle Records mit Node 2 oder 3");
    }

    /// Prüft, dass find_first_by_node_id den ersten Record findet.
    #[test]
    fn find_first_by_node_id_findet_record() {
        let mut registry = GroupRegistry::new();
        registry.register(make_test_record(0, vec![1, 2], vec![], false));
        registry.register(make_test_record(1, vec![3, 4], vec![], false));

        assert!(registry.find_first_by_node_id(1).is_some());
        assert!(registry.find_first_by_node_id(99).is_none());
    }

    /// Prüft, dass remove bei nicht-existierender ID nicht panikt.
    #[test]
    fn remove_nicht_existierend_ist_noop() {
        let mut registry = GroupRegistry::new();
        registry.register(make_test_record(0, vec![1, 2], vec![], false));

        // Doppeltes Remove sollte kein Panic verursachen
        registry.remove(0);
        registry.remove(0);
        registry.remove(99);

        assert!(registry.is_empty());
    }

    /// Prüft Operationen auf leerer Registry.
    #[test]
    fn leere_registry_edge_cases() {
        let registry = GroupRegistry::new();

        assert!(registry.groups_for_node(1).is_empty());
        assert!(registry.expand_locked_selection(&[1, 2]).is_empty());
        assert!(registry.find_first_by_node_id(1).is_none());

        let query: indexmap::IndexSet<u64> = [1, 2].into_iter().collect();
        assert!(registry.find_by_node_ids(&query).is_empty());

        assert!(registry.is_empty());
        assert_eq!(registry.len(), 0);
    }

    // --- Tests fuer remove_nodes_from_record ---

    /// Prüft, dass Nodes korrekt aus einem Record entfernt werden.
    #[test]
    fn remove_nodes_from_record_entfernt_subset() {
        let mut registry = GroupRegistry::new();
        let positions = vec![Vec2::ZERO, Vec2::X, Vec2::Y, Vec2::ONE];
        registry.register(make_test_record(0, vec![1, 2, 3, 4], positions, false));

        let still_alive = registry.remove_nodes_from_record(0, &[2, 3]);
        assert!(
            still_alive,
            "Record sollte bestehen bleiben (2 Nodes uebrig)"
        );

        let record = registry.get(0).expect("Record vorhanden");
        assert_eq!(record.node_ids, vec![1, 4]);
        assert_eq!(record.original_positions.len(), 2);

        // Reverse-Index: entfernte Nodes sollten weg sein
        assert!(registry.groups_for_node(2).is_empty());
        assert!(registry.groups_for_node(3).is_empty());
        // Verbleibende Nodes weiterhin zugeordnet
        assert_eq!(registry.groups_for_node(1), vec![0]);
        assert_eq!(registry.groups_for_node(4), vec![0]);
    }

    /// Prüft, dass der Record aufgeloest wird wenn weniger als 2 Nodes verbleiben.
    #[test]
    fn remove_nodes_from_record_dissolve_bei_weniger_als_2() {
        let mut registry = GroupRegistry::new();
        let positions = vec![Vec2::ZERO, Vec2::X, Vec2::Y];
        registry.register(make_test_record(0, vec![1, 2, 3], positions, false));

        let still_alive = registry.remove_nodes_from_record(0, &[1, 2]);
        assert!(
            !still_alive,
            "Record sollte aufgeloest worden sein (<2 Nodes)"
        );
        assert!(
            registry.get(0).is_none(),
            "Record darf nicht mehr existieren"
        );

        // Alle Nodes aus dem Reverse-Index entfernt
        assert!(registry.groups_for_node(1).is_empty());
        assert!(registry.groups_for_node(2).is_empty());
        assert!(registry.groups_for_node(3).is_empty());
    }

    /// Prüft, dass nicht-existierende Nodes im remove-Set keine Probleme verursachen.
    #[test]
    fn remove_nodes_from_record_ignoriert_unbekannte_nodes() {
        let mut registry = GroupRegistry::new();
        let positions = vec![Vec2::ZERO, Vec2::X, Vec2::Y];
        registry.register(make_test_record(0, vec![1, 2, 3], positions, false));

        let still_alive = registry.remove_nodes_from_record(0, &[99, 100]);
        assert!(still_alive, "Record bleibt (keine echten Entfernungen)");

        let record = registry.get(0).expect("Record vorhanden");
        assert_eq!(record.node_ids.len(), 3, "Keine Nodes entfernt");
    }

    /// Prüft, dass remove_nodes_from_record false bei unbekannter Record-ID liefert.
    #[test]
    fn remove_nodes_from_record_unbekannte_id() {
        let mut registry = GroupRegistry::new();
        let result = registry.remove_nodes_from_record(42, &[1, 2]);
        assert!(!result, "Nicht-existierende ID sollte false ergeben");
    }

    /// Prüft, dass der Boundary-Cache nach Node-Entfernung invalidiert wird.
    #[test]
    fn remove_nodes_from_record_invalidiert_boundary_cache() {
        let mut map = RoadMap::new(4);
        map.add_node(MapNode::new(1, Vec2::new(0.0, 0.0), NodeFlag::Regular));
        map.add_node(MapNode::new(2, Vec2::new(10.0, 0.0), NodeFlag::Regular));
        map.add_node(MapNode::new(3, Vec2::new(20.0, 0.0), NodeFlag::Regular));
        map.add_node(MapNode::new(4, Vec2::new(30.0, 0.0), NodeFlag::Regular));

        let mut registry = GroupRegistry::new();
        let positions = vec![
            Vec2::new(0.0, 0.0),
            Vec2::new(10.0, 0.0),
            Vec2::new(20.0, 0.0),
            Vec2::new(30.0, 0.0),
        ];
        registry.register(make_test_record(0, vec![1, 2, 3, 4], positions, false));

        // Cache aufwaermen
        registry.warm_boundary_cache(&map);
        assert!(
            registry.boundary_cache_for(0).is_some(),
            "Cache sollte nach warm vorhanden sein"
        );

        // Node entfernen → Cache muss invalidiert sein
        registry.remove_nodes_from_record(0, &[3]);
        assert!(
            registry.boundary_cache_for(0).is_none(),
            "Cache muss nach Node-Entfernung invalidiert sein"
        );
    }
}
