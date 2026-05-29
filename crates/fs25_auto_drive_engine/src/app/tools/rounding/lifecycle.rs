//! RouteTool-Lifecycle fuer das Arc-only-Verrundungs-Tool.

use super::geometry::{
    arc_replace_path, build_arc_plan_from_payload, collect_transitions, ArcPlan, ArcTransition,
    ArcValidation,
};
use super::state::{clamp_arc_max_angle_deg, ArcOnePointState, RoundingTool};
use crate::app::tool_editing::{RoundingTransitionSnapshot, RouteToolEditPayload};
use crate::app::tools::common::ToolResultBuilder;
use crate::app::tools::{
    RouteTool, RouteToolCore, RouteToolGroupEdit, RouteToolHostSync, RouteToolRecreate,
    RouteToolSelectionInput, RouteToolSelectionSeed, ToolAction, ToolAnchor, ToolHostContext,
    ToolPreview, ToolResult,
};
use crate::core::{ConnectionDirection, ConnectionPriority, NodeFlag, RoadMap};
use glam::Vec2;

impl RouteToolCore for RoundingTool {
    fn on_click(&mut self, _pos: Vec2, _road_map: &RoadMap, _ctrl: bool) -> ToolAction {
        ToolAction::Continue
    }

    fn preview(&self, _cursor_pos: Vec2, road_map: &RoadMap) -> ToolPreview {
        if self.arc.validation == ArcValidation::Ready {
            preview_arc(self, road_map)
        } else if self.lifecycle.restored_for_edit {
            preview_arc_from_payload(self, road_map)
        } else {
            ToolPreview::default()
        }
    }

    fn execute(&self, road_map: &RoadMap) -> Option<ToolResult> {
        execute_arc(self, road_map).or_else(|| {
            self.lifecycle
                .restored_for_edit
                .then(|| execute_from_payload(self, road_map))
                .flatten()
        })
    }

    fn reset(&mut self) {
        self.reset_runtime_state();
    }

    fn is_ready(&self) -> bool {
        if self.lifecycle.restored_for_edit && self.has_restored_payload_for_active_mode() {
            return true;
        }

        self.arc.validation == ArcValidation::Ready
    }

    fn has_pending_input(&self) -> bool {
        if self.lifecycle.restored_for_edit && self.has_restored_payload_for_active_mode() {
            return true;
        }

        !self.arc.selected_node_ids.is_empty()
    }
}

impl RouteToolHostSync for RoundingTool {
    fn sync_host(&mut self, context: &ToolHostContext) {
        self.snap_radius = context.snap_radius;
    }
}

impl RouteToolRecreate for RoundingTool {
    fn on_applied(&mut self, ids: &[u64], road_map: &RoadMap) {
        self.lifecycle.last_created_ids.clear();
        self.lifecycle.last_created_ids.extend_from_slice(ids);
        self.lifecycle.recreate_needed = false;
        self.lifecycle.restored_for_edit = false;
        self.lifecycle.edit_payload = build_runtime_edit_payload(self, road_map)
            .or_else(|| self.lifecycle.edit_payload.clone());
    }

    fn last_created_ids(&self) -> &[u64] {
        &self.lifecycle.last_created_ids
    }

    fn last_end_anchor(&self) -> Option<ToolAnchor> {
        None
    }

    fn needs_recreate(&self) -> bool {
        self.lifecycle.recreate_needed
    }

    fn clear_recreate_flag(&mut self) {
        self.lifecycle.recreate_needed = false;
    }

    fn execute_from_anchors(&self, road_map: &RoadMap) -> Option<ToolResult> {
        execute_from_payload(self, road_map)
    }
}

impl RouteToolSelectionInput for RoundingTool {
    fn load_selection(&mut self, selection: RouteToolSelectionSeed) {
        self.load_selection_seed(selection);
    }
}

impl RouteTool for RoundingTool {
    fn as_recreate(&self) -> Option<&dyn RouteToolRecreate> {
        Some(self)
    }

    fn as_recreate_mut(&mut self) -> Option<&mut dyn RouteToolRecreate> {
        Some(self)
    }

    fn as_selection_input(&self) -> Option<&dyn RouteToolSelectionInput> {
        Some(self)
    }

    fn as_selection_input_mut(&mut self) -> Option<&mut dyn RouteToolSelectionInput> {
        Some(self)
    }

    fn as_group_edit(&self) -> Option<&dyn RouteToolGroupEdit> {
        Some(self)
    }

    fn as_group_edit_mut(&mut self) -> Option<&mut dyn RouteToolGroupEdit> {
        Some(self)
    }
}

impl RouteToolGroupEdit for RoundingTool {
    fn build_edit_payload(&self) -> Option<RouteToolEditPayload> {
        self.lifecycle.edit_payload.clone()
    }

    fn restore_edit_payload(&mut self, payload: &RouteToolEditPayload) {
        match payload {
            RouteToolEditPayload::RoundingArc {
                radius_m,
                max_angle_deg,
                ..
            } => {
                self.arc = ArcOnePointState {
                    radius_m: *radius_m,
                    max_angle_deg: clamp_arc_max_angle_deg(*max_angle_deg),
                    ..ArcOnePointState::default()
                };
            }
            _ => return,
        }

        self.lifecycle.last_created_ids.clear();
        self.lifecycle.recreate_needed = false;
        self.lifecycle.edit_payload = Some(payload.clone());
        self.lifecycle.restored_for_edit = true;
    }
}

fn preview_arc(tool: &RoundingTool, road_map: &RoadMap) -> ToolPreview {
    if tool.arc.validation != ArcValidation::Ready {
        return ToolPreview::default();
    }

    let Some(corner_id) = tool.arc.corner_node_id else {
        return ToolPreview::default();
    };
    let Some(plan) = &tool.arc.plan else {
        return ToolPreview::default();
    };

    let transitions = collect_transitions(road_map, corner_id, plan);
    if transitions.is_empty() {
        return ToolPreview::default();
    }

    let mut nodes = Vec::with_capacity(plan.arc_positions.len() + 2);
    nodes.push(plan.first_side.anchor_position);
    nodes.extend(plan.arc_positions.iter().copied());
    nodes.push(plan.second_side.anchor_position);

    ToolPreview::from_polyline(
        nodes,
        preview_direction(&transitions),
        preview_priority(&transitions),
    )
}

fn preview_arc_from_payload(tool: &RoundingTool, road_map: &RoadMap) -> ToolPreview {
    let Some(RouteToolEditPayload::RoundingArc {
        first_anchor_id,
        second_anchor_id,
        corner_position,
        radius_m,
        max_angle_deg,
        transitions,
    }) = tool.lifecycle.edit_payload.as_ref()
    else {
        return ToolPreview::default();
    };

    let Some(first_position) = road_map.node_position(*first_anchor_id) else {
        return ToolPreview::default();
    };
    let Some(second_position) = road_map.node_position(*second_anchor_id) else {
        return ToolPreview::default();
    };
    let Some(plan) = build_arc_plan_from_payload(
        *corner_position,
        *first_anchor_id,
        first_position,
        *second_anchor_id,
        second_position,
        *radius_m,
        *max_angle_deg,
    ) else {
        return ToolPreview::default();
    };

    let transitions = snapshot_transitions_to_runtime(transitions);
    if transitions.is_empty() {
        return ToolPreview::default();
    }

    let mut nodes = Vec::with_capacity(plan.arc_positions.len() + 2);
    nodes.push(first_position);
    nodes.extend(plan.arc_positions.iter().copied());
    nodes.push(second_position);

    ToolPreview::from_polyline(
        nodes,
        preview_direction(&transitions),
        preview_priority(&transitions),
    )
}

fn execute_arc(tool: &RoundingTool, road_map: &RoadMap) -> Option<ToolResult> {
    if tool.arc.validation != ArcValidation::Ready {
        return None;
    }

    let corner_id = tool.arc.corner_node_id?;
    let plan = tool.arc.plan.as_ref()?;
    if !road_map.contains_node(corner_id)
        || !road_map.contains_node(plan.first_side.anchor_id)
        || !road_map.contains_node(plan.second_side.anchor_id)
    {
        return None;
    }

    let transitions = collect_transitions(road_map, corner_id, plan);
    if transitions.is_empty() {
        return None;
    }

    Some(build_arc_tool_result(plan, corner_id, &transitions))
}

fn execute_from_payload(tool: &RoundingTool, road_map: &RoadMap) -> Option<ToolResult> {
    match tool.lifecycle.edit_payload.as_ref()? {
        RouteToolEditPayload::RoundingArc {
            first_anchor_id,
            second_anchor_id,
            corner_position,
            radius_m,
            max_angle_deg,
            transitions,
        } => {
            let first_position = road_map.node_position(*first_anchor_id)?;
            let second_position = road_map.node_position(*second_anchor_id)?;
            let plan = build_arc_plan_from_payload(
                *corner_position,
                *first_anchor_id,
                first_position,
                *second_anchor_id,
                second_position,
                *radius_m,
                *max_angle_deg,
            )?;

            Some(build_arc_tool_result_from_transitions(
                &plan,
                transitions,
                Vec::new(),
            ))
        }
        _ => None,
    }
}

fn build_arc_tool_result(
    plan: &ArcPlan,
    corner_id: u64,
    transitions: &[ArcTransition],
) -> ToolResult {
    let snapshots: Vec<RoundingTransitionSnapshot> = transitions
        .iter()
        .map(|transition| RoundingTransitionSnapshot {
            forward: transition.forward,
            direction: transition.direction,
            priority: transition.priority,
        })
        .collect();

    let replace_path = arc_replace_path(corner_id, plan);
    let nodes_to_remove = replace_path
        .get(1..replace_path.len().saturating_sub(1))
        .map_or_else(Vec::new, |inner| inner.to_vec());

    build_arc_tool_result_from_transitions(plan, &snapshots, nodes_to_remove)
}

fn build_arc_tool_result_from_transitions(
    plan: &ArcPlan,
    transitions: &[RoundingTransitionSnapshot],
    nodes_to_remove: Vec<u64>,
) -> ToolResult {
    let new_nodes: Vec<(Vec2, NodeFlag)> = plan
        .arc_positions
        .iter()
        .copied()
        .map(|position| (position, NodeFlag::RoundedCorner))
        .collect();

    let last_index = new_nodes.len().saturating_sub(1);
    let mut internal_connections = Vec::new();
    let mut external_connections = Vec::new();

    for transition in transitions {
        if transition.forward {
            external_connections.push((
                0,
                plan.first_side.anchor_id,
                true,
                transition.direction,
                transition.priority,
            ));
            for index in 0..last_index {
                internal_connections.push((
                    index,
                    index + 1,
                    transition.direction,
                    transition.priority,
                ));
            }
            external_connections.push((
                last_index,
                plan.second_side.anchor_id,
                false,
                transition.direction,
                transition.priority,
            ));
        } else {
            external_connections.push((
                last_index,
                plan.second_side.anchor_id,
                true,
                transition.direction,
                transition.priority,
            ));
            for index in (1..=last_index).rev() {
                internal_connections.push((
                    index,
                    index - 1,
                    transition.direction,
                    transition.priority,
                ));
            }
            external_connections.push((
                0,
                plan.first_side.anchor_id,
                false,
                transition.direction,
                transition.priority,
            ));
        }
    }

    ToolResultBuilder::new(new_nodes, internal_connections)
        .with_external_connections(external_connections)
        .with_source_group_node_ids(nodes_to_remove.clone())
        .with_nodes_to_remove(nodes_to_remove)
        .build()
}

fn build_runtime_edit_payload(
    tool: &RoundingTool,
    road_map: &RoadMap,
) -> Option<RouteToolEditPayload> {
    let corner_position = tool.arc.corner_position?;
    let plan = tool.arc.plan.as_ref()?;
    let corner_id = tool.arc.corner_node_id?;
    let transitions = collect_transitions(road_map, corner_id, plan);
    if transitions.is_empty() {
        return None;
    }

    Some(RouteToolEditPayload::RoundingArc {
        first_anchor_id: plan.first_side.anchor_id,
        second_anchor_id: plan.second_side.anchor_id,
        corner_position,
        radius_m: tool.arc.radius_m,
        max_angle_deg: tool.arc.max_angle_deg,
        transitions: transitions
            .into_iter()
            .map(|transition| RoundingTransitionSnapshot {
                forward: transition.forward,
                direction: transition.direction,
                priority: transition.priority,
            })
            .collect(),
    })
}

fn snapshot_transitions_to_runtime(
    transitions: &[RoundingTransitionSnapshot],
) -> Vec<ArcTransition> {
    transitions
        .iter()
        .map(|transition| ArcTransition {
            forward: transition.forward,
            direction: transition.direction,
            priority: transition.priority,
        })
        .collect()
}

fn preview_direction(transitions: &[ArcTransition]) -> ConnectionDirection {
    let has_forward = transitions.iter().any(|transition| transition.forward);
    let has_reverse = transitions.iter().any(|transition| !transition.forward);
    match (has_forward, has_reverse) {
        (true, true) => ConnectionDirection::Dual,
        (true, false) => transitions
            .iter()
            .find(|transition| transition.forward)
            .map(|transition| transition.direction)
            .unwrap_or(ConnectionDirection::Regular),
        (false, true) => transitions
            .iter()
            .find(|transition| !transition.forward)
            .map(|transition| transition.direction)
            .unwrap_or(ConnectionDirection::Reverse),
        (false, false) => ConnectionDirection::Regular,
    }
}

fn preview_priority(transitions: &[ArcTransition]) -> ConnectionPriority {
    if transitions
        .iter()
        .all(|transition| transition.priority == ConnectionPriority::SubPriority)
    {
        ConnectionPriority::SubPriority
    } else {
        ConnectionPriority::Regular
    }
}
