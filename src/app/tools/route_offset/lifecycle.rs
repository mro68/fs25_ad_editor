//! Route-Tool-Implementierung fuer das Strecken-Versatz-Tool.

use super::geometry::compute_offset_positions;
use super::state::RouteOffsetTool;
use crate::app::group_registry::{GroupBase, GroupKind, GroupRecord};
use crate::app::tools::common::{
    record_applied_tool_state, sync_tool_host, ToolLifecycleState, ToolResultBuilder,
};
use crate::app::tools::{
    OrderedNodeChain, RouteTool, RouteToolChainInput, RouteToolCore, RouteToolHostSync,
    RouteToolId, RouteToolPanelBridge, RouteToolRecreate, ToolAction, ToolAnchor, ToolHostContext,
    ToolPreview, ToolResult,
};
use crate::app::ui_contract::{RouteToolConfigState, RouteToolPanelAction, RouteToolPanelEffect};
use crate::core::{ConnectionDirection, ConnectionPriority, NodeFlag, RoadMap};
use glam::Vec2;

impl RouteOffsetTool {
    pub(crate) fn load_chain(&mut self, positions: Vec<Vec2>, start_id: u64, end_id: u64) {
        let inferred: Vec<u64> = if positions.len() >= 3
            && end_id > start_id
            && (end_id - start_id) == (positions.len() as u64 - 1)
        {
            (start_id + 1..end_id).collect()
        } else {
            Vec::new()
        };
        self.chain_positions = positions;
        self.chain_start_id = start_id;
        self.chain_end_id = end_id;
        self.chain_inner_ids = inferred;
    }
}

impl RouteToolPanelBridge for RouteOffsetTool {
    fn status_text(&self) -> &str {
        if self.has_chain() {
            "Bereit — Enter zum Ausfuehren, Escape zum Abbrechen"
        } else {
            "Kette selektieren, dann Route-Tool neu aktivieren"
        }
    }

    fn panel_state(&self) -> RouteToolConfigState {
        RouteToolConfigState::RouteOffset(self.panel_state())
    }

    fn apply_panel_action(&mut self, action: RouteToolPanelAction) -> RouteToolPanelEffect {
        let RouteToolPanelAction::RouteOffset(action) = action else {
            return RouteToolPanelEffect::default();
        };

        self.apply_panel_action(action)
    }
}

impl RouteToolCore for RouteOffsetTool {
    fn on_click(&mut self, _pos: Vec2, _road_map: &RoadMap, _ctrl: bool) -> ToolAction {
        ToolAction::Continue
    }

    fn preview(&self, _cursor_pos: Vec2, _road_map: &RoadMap) -> ToolPreview {
        if !self.has_chain() {
            return ToolPreview::default();
        }

        let mut nodes: Vec<Vec2> = Vec::new();
        let mut connections: Vec<(usize, usize)> = Vec::new();
        let mut styles: Vec<(ConnectionDirection, ConnectionPriority)> = Vec::new();

        let orig_start = nodes.len();
        nodes.extend_from_slice(&self.chain_positions);
        for i in 0..self.chain_positions.len().saturating_sub(1) {
            connections.push((orig_start + i, orig_start + i + 1));
            styles.push((self.direction, self.priority));
        }

        if self.config.left_enabled {
            if let Some(pts) = compute_offset_positions(
                &self.chain_positions,
                self.config.left_distance,
                self.config.base_spacing,
            ) {
                let start = nodes.len();
                nodes.extend_from_slice(&pts);
                for i in 0..pts.len().saturating_sub(1) {
                    connections.push((start + i, start + i + 1));
                    styles.push((self.direction, self.priority));
                }
            }
        }

        if self.config.right_enabled {
            if let Some(pts) = compute_offset_positions(
                &self.chain_positions,
                -self.config.right_distance,
                self.config.base_spacing,
            ) {
                let start = nodes.len();
                nodes.extend_from_slice(&pts);
                for i in 0..pts.len().saturating_sub(1) {
                    connections.push((start + i, start + i + 1));
                    styles.push((self.direction, self.priority));
                }
            }
        }

        if nodes.is_empty() {
            return ToolPreview::default();
        }

        ToolPreview {
            nodes,
            connections,
            connection_styles: styles,
            labels: vec![],
        }
    }

    fn execute(&self, _road_map: &RoadMap) -> Option<ToolResult> {
        if !self.has_chain() {
            return None;
        }
        if !self.config.left_enabled && !self.config.right_enabled {
            return None;
        }

        let mut new_nodes: Vec<(Vec2, NodeFlag)> = Vec::new();
        let mut internal_connections: Vec<(usize, usize, ConnectionDirection, ConnectionPriority)> =
            Vec::new();
        let mut external_connections: Vec<(
            usize,
            u64,
            bool,
            ConnectionDirection,
            ConnectionPriority,
        )> = Vec::new();

        let mut add_side = |offset: f32| {
            let Some(pts) =
                compute_offset_positions(&self.chain_positions, offset, self.config.base_spacing)
            else {
                return;
            };
            let base = new_nodes.len();
            for &p in &pts {
                new_nodes.push((p, NodeFlag::Regular));
            }
            for i in 0..pts.len().saturating_sub(1) {
                internal_connections.push((base + i, base + i + 1, self.direction, self.priority));
            }
            let first = base;
            let last = base + pts.len() - 1;
            external_connections.push((
                first,
                self.chain_start_id,
                true,
                self.direction,
                self.priority,
            ));
            external_connections.push((
                last,
                self.chain_end_id,
                false,
                self.direction,
                self.priority,
            ));
        };

        if self.config.left_enabled {
            add_side(self.config.left_distance);
        }
        if self.config.right_enabled {
            add_side(-self.config.right_distance);
        }

        if new_nodes.is_empty() {
            return None;
        }

        let nodes_to_remove = if !self.config.keep_original {
            self.chain_inner_ids.clone()
        } else {
            Vec::new()
        };

        Some(
            ToolResultBuilder::new(new_nodes, internal_connections)
                .with_external_connections(external_connections)
                .with_nodes_to_remove(nodes_to_remove)
                .build(),
        )
    }

    fn reset(&mut self) {
        self.chain_positions.clear();
        self.chain_start_id = 0;
        self.chain_end_id = 0;
        self.chain_inner_ids.clear();
        let snap_radius = self.lifecycle.snap_radius;
        self.lifecycle = ToolLifecycleState::new(snap_radius);
    }

    fn is_ready(&self) -> bool {
        self.has_chain()
    }

    fn has_pending_input(&self) -> bool {
        self.has_chain()
    }
}

impl RouteToolHostSync for RouteOffsetTool {
    fn sync_host(&mut self, context: &ToolHostContext) {
        sync_tool_host(
            &mut self.direction,
            &mut self.priority,
            &mut self.lifecycle,
            context,
        );
    }
}

impl RouteToolRecreate for RouteOffsetTool {
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

impl RouteToolChainInput for RouteOffsetTool {
    fn load_chain(&mut self, chain: OrderedNodeChain) {
        let inner_ids = chain.inner_ids;
        self.load_chain(chain.positions, chain.start_id, chain.end_id);
        if !inner_ids.is_empty() {
            self.chain_inner_ids = inner_ids;
        }
    }
}

impl RouteTool for RouteOffsetTool {
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

    fn make_group_record(&self, id: u64, node_ids: &[u64]) -> Option<GroupRecord> {
        if !self.has_chain() {
            return None;
        }
        let start_pos = *self.chain_positions.first()?;
        let end_pos = *self.chain_positions.last()?;
        Some(GroupRecord {
            id,
            tool_id: Some(RouteToolId::RouteOffset),
            node_ids: node_ids.to_vec(),
            start_anchor: ToolAnchor::ExistingNode(self.chain_start_id, start_pos),
            end_anchor: ToolAnchor::ExistingNode(self.chain_end_id, end_pos),
            original_positions: Vec::new(),
            marker_node_ids: Vec::new(),
            locked: true,
            entry_node_id: None,
            exit_node_id: None,
            kind: GroupKind::RouteOffset {
                chain_positions: self.chain_positions.clone(),
                chain_start_id: self.chain_start_id,
                chain_end_id: self.chain_end_id,
                offset_left: if self.config.left_enabled {
                    self.config.left_distance
                } else {
                    0.0
                },
                offset_right: if self.config.right_enabled {
                    self.config.right_distance
                } else {
                    0.0
                },
                keep_original: self.config.keep_original,
                base_spacing: self.config.base_spacing,
                base: GroupBase {
                    direction: self.direction,
                    priority: self.priority,
                    max_segment_length: self.config.base_spacing,
                },
            },
        })
    }

    fn load_for_edit(&mut self, _record: &GroupRecord, kind: &GroupKind) {
        let GroupKind::RouteOffset {
            chain_positions,
            chain_start_id,
            chain_end_id,
            offset_left,
            offset_right,
            keep_original,
            base_spacing,
            base,
        } = kind
        else {
            return;
        };
        self.load_chain(chain_positions.clone(), *chain_start_id, *chain_end_id);
        self.config.left_enabled = *offset_left > 0.0;
        self.config.right_enabled = *offset_right > 0.0;
        if *offset_left > 0.0 {
            self.config.left_distance = *offset_left;
        }
        if *offset_right > 0.0 {
            self.config.right_distance = *offset_right;
        }
        self.config.keep_original = *keep_original;
        self.config.base_spacing = *base_spacing;
        self.direction = base.direction;
        self.priority = base.priority;
    }
}
