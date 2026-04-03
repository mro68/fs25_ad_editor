//! Boundary-Cache-Logik der [`GroupRegistry`].

use super::{BoundaryDirection, BoundaryInfo, GroupRegistry};
use crate::core::RoadMap;
use std::collections::HashMap;

impl GroupRegistry {
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

            for &node_id in &group_set {
                for &(nb_id, is_outgoing) in road_map.neighbors(node_id) {
                    let nb_in = group_set.contains(&nb_id);

                    // Verbindung fuer Winkel nachschlagen — O(1)
                    let conn = if is_outgoing {
                        road_map.find_connection(node_id, nb_id)
                    } else {
                        road_map.find_connection(nb_id, node_id)
                    };
                    let Some(c) = conn else { continue };
                    let angle_from_node = if is_outgoing {
                        c.angle
                    } else {
                        c.angle + std::f32::consts::PI
                    };

                    if nb_in {
                        // Interne Verbindung: Winkel aus Sicht dieses Nodes
                        internal_angles
                            .entry(node_id)
                            .or_default()
                            .push(angle_from_node);
                    } else {
                        // Externe Verbindung: Flags und Winkel setzen
                        let entry = node_info.entry(node_id).or_insert((false, false, false));
                        if is_outgoing {
                            entry.1 = true; // has_outgoing
                        } else {
                            entry.0 = true; // has_incoming
                        }
                        if !all_grouped_ids.contains(&nb_id) {
                            entry.2 = true; // Nachbar ausserhalb jeder Gruppe
                        }
                        external_angles
                            .entry(node_id)
                            .or_default()
                            .push(angle_from_node);
                    }
                }
            }

            let mut infos: Vec<BoundaryInfo> = node_info
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

            // Explizite Entry-/Exit-Knoten immer im Boundary-Overlay halten,
            // auch wenn dort aktuell keine externe Verbindung mehr existiert.
            let existing_ids: HashSet<u64> = infos.iter().map(|bi| bi.node_id).collect();

            if let Some(entry_id) = record.entry_node_id {
                if !existing_ids.contains(&entry_id) {
                    infos.push(BoundaryInfo {
                        node_id: entry_id,
                        has_external_connection: false,
                        direction: BoundaryDirection::Entry,
                        max_external_angle_deviation: None, // None = immer Icon anzeigen
                    });
                }
            }
            if let Some(exit_id) = record.exit_node_id {
                if !existing_ids.contains(&exit_id) {
                    infos.push(BoundaryInfo {
                        node_id: exit_id,
                        has_external_connection: false,
                        direction: BoundaryDirection::Exit,
                        max_external_angle_deviation: None,
                    });
                }
            }
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
}
