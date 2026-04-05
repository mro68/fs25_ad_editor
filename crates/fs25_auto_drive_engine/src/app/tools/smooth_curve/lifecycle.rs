//! Lifecycle-Methoden des SmoothCurveTool.

use super::super::common::{linear_connections, record_applied_tool_state, sync_tool_host};
use super::super::{
    RouteTool, RouteToolCore, RouteToolDrag, RouteToolGroupEdit, RouteToolHostSync,
    RouteToolPanelBridge, RouteToolRecreate, RouteToolSegmentAdjustments, ToolAction,
    ToolHostContext, ToolPreview, ToolResult,
};
use super::geometry::{build_result, BuildResultParams};
use super::state::{Phase, SmoothCurveTool};
use crate::app::tool_editing::{RouteToolEditPayload, ToolEditAnchors, ToolRouteBase};
use crate::app::ui_contract::{RouteToolConfigState, RouteToolPanelAction, RouteToolPanelEffect};
use crate::core::RoadMap;
use glam::Vec2;

impl RouteToolPanelBridge for SmoothCurveTool {
    fn status_text(&self) -> &str {
        match self.phase {
            Phase::Start => "Startpunkt klicken",
            Phase::End => "Endpunkt klicken",
            Phase::ControlNodes => {
                "Kontrollpunkte klicken (Enter bestaetigt, Rechtsklick entfernt letzten)"
            }
        }
    }

    fn panel_state(&self) -> RouteToolConfigState {
        RouteToolConfigState::SmoothCurve(self.panel_state())
    }

    fn apply_panel_action(&mut self, action: RouteToolPanelAction) -> RouteToolPanelEffect {
        let RouteToolPanelAction::SmoothCurve(action) = action else {
            return RouteToolPanelEffect::default();
        };

        self.apply_panel_action(action)
    }
}

impl RouteToolCore for SmoothCurveTool {
    fn on_click(&mut self, pos: Vec2, road_map: &RoadMap, _ctrl: bool) -> ToolAction {
        let (anchor, _neighbors) = self.lifecycle.snap_with_neighbors(pos, road_map);

        match self.phase {
            Phase::Start => {
                if let Some(last_end) = self.lifecycle.chaining_start_anchor() {
                    self.lifecycle.prepare_for_chaining();
                    self.last_start_anchor = None;
                    self.last_control_nodes.clear();
                    self.start = Some(last_end);
                    self.start_neighbor_dirs =
                        SmoothCurveTool::collect_neighbor_dirs(&last_end, road_map);
                    self.end = Some(anchor);
                    self.end_neighbor_dirs =
                        SmoothCurveTool::collect_neighbor_dirs(&anchor, road_map);
                    self.phase = Phase::ControlNodes;
                    self.sync_derived();
                    self.update_preview();
                    ToolAction::UpdatePreview
                } else {
                    self.start = Some(anchor);
                    self.start_neighbor_dirs =
                        SmoothCurveTool::collect_neighbor_dirs(&anchor, road_map);
                    self.phase = Phase::End;
                    ToolAction::Continue
                }
            }
            Phase::End => {
                self.end = Some(anchor);
                self.end_neighbor_dirs = SmoothCurveTool::collect_neighbor_dirs(&anchor, road_map);
                self.phase = Phase::ControlNodes;
                self.sync_derived();
                self.update_preview();
                ToolAction::UpdatePreview
            }
            Phase::ControlNodes => {
                self.control_nodes.push(pos);
                self.sync_derived();
                self.update_preview();
                ToolAction::UpdatePreview
            }
        }
    }

    fn preview(&self, cursor_pos: Vec2, road_map: &RoadMap) -> ToolPreview {
        match self.phase {
            Phase::Start => ToolPreview::default(),
            Phase::End => {
                let start_pos = match &self.start {
                    Some(a) => a.position(),
                    None => return ToolPreview::default(),
                };
                let snapped = self.lifecycle.snap_at(cursor_pos, road_map);
                let end_pos = snapped.position();
                let nodes = vec![start_pos, end_pos];
                ToolPreview::from_polyline(nodes, self.direction, self.priority)
            }
            Phase::ControlNodes => {
                if self.preview_positions.is_empty() {
                    return ToolPreview::default();
                }
                let connections: Vec<(usize, usize)> = if self.preview_connections.is_empty() {
                    linear_connections(self.preview_positions.len())
                } else {
                    self.preview_connections.clone()
                };
                let styles = vec![(self.direction, self.priority); connections.len()];
                let mut nodes = self.preview_positions.clone();

                if let Some(ap) = self.approach_steerer {
                    nodes.push(ap);
                }
                if let Some(dp) = self.departure_steerer {
                    nodes.push(dp);
                }
                for &cp in &self.control_nodes {
                    nodes.push(cp);
                }

                ToolPreview {
                    nodes,
                    connections,
                    connection_styles: styles,
                    labels: vec![],
                }
            }
        }
    }

    fn execute(&self, road_map: &RoadMap) -> Option<ToolResult> {
        let start = *self.start.as_ref()?;
        let end = *self.end.as_ref()?;

        // Kontrollpunkte inkl. manuell verschobener Steuerpunkte zusammenbauen
        let mut solver_control = Vec::new();
        if self.approach_manual
            && let Some(ap) = self.approach_steerer
        {
            solver_control.push(ap);
        }
        solver_control.extend_from_slice(&self.control_nodes);
        if self.departure_manual
            && let Some(dp) = self.departure_steerer
        {
            solver_control.push(dp);
        }

        build_result(
            &BuildResultParams {
                start,
                end,
                control_nodes: &solver_control,
                max_segment_length: self.seg.max_segment_length,
                max_angle_deg: self.max_angle_deg,
                start_neighbor_dirs: if self.approach_manual {
                    &[]
                } else {
                    &self.start_neighbor_dirs
                },
                end_neighbor_dirs: if self.departure_manual {
                    &[]
                } else {
                    &self.end_neighbor_dirs
                },
                min_distance: self.min_distance,
                direction: self.direction,
                priority: self.priority,
            },
            road_map,
        )
    }

    fn reset(&mut self) {
        self.start = None;
        self.end = None;
        self.control_nodes.clear();
        self.phase = Phase::Start;
        self.dragging = None;
        self.preview_positions.clear();
        self.preview_connections.clear();
        self.start_neighbor_dirs.clear();
        self.end_neighbor_dirs.clear();
        self.approach_steerer = None;
        self.departure_steerer = None;
        self.approach_manual = false;
        self.departure_manual = false;
    }

    fn is_ready(&self) -> bool {
        self.start.is_some() && self.end.is_some()
    }

    fn has_pending_input(&self) -> bool {
        self.phase != Phase::Start
    }
}

impl RouteToolHostSync for SmoothCurveTool {
    fn sync_host(&mut self, context: &ToolHostContext) {
        sync_tool_host(
            &mut self.direction,
            &mut self.priority,
            &mut self.lifecycle,
            context,
        );
    }
}

impl RouteToolRecreate for SmoothCurveTool {
    fn on_applied(&mut self, ids: &[u64], _road_map: &RoadMap) {
        if self.start.is_some() {
            self.last_start_anchor = self.start;
        }
        if !self.control_nodes.is_empty() {
            self.last_control_nodes = self.control_nodes.clone();
        }
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
        let start = self.last_start_anchor?;
        let end = self.lifecycle.last_end_anchor?;
        build_result(
            &BuildResultParams {
                start,
                end,
                control_nodes: &self.last_control_nodes,
                max_segment_length: self.seg.max_segment_length,
                max_angle_deg: self.max_angle_deg,
                start_neighbor_dirs: &self.start_neighbor_dirs,
                end_neighbor_dirs: &self.end_neighbor_dirs,
                min_distance: self.min_distance,
                direction: self.direction,
                priority: self.priority,
            },
            road_map,
        )
    }
}

impl RouteToolDrag for SmoothCurveTool {
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

impl RouteToolSegmentAdjustments for SmoothCurveTool {
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

impl RouteTool for SmoothCurveTool {
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

impl RouteToolGroupEdit for SmoothCurveTool {
    fn build_edit_payload(&self) -> Option<RouteToolEditPayload> {
        let start = self.last_start_anchor?;
        let end = self.lifecycle.last_end_anchor?;
        Some(RouteToolEditPayload::SmoothCurve {
            anchors: ToolEditAnchors { start, end },
            control_nodes: self.last_control_nodes.clone(),
            max_angle_deg: self.max_angle_deg,
            min_distance: self.min_distance,
            base: ToolRouteBase {
                direction: self.direction,
                priority: self.priority,
                max_segment_length: self.seg.max_segment_length,
            },
        })
    }

    fn restore_edit_payload(&mut self, payload: &RouteToolEditPayload) {
        let RouteToolEditPayload::SmoothCurve {
            anchors,
            control_nodes,
            max_angle_deg,
            min_distance,
            base,
        } = payload
        else {
            return;
        };
        self.start = Some(anchors.start);
        self.end = Some(anchors.end);
        self.control_nodes = control_nodes.clone();
        self.max_angle_deg = *max_angle_deg;
        self.direction = base.direction;
        self.priority = base.priority;
        self.seg.max_segment_length = base.max_segment_length;
        self.min_distance = *min_distance;
        self.phase = Phase::ControlNodes;
        self.sync_derived();
        self.update_preview();
    }
}
