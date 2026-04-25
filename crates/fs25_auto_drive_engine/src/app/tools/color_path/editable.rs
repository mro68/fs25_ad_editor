//! Editierbares Zwischenmodell zwischen Stage E (Skelett-Netz) und Stage F
//! (PreparedSegments) des ColorPath-Wizards.
//!
//! `EditableCenterlines` uebernimmt in CP-06 die stabile, ID-basierte Sicht auf
//! das extrahierte Netz und bildet damit die Grundlage fuer die folgenden
//! Commit-Punkte:
//!
//! - CP-07: Stage F liest Junction-Positionen aus `EditableCenterlines` (heute
//!   noch aus [`super::skeleton::SkeletonNetwork`]).
//! - CP-08: `RouteToolDrag` mutiert Junction-Positionen live, bumpt die
//!   `revision` und invalidiert dadurch den Stage-F-Cache.
//!
//! Forward-Compatibility fuer zweispurige Strassen aus Selektion
//! ([planner_analysis.md](../../../../../../../memories/session/20260424_201526-colorpath-phase-wizard/planner_analysis.md)
//! §1.5): Die per [`EditableCenterline::lane_spec`] hinterlegte
//! [`LaneSpec`]-Platzhalter-Struktur haelt spaeter die Fahrstreifen-Anzahl pro
//! ausgewaehlter Centerline, ohne dass Datenmodelle erneut gedreht werden muessen.
//!
//! CP-06 baut nur Struktur + Befuellung. Die Felder werden erst ab CP-07
//! (Stage F liest Junctions) bzw. CP-08 (Junction-Drag) gelesen. Damit der
//! Build bis dahin warning-frei bleibt, markieren wir sie modulweit als
//! `allow(dead_code)`.

#![allow(dead_code)]

use std::collections::HashMap;

use glam::Vec2;

use super::skeleton::SkeletonNetwork;

/// Stabile ID einer Junction im editierbaren Zwischenmodell.
///
/// Die ID wird waehrend des Aufbaus aus [`SkeletonNetwork`] einmalig vergeben
/// und bleibt ueber Drags, Revisions-Bumps und Phase-Wechsel konstant, solange
/// das Netz nicht neu extrahiert wird.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub struct EditableJunctionId(pub u32);

/// Stabile ID einer Centerline (Segment zwischen zwei Junctions).
///
/// Wie [`EditableJunctionId`] stabil ueber die gesamte Lebensdauer einer
/// Stage-E-Revision; wird in CP-09 auch als Handle fuer die spaetere
/// Zweispur-Selektion wiederverwendet.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub struct EditableCenterlineId(pub u32);

/// Platzhalter-Spezifikation fuer die spaetere Zweispur-Erweiterung.
///
/// Heute traegt jede Centerline implizit eine Spur (`lane_count = 1`). Die
/// Struktur existiert bereits, damit zukuenftige Commits (Zweispurige
/// Strassen aus Selektion) weitere Felder additiv ergaenzen koennen, ohne den
/// Typ der Centerlines zu aendern.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct LaneSpec {
    /// Anzahl der Fahrstreifen entlang dieser Centerline.
    pub lane_count: u8,
}

impl Default for LaneSpec {
    fn default() -> Self {
        Self { lane_count: 1 }
    }
}

/// Editierbare Repraesentation eines Graph-Knotens (Kreuzung, offenes Ende
/// oder Schleifen-Anker) aus dem Skelett-Netz.
#[derive(Debug, Clone)]
pub struct EditableJunction {
    /// Stabile ID innerhalb des aktuellen Editable-Netzes.
    pub id: EditableJunctionId,
    /// Aktuelle (ggf. gedraggte) Weltposition.
    pub world_pos: Vec2,
    /// Urspruengliche Position direkt nach der Stage-E-Extraktion.
    ///
    /// Dient spaeter als Referenz fuer „Reset auf Original" (CP-08) und zur
    /// Invalidierung, wenn der User die gleiche Junction wiederholt draggt.
    pub original_pos: Vec2,
    /// IDs aller an dieser Junction anliegenden Centerlines.
    pub incident_centerlines: Vec<EditableCenterlineId>,
}

/// Editierbare Repraesentation eines Segments zwischen zwei Junctions.
#[derive(Debug, Clone)]
pub struct EditableCenterline {
    /// Stabile ID innerhalb des aktuellen Editable-Netzes.
    pub id: EditableCenterlineId,
    /// Polyline in Weltkoordinaten inklusive Start- und Endpunkt.
    pub polyline: Vec<Vec2>,
    /// Start-Junction der Polyline (falls der Knoten als Junction gefuehrt wird).
    pub start_junction: Option<EditableJunctionId>,
    /// End-Junction der Polyline (falls der Knoten als Junction gefuehrt wird).
    pub end_junction: Option<EditableJunctionId>,
    /// Platzhalter fuer die Zweispur-Spezifikation (siehe [`LaneSpec`]).
    pub lane_spec: LaneSpec,
    /// Selektionsflag fuer kuenftige Multi-Centerline-Auswahl.
    ///
    /// CP-06 setzt den Wert noch nie auf `true`; das Feld existiert ausschliesslich
    /// fuer die Zweispur-Erweiterung.
    pub selected: bool,
}

/// Vollstaendiges, editierbares Netz aus Junctions und Centerlines.
///
/// CP-06 befuellt die Struktur einmalig am Eintritt in die Editing-Phase
/// aus dem aktuellen [`SkeletonNetwork`]. Mutationen (Drag in CP-08,
/// Selektion der Zweispur-Erweiterung) bumpen die [`Self::revision`] und
/// signalisieren damit spaeteren Stages den Cache-Ungueltig-Stempel.
#[derive(Debug, Clone, Default)]
pub struct EditableCenterlines {
    /// Alle editierbaren Junctions, indiziert ueber ihre stabile ID.
    pub junctions: HashMap<EditableJunctionId, EditableJunction>,
    /// Alle editierbaren Centerlines, indiziert ueber ihre stabile ID.
    pub centerlines: HashMap<EditableCenterlineId, EditableCenterline>,
    /// Monoton steigender Revisionszaehler (0 = „frisch erzeugt, keine Mutation").
    pub revision: u64,
}

impl EditableCenterlines {
    /// Baut das editierbare Netz aus einem bereits extrahierten Skelett-Netz auf.
    ///
    /// Die ID-Vergabe folgt strikt den Vec-Indizes in [`SkeletonNetwork`]:
    /// - `nodes[i]` → `EditableJunctionId(i as u32)`
    /// - `segments[i]` → `EditableCenterlineId(i as u32)`
    ///
    /// Dadurch bleiben die IDs stabil, solange das Netz nicht neu extrahiert
    /// wird. Segmente, deren Start-/End-Node ausserhalb des bekannten
    /// Node-Bereichs liegt, werden robust mit `None` markiert statt in Panics
    /// zu laufen.
    pub fn from_skeleton_network(network: &SkeletonNetwork) -> Self {
        let mut junctions: HashMap<EditableJunctionId, EditableJunction> =
            HashMap::with_capacity(network.nodes.len());
        for (idx, node) in network.nodes.iter().enumerate() {
            let id = EditableJunctionId(idx as u32);
            junctions.insert(
                id,
                EditableJunction {
                    id,
                    world_pos: node.world_position,
                    original_pos: node.world_position,
                    incident_centerlines: Vec::new(),
                },
            );
        }

        let mut centerlines: HashMap<EditableCenterlineId, EditableCenterline> =
            HashMap::with_capacity(network.segments.len());
        for (idx, segment) in network.segments.iter().enumerate() {
            let centerline_id = EditableCenterlineId(idx as u32);
            let start_junction = node_index_to_id(segment.start_node, network.nodes.len());
            let end_junction = node_index_to_id(segment.end_node, network.nodes.len());

            if let Some(start_id) = start_junction
                && let Some(junction) = junctions.get_mut(&start_id)
            {
                junction.incident_centerlines.push(centerline_id);
            }
            if let Some(end_id) = end_junction
                && end_junction != start_junction
                && let Some(junction) = junctions.get_mut(&end_id)
            {
                junction.incident_centerlines.push(centerline_id);
            }

            centerlines.insert(
                centerline_id,
                EditableCenterline {
                    id: centerline_id,
                    polyline: segment.polyline.clone(),
                    start_junction,
                    end_junction,
                    lane_spec: LaneSpec::default(),
                    selected: false,
                },
            );
        }

        Self {
            junctions,
            centerlines,
            revision: 0,
        }
    }

    /// Erhoeht den Revisionszaehler monoton und ueberspringt dabei den Wert 0.
    ///
    /// Wird von Phase-Wechseln (CP-06) und spaeter vom Junction-Drag
    /// (CP-08) aufgerufen, um abgeleitete Cache-Keys zu invalidieren.
    pub fn bump_revision(&mut self) {
        self.revision = self.revision.wrapping_add(1);
        if self.revision == 0 {
            self.revision = 1;
        }
    }

    /// Verschiebt eine Junction auf eine neue Weltposition und bumpt die Revision.
    ///
    /// Gibt `true` zurueck, wenn die Junction existiert und aktualisiert wurde.
    /// CP-06 passt die angrenzenden Polyline-Endpunkte bewusst **nicht** an —
    /// das wird Aufgabe des CP-07-Stage-F-Rebuilds bzw. der CP-08-Drag-Logik.
    pub fn move_junction(&mut self, id: EditableJunctionId, new_pos: Vec2) -> bool {
        let Some(junction) = self.junctions.get_mut(&id) else {
            return false;
        };
        if junction.world_pos == new_pos {
            return true;
        }
        junction.world_pos = new_pos;
        self.bump_revision();
        true
    }
}

/// Uebersetzt einen Skelett-Node-Index in die zugehoerige `EditableJunctionId`,
/// sofern er innerhalb der bekannten Node-Liste liegt.
fn node_index_to_id(node_index: usize, node_count: usize) -> Option<EditableJunctionId> {
    if node_index < node_count {
        Some(EditableJunctionId(node_index as u32))
    } else {
        None
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::app::tools::color_path::skeleton::{
        SkeletonGraphNode, SkeletonGraphNodeKind, SkeletonGraphSegment, SkeletonNetwork,
    };

    fn sample_network() -> SkeletonNetwork {
        SkeletonNetwork {
            nodes: vec![
                SkeletonGraphNode {
                    kind: SkeletonGraphNodeKind::OpenEnd,
                    pixel_position: Vec2::ZERO,
                    world_position: Vec2::new(0.0, 0.0),
                },
                SkeletonGraphNode {
                    kind: SkeletonGraphNodeKind::Junction,
                    pixel_position: Vec2::ZERO,
                    world_position: Vec2::new(10.0, 0.0),
                },
                SkeletonGraphNode {
                    kind: SkeletonGraphNodeKind::OpenEnd,
                    pixel_position: Vec2::ZERO,
                    world_position: Vec2::new(10.0, 10.0),
                },
            ],
            segments: vec![
                SkeletonGraphSegment {
                    start_node: 0,
                    end_node: 1,
                    polyline: vec![
                        Vec2::new(0.0, 0.0),
                        Vec2::new(5.0, 0.0),
                        Vec2::new(10.0, 0.0),
                    ],
                },
                SkeletonGraphSegment {
                    start_node: 1,
                    end_node: 2,
                    polyline: vec![Vec2::new(10.0, 0.0), Vec2::new(10.0, 10.0)],
                },
            ],
        }
    }

    #[test]
    fn from_skeleton_network_assigns_stable_ids_and_links_incidences() {
        let network = sample_network();
        let editable = EditableCenterlines::from_skeleton_network(&network);

        assert_eq!(editable.junctions.len(), 3);
        assert_eq!(editable.centerlines.len(), 2);
        assert_eq!(editable.revision, 0);

        let center_id_0 = EditableCenterlineId(0);
        let center_id_1 = EditableCenterlineId(1);
        let junction_0 = &editable.junctions[&EditableJunctionId(0)];
        let junction_1 = &editable.junctions[&EditableJunctionId(1)];
        let junction_2 = &editable.junctions[&EditableJunctionId(2)];

        assert_eq!(junction_0.world_pos, junction_0.original_pos);
        assert_eq!(junction_0.incident_centerlines, vec![center_id_0]);
        assert_eq!(
            junction_1.incident_centerlines,
            vec![center_id_0, center_id_1]
        );
        assert_eq!(junction_2.incident_centerlines, vec![center_id_1]);

        let centerline_0 = &editable.centerlines[&center_id_0];
        assert_eq!(centerline_0.start_junction, Some(EditableJunctionId(0)));
        assert_eq!(centerline_0.end_junction, Some(EditableJunctionId(1)));
        assert_eq!(centerline_0.polyline.len(), 3);
        assert_eq!(centerline_0.lane_spec, LaneSpec::default());
        assert!(!centerline_0.selected);
    }

    #[test]
    fn bump_revision_is_monotonic_and_skips_zero() {
        let mut editable = EditableCenterlines::default();
        editable.revision = u64::MAX;
        editable.bump_revision();
        assert_eq!(editable.revision, 1);
        editable.bump_revision();
        assert_eq!(editable.revision, 2);
    }

    #[test]
    fn move_junction_updates_position_and_bumps_revision() {
        let network = sample_network();
        let mut editable = EditableCenterlines::from_skeleton_network(&network);
        let target = Vec2::new(42.0, -7.0);

        assert!(editable.move_junction(EditableJunctionId(1), target));
        assert_eq!(editable.junctions[&EditableJunctionId(1)].world_pos, target);
        assert_eq!(
            editable.junctions[&EditableJunctionId(1)].original_pos,
            Vec2::new(10.0, 0.0)
        );
        assert_eq!(editable.revision, 1);

        // Identische Position bumpt nicht erneut.
        assert!(editable.move_junction(EditableJunctionId(1), target));
        assert_eq!(editable.revision, 1);

        // Unbekannte ID wird sauber abgelehnt.
        assert!(!editable.move_junction(EditableJunctionId(99), Vec2::ZERO));
    }

    #[test]
    fn segments_with_out_of_bounds_nodes_fall_back_to_none() {
        let mut network = sample_network();
        network.segments.push(SkeletonGraphSegment {
            start_node: 42,
            end_node: 99,
            polyline: vec![Vec2::ZERO, Vec2::ONE],
        });

        let editable = EditableCenterlines::from_skeleton_network(&network);
        let stray = &editable.centerlines[&EditableCenterlineId(2)];
        assert!(stray.start_junction.is_none());
        assert!(stray.end_junction.is_none());
    }
}
