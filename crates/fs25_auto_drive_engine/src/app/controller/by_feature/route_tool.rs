//! Command-Dispatch fuer Route-Tool-Interaktionen.

use crate::app::handlers;
use crate::app::{AppCommand, AppState};

/// Fuehrt Route-Tool-Commands aus.
pub(super) fn handle(state: &mut AppState, command: AppCommand) -> anyhow::Result<()> {
    match command {
        AppCommand::RouteToolClick { world_pos, ctrl } => {
            handlers::route_tool::click(state, world_pos, ctrl);
            Ok(())
        }
        AppCommand::RouteToolExecute => {
            handlers::route_tool::execute(state);
            Ok(())
        }
        AppCommand::RouteToolCancel => {
            handlers::route_tool::cancel(state);
            Ok(())
        }
        AppCommand::SelectRouteTool { tool_id } => {
            handlers::route_tool::select(state, tool_id);
            Ok(())
        }
        AppCommand::RouteToolWithAnchors {
            tool_id,
            start_node_id,
            end_node_id,
        } => {
            handlers::route_tool::select_with_anchors(state, tool_id, start_node_id, end_node_id);
            Ok(())
        }
        AppCommand::RouteToolRecreate => {
            handlers::route_tool::recreate(state);
            Ok(())
        }
        AppCommand::RouteToolPanelAction { action } => {
            handlers::route_tool::apply_panel_action(state, action);
            Ok(())
        }
        AppCommand::RouteToolApplyTangent { start, end } => {
            handlers::route_tool::apply_tangent(state, start, end);
            Ok(())
        }
        AppCommand::RouteToolDragStart { world_pos } => {
            handlers::route_tool::drag_start(state, world_pos);
            Ok(())
        }
        AppCommand::RouteToolDragUpdate { world_pos } => {
            handlers::route_tool::drag_update(state, world_pos);
            Ok(())
        }
        AppCommand::RouteToolDragEnd => {
            handlers::route_tool::drag_end(state);
            Ok(())
        }
        AppCommand::RouteToolRotate { delta } => {
            handlers::route_tool::rotate(state, delta);
            Ok(())
        }
        AppCommand::RouteToolLassoCompleted { polygon } => {
            handlers::route_tool::lasso_completed(state, polygon);
            Ok(())
        }
        AppCommand::IncreaseRouteToolNodeCount => {
            handlers::route_tool::increase_node_count(state);
            Ok(())
        }
        AppCommand::DecreaseRouteToolNodeCount => {
            handlers::route_tool::decrease_node_count(state);
            Ok(())
        }
        AppCommand::IncreaseRouteToolSegmentLength => {
            handlers::route_tool::increase_segment_length(state);
            Ok(())
        }
        AppCommand::DecreaseRouteToolSegmentLength => {
            handlers::route_tool::decrease_segment_length(state);
            Ok(())
        }
        other => anyhow::bail!("unerwarteter RouteTool-Command: {other:?}"),
    }
}
