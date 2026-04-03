//! Lifecycle-Methoden des SplineTool.

use super::super::{
    common::{
        linear_connections, populate_neighbors, record_applied_tool_state, sync_tool_host,
        tangent_options,
    },
    RouteTool, RouteToolCore, RouteToolHostSync, RouteToolId, RouteToolPanelBridge,
    RouteToolRecreate, RouteToolSegmentAdjustments, RouteToolTangent, ToolAction, ToolHostContext,
    ToolPreview, ToolResult,
};
use super::state::SplineTool;
use crate::app::group_registry::{GroupBase, GroupKind, GroupRecord};
use crate::app::tool_contract::TangentSource;
use crate::app::ui_contract::{
    RouteToolConfigState, RouteToolPanelAction, RouteToolPanelEffect, TangentMenuData,
};
use crate::core::RoadMap;
use glam::Vec2;

impl RouteToolPanelBridge for SplineTool {
    fn status_text(&self) -> &str {
        match self.anchors.len() {
            0 => "Startpunkt klicken",
            1 => "Naechsten Punkt klicken (mind. 2 Punkte)",
            _ => "Weitere Punkte klicken — Enter bestaetigt, Escape abbrechen",
        }
    }

    fn panel_state(&self) -> RouteToolConfigState {
        RouteToolConfigState::Spline(self.panel_state())
    }

    fn apply_panel_action(&mut self, action: RouteToolPanelAction) -> RouteToolPanelEffect {
        let RouteToolPanelAction::Spline(action) = action else {
            return RouteToolPanelEffect::default();
        };

        self.apply_panel_action(action)
    }
}

impl RouteToolCore for SplineTool {
    fn on_click(&mut self, pos: Vec2, road_map: &RoadMap, _ctrl: bool) -> ToolAction {
        let (anchor, _neighbors) = self.lifecycle.snap_with_neighbors(pos, road_map);

        if self.anchors.is_empty() {
            if let Some(last_end) = self.lifecycle.chaining_start_anchor() {
                self.lifecycle.prepare_for_chaining();
                self.last_anchors.clear();
                self.tangents.reset_tangents();
                self.anchors.push(last_end);
                self.anchors.push(anchor);
                self.sync_derived();
                return ToolAction::UpdatePreview;
            }
        }

        self.anchors.push(anchor);

        if self.anchors.len() >= 2 {
            self.sync_derived();
            ToolAction::UpdatePreview
        } else {
            ToolAction::Continue
        }
    }

    fn preview(&self, cursor_pos: Vec2, road_map: &RoadMap) -> ToolPreview {
        if self.anchors.is_empty() {
            return ToolPreview::default();
        }

        let snapped_cursor = self.lifecycle.snap_at(cursor_pos, road_map).position();

        let positions = if self.anchors.len() == 1 {
            let start = self.anchors[0].position();
            vec![start, snapped_cursor]
        } else {
            self.compute_resampled(Some(snapped_cursor))
        };

        let connections = linear_connections(positions.len());
        let styles = vec![(self.direction, self.priority); connections.len()];

        let mut nodes = positions;
        for anchor in &self.anchors {
            nodes.push(anchor.position());
        }
        nodes.push(snapped_cursor);

        ToolPreview {
            nodes,
            connections,
            connection_styles: styles,
            labels: vec![],
        }
    }

    fn execute(&self, road_map: &RoadMap) -> Option<ToolResult> {
        Self::build_result_from_anchors(
            &self.anchors,
            self.seg.max_segment_length,
            self.direction,
            self.priority,
            self.tangents.tangent_start,
            self.tangents.tangent_end,
            road_map,
        )
    }

    fn reset(&mut self) {
        self.anchors.clear();
        self.tangents.reset_tangents();
    }

    fn is_ready(&self) -> bool {
        self.anchors.len() >= 2
    }

    fn has_pending_input(&self) -> bool {
        !self.anchors.is_empty()
    }
}

impl RouteToolHostSync for SplineTool {
    fn sync_host(&mut self, context: &ToolHostContext) {
        sync_tool_host(
            &mut self.direction,
            &mut self.priority,
            &mut self.lifecycle,
            context,
        );
    }
}

impl RouteToolRecreate for SplineTool {
    fn on_applied(&mut self, ids: &[u64], road_map: &RoadMap) {
        if !self.anchors.is_empty() {
            self.last_anchors = self.anchors.clone();
        }
        let source = if !self.anchors.is_empty() {
            &self.anchors
        } else {
            &self.last_anchors
        };
        if let Some(first) = source.first() {
            self.tangents.start_neighbors = populate_neighbors(first, road_map);
        }
        if let Some(last) = source.last() {
            self.tangents.end_neighbors = populate_neighbors(last, road_map);
        }
        self.tangents.save_for_recreate();
        let end_anchor = self
            .anchors
            .last()
            .copied()
            .or_else(|| self.last_anchors.last().copied())
            .or(self.lifecycle.last_end_anchor);
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
        // Aktuelle Tangenten verwenden (nicht last_tangent_*),
        // damit Aenderungen im Nachbearbeitungs-Modus wirksam werden
        Self::build_result_from_anchors(
            &self.last_anchors,
            self.seg.max_segment_length,
            self.direction,
            self.priority,
            self.tangents.tangent_start,
            self.tangents.tangent_end,
            road_map,
        )
    }
}

impl RouteToolTangent for SplineTool {
    fn tangent_menu_data(&self) -> Option<TangentMenuData> {
        let adjusting = !self.lifecycle.last_created_ids.is_empty() && self.last_anchors.len() >= 2;
        if !adjusting {
            return None;
        }

        let has_start = !self.tangents.start_neighbors.is_empty();
        let has_end = !self.tangents.end_neighbors.is_empty();
        if !has_start && !has_end {
            return None;
        }

        Some(TangentMenuData {
            start_options: tangent_options(&self.tangents.start_neighbors),
            end_options: tangent_options(&self.tangents.end_neighbors),
            current_start: self.tangents.tangent_start,
            current_end: self.tangents.tangent_end,
        })
    }

    fn apply_tangent_selection(&mut self, start: TangentSource, end: TangentSource) {
        self.tangents.tangent_start = start;
        self.tangents.tangent_end = end;
        self.sync_derived();
        if self.lifecycle.has_last_created() {
            self.lifecycle.recreate_needed = true;
        }
    }
}

impl RouteToolSegmentAdjustments for SplineTool {
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

impl RouteTool for SplineTool {
    fn as_recreate(&self) -> Option<&dyn RouteToolRecreate> {
        Some(self)
    }

    fn as_recreate_mut(&mut self) -> Option<&mut dyn RouteToolRecreate> {
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

    fn make_group_record(&self, id: u64, node_ids: &[u64]) -> Option<GroupRecord> {
        if self.last_anchors.len() < 2 {
            return None;
        }
        let start = *self.last_anchors.first()?;
        let end = *self.last_anchors.last()?;
        Some(GroupRecord {
            id,
            tool_id: Some(RouteToolId::Spline),
            node_ids: node_ids.to_vec(),
            start_anchor: start,
            end_anchor: end,
            original_positions: Vec::new(), // wird im Handler befüllt
            marker_node_ids: Vec::new(),
            locked: true,
            entry_node_id: None,
            exit_node_id: None,
            kind: GroupKind::Spline {
                anchors: self.last_anchors.clone(),
                tangent_start: self.tangents.last_tangent_start,
                tangent_end: self.tangents.last_tangent_end,
                base: GroupBase {
                    direction: self.direction,
                    priority: self.priority,
                    max_segment_length: self.seg.max_segment_length,
                },
            },
        })
    }

    fn load_for_edit(&mut self, _record: &GroupRecord, kind: &GroupKind) {
        let GroupKind::Spline {
            anchors,
            tangent_start,
            tangent_end,
            base,
        } = kind
        else {
            return;
        };
        self.anchors = anchors.clone();
        self.last_anchors = anchors.clone();
        self.tangents.tangent_start = *tangent_start;
        self.tangents.tangent_end = *tangent_end;
        self.tangents.last_tangent_start = *tangent_start;
        self.tangents.last_tangent_end = *tangent_end;
        self.direction = base.direction;
        self.priority = base.priority;
        self.seg.max_segment_length = base.max_segment_length;
        self.sync_derived();
    }
}
