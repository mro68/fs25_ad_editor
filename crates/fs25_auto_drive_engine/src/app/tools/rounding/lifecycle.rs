//! RouteTool-Lifecycle fuer den Arc-Pfad des Verrundungs-Tools.

use super::geometry::{
    collect_transitions, preview_direction, preview_priority, ArcTransition, ArcValidation,
};
use super::state::{RoundingMode, RoundingTool};
use crate::app::tools::common::ToolResultBuilder;
use crate::app::tools::{
    RouteTool, RouteToolCore, RouteToolHostSync, RouteToolSelectionInput, RouteToolSelectionSeed,
    ToolAction, ToolHostContext, ToolPreview, ToolResult,
};
use crate::core::{NodeFlag, RoadMap};
use glam::Vec2;

impl RouteToolCore for RoundingTool {
    fn on_click(&mut self, _pos: Vec2, _road_map: &RoadMap, _ctrl: bool) -> ToolAction {
        ToolAction::Continue
    }

    fn preview(&self, _cursor_pos: Vec2, road_map: &RoadMap) -> ToolPreview {
        match self.mode {
            RoundingMode::ArcOnePoint if self.arc.validation == ArcValidation::Ready => {
                preview_arc(self, road_map)
            }
            _ => ToolPreview::default(),
        }
    }

    fn execute(&self, road_map: &RoadMap) -> Option<ToolResult> {
        match self.mode {
            RoundingMode::ArcOnePoint => execute_arc(self, road_map),
            RoundingMode::QuadraticThreePoint => None,
        }
    }

    fn reset(&mut self) {
        self.reset_runtime_state();
    }

    fn is_ready(&self) -> bool {
        matches!(self.mode, RoundingMode::ArcOnePoint)
            && self.arc.validation == ArcValidation::Ready
    }

    fn has_pending_input(&self) -> bool {
        !self.arc.selected_node_ids.is_empty()
    }
}

impl RouteToolHostSync for RoundingTool {
    fn sync_host(&mut self, context: &ToolHostContext) {
        self.snap_radius = context.snap_radius;
    }
}

impl RouteToolSelectionInput for RoundingTool {
    fn load_selection(&mut self, selection: RouteToolSelectionSeed) {
        self.load_selection_seed(selection);
    }
}

impl RouteTool for RoundingTool {
    fn as_selection_input(&self) -> Option<&dyn RouteToolSelectionInput> {
        Some(self)
    }

    fn as_selection_input_mut(&mut self) -> Option<&mut dyn RouteToolSelectionInput> {
        Some(self)
    }
}

fn preview_arc(tool: &RoundingTool, road_map: &RoadMap) -> ToolPreview {
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
    nodes.push(plan.first_side.neighbor_position);
    nodes.extend(plan.arc_positions.iter().copied());
    nodes.push(plan.second_side.neighbor_position);

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
        || !road_map.contains_node(plan.first_side.neighbor_id)
        || !road_map.contains_node(plan.second_side.neighbor_id)
    {
        return None;
    }

    let transitions = collect_transitions(road_map, corner_id, plan);
    if transitions.is_empty() {
        return None;
    }

    Some(build_arc_tool_result(plan, corner_id, &transitions))
}

fn build_arc_tool_result(
    plan: &super::geometry::ArcPlan,
    corner_id: u64,
    transitions: &[ArcTransition],
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
                plan.first_side.neighbor_id,
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
                plan.second_side.neighbor_id,
                false,
                transition.direction,
                transition.priority,
            ));
        } else {
            external_connections.push((
                last_index,
                plan.second_side.neighbor_id,
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
                plan.first_side.neighbor_id,
                false,
                transition.direction,
                transition.priority,
            ));
        }
    }

    ToolResultBuilder::new(new_nodes, internal_connections)
        .with_external_connections(external_connections)
        .with_nodes_to_remove(vec![corner_id])
        .build()
}
