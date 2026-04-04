//! Intent-Mapping fuer Gruppen- und Segment-Operationen.

use crate::app::{AppCommand, AppIntent, AppState};

/// Mappt Gruppen-Intents auf Commands.
pub(super) fn map(_state: &AppState, intent: AppIntent) -> Vec<AppCommand> {
    match intent {
        AppIntent::EditGroupRequested { record_id } => vec![AppCommand::EditGroup { record_id }],
        AppIntent::GroupEditStartRequested { record_id } => {
            vec![AppCommand::GroupEditStart { record_id }]
        }
        AppIntent::GroupEditApplyRequested => vec![AppCommand::GroupEditApply],
        AppIntent::GroupEditCancelRequested => vec![AppCommand::GroupEditCancel],
        AppIntent::GroupEditToolRequested { record_id } => {
            vec![AppCommand::BeginToolEditFromGroup { record_id }]
        }
        AppIntent::GroupSelectionAsGroupRequested => vec![AppCommand::GroupSelectionAsGroup],
        AppIntent::RemoveSelectedNodesFromGroupRequested => {
            vec![AppCommand::RemoveSelectedNodesFromGroups]
        }
        AppIntent::SetGroupBoundaryNodes {
            record_id,
            entry_node_id,
            exit_node_id,
        } => vec![AppCommand::SetGroupBoundaryNodes {
            record_id,
            entry_node_id,
            exit_node_id,
        }],
        AppIntent::ToggleGroupLockRequested { segment_id } => {
            vec![AppCommand::ToggleGroupLock { segment_id }]
        }
        AppIntent::DissolveGroupRequested { segment_id } => {
            vec![AppCommand::OpenDissolveConfirmDialog { segment_id }]
        }
        AppIntent::DissolveGroupConfirmed { segment_id } => {
            vec![AppCommand::DissolveGroup { segment_id }]
        }
        other => unreachable!("unerwarteter Group-Intent: {other:?}"),
    }
}
