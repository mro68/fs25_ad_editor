//! Lifecycle-Methoden des CurveTool (on_click, preview, execute, reset, etc.).

use super::super::{
    common::populate_neighbors, snap_to_node, RouteTool, ToolAction, ToolAnchor, ToolPreview,
    ToolResult,
};
use super::geometry::{
    build_tool_result, compute_curve_positions, cubic_bezier, quadratic_bezier, CurveParams,
};
use super::state::{CurveDegree, CurveTool, Phase};
use crate::app::segment_registry::{SegmentKind, SegmentRecord};
use crate::core::{ConnectionDirection, ConnectionPriority, RoadMap};
use glam::Vec2;

impl RouteTool for CurveTool {
    fn name(&self) -> &str {
        self.tool_name
    }

    fn icon(&self) -> &str {
        match self.degree {
            CurveDegree::Quadratic => "⌒",
            CurveDegree::Cubic => "〜",
        }
    }

    fn description(&self) -> &str {
        match self.degree {
            CurveDegree::Quadratic => "Zeichnet eine quadratische Bézier-Kurve (1 Steuerpunkt)",
            CurveDegree::Cubic => "Zeichnet eine kubische Bézier-Kurve (2 Steuerpunkte)",
        }
    }

    fn status_text(&self) -> &str {
        match self.phase {
            Phase::Start => "Startpunkt klicken",
            Phase::End => "Endpunkt klicken",
            Phase::Control => match self.degree {
                CurveDegree::Quadratic => {
                    if self.control_point1.is_some() {
                        "Punkte per Drag anpassen — Enter bestätigt"
                    } else {
                        "Steuerpunkt klicken"
                    }
                }
                CurveDegree::Cubic => {
                    let has1 = self.control_point1.is_some();
                    let has2 = self.control_point2.is_some();
                    if has1 && has2 {
                        "Scheitelpunkt (Mitte) per Drag anpassen — Enter bestätigt"
                    } else if has1 {
                        "2. Steuerpunkt klicken"
                    } else {
                        "1. Steuerpunkt klicken"
                    }
                }
            },
        }
    }

    fn on_click(&mut self, pos: Vec2, road_map: &RoadMap, _ctrl: bool) -> ToolAction {
        match self.phase {
            Phase::Start => {
                // Verkettung: letzten Endpunkt als Start verwenden
                if let Some(last_end) = self.lifecycle.chaining_start_anchor() {
                    self.lifecycle.last_created_ids.clear();
                    self.last_start_anchor = None;
                    self.lifecycle.last_end_anchor = None;
                    self.last_control_point1 = None;
                    self.last_control_point2 = None;
                    self.lifecycle.recreate_needed = false;
                    self.start = Some(last_end);
                    self.tangents.start_neighbors = populate_neighbors(&last_end, road_map);
                    let end_anchor = snap_to_node(pos, road_map, self.lifecycle.snap_radius);
                    self.tangents.end_neighbors = populate_neighbors(&end_anchor, road_map);
                    self.end = Some(end_anchor);
                    self.tangents.reset_tangents();
                    self.phase = Phase::Control;
                    // Auto-Tangente + beide CPs + Apex initialisieren
                    if self.degree == CurveDegree::Cubic {
                        self.auto_suggest_start_tangent();
                    }
                    self.apply_tangent_to_cp();
                    self.set_default_cp2_if_missing();
                    self.init_apex();
                    ToolAction::Continue
                } else {
                    let start_anchor = snap_to_node(pos, road_map, self.lifecycle.snap_radius);
                    self.tangents.start_neighbors = populate_neighbors(&start_anchor, road_map);
                    self.tangents.tangent_start = super::super::common::TangentSource::None;
                    self.start = Some(start_anchor);
                    self.phase = Phase::End;
                    ToolAction::Continue
                }
            }
            Phase::End => {
                let end_anchor = snap_to_node(pos, road_map, self.lifecycle.snap_radius);
                self.tangents.end_neighbors = populate_neighbors(&end_anchor, road_map);
                self.tangents.tangent_end = super::super::common::TangentSource::None;
                self.end = Some(end_anchor);
                self.phase = Phase::Control;
                // Auto-Tangente + beide CPs + Apex initialisieren
                if self.degree == CurveDegree::Cubic {
                    self.auto_suggest_start_tangent();
                }
                self.apply_tangent_to_cp();
                self.set_default_cp2_if_missing();
                self.init_apex();
                ToolAction::Continue
            }
            Phase::Control => {
                match self.degree {
                    CurveDegree::Quadratic => {
                        if self.control_point1.is_none() {
                            self.control_point1 = Some(pos);
                        }
                    }
                    CurveDegree::Cubic => {
                        if self.control_point1.is_none() {
                            self.control_point1 = Some(pos);
                            self.tangents.tangent_start = super::super::common::TangentSource::None;
                        } else if self.control_point2.is_none() {
                            self.control_point2 = Some(pos);
                            self.tangents.tangent_end = super::super::common::TangentSource::None;
                        }
                    }
                }
                self.sync_derived();
                self.init_apex();
                ToolAction::UpdatePreview
            }
        }
    }

    fn preview(&self, cursor_pos: Vec2, road_map: &RoadMap) -> ToolPreview {
        let start_pos = match &self.start {
            Some(anchor) => anchor.position(),
            None => return ToolPreview::default(),
        };

        match self.phase {
            Phase::End => {
                let end_pos =
                    snap_to_node(cursor_pos, road_map, self.lifecycle.snap_radius).position();
                ToolPreview {
                    nodes: vec![start_pos, end_pos],
                    connections: vec![(0, 1)],
                }
            }
            Phase::Control => {
                let end_pos = match &self.end {
                    Some(anchor) => anchor.position(),
                    None => return ToolPreview::default(),
                };

                let cp1 = self.control_point1.unwrap_or(cursor_pos);

                let positions = match self.degree {
                    CurveDegree::Quadratic => compute_curve_positions(
                        |t| quadratic_bezier(start_pos, cp1, end_pos, t),
                        self.seg.max_segment_length,
                    ),
                    CurveDegree::Cubic => {
                        let cp2 = self.control_point2.unwrap_or(cursor_pos);
                        compute_curve_positions(
                            |t| cubic_bezier(start_pos, cp1, cp2, end_pos, t),
                            self.seg.max_segment_length,
                        )
                    }
                };

                let connections: Vec<(usize, usize)> = (0..positions.len().saturating_sub(1))
                    .map(|i| (i, i + 1))
                    .collect();

                // Steuerpunkte als zusätzliche Vorschau-Nodes
                let mut nodes = positions;
                nodes.push(cp1);
                if self.degree == CurveDegree::Cubic {
                    let cp2 = self.control_point2.unwrap_or(cursor_pos);
                    nodes.push(cp2);
                    // Virtueller Scheitelpunkt B(0.5) als draggbares Handle
                    let apex = self
                        .virtual_apex
                        .unwrap_or_else(|| cubic_bezier(start_pos, cp1, cp2, end_pos, 0.5));
                    nodes.push(apex);
                }

                ToolPreview { nodes, connections }
            }
            _ => ToolPreview::default(),
        }
    }

    fn render_config(&mut self, ui: &mut egui::Ui) -> bool {
        self.render_config_view(ui)
    }

    fn execute(&self, road_map: &RoadMap) -> Option<ToolResult> {
        let start = self.start.as_ref()?;
        let end = self.end.as_ref()?;
        let params = CurveParams {
            degree: self.degree,
            cp1: self.control_point1?,
            cp2: self.control_point2,
            max_segment_length: self.seg.max_segment_length,
            direction: self.direction,
            priority: self.priority,
        };
        build_tool_result(start, end, &params, road_map)
    }

    fn reset(&mut self) {
        self.start = None;
        self.end = None;
        self.control_point1 = None;
        self.control_point2 = None;
        self.virtual_apex = None;
        self.phase = Phase::Start;
        self.tangents.reset_all();
    }

    fn is_ready(&self) -> bool {
        self.start.is_some() && self.end.is_some() && self.controls_complete()
    }

    fn set_direction(&mut self, dir: ConnectionDirection) {
        self.direction = dir;
    }

    fn set_priority(&mut self, prio: ConnectionPriority) {
        self.priority = prio;
    }

    fn set_snap_radius(&mut self, radius: f32) {
        self.lifecycle.snap_radius = radius;
    }

    fn set_last_created(&mut self, ids: Vec<u64>, _road_map: &RoadMap) {
        if self.start.is_some() {
            self.last_start_anchor = self.start;
        }
        if self.end.is_some() {
            self.lifecycle.last_end_anchor = self.end;
        }
        if self.control_point1.is_some() {
            self.last_control_point1 = self.control_point1;
        }
        if self.control_point2.is_some() {
            self.last_control_point2 = self.control_point2;
        }
        self.tangents.save_for_recreate();
        self.lifecycle.last_created_ids = ids;
        self.lifecycle.recreate_needed = false;
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

    fn execute_from_anchors(&self, road_map: &RoadMap) -> Option<ToolResult> {
        let start = self.last_start_anchor.as_ref()?;
        let end = self.lifecycle.last_end_anchor.as_ref()?;
        let params = CurveParams {
            degree: self.degree,
            cp1: self.last_control_point1?,
            cp2: self.last_control_point2,
            max_segment_length: self.seg.max_segment_length,
            direction: self.direction,
            priority: self.priority,
        };
        build_tool_result(start, end, &params, road_map)
    }

    fn drag_targets(&self) -> Vec<Vec2> {
        super::drag::drag_targets(self)
    }

    fn on_drag_start(&mut self, pos: Vec2, road_map: &RoadMap, pick_radius: f32) -> bool {
        super::drag::on_drag_start(self, pos, road_map, pick_radius)
    }

    fn on_drag_update(&mut self, pos: Vec2) {
        super::drag::on_drag_update(self, pos);
    }

    fn on_drag_end(&mut self, road_map: &RoadMap) {
        super::drag::on_drag_end(self, road_map);
    }

    fn render_context_menu(&mut self, response: &egui::Response) -> bool {
        self.render_tangent_context_menu(response)
    }

    fn make_segment_record(&self, id: u64, node_ids: Vec<u64>) -> Option<SegmentRecord> {
        let start = self.last_start_anchor?;
        let end = self.lifecycle.last_end_anchor?;
        let cp1 = self.last_control_point1?;
        let kind = match self.degree {
            CurveDegree::Quadratic => SegmentKind::CurveQuad {
                cp1,
                direction: self.direction,
                priority: self.priority,
                max_segment_length: self.seg.max_segment_length,
            },
            CurveDegree::Cubic => {
                let cp2 = self.last_control_point2.unwrap_or(cp1);
                SegmentKind::CurveCubic {
                    cp1,
                    cp2,
                    tangent_start: self.tangents.last_tangent_start,
                    tangent_end: self.tangents.last_tangent_end,
                    direction: self.direction,
                    priority: self.priority,
                    max_segment_length: self.seg.max_segment_length,
                }
            }
        };
        Some(SegmentRecord {
            id,
            node_ids,
            start_anchor: start,
            end_anchor: end,
            kind,
        })
    }

    fn load_for_edit(&mut self, record: &SegmentRecord, kind: &SegmentKind) {
        self.start = Some(record.start_anchor);
        self.end = Some(record.end_anchor);
        match kind {
            SegmentKind::CurveQuad {
                cp1,
                direction,
                priority,
                max_segment_length,
            } => {
                self.control_point1 = Some(*cp1);
                self.control_point2 = None;
                self.direction = *direction;
                self.priority = *priority;
                self.seg.max_segment_length = *max_segment_length;
            }
            SegmentKind::CurveCubic {
                cp1,
                cp2,
                tangent_start,
                tangent_end,
                direction,
                priority,
                max_segment_length,
            } => {
                self.control_point1 = Some(*cp1);
                self.control_point2 = Some(*cp2);
                self.tangents.tangent_start = *tangent_start;
                self.tangents.tangent_end = *tangent_end;
                self.direction = *direction;
                self.priority = *priority;
                self.seg.max_segment_length = *max_segment_length;
            }
            _ => return,
        }
        self.phase = Phase::Control;
        self.init_apex();
    }
}
