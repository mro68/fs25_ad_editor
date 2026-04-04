//! Command-Dispatch fuer Gruppen- und Segment-Operationen.

use crate::app::handlers;
use crate::app::{AppCommand, AppState};

/// Fuehrt Gruppen-Commands aus.
pub(super) fn handle(state: &mut AppState, command: AppCommand) -> anyhow::Result<()> {
    match command {
        AppCommand::EditGroup { record_id } => {
            handlers::editing::edit_group(state, record_id);
            Ok(())
        }
        AppCommand::ToggleGroupLock { segment_id } => {
            handlers::group::toggle_lock(state, segment_id);
            Ok(())
        }
        AppCommand::DissolveGroup { segment_id } => {
            handlers::group::dissolve(state, segment_id);
            Ok(())
        }
        AppCommand::OpenDissolveConfirmDialog { segment_id } => {
            handlers::group::open_dissolve_confirm_dialog(state, segment_id);
            Ok(())
        }
        AppCommand::GroupSelectionAsGroup => {
            handlers::group::group_selection(state);
            Ok(())
        }
        AppCommand::RemoveSelectedNodesFromGroups => {
            handlers::group::remove_selected_from_groups(state);
            Ok(())
        }
        AppCommand::SetGroupBoundaryNodes {
            record_id,
            entry_node_id,
            exit_node_id,
        } => {
            handlers::group::set_boundary_nodes(state, record_id, entry_node_id, exit_node_id);
            Ok(())
        }
        AppCommand::GroupEditStart { record_id } => {
            handlers::group::start_group_edit(state, record_id);
            Ok(())
        }
        AppCommand::GroupEditApply => {
            handlers::group::apply_group_edit(state);
            Ok(())
        }
        AppCommand::GroupEditCancel => {
            handlers::group::cancel_group_edit(state);
            Ok(())
        }
        AppCommand::BeginToolEditFromGroup { record_id } => {
            handlers::group::begin_tool_edit_from_group(state, record_id);
            Ok(())
        }
        AppCommand::OpenGroupSettingsPopup { world_pos } => {
            handlers::group::open_settings_popup(state, world_pos);
            Ok(())
        }
        other => unreachable!("unerwarteter Group-Command: {other:?}"),
    }
}
