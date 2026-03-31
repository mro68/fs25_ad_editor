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
//! - [`types`]: Datentypes (`GroupBase`, `GroupKind`, `GroupRecord`) und der explizite Tool-Vertrag
//! - [`query`]: Lookup- und Query-Methoden
//! - [`lock`]: Lock- und Edit-Guard-Methoden
//! - [`mutation`]: Mutierende Methoden
//! - [`boundary_cache`]: Boundary-Cache-Logik
//! - Dieses Modul: [`GroupRegistry`] mit CRUD-Kernoperationen

mod types;
pub use types::*;

mod boundary_cache;
mod lock;
mod mutation;
mod query;
#[cfg(test)]
mod tests;

use crate::core::RoadMap;
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
    pub(super) records: HashMap<u64, GroupRecord>,
    pub(super) next_id: u64,
    /// Record-ID, die von automatischer Invalidierung ausgenommen ist (aktiver Group-Edit).
    pub(super) edit_guard_id: Option<u64>,
    /// Reverse-Index: Node-ID → Liste der zugehoerigen Record-IDs.
    pub(super) node_to_records: HashMap<u64, Vec<u64>>,
    /// Cache fuer gecachte Boundary-Infos pro Record (Key = record_id).
    /// Wird bei register/remove/update_record invalidiert;
    /// bei neuem RoadMap-Pointer komplett geleert.
    pub(super) boundary_cache: HashMap<u64, Vec<BoundaryInfo>>,
    /// Adresse der zuletzt verwendeten RoadMap fuer PTR-basierten Cache-Reset.
    pub(super) last_roadmap_ptr: usize,
    /// Monoton steigender Zaehler: wird bei jeder Mutation erhoehen, die node_ids veraendert.
    /// Dient als Invalidierungs-Token fuer den `dimmed_ids`-Cache in `AppState`.
    pub(crate) dimmed_generation: u64,
}

impl GroupRegistry {
    /// Erstellt eine leere Registry.
    pub fn new() -> Self {
        Self::default()
    }

    /// Fuegt alle Node-IDs eines Records zum Reverse-Index hinzu.
    pub(super) fn add_to_index(&mut self, record_id: u64, node_ids: &[u64]) {
        for &nid in node_ids {
            self.node_to_records.entry(nid).or_default().push(record_id);
        }
    }

    /// Entfernt alle Node-IDs eines Records aus dem Reverse-Index.
    pub(super) fn remove_from_index(&mut self, record_id: u64, node_ids: &[u64]) {
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
        self.dimmed_generation += 1;
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
        self.dimmed_generation += 1;
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

    /// Gibt eine mutable Referenz auf den Record mit der angegebenen ID zurueck.
    pub(super) fn get_mut(&mut self, record_id: u64) -> Option<&mut GroupRecord> {
        self.records.get_mut(&record_id)
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
