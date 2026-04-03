//! Route-Tool-Implementierung fuer das Ausweichstrecken-Tool.

use std::borrow::Cow;

use super::geometry::compute_bypass_positions;
use super::state::BypassTool;
use crate::app::tool_editing::{RouteToolEditPayload, ToolRouteBase};
use crate::app::tools::common::{assemble_tool_result, record_applied_tool_state, sync_tool_host};
use crate::app::tools::{
    OrderedNodeChain, RouteTool, RouteToolChainInput, RouteToolCore, RouteToolGroupEdit,
    RouteToolHostSync, RouteToolPanelBridge, RouteToolRecreate, ToolAction, ToolAnchor,
    ToolHostContext, ToolPreview, ToolResult,
};
use crate::app::ui_contract::{RouteToolConfigState, RouteToolPanelAction, RouteToolPanelEffect};
use crate::core::RoadMap;
use glam::Vec2;

impl BypassTool {
    pub(crate) fn load_chain(&mut self, positions: Vec<Vec2>, start_id: u64, end_id: u64) {
        self.chain_positions = positions;
        self.chain_start_id = start_id;
        self.chain_end_id = end_id;
        self.cached_positions = None;
        self.cached_connections = None;
        self.d_blend = 0.0;
    }
}

impl RouteToolPanelBridge for BypassTool {
    fn status_text(&self) -> &str {
        if self.has_chain() {
            "Bereit — Enter zum Ausfuehren, Escape zum Abbrechen"
        } else {
            "Kette selektieren, dann Route-Tool neu aktivieren"
        }
    }

    fn panel_state(&self) -> RouteToolConfigState {
        RouteToolConfigState::Bypass(self.panel_state())
    }

    fn apply_panel_action(&mut self, action: RouteToolPanelAction) -> RouteToolPanelEffect {
        let RouteToolPanelAction::Bypass(action) = action else {
            return RouteToolPanelEffect::default();
        };

        self.apply_panel_action(action)
    }
}

impl RouteToolCore for BypassTool {
    fn on_click(&mut self, _pos: Vec2, _road_map: &RoadMap, _ctrl: bool) -> ToolAction {
        ToolAction::Continue
    }

    fn preview(&self, _cursor_pos: Vec2, _road_map: &RoadMap) -> ToolPreview {
        if !self.has_chain() {
            return ToolPreview::default();
        }

        let positions: Cow<'_, [Vec2]> = if let Some(cached) = &self.cached_positions {
            Cow::Borrowed(cached.as_slice())
        } else {
            let Some((new_pts, _d_blend)) =
                compute_bypass_positions(&self.chain_positions, self.offset, self.base_spacing)
            else {
                return ToolPreview::default();
            };
            Cow::Owned(new_pts)
        };

        let chain_start = *self
            .chain_positions
            .first()
            .expect("invariant: chain_positions ist nicht-leer nach load_chain()");
        let chain_end = *self
            .chain_positions
            .last()
            .expect("invariant: chain_positions ist nicht-leer nach load_chain()");

        let mut nodes = Vec::with_capacity(positions.len() + 2);
        nodes.push(chain_start);
        nodes.extend_from_slice(&positions);
        nodes.push(chain_end);

        let connections: Vec<(usize, usize)> = if let Some(cached) = &self.cached_connections {
            cached.clone()
        } else {
            (0..nodes.len().saturating_sub(1))
                .map(|i| (i, i + 1))
                .collect()
        };
        let styles = vec![(self.direction, self.priority); connections.len()];

        ToolPreview {
            nodes,
            connections,
            connection_styles: styles,
            labels: vec![],
        }
    }

    fn execute(&self, road_map: &RoadMap) -> Option<ToolResult> {
        if !self.has_chain() {
            return None;
        }

        let (new_positions, _d_blend) =
            compute_bypass_positions(&self.chain_positions, self.offset, self.base_spacing)?;

        let chain_start_pos = *self
            .chain_positions
            .first()
            .expect("invariant: chain_positions ist nicht-leer nach load_chain()");
        let chain_end_pos = *self
            .chain_positions
            .last()
            .expect("invariant: chain_positions ist nicht-leer nach load_chain()");

        let mut all_positions = Vec::with_capacity(new_positions.len() + 2);
        all_positions.push(chain_start_pos);
        all_positions.extend_from_slice(&new_positions);
        all_positions.push(chain_end_pos);

        Some(assemble_tool_result(
            &all_positions,
            &ToolAnchor::ExistingNode(self.chain_start_id, chain_start_pos),
            &ToolAnchor::ExistingNode(self.chain_end_id, chain_end_pos),
            self.direction,
            self.priority,
            road_map,
        ))
    }

    fn reset(&mut self) {
        self.chain_positions.clear();
        self.cached_positions = None;
        self.cached_connections = None;
        self.d_blend = 0.0;
        let snap_radius = self.lifecycle.snap_radius;
        self.lifecycle = crate::app::tools::common::ToolLifecycleState::new(snap_radius);
    }

    fn is_ready(&self) -> bool {
        self.has_chain()
    }

    fn has_pending_input(&self) -> bool {
        self.has_chain()
    }
}

impl RouteToolHostSync for BypassTool {
    fn sync_host(&mut self, context: &ToolHostContext) {
        sync_tool_host(
            &mut self.direction,
            &mut self.priority,
            &mut self.lifecycle,
            context,
        );
    }
}

impl RouteToolRecreate for BypassTool {
    fn on_applied(&mut self, ids: &[u64], _road_map: &RoadMap) {
        record_applied_tool_state(&mut self.lifecycle, ids, None);
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
        self.execute(road_map)
    }
}

impl RouteToolChainInput for BypassTool {
    fn load_chain(&mut self, chain: OrderedNodeChain) {
        self.load_chain(chain.positions, chain.start_id, chain.end_id);
    }
}

impl RouteTool for BypassTool {
    fn as_recreate(&self) -> Option<&dyn RouteToolRecreate> {
        Some(self)
    }

    fn as_recreate_mut(&mut self) -> Option<&mut dyn RouteToolRecreate> {
        Some(self)
    }

    fn as_chain_input(&self) -> Option<&dyn RouteToolChainInput> {
        Some(self)
    }

    fn as_chain_input_mut(&mut self) -> Option<&mut dyn RouteToolChainInput> {
        Some(self)
    }

    fn as_group_edit(&self) -> Option<&dyn RouteToolGroupEdit> {
        Some(self)
    }

    fn as_group_edit_mut(&mut self) -> Option<&mut dyn RouteToolGroupEdit> {
        Some(self)
    }
}

impl RouteToolGroupEdit for BypassTool {
    fn build_edit_payload(&self) -> Option<RouteToolEditPayload> {
        if !self.has_chain() {
            return None;
        }
        Some(RouteToolEditPayload::Bypass {
            chain_positions: self.chain_positions.clone(),
            chain_start_id: self.chain_start_id,
            chain_end_id: self.chain_end_id,
            offset: self.offset,
            base_spacing: self.base_spacing,
            base: ToolRouteBase {
                direction: self.direction,
                priority: self.priority,
                max_segment_length: self.base_spacing,
            },
        })
    }

    fn restore_edit_payload(&mut self, payload: &RouteToolEditPayload) {
        let RouteToolEditPayload::Bypass {
            chain_positions,
            chain_start_id,
            chain_end_id,
            offset,
            base_spacing,
            base,
        } = payload
        else {
            return;
        };
        self.load_chain(chain_positions.clone(), *chain_start_id, *chain_end_id);
        self.offset = *offset;
        self.base_spacing = *base_spacing;
        self.direction = base.direction;
        self.priority = base.priority;
    }
}
