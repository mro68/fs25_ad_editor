//! Command-Dispatch fuer Selektion und Move/Rotate-Lifecycles.

use crate::app::handlers;
use crate::app::{AppCommand, AppState};

/// Fuehrt Selektions-Commands aus.
pub(super) fn handle(state: &mut AppState, command: AppCommand) -> anyhow::Result<()> {
    match command {
        AppCommand::SelectNearestNode {
            world_pos,
            max_distance,
            additive,
            extend_path,
        } => {
            handlers::group::close_settings_popup(state);
            handlers::selection::select_nearest_node(
                state,
                world_pos,
                max_distance,
                additive,
                extend_path,
            );
            Ok(())
        }
        AppCommand::SelectSegmentBetweenNearestIntersections {
            world_pos,
            max_distance,
            additive,
            stop_at_junction,
            max_angle_deg,
        } => {
            handlers::selection::select_segment(
                state,
                world_pos,
                max_distance,
                additive,
                stop_at_junction,
                max_angle_deg,
            );
            Ok(())
        }
        AppCommand::SelectGroupByNearestNode {
            world_pos,
            max_distance,
            additive,
        } => {
            handlers::selection::select_group_nodes(state, world_pos, max_distance, additive);
            Ok(())
        }
        AppCommand::SelectNodesInRect { min, max, additive } => {
            handlers::selection::select_in_rect(state, min, max, additive);
            Ok(())
        }
        AppCommand::SelectNodesInLasso { polygon, additive } => {
            handlers::selection::select_in_lasso(state, &polygon, additive);
            Ok(())
        }
        AppCommand::MoveSelectedNodes { delta_world } => {
            handlers::selection::move_selected(state, delta_world);
            Ok(())
        }
        AppCommand::BeginMoveSelectedNodes => {
            handlers::selection::begin_move(state);
            Ok(())
        }
        AppCommand::EndMoveSelectedNodes => {
            handlers::selection::end_move(state);
            Ok(())
        }
        AppCommand::BeginRotateSelectedNodes => {
            handlers::selection::begin_rotate(state);
            Ok(())
        }
        AppCommand::RotateSelectedNodes { delta_angle } => {
            handlers::selection::rotate_selected(state, delta_angle);
            Ok(())
        }
        AppCommand::EndRotateSelectedNodes => {
            handlers::selection::end_rotate(state);
            Ok(())
        }
        AppCommand::ClearSelection => {
            handlers::group::close_settings_popup(state);
            handlers::selection::clear(state);
            Ok(())
        }
        AppCommand::SelectAllNodes => {
            handlers::selection::select_all(state);
            Ok(())
        }
        AppCommand::InvertSelection => {
            handlers::selection::invert(state);
            Ok(())
        }
        other => unreachable!("unerwarteter Selection-Command: {other:?}"),
    }
}
