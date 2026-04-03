//! Intent-Mapping fuer Route-Tool-Interaktionen.

use crate::app::{AppCommand, AppIntent, AppState};

/// Mappt Route-Tool-Intents auf Commands.
pub(super) fn map(_state: &AppState, intent: AppIntent) -> Vec<AppCommand> {
    match intent {
        AppIntent::RouteToolClicked { world_pos, ctrl } => {
            vec![AppCommand::RouteToolClick { world_pos, ctrl }]
        }
        AppIntent::RouteToolExecuteRequested => vec![AppCommand::RouteToolExecute],
        AppIntent::RouteToolCancelled => vec![AppCommand::RouteToolCancel],
        AppIntent::SelectRouteToolRequested { tool_id } => {
            vec![AppCommand::SelectRouteTool { tool_id }]
        }
        AppIntent::RouteToolWithAnchorsRequested {
            tool_id,
            start_node_id,
            end_node_id,
        } => vec![AppCommand::RouteToolWithAnchors {
            tool_id,
            start_node_id,
            end_node_id,
        }],
        AppIntent::RouteToolConfigChanged | AppIntent::RouteToolRecreateRequested => {
            vec![AppCommand::RouteToolRecreate]
        }
        AppIntent::RouteToolPanelActionRequested { action } => {
            vec![AppCommand::RouteToolPanelAction { action }]
        }
        AppIntent::RouteToolTangentSelected { start, end } => {
            vec![AppCommand::RouteToolApplyTangent { start, end }]
        }
        AppIntent::RouteToolLassoCompleted { polygon } => {
            vec![AppCommand::RouteToolLassoCompleted { polygon }]
        }
        AppIntent::RouteToolDragStarted { world_pos } => {
            vec![AppCommand::RouteToolDragStart { world_pos }]
        }
        AppIntent::RouteToolDragUpdated { world_pos } => {
            vec![AppCommand::RouteToolDragUpdate { world_pos }]
        }
        AppIntent::RouteToolDragEnded => vec![AppCommand::RouteToolDragEnd],
        AppIntent::RouteToolScrollRotated { delta } => vec![AppCommand::RouteToolRotate { delta }],
        AppIntent::IncreaseRouteToolNodeCount => vec![AppCommand::IncreaseRouteToolNodeCount],
        AppIntent::DecreaseRouteToolNodeCount => vec![AppCommand::DecreaseRouteToolNodeCount],
        AppIntent::IncreaseRouteToolSegmentLength => {
            vec![AppCommand::IncreaseRouteToolSegmentLength]
        }
        AppIntent::DecreaseRouteToolSegmentLength => {
            vec![AppCommand::DecreaseRouteToolSegmentLength]
        }
        other => unreachable!("unerwarteter RouteTool-Intent: {other:?}"),
    }
}
