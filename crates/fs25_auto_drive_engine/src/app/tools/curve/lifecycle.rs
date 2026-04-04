//! Lifecycle-Methoden des CurveTool.

use super::super::{
    common::{linear_connections, populate_neighbors, record_applied_tool_state, sync_tool_host},
    RouteTool, RouteToolCore, RouteToolDrag, RouteToolGroupEdit, RouteToolHostSync,
    RouteToolPanelBridge, RouteToolRecreate, RouteToolSegmentAdjustments, RouteToolTangent,
    ToolAction, ToolHostContext, ToolPreview, ToolResult,
};
use super::geometry::{build_tool_result, cubic_bezier, CurveParams};
use super::state::{CurveDegree, CurvePreviewCacheKey, CurveTool, Phase};
use crate::app::tool_contract::TangentSource;
use crate::app::tool_editing::{RouteToolEditPayload, ToolEditAnchors, ToolRouteBase};
use crate::app::ui_contract::{
    RouteToolConfigState, RouteToolPanelAction, RouteToolPanelEffect, TangentMenuData,
};
use crate::core::RoadMap;
use glam::Vec2;

impl RouteToolPanelBridge for CurveTool {
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

    fn panel_state(&self) -> RouteToolConfigState {
        RouteToolConfigState::Curve(self.panel_state())
    }

    fn apply_panel_action(&mut self, action: RouteToolPanelAction) -> RouteToolPanelEffect {
        let RouteToolPanelAction::Curve(action) = action else {
            return RouteToolPanelEffect::default();
        };

        self.apply_panel_action(action)
    }
}

impl RouteToolCore for CurveTool {
    fn on_click(&mut self, pos: Vec2, road_map: &RoadMap, _ctrl: bool) -> ToolAction {
        self.invalidate_preview_cache();
        match self.phase {
            Phase::Start => {
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
                    self.tangents.tangent_start = TangentSource::None;
                    self.start = Some(start_anchor);
                    self.phase = Phase::End;
                    ToolAction::Continue
                }
            }
            Phase::End => {
                let (end_anchor, end_neighbors) = self.lifecycle.snap_with_neighbors(pos, road_map);
                self.tangents.end_neighbors = end_neighbors;
                self.tangents.tangent_end = TangentSource::None;
                self.end = Some(end_anchor);
                self.phase = Phase::Control;
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
                            self.tangents.tangent_start = TangentSource::None;
                        } else if self.control_point2.is_none() {
                            self.control_point2 = Some(pos);
                            self.tangents.tangent_end = TangentSource::None;
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
                let end_pos = self.lifecycle.snap_at(cursor_pos, road_map).position();
                ToolPreview::from_polyline(vec![start_pos, end_pos], self.direction, self.priority)
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

                let mut nodes = positions;
                nodes.push(cp1);
                if self.degree == CurveDegree::Cubic {
                    let cp2 = cp2.unwrap_or_else(|| self.control_point2.unwrap_or(cursor_pos));
                    nodes.push(cp2);
                    let apex = self
                        .virtual_apex
                        .unwrap_or_else(|| cubic_bezier(start_pos, cp1, cp2, end_pos, 0.5));
                    nodes.push(apex);
                }

                ToolPreview {
                    nodes,
                    connections,
                    connection_styles: styles,
                    labels: vec![],
                }
            }
            _ => ToolPreview::default(),
        }
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
}

impl RouteToolHostSync for CurveTool {
    fn sync_host(&mut self, context: &ToolHostContext) {
        sync_tool_host(
            &mut self.direction,
            &mut self.priority,
            &mut self.lifecycle,
            context,
        );
    }
}

impl RouteToolRecreate for CurveTool {
    fn on_applied(&mut self, ids: &[u64], _road_map: &RoadMap) {
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
        let end_anchor = self.end.or(self.lifecycle.last_end_anchor);
        record_applied_tool_state(&mut self.lifecycle, ids, end_anchor);
    }

    fn last_created_ids(&self) -> &[u64] {
        &self.lifecycle.last_created_ids
    }

    fn last_end_anchor(&self) -> Option<super::super::ToolAnchor> {
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
}

impl RouteToolDrag for CurveTool {
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
}

impl RouteToolTangent for CurveTool {
    fn tangent_menu_data(&self) -> Option<TangentMenuData> {
        self.build_tangent_menu_data()
    }

    fn apply_tangent_selection(&mut self, start: TangentSource, end: TangentSource) {
        self.apply_tangent_from_menu(start, end);
    }
}

impl RouteToolSegmentAdjustments for CurveTool {
    fn increase_node_count(&mut self) {
        self.seg.increase_node_count();
        self.lifecycle.recreate_needed = true;
    }

    fn decrease_node_count(&mut self) {
        self.seg.decrease_node_count();
        self.lifecycle.recreate_needed = true;
    }

    fn increase_segment_length(&mut self) {
        self.seg.increase_segment_length();
        self.lifecycle.recreate_needed = true;
    }

    fn decrease_segment_length(&mut self) {
        self.seg.decrease_segment_length();
        self.lifecycle.recreate_needed = true;
    }
}

impl RouteTool for CurveTool {
    fn as_recreate(&self) -> Option<&dyn RouteToolRecreate> {
        Some(self)
    }

    fn as_recreate_mut(&mut self) -> Option<&mut dyn RouteToolRecreate> {
        Some(self)
    }

    fn as_drag(&self) -> Option<&dyn RouteToolDrag> {
        Some(self)
    }

    fn as_drag_mut(&mut self) -> Option<&mut dyn RouteToolDrag> {
        Some(self)
    }

    fn as_tangent(&self) -> Option<&dyn RouteToolTangent> {
        Some(self)
    }

    fn as_tangent_mut(&mut self) -> Option<&mut dyn RouteToolTangent> {
        Some(self)
    }

    fn as_segment_adjustments(&self) -> Option<&dyn RouteToolSegmentAdjustments> {
        Some(self)
    }

    fn as_segment_adjustments_mut(&mut self) -> Option<&mut dyn RouteToolSegmentAdjustments> {
        Some(self)
    }

    fn as_group_edit(&self) -> Option<&dyn RouteToolGroupEdit> {
        Some(self)
    }

    fn as_group_edit_mut(&mut self) -> Option<&mut dyn RouteToolGroupEdit> {
        Some(self)
    }
}

impl RouteToolGroupEdit for CurveTool {
    fn build_edit_payload(&self) -> Option<RouteToolEditPayload> {
        let start = self.last_start_anchor?;
        let end = self.lifecycle.last_end_anchor?;
        let cp1 = self.last_control_point1?;
        let anchors = ToolEditAnchors { start, end };
        match self.degree {
            CurveDegree::Quadratic => Some(RouteToolEditPayload::CurveQuad {
                anchors,
                cp1,
                base: ToolRouteBase {
                    direction: self.direction,
                    priority: self.priority,
                    max_segment_length: self.seg.max_segment_length,
                },
            }),
            CurveDegree::Cubic => {
                let cp2 = self.last_control_point2.unwrap_or(cp1);
                Some(RouteToolEditPayload::CurveCubic {
                    anchors,
                    cp1,
                    cp2,
                    tangent_start: self.tangents.last_tangent_start,
                    tangent_end: self.tangents.last_tangent_end,
                    base: ToolRouteBase {
                        direction: self.direction,
                        priority: self.priority,
                        max_segment_length: self.seg.max_segment_length,
                    },
                })
            }
        }
    }

    fn restore_edit_payload(&mut self, payload: &RouteToolEditPayload) {
        match payload {
            RouteToolEditPayload::CurveQuad { anchors, cp1, base } => {
                self.start = Some(anchors.start);
                self.end = Some(anchors.end);
                self.control_point1 = Some(*cp1);
                self.control_point2 = None;
                self.direction = base.direction;
                self.priority = base.priority;
                self.seg.max_segment_length = base.max_segment_length;
            }
            RouteToolEditPayload::CurveCubic {
                anchors,
                cp1,
                cp2,
                tangent_start,
                tangent_end,
                base,
            } => {
                self.start = Some(anchors.start);
                self.end = Some(anchors.end);
                self.control_point1 = Some(*cp1);
                self.control_point2 = Some(*cp2);
                self.tangents.tangent_start = *tangent_start;
                self.tangents.tangent_end = *tangent_end;
                self.direction = base.direction;
                self.priority = base.priority;
                self.seg.max_segment_length = base.max_segment_length;
            }
            _ => return,
        }
        self.phase = Phase::Control;
        self.init_apex();
    }
}
