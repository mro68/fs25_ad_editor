//! RouteTool-Implementierung fuer das FieldBoundaryTool.

use std::sync::Arc;

use crate::app::group_registry::{GroupBase, GroupKind, GroupRecord};
use crate::app::tools::{ToolAction, ToolAnchor, ToolPreview, ToolResult};
use crate::core::{
    find_polygon_at, offset_polygon, simplify_polygon, ConnectionDirection, ConnectionPriority,
    FieldPolygon, NodeFlag, RoadMap,
};
use crate::shared::spline_geometry::resample_by_distance;
use glam::Vec2;

use super::state::{FieldBoundaryPhase, FieldBoundaryTool};

impl crate::app::tools::RouteTool for FieldBoundaryTool {
    fn name(&self) -> &str {
        "Feld erkennen"
    }

    fn icon(&self) -> &str {
        "\u{1f33e}" // 🌾
    }

    fn description(&self) -> &str {
        "Erzeugt eine Route entlang der erkannten Feldgrenze"
    }

    fn status_text(&self) -> &str {
        match self.phase {
            FieldBoundaryPhase::Idle => "In ein Feld klicken zum Erkennen der Grenze",
            FieldBoundaryPhase::Configuring => {
                "Einstellungen anpassen \u{2014} Best\u{e4}tigen oder Abbrechen"
            }
        }
    }

    fn on_click(&mut self, pos: Vec2, _road_map: &RoadMap, _ctrl: bool) -> ToolAction {
        match self.phase {
            FieldBoundaryPhase::Idle => {
                // Feldpolygon an Klickposition suchen
                if let Some(data) = &self.farmland_data {
                    if let Some(polygon) = find_polygon_at(pos, data) {
                        self.selected_polygon = Some(polygon.clone());
                        self.phase = FieldBoundaryPhase::Configuring;
                    } else {
                        log::info!(
                            "Kein Feld an Position ({:.1}, {:.1}) gefunden",
                            pos.x,
                            pos.y
                        );
                    }
                } else {
                    log::warn!("Keine Farmland-Daten geladen – Feld-Erkennung nicht moeglich");
                }
                ToolAction::Continue
            }
            FieldBoundaryPhase::Configuring => {
                // Erneuter Klick → Auswahl zuruecksetzen, neues Feld suchen
                self.selected_polygon = None;
                self.phase = FieldBoundaryPhase::Idle;
                ToolAction::Continue
            }
        }
    }

    fn preview(&self, _cursor_pos: Vec2, _road_map: &RoadMap) -> ToolPreview {
        let Some(polygon) = &self.selected_polygon else {
            return ToolPreview::default();
        };
        let corner_threshold = if self.corner_detection_enabled {
            Some(self.corner_angle_threshold_deg)
        } else {
            None
        };
        let nodes = compute_ring(
            &polygon.vertices,
            self.offset,
            self.straighten_tolerance,
            self.node_spacing,
            corner_threshold,
        );
        if nodes.len() < 2 {
            return ToolPreview::default();
        }
        let n = nodes.len();
        let connections: Vec<(usize, usize)> = (0..n).map(|i| (i, (i + 1) % n)).collect();
        let style = (self.direction, self.priority);
        let connection_styles = vec![style; connections.len()];
        ToolPreview {
            nodes,
            connections,
            connection_styles,
            labels: vec![],
        }
    }

    fn render_config(&mut self, ui: &mut egui::Ui, distance_wheel_step_m: f32) -> bool {
        self.render_config_view(ui, distance_wheel_step_m)
    }

    fn execute(&self, _road_map: &RoadMap) -> Option<ToolResult> {
        if self.phase != FieldBoundaryPhase::Configuring {
            return None;
        }
        let polygon = self.selected_polygon.as_ref()?;
        let corner_threshold = if self.corner_detection_enabled {
            Some(self.corner_angle_threshold_deg)
        } else {
            None
        };
        let positions = compute_ring(
            &polygon.vertices,
            self.offset,
            self.straighten_tolerance,
            self.node_spacing,
            corner_threshold,
        );
        if positions.len() < 2 {
            return None;
        }
        let n = positions.len();
        let new_nodes: Vec<(Vec2, NodeFlag)> = positions
            .into_iter()
            .map(|p| (p, NodeFlag::Regular))
            .collect();
        let internal_connections = (0..n)
            .map(|i| (i, (i + 1) % n, self.direction, self.priority))
            .collect();
        Some(ToolResult {
            new_nodes,
            internal_connections,
            external_connections: Vec::new(),
            markers: Vec::new(),
            nodes_to_remove: Vec::new(),
        })
    }

    fn reset(&mut self) {
        self.phase = FieldBoundaryPhase::Idle;
        self.selected_polygon = None;
    }

    fn is_ready(&self) -> bool {
        self.phase == FieldBoundaryPhase::Configuring && self.selected_polygon.is_some()
    }

    fn has_pending_input(&self) -> bool {
        self.phase == FieldBoundaryPhase::Configuring
    }

    // ── Lifecycle-Delegation (manuell, da kein SegmentConfig) ────

    fn set_direction(&mut self, dir: ConnectionDirection) {
        self.direction = dir;
    }

    fn set_priority(&mut self, prio: ConnectionPriority) {
        self.priority = prio;
    }

    fn set_snap_radius(&mut self, radius: f32) {
        self.lifecycle.snap_radius = radius;
    }

    fn set_farmland_data(&mut self, data: Option<Arc<Vec<FieldPolygon>>>) {
        self.farmland_data = data;
    }

    fn last_created_ids(&self) -> &[u64] {
        &self.lifecycle.last_created_ids
    }

    fn last_end_anchor(&self) -> Option<ToolAnchor> {
        self.lifecycle.last_end_anchor
    }

    fn needs_recreate(&self) -> bool {
        self.lifecycle.recreate_needed
    }

    fn clear_recreate_flag(&mut self) {
        self.lifecycle.recreate_needed = false;
    }

    fn set_last_created(&mut self, ids: &[u64], _road_map: &RoadMap) {
        self.lifecycle.save_created_ids(ids);
    }

    fn make_group_record(&self, id: u64, node_ids: &[u64]) -> Option<GroupRecord> {
        let polygon = self.selected_polygon.as_ref()?;
        Some(GroupRecord {
            id,
            node_ids: node_ids.to_vec(),
            start_anchor: ToolAnchor::NewPosition(Vec2::ZERO),
            end_anchor: ToolAnchor::NewPosition(Vec2::ZERO),
            original_positions: Vec::new(),
            marker_node_ids: Vec::new(),
            locked: true,
            entry_node_id: None,
            exit_node_id: None,
            kind: GroupKind::FieldBoundary {
                field_id: polygon.id,
                node_spacing: self.node_spacing,
                offset: self.offset,
                straighten_tolerance: self.straighten_tolerance,
                corner_angle_threshold: if self.corner_detection_enabled {
                    Some(self.corner_angle_threshold_deg)
                } else {
                    None
                },
                base: GroupBase {
                    direction: self.direction,
                    priority: self.priority,
                    max_segment_length: 0.0,
                },
            },
        })
    }

    fn load_for_edit(&mut self, _record: &GroupRecord, kind: &GroupKind) {
        let GroupKind::FieldBoundary {
            field_id,
            node_spacing,
            offset,
            straighten_tolerance,
            corner_angle_threshold,
            base,
        } = kind
        else {
            return;
        };

        self.node_spacing = *node_spacing;
        self.offset = *offset;
        self.straighten_tolerance = *straighten_tolerance;
        if let Some(threshold) = corner_angle_threshold {
            self.corner_detection_enabled = true;
            self.corner_angle_threshold_deg = *threshold;
        } else {
            self.corner_detection_enabled = false;
            self.corner_angle_threshold_deg = 90.0;
        }
        self.direction = base.direction;
        self.priority = base.priority;

        let Some(polygons) = &self.farmland_data else {
            log::warn!(
                "FieldBoundary edit: keine Farmland-Daten verfuegbar fuer Feld-ID {}",
                field_id
            );
            self.selected_polygon = None;
            self.phase = FieldBoundaryPhase::Idle;
            return;
        };

        if let Some(polygon) = polygons.iter().find(|polygon| polygon.id == *field_id) {
            self.selected_polygon = Some(polygon.clone());
            self.phase = FieldBoundaryPhase::Configuring;
        } else {
            log::warn!(
                "FieldBoundary edit: Feld-ID {} nicht in Farmland-Daten gefunden",
                field_id
            );
            self.selected_polygon = None;
            self.phase = FieldBoundaryPhase::Idle;
        }
    }
}

/// Erkennt Eckpunkte eines Polygons anhand des Ablenkungswinkels zwischen aufeinanderfolgenden Segmenten.
///
/// Ein Vertex gilt als Ecke, wenn der Ablenkungswinkel >= `angle_threshold_rad`.
/// Kleinerer Schwellwert → mehr Ecken erkannt.
///
/// Gibt sortierte Indizes der erkannten Eckpunkte zurueck.
fn detect_corners(vertices: &[Vec2], angle_threshold_rad: f32) -> Vec<usize> {
    let n = vertices.len();
    if n < 3 {
        return Vec::new();
    }
    let mut corners = Vec::new();
    for i in 0..n {
        let prev = vertices[(i + n - 1) % n];
        let curr = vertices[i];
        let next = vertices[(i + 1) % n];
        let seg_a = (curr - prev).normalize_or_zero();
        let seg_b = (next - curr).normalize_or_zero();
        if seg_a == Vec2::ZERO || seg_b == Vec2::ZERO {
            continue;
        }
        // Ablenkungswinkel als Bogenwinkel (0 = Gerade, PI = U-Turn)
        let cos_angle = seg_a.dot(seg_b).clamp(-1.0, 1.0);
        let deflection = cos_angle.acos();
        if deflection >= angle_threshold_rad {
            corners.push(i);
        }
    }
    corners
}

/// Resampled einen Polygon-Ring segmentweise mit Eckpunkten als festen Ankerpunkten.
///
/// - `simplified`: Vereinfachtes Polygon (nicht geschlossen, ohne letzten==ersten Punkt)
/// - `corner_indices`: Sortierte Indizes der Eckpunkte
/// - `spacing`: maximaler Segment-Abstand beim Resampling
///
/// Gibt den resamplten Ring zurueck (ohne abschliessenden Duplikat-Punkt).
fn resample_ring_with_corners(
    simplified: &[Vec2],
    corner_indices: &[usize],
    spacing: f32,
) -> Vec<Vec2> {
    if corner_indices.is_empty() {
        // Keine Ecken → gesamten Ring normal resamplen
        let mut closed = simplified.to_vec();
        closed.push(simplified[0]);
        let mut r = resample_by_distance(&closed, spacing.max(0.1));
        if r.len() > 1 {
            r.pop();
        }
        return r;
    }

    let nc = corner_indices.len();
    let mut result = Vec::new();

    for c in 0..nc {
        let c_start = corner_indices[c];
        let c_end = corner_indices[(c + 1) % nc];

        // Segment vom aktuellen Eckpunkt zum naechsten aufbauen (mit Ring-Umbruch)
        let segment: Vec<Vec2> = if c_end >= c_start {
            simplified[c_start..=c_end].to_vec()
        } else {
            let mut seg = simplified[c_start..].to_vec();
            seg.extend_from_slice(&simplified[..=c_end]);
            seg
        };

        let resampled = resample_by_distance(&segment, spacing.max(0.1));

        // Letzten Punkt weglassen — er ist der Startpunkt des naechsten Segments
        let take = if resampled.len() > 1 {
            resampled.len() - 1
        } else {
            resampled.len()
        };
        result.extend_from_slice(&resampled[..take]);
    }

    result
}

/// Berechnet einen gleichmaessig abgetasteten, geschlossenen Ring aus einem Polygon.
///
/// - `offset`: Verschiebung der Vertices nach innen (negativ) oder aussen (positiv)
/// - `tolerance`: Douglas-Peucker-Vereinfachung (0 = keine)
/// - `spacing`: maximaler Segment-Abstand beim Resampling
/// - `corner_angle_threshold`: Winkel-Schwellwert in Grad fuer Ecken-Erkennung (None = deaktiviert)
pub fn compute_ring(
    vertices: &[Vec2],
    offset: f32,
    tolerance: f32,
    spacing: f32,
    corner_angle_threshold: Option<f32>,
) -> Vec<Vec2> {
    let offsetted = offset_polygon(vertices, offset);
    let simplified = simplify_polygon(&offsetted, tolerance);
    if simplified.len() < 3 {
        return Vec::new();
    }

    if let Some(threshold_deg) = corner_angle_threshold {
        let threshold_rad = threshold_deg.to_radians();
        let corners = detect_corners(&simplified, threshold_rad);
        resample_ring_with_corners(&simplified, &corners, spacing)
    } else {
        // Geschlossenen Ring fuer Resampling: letzter Punkt = erster Punkt
        let mut closed = simplified.clone();
        closed.push(simplified[0]);
        let mut resampled = resample_by_distance(&closed, spacing.max(0.1));
        // Letzten Punkt entfernen (Duplikat des ersten Punktes)
        if resampled.len() > 1 {
            resampled.pop();
        }
        resampled
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// Hilfsfunktion: Rechteck-Vertices aufbauen
    fn rectangle_vertices() -> Vec<Vec2> {
        vec![
            Vec2::new(0.0, 0.0),
            Vec2::new(100.0, 0.0),
            Vec2::new(100.0, 50.0),
            Vec2::new(0.0, 50.0),
        ]
    }

    #[test]
    fn test_compute_ring_ohne_ecken_identisch_zu_vorher() {
        // Ohne Ecken-Erkennung muss compute_ring wie bisher Resampling durchfuehren
        let verts = rectangle_vertices();
        let ring_ohne = compute_ring(&verts, 0.0, 0.0, 10.0, None);
        // Ring muss mindestens so viele Punkte haben wie der Umfang / Abstand
        let umfang = 2.0 * (100.0 + 50.0_f32);
        let erwartete_punkte = (umfang / 10.0).round() as usize;
        assert!(
            (ring_ohne.len() as i32 - erwartete_punkte as i32).abs() <= 2,
            "Erwartete ~{} Punkte, bekam {}",
            erwartete_punkte,
            ring_ohne.len()
        );
        // Kein Punkt darf doppelt sein (kein Closing-Duplikat)
        assert_ne!(
            ring_ohne.first(),
            ring_ohne.last(),
            "Erster != Letzter Punkt"
        );
    }

    #[test]
    fn test_detect_corners_rechteck_vier_ecken() {
        // Rechteck hat 4 rechte Winkel — alle sollen bei 80° Schwellwert erkannt werden
        let verts = rectangle_vertices();
        let threshold_rad = 80_f32.to_radians();
        let corners = detect_corners(&verts, threshold_rad);
        assert_eq!(
            corners.len(),
            4,
            "Rechteck sollte 4 Ecken haben, bekam: {:?}",
            corners
        );
    }

    #[test]
    fn test_detect_corners_kein_ergebnis_bei_hohem_schwellwert() {
        // Rechteck hat ~90° Ecken — bei 150° Schwellwert sollen keine erkannt werden
        let verts = rectangle_vertices();
        let threshold_rad = 150_f32.to_radians();
        let corners = detect_corners(&verts, threshold_rad);
        assert!(
            corners.is_empty(),
            "Bei 150° Schwellwert keine Ecken erwartet, bekam: {:?}",
            corners
        );
    }

    #[test]
    fn test_compute_ring_mit_ecken_rechteck_enthaelt_ecken() {
        // Mit Ecken-Erkennung bei 80° → alle 4 Ecken des Rechtecks bleiben als Pflichtpunkte
        let verts = rectangle_vertices();
        let ring_mit = compute_ring(&verts, 0.0, 0.0, 10.0, Some(80.0));
        assert!(
            ring_mit.len() >= 4,
            "Ring sollte mindestens 4 Punkte (= Ecken) haben"
        );
        // Die 4 Ecken muessen im Ring enthalten sein
        let ecken = [
            Vec2::new(0.0, 0.0),
            Vec2::new(100.0, 0.0),
            Vec2::new(100.0, 50.0),
            Vec2::new(0.0, 50.0),
        ];
        for ecke in &ecken {
            assert!(
                ring_mit.iter().any(|p| (*p - *ecke).length() < 1e-3),
                "Ecke {:?} fehlt im Ring",
                ecke
            );
        }
    }
}
