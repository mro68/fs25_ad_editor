//! Intent-Mapping fuer Selektion und Move/Rotate-Lifecycles.

use crate::app::{AppCommand, AppIntent, AppState};

fn map_node_pick(
    state: &AppState,
    world_pos: glam::Vec2,
    additive: bool,
    extend_path: bool,
) -> Vec<AppCommand> {
    let base_max_distance = state.options.hitbox_radius();
    let increased_max_distance = base_max_distance * state.options.selection_size_multiplier();

    let mut max_distance = base_max_distance;
    if let Some(rm) = state.road_map.as_ref() {
        for id in state.selection.selected_node_ids.iter() {
            if let Some(node) = rm.node(*id) {
                if (node.position - world_pos).length() <= increased_max_distance {
                    max_distance = increased_max_distance;
                    break;
                }
            }
        }
    }

    vec![AppCommand::SelectNearestNode {
        world_pos,
        max_distance,
        additive,
        extend_path,
    }]
}

fn map_segment_pick(state: &AppState, world_pos: glam::Vec2, additive: bool) -> Vec<AppCommand> {
    let max_distance = state.options.hitbox_radius();

    if let Some(rm) = state.road_map.as_deref() {
        if let Some(hit) = rm
            .nearest_node(world_pos)
            .filter(|h| h.distance <= max_distance)
        {
            if state
                .group_registry
                .find_first_by_node_id(hit.node_id)
                .is_some()
            {
                return vec![
                    AppCommand::SelectGroupByNearestNode {
                        world_pos,
                        max_distance,
                        additive,
                    },
                    AppCommand::OpenGroupSettingsPopup { world_pos },
                ];
            }
        }
    }

    vec![
        AppCommand::SelectSegmentBetweenNearestIntersections {
            world_pos,
            max_distance,
            additive,
            stop_at_junction: state.options.segment_stop_at_junction,
            max_angle_deg: state.options.segment_max_angle_deg,
        },
        AppCommand::OpenGroupSettingsPopup { world_pos },
    ]
}

/// Mappt Selektions-Intents auf Commands.
pub(super) fn map(state: &AppState, intent: AppIntent) -> Vec<AppCommand> {
    match intent {
        AppIntent::NodePickRequested {
            world_pos,
            additive,
            extend_path,
        } => map_node_pick(state, world_pos, additive, extend_path),
        AppIntent::NodeSegmentBetweenIntersectionsRequested {
            world_pos,
            additive,
        } => map_segment_pick(state, world_pos, additive),
        AppIntent::SelectNodesInRectRequested { min, max, additive } => {
            vec![AppCommand::SelectNodesInRect { min, max, additive }]
        }
        AppIntent::SelectNodesInLassoRequested { polygon, additive } => {
            vec![AppCommand::SelectNodesInLasso { polygon, additive }]
        }
        AppIntent::BeginMoveSelectedNodesRequested => vec![AppCommand::BeginMoveSelectedNodes],
        AppIntent::MoveSelectedNodesRequested { delta_world } => {
            vec![AppCommand::MoveSelectedNodes { delta_world }]
        }
        AppIntent::EndMoveSelectedNodesRequested => vec![AppCommand::EndMoveSelectedNodes],
        AppIntent::BeginRotateSelectedNodesRequested => vec![AppCommand::BeginRotateSelectedNodes],
        AppIntent::RotateSelectedNodesRequested { delta_angle } => {
            vec![AppCommand::RotateSelectedNodes { delta_angle }]
        }
        AppIntent::EndRotateSelectedNodesRequested => vec![AppCommand::EndRotateSelectedNodes],
        AppIntent::ClearSelectionRequested => vec![AppCommand::ClearSelection],
        AppIntent::SelectAllRequested => vec![AppCommand::SelectAllNodes],
        AppIntent::InvertSelectionRequested => vec![AppCommand::InvertSelection],
        other => unreachable!("unerwarteter Selection-Intent: {other:?}"),
    }
}
