//! RouteTool-Implementierung fuer das FieldBoundaryTool.

use crate::app::tool_editing::{RouteToolEditPayload, ToolRouteBase};
use crate::app::tools::common::{sync_tool_host, ToolResultBuilder};
use crate::app::tools::{
    RouteTool, RouteToolCore, RouteToolGroupEdit, RouteToolHostSync, RouteToolPanelBridge,
    ToolAction, ToolHostContext, ToolPreview, ToolResult,
};
use crate::app::ui_contract::{RouteToolConfigState, RouteToolPanelAction, RouteToolPanelEffect};
use crate::core::{find_polygon_at, offset_polygon, simplify_polygon, NodeFlag, RoadMap};
use glam::Vec2;

use super::geometry::{detect_corners, resample_ring_with_corners, RingNodeKind};
use super::state::{FieldBoundaryPhase, FieldBoundaryTool};

impl RouteToolPanelBridge for FieldBoundaryTool {
    fn status_text(&self) -> &str {
        match self.phase {
            FieldBoundaryPhase::Idle => "In ein Feld klicken zum Erkennen der Grenze",
            FieldBoundaryPhase::Configuring => {
                "Einstellungen anpassen \u{2014} Best\u{e4}tigen oder Abbrechen"
            }
        }
    }

    fn panel_state(&self) -> RouteToolConfigState {
        RouteToolConfigState::FieldBoundary(self.panel_state())
    }

    fn apply_panel_action(&mut self, action: RouteToolPanelAction) -> RouteToolPanelEffect {
        let RouteToolPanelAction::FieldBoundary(action) = action else {
            return RouteToolPanelEffect::default();
        };

        self.apply_panel_action(action)
    }
}

impl RouteToolCore for FieldBoundaryTool {
    fn on_click(&mut self, pos: Vec2, _road_map: &RoadMap, _ctrl: bool) -> ToolAction {
        match self.phase {
            FieldBoundaryPhase::Idle => {
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
        let rounding_radius = if self.corner_detection_enabled && self.corner_rounding_enabled {
            Some(self.corner_rounding_radius)
        } else {
            None
        };
        let max_angle_deg = if self.corner_detection_enabled && self.corner_rounding_enabled {
            Some(self.corner_rounding_max_angle_deg)
        } else {
            None
        };
        let ring = compute_ring(
            &polygon.vertices,
            self.offset,
            self.straighten_tolerance,
            self.node_spacing,
            corner_threshold,
            rounding_radius,
            max_angle_deg,
        );
        if ring.len() < 2 {
            return ToolPreview::default();
        }
        let n = ring.len();
        let nodes: Vec<Vec2> = ring.into_iter().map(|(p, _)| p).collect();
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
        let rounding_radius = if self.corner_detection_enabled && self.corner_rounding_enabled {
            Some(self.corner_rounding_radius)
        } else {
            None
        };
        let max_angle_deg = if self.corner_detection_enabled && self.corner_rounding_enabled {
            Some(self.corner_rounding_max_angle_deg)
        } else {
            None
        };
        let ring = compute_ring(
            &polygon.vertices,
            self.offset,
            self.straighten_tolerance,
            self.node_spacing,
            corner_threshold,
            rounding_radius,
            max_angle_deg,
        );
        if ring.len() < 2 {
            return None;
        }
        let n = ring.len();
        let new_nodes: Vec<(Vec2, NodeFlag)> = ring
            .into_iter()
            .map(|(p, kind)| {
                let flag = if kind == RingNodeKind::RoundedCorner {
                    NodeFlag::RoundedCorner
                } else {
                    NodeFlag::Regular
                };
                (p, flag)
            })
            .collect();
        let internal_connections = (0..n)
            .map(|i| (i, (i + 1) % n, self.direction, self.priority))
            .collect();
        Some(ToolResultBuilder::new(new_nodes, internal_connections).build())
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
}

impl RouteToolHostSync for FieldBoundaryTool {
    fn sync_host(&mut self, context: &ToolHostContext) {
        sync_tool_host(
            &mut self.direction,
            &mut self.priority,
            &mut self.lifecycle,
            context,
        );
        self.farmland_data = context.farmland_data.clone();
    }
}

impl RouteTool for FieldBoundaryTool {
    fn as_group_edit(&self) -> Option<&dyn RouteToolGroupEdit> {
        Some(self)
    }

    fn as_group_edit_mut(&mut self) -> Option<&mut dyn RouteToolGroupEdit> {
        Some(self)
    }
}

impl RouteToolGroupEdit for FieldBoundaryTool {
    fn build_edit_payload(&self) -> Option<RouteToolEditPayload> {
        let polygon = self.selected_polygon.as_ref()?;
        Some(RouteToolEditPayload::FieldBoundary {
            field_id: polygon.id,
            node_spacing: self.node_spacing,
            offset: self.offset,
            straighten_tolerance: self.straighten_tolerance,
            corner_angle_threshold: if self.corner_detection_enabled {
                Some(self.corner_angle_threshold_deg)
            } else {
                None
            },
            corner_rounding_radius: if self.corner_detection_enabled && self.corner_rounding_enabled
            {
                Some(self.corner_rounding_radius)
            } else {
                None
            },
            corner_rounding_max_angle_deg: if self.corner_detection_enabled
                && self.corner_rounding_enabled
            {
                Some(self.corner_rounding_max_angle_deg)
            } else {
                None
            },
            base: ToolRouteBase {
                direction: self.direction,
                priority: self.priority,
                max_segment_length: 0.0,
            },
        })
    }

    fn restore_edit_payload(&mut self, payload: &RouteToolEditPayload) {
        let RouteToolEditPayload::FieldBoundary {
            field_id,
            node_spacing,
            offset,
            straighten_tolerance,
            corner_angle_threshold,
            corner_rounding_radius,
            corner_rounding_max_angle_deg,
            base,
        } = payload
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
        if let Some(radius) = corner_rounding_radius {
            self.corner_rounding_enabled = true;
            self.corner_rounding_radius = *radius;
        } else {
            self.corner_rounding_enabled = false;
            self.corner_rounding_radius = 5.0;
        }
        if let Some(angle) = corner_rounding_max_angle_deg {
            self.corner_rounding_max_angle_deg = *angle;
        } else {
            self.corner_rounding_max_angle_deg = 15.0;
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

/// Berechnet einen gleichmaessig abgetasteten, geschlossenen Ring aus einem Polygon.
///
/// - `offset`: Verschiebung der Vertices nach innen (negativ) oder aussen (positiv)
/// - `tolerance`: Douglas-Peucker-Vereinfachung (0 = keine)
/// - `spacing`: maximaler Segment-Abstand beim Resampling der geraden Segmente
/// - `corner_angle_threshold`: Winkel-Schwellwert in Grad fuer Ecken-Erkennung (None = deaktiviert)
/// - `rounding_radius`: Verrundungsradius fuer konvexe Ecken in Metern (None = deaktiviert)
/// - `max_angle_deg`: Maximale Winkelabweichung zwischen Bogenpunkten in Grad (None = 15°)
///
/// Ruckgabe: Alle Ring-Positionen mit `RingNodeKind`-Markierung.
pub fn compute_ring(
    vertices: &[Vec2],
    offset: f32,
    tolerance: f32,
    spacing: f32,
    corner_angle_threshold: Option<f32>,
    rounding_radius: Option<f32>,
    max_angle_deg: Option<f32>,
) -> Vec<(Vec2, RingNodeKind)> {
    use crate::shared::spline_geometry::resample_by_distance;

    // Node-Abstand muss mindestens so gross sein wie der Verrundungsradius,
    // damit Bogensegmente nicht staerker abgetastet werden als der Bogen lang ist.
    let spacing = if let Some(r) = rounding_radius {
        spacing.max(r)
    } else {
        spacing
    };

    let offsetted = offset_polygon(vertices, offset);
    let simplified = simplify_polygon(&offsetted, tolerance);
    if simplified.len() < 3 {
        return Vec::new();
    }

    if let Some(threshold_deg) = corner_angle_threshold {
        let threshold_rad = threshold_deg.to_radians();
        let corners = detect_corners(&simplified, threshold_rad);
        let angle = max_angle_deg.unwrap_or(15.0).clamp(1.0, 45.0);
        resample_ring_with_corners(&simplified, &corners, spacing, rounding_radius, angle)
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
            .into_iter()
            .map(|p| (p, RingNodeKind::Regular))
            .collect()
    }
}

#[cfg(test)]
mod tests {
    use super::super::geometry::detect_corners;
    use super::*;
    use crate::app::tools::RouteToolCore;
    use crate::core::{FieldPolygon, RoadMap};

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
        let ring_ohne = compute_ring(&verts, 0.0, 0.0, 10.0, None, None, None);
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
            ring_ohne.first().map(|(p, _)| p),
            ring_ohne.last().map(|(p, _)| p),
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
        let ring_mit = compute_ring(&verts, 0.0, 0.0, 10.0, Some(80.0), None, None);
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
                ring_mit.iter().any(|(p, _)| (*p - *ecke).length() < 1e-3),
                "Ecke {:?} fehlt im Ring",
                ecke
            );
        }
    }

    #[test]
    fn test_execute_kanonisiert_leere_tool_result_seitenkanaele() {
        let mut tool = FieldBoundaryTool::new();
        tool.phase = FieldBoundaryPhase::Configuring;
        tool.selected_polygon = Some(FieldPolygon {
            id: 7,
            vertices: rectangle_vertices(),
        });

        let result = tool
            .execute(&RoadMap::new(2))
            .expect("Konfiguriertes Rechteck muss ein ToolResult liefern");

        assert!(!result.new_nodes.is_empty());
        assert!(!result.internal_connections.is_empty());
        assert!(result.external_connections.is_empty());
        assert!(result.markers.is_empty());
        assert!(result.nodes_to_remove.is_empty());
    }
}
