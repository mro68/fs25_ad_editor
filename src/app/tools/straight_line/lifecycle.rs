//! Lifecycle-Methoden des StraightLineTool.

use super::super::{
    common::{record_applied_tool_state, sync_tool_host},
    RouteTool, RouteToolCore, RouteToolHostSync, RouteToolId, RouteToolPanelBridge,
    RouteToolRecreate, RouteToolSegmentAdjustments, ToolAction, ToolHostContext, ToolPreview,
    ToolResult,
};
use super::geometry::{build_result, compute_line_positions};
use super::state::StraightLineTool;
use crate::app::group_registry::{GroupBase, GroupKind, GroupRecord};
use crate::app::ui_contract::{RouteToolConfigState, RouteToolPanelAction, RouteToolPanelEffect};
use crate::core::RoadMap;
use glam::Vec2;

impl RouteToolPanelBridge for StraightLineTool {
    fn status_text(&self) -> &str {
        match (&self.start, &self.end) {
            (None, _) => "Startpunkt klicken",
            (Some(_), None) => "Endpunkt klicken",
            (Some(_), Some(_)) => "Bereit — Enter zum Ausfuehren, Escape zum Abbrechen",
        }
    }

    fn panel_state(&self) -> RouteToolConfigState {
        RouteToolConfigState::Straight(self.panel_state())
    }

    fn apply_panel_action(&mut self, action: RouteToolPanelAction) -> RouteToolPanelEffect {
        let RouteToolPanelAction::Straight(action) = action else {
            return RouteToolPanelEffect::default();
        };

        self.apply_panel_action(action)
    }
}

impl RouteToolCore for StraightLineTool {
    fn on_click(&mut self, pos: Vec2, road_map: &RoadMap, _ctrl: bool) -> ToolAction {
        let anchor = self.lifecycle.snap_at(pos, road_map);

        if self.start.is_none() {
            if let Some(last_end) = self.lifecycle.chaining_start_anchor() {
                self.lifecycle.prepare_for_chaining();
                self.last_start_anchor = None;
                self.start = Some(last_end);
                self.end = Some(anchor);
                self.sync_derived();
                ToolAction::ReadyToExecute
            } else {
                self.start = Some(anchor);
                ToolAction::Continue
            }
        } else {
            self.end = Some(anchor);
            self.sync_derived();
            ToolAction::ReadyToExecute
        }
    }

    fn preview(&self, cursor_pos: Vec2, road_map: &RoadMap) -> ToolPreview {
        let start_pos = match &self.start {
            Some(anchor) => anchor.position(),
            None => return ToolPreview::default(),
        };

        let end_pos = match &self.end {
            Some(anchor) => anchor.position(),
            None => self.lifecycle.snap_at(cursor_pos, road_map).position(),
        };

        let positions = compute_line_positions(start_pos, end_pos, self.seg.max_segment_length);
        ToolPreview::from_polyline(positions, self.direction, self.priority)
    }

    fn execute(&self, road_map: &RoadMap) -> Option<ToolResult> {
        let start = *self.start.as_ref()?;
        let end = *self.end.as_ref()?;
        build_result(
            start,
            end,
            self.seg.max_segment_length,
            self.direction,
            self.priority,
            road_map,
        )
    }

    fn reset(&mut self) {
        self.start = None;
        self.end = None;
    }

    fn is_ready(&self) -> bool {
        self.start.is_some() && self.end.is_some()
    }

    fn has_pending_input(&self) -> bool {
        self.start.is_some()
    }
}

impl RouteToolHostSync for StraightLineTool {
    fn sync_host(&mut self, context: &ToolHostContext) {
        sync_tool_host(
            &mut self.direction,
            &mut self.priority,
            &mut self.lifecycle,
            context,
        );
    }
}

impl RouteToolRecreate for StraightLineTool {
    fn on_applied(&mut self, ids: &[u64], _road_map: &RoadMap) {
        if self.start.is_some() {
            self.last_start_anchor = self.start;
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
            start,
            end,
            self.seg.max_segment_length,
            self.direction,
            self.priority,
            road_map,
        )
    }
}

impl RouteToolSegmentAdjustments for StraightLineTool {
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

impl RouteTool for StraightLineTool {
    fn as_recreate(&self) -> Option<&dyn RouteToolRecreate> {
        Some(self)
    }

    fn as_recreate_mut(&mut self) -> Option<&mut dyn RouteToolRecreate> {
        Some(self)
    }

    fn as_segment_adjustments(&self) -> Option<&dyn RouteToolSegmentAdjustments> {
        Some(self)
    }

    fn as_segment_adjustments_mut(&mut self) -> Option<&mut dyn RouteToolSegmentAdjustments> {
        Some(self)
    }

    fn make_group_record(&self, id: u64, node_ids: &[u64]) -> Option<GroupRecord> {
        let start = self.last_start_anchor?;
        let end = self.lifecycle.last_end_anchor?;
        Some(GroupRecord {
            id,
            tool_id: Some(RouteToolId::Straight),
            node_ids: node_ids.to_vec(),
            start_anchor: start,
            end_anchor: end,
            original_positions: Vec::new(), // wird im Handler befüllt
            marker_node_ids: Vec::new(),
            locked: true,
            entry_node_id: None,
            exit_node_id: None,
            kind: GroupKind::Straight {
                base: GroupBase {
                    direction: self.direction,
                    priority: self.priority,
                    max_segment_length: self.seg.max_segment_length,
                },
            },
        })
    }

    fn load_for_edit(&mut self, record: &GroupRecord, kind: &GroupKind) {
        let GroupKind::Straight { base } = kind else {
            return;
        };
        self.start = Some(record.start_anchor);
        self.end = Some(record.end_anchor);
        self.direction = base.direction;
        self.priority = base.priority;
        self.seg.max_segment_length = base.max_segment_length;
        self.sync_derived();
    }
}
