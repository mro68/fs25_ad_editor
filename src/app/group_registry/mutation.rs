//! Mutierende Methoden der [`GroupRegistry`].

use super::GroupRegistry;
use crate::core::RoadMap;
use glam::Vec2;

impl GroupRegistry {
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

    /// Setzt die Entry- und Exit-Node-IDs eines Segments.
    ///
    /// Validiert, dass die angegebenen Node-IDs im Record enthalten sind.
    /// Invalidiert den Boundary-Cache fuer diesen Record.
    /// Gibt `false` zurueck wenn das Segment nicht existiert oder eine ID ungueltig ist.
    pub fn set_entry_exit(
        &mut self,
        record_id: u64,
        entry: Option<u64>,
        exit: Option<u64>,
    ) -> bool {
        let Some(record) = self.records.get_mut(&record_id) else {
            return false;
        };
        if let Some(eid) = entry {
            if !record.node_ids.contains(&eid) {
                return false;
            }
        }
        if let Some(eid) = exit {
            if !record.node_ids.contains(&eid) {
                return false;
            }
        }
        record.entry_node_id = entry;
        record.exit_node_id = exit;
        self.boundary_cache.remove(&record_id);
        true
    }

    /// Aktualisiert einen bestehenden Record in-place (ID und locked-Status bleiben erhalten).
    ///
    /// Passt den Reverse-Index an die neuen Node-IDs an.
    /// Gibt `false` zurueck wenn kein Record mit dieser ID existiert.
    pub fn update_record(
        &mut self,
        record_id: u64,
        node_ids: Vec<u64>,
        original_positions: Vec<Vec2>,
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
            // Entry/Exit validieren: Node nicht mehr im Record → zuruecksetzen
            let new_ids = &record.node_ids;
            if record
                .entry_node_id
                .is_some_and(|id| !new_ids.contains(&id))
            {
                record.entry_node_id = None;
            }
            if record.exit_node_id.is_some_and(|id| !new_ids.contains(&id)) {
                record.exit_node_id = None;
            }
        }
        // Cache-Eintrag invalidieren (Boundary-Bild veraltet nach Node-Aenderung)
        self.boundary_cache.remove(&record_id);
        self.dimmed_generation += 1;
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
            // Entry/Exit: geloeschte Nodes auf None setzen
            if record
                .entry_node_id
                .is_some_and(|id| remove_set.contains(&id))
            {
                record.entry_node_id = None;
            }
            if record
                .exit_node_id
                .is_some_and(|id| remove_set.contains(&id))
            {
                record.exit_node_id = None;
            }
        }
        self.boundary_cache.remove(&record_id);
        self.dimmed_generation += 1;
        true
    }
}
