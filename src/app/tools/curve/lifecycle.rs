//! Lifecycle-Methoden des CurveTool (on_click, preview, execute, reset, etc.).

use super::super::{
    common::{linear_connections, populate_neighbors, TangentMenuData},
    RouteTool, ToolAction, ToolPreview, ToolResult,
};
use super::geometry::{build_tool_result, cubic_bezier, CurveParams};
use super::state::{CurveDegree, CurvePreviewCacheKey, CurveTool, Phase};
use crate::app::segment_registry::{SegmentKind, SegmentRecord};
use crate::core::RoadMap;
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
                        "Punkte per Drag anpassen — Enter bestaetigt"
                    } else {
                        "Steuerpunkt klicken"
                    }
                }
                CurveDegree::Cubic => {
                    let has1 = self.control_point1.is_some();
                    let has2 = self.control_point2.is_some();
                    if has1 && has2 {
                        "Scheitelpunkt (Mitte) per Drag anpassen — Enter bestaetigt"
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
        self.invalidate_preview_cache();
        match self.phase {
            Phase::Start => {
                // Verkettung: letzten Endpunkt als Start verwenden
                if let Some(last_end) = self.lifecycle.chaining_start_anchor() {
                    self.lifecycle.prepare_for_chaining();
                    self.last_start_anchor = None;
                    self.last_control_point1 = None;
                    self.last_control_point2 = None;
                    self.start = Some(last_end);
                    self.tangents.start_neighbors = populate_neighbors(&last_end, road_map);
                    let (end_anchor, end_neighbors) =
                        self.lifecycle.snap_with_neighbors(pos, road_map);
                    self.tangents.end_neighbors = end_neighbors;
                    self.end = Some(end_anchor);
                    self.tangents.reset_tangents();
                    self.phase = Phase::Control;
                    // Auto-Tangente + beide CPs + Apex initialisieren
                    if self.degree == CurveDegree::Cubic {
                        self.auto_suggest_start_tangent();
                        self.auto_suggest_end_tangent();
                    }
                    self.apply_tangent_to_cp();
                    self.set_default_cp2_if_missing();
                    self.init_apex();
                    ToolAction::Continue
                } else {
                    let (start_anchor, start_neighbors) =
                        self.lifecycle.snap_with_neighbors(pos, road_map);
                    self.tangents.start_neighbors = start_neighbors;
                    self.tangents.tangent_start = super::super::common::TangentSource::None;
                    self.start = Some(start_anchor);
                    self.phase = Phase::End;
                    ToolAction::Continue
                }
            }
            Phase::End => {
                let (end_anchor, end_neighbors) =
                    self.lifecycle.snap_with_neighbors(pos, road_map);
                self.tangents.end_neighbors = end_neighbors;
                self.tangents.tangent_end = super::super::common::TangentSource::None;
                self.end = Some(end_anchor);
                self.phase = Phase::Control;
                // Auto-Tangente + beide CPs + Apex initialisieren
                if self.degree == CurveDegree::Cubic {
                    self.auto_suggest_start_tangent();
                    self.auto_suggest_end_tangent();
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
                    self.lifecycle.snap_at(cursor_pos, road_map).position();
                let connections = vec![(0, 1)];
                let styles = vec![(self.direction, self.priority)];
                ToolPreview {
                    nodes: vec![start_pos, end_pos],
                    connections,
                    connection_styles: styles,
                }
            }
            Phase::Control => {
                let end_pos = match &self.end {
                    Some(anchor) => anchor.position(),
                    None => return ToolPreview::default(),
                };

                let cp1 = self.control_point1.unwrap_or(cursor_pos);
                let cp2 = if self.degree == CurveDegree::Cubic {
                    Some(self.control_point2.unwrap_or(cursor_pos))
                } else {
                    None
                };
                let positions = self.preview_positions_for(CurvePreviewCacheKey {
                    degree: self.degree,
                    start: start_pos,
                    end: end_pos,
                    cp1,
                    cp2,
                    max_segment_length: self.seg.max_segment_length,
                });

                let connections = linear_connections(positions.len());
                let styles = vec![(self.direction, self.priority); connections.len()];

                // Steuerpunkte als zusaetzliche Vorschau-Nodes
                let mut nodes = positions;
                nodes.push(cp1);
                if self.degree == CurveDegree::Cubic {
                    let cp2 = cp2.unwrap_or_else(|| self.control_point2.unwrap_or(cursor_pos));
                    nodes.push(cp2);
                    // Virtueller Scheitelpunkt B(0.5) als draggbares Handle
                    let apex = self
                        .virtual_apex
                        .unwrap_or_else(|| cubic_bezier(start_pos, cp1, cp2, end_pos, 0.5));
                    nodes.push(apex);
                }

                ToolPreview {
                    nodes,
                    connections,
                    connection_styles: styles,
                }
            }
            _ => ToolPreview::default(),
        }
    }

    fn render_config(&mut self, ui: &mut egui::Ui, distance_wheel_step_m: f32) -> bool {
        self.render_config_view(ui, distance_wheel_step_m)
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

    /// Setzt das Tool vollstaendig zurueck (inkl. Tangenten und Phase).
    ///
    /// Im Gegensatz zu StraightLine/Spline werden hier auch Tangenten
    /// komplett zurueckgesetzt, da Control Points die primaere Steuerung sind.
    /// `lifecycle.last_created_ids` und `last_*_anchor` bleiben erhalten.
    fn reset(&mut self) {
        self.invalidate_preview_cache();
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

    fn has_pending_input(&self) -> bool {
        self.phase != Phase::Start
    }

    crate::impl_lifecycle_delegation!();

    fn current_end_anchor(&self) -> Option<super::super::ToolAnchor> {
        self.end.or(self.lifecycle.last_end_anchor)
    }

    fn save_anchors_for_recreate(&mut self, _road_map: &RoadMap) {
        if self.start.is_some() {
            self.last_start_anchor = self.start;
        }
        if self.control_point1.is_some() {
            self.last_control_point1 = self.control_point1;
        }
        if self.control_point2.is_some() {
            self.last_control_point2 = self.control_point2;
        }
        self.tangents.save_for_recreate();
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

    fn tangent_menu_data(&self) -> Option<TangentMenuData> {
        self.build_tangent_menu_data()
    }

    fn apply_tangent_selection(
        &mut self,
        start: super::super::common::TangentSource,
        end: super::super::common::TangentSource,
    ) {
        self.apply_tangent_from_menu(start, end);
    }

    fn make_segment_record(&self, id: u64, node_ids: &[u64]) -> Option<SegmentRecord> {
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
            node_ids: node_ids.to_vec(),
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
