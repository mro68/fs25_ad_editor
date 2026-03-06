//! RouteTool-Implementierung fuer das FieldBoundaryTool.

use std::sync::Arc;

use crate::app::segment_registry::{SegmentBase, SegmentKind, SegmentRecord};
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
                    }
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
        let nodes = compute_ring(
            &polygon.vertices,
            self.offset,
            self.straighten_tolerance,
            self.node_spacing,
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
        let positions = compute_ring(
            &polygon.vertices,
            self.offset,
            self.straighten_tolerance,
            self.node_spacing,
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

    fn make_segment_record(&self, id: u64, node_ids: &[u64]) -> Option<SegmentRecord> {
        let polygon = self.selected_polygon.as_ref()?;
        Some(SegmentRecord {
            id,
            node_ids: node_ids.to_vec(),
            start_anchor: ToolAnchor::NewPosition(Vec2::ZERO),
            end_anchor: ToolAnchor::NewPosition(Vec2::ZERO),
            original_positions: Vec::new(),
            marker_node_ids: Vec::new(),
            locked: true,
            kind: SegmentKind::FieldBoundary {
                field_id: polygon.id,
                node_spacing: self.node_spacing,
                offset: self.offset,
                straighten_tolerance: self.straighten_tolerance,
                base: SegmentBase {
                    direction: self.direction,
                    priority: self.priority,
                    max_segment_length: 0.0,
                },
            },
        })
    }
}

/// Berechnet einen gleichmaessig abgetasteten, geschlossenen Ring aus einem Polygon.
///
/// - `offset`: Verschiebung der Vertices nach innen (negativ) oder aussen (positiv)
/// - `tolerance`: Douglas-Peucker-Vereinfachung (0 = keine)
/// - `spacing`: maximaler Segment-Abstand beim Resampling
fn compute_ring(vertices: &[Vec2], offset: f32, tolerance: f32, spacing: f32) -> Vec<Vec2> {
    let offsetted = offset_polygon(vertices, offset);
    let simplified = simplify_polygon(&offsetted, tolerance);
    if simplified.len() < 3 {
        return Vec::new();
    }
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
