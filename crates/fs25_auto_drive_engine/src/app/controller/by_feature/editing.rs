//! Command-Dispatch fuer Editing, Marker, Copy/Paste und Editing-Extras.

use crate::app::handlers;
use crate::app::{AppCommand, AppState};

/// Fuehrt Editing-Commands aus.
pub(super) fn handle(state: &mut AppState, command: AppCommand) -> anyhow::Result<()> {
    match command {
        AppCommand::SetEditorTool { tool } => {
            handlers::editing::set_editor_tool(state, tool);
            Ok(())
        }
        AppCommand::AddNodeAtPosition { world_pos } => {
            handlers::editing::add_node(state, world_pos);
            Ok(())
        }
        AppCommand::DeleteSelectedNodes => {
            handlers::editing::delete_selected(state);
            Ok(())
        }
        AppCommand::ConnectToolPickNode {
            world_pos,
            max_distance,
        } => {
            handlers::editing::connect_tool_pick(state, world_pos, max_distance);
            Ok(())
        }
        AppCommand::AddConnection {
            from_id,
            to_id,
            direction,
            priority,
        } => {
            handlers::editing::add_connection(state, from_id, to_id, direction, priority);
            Ok(())
        }
        AppCommand::RemoveConnectionBetween { node_a, node_b } => {
            handlers::editing::remove_connection_between(state, node_a, node_b);
            Ok(())
        }
        AppCommand::SetConnectionDirection {
            start_id,
            end_id,
            direction,
        } => {
            handlers::editing::set_connection_direction(state, start_id, end_id, direction);
            Ok(())
        }
        AppCommand::SetConnectionPriority {
            start_id,
            end_id,
            priority,
        } => {
            handlers::editing::set_connection_priority(state, start_id, end_id, priority);
            Ok(())
        }
        AppCommand::SetNodeFlag { node_id, flag } => {
            handlers::editing::set_node_flag(state, node_id, flag);
            Ok(())
        }
        AppCommand::SetDefaultDirection { direction } => {
            handlers::editing::set_default_direction(state, direction);
            Ok(())
        }
        AppCommand::SetDefaultPriority { priority } => {
            handlers::editing::set_default_priority(state, priority);
            Ok(())
        }
        AppCommand::SetAllConnectionsDirectionBetweenSelected { direction } => {
            handlers::editing::set_all_directions_between_selected(state, direction);
            Ok(())
        }
        AppCommand::RemoveAllConnectionsBetweenSelected => {
            handlers::editing::remove_all_between_selected(state);
            Ok(())
        }
        AppCommand::InvertAllConnectionsBetweenSelected => {
            handlers::editing::invert_all_between_selected(state);
            Ok(())
        }
        AppCommand::SetAllConnectionsPriorityBetweenSelected { priority } => {
            handlers::editing::set_all_priorities_between_selected(state, priority);
            Ok(())
        }
        AppCommand::ConnectSelectedNodes => {
            handlers::editing::connect_selected(state);
            Ok(())
        }
        AppCommand::CreateMarker {
            node_id,
            name,
            group,
        } => {
            handlers::editing::create_marker(state, node_id, &name, &group);
            Ok(())
        }
        AppCommand::RemoveMarker { node_id } => {
            handlers::editing::remove_marker(state, node_id);
            Ok(())
        }
        AppCommand::OpenMarkerDialog { node_id, is_new } => {
            handlers::editing::open_marker_dialog(state, node_id, is_new);
            Ok(())
        }
        AppCommand::UpdateMarker {
            node_id,
            name,
            group,
        } => {
            handlers::editing::update_marker(state, node_id, &name, &group);
            Ok(())
        }
        AppCommand::ResamplePath => {
            handlers::editing::resample_path(state);
            Ok(())
        }
        AppCommand::StreckenteilungAktivieren => {
            handlers::editing::streckenteilung_aktivieren(state);
            Ok(())
        }
        AppCommand::CopySelection => {
            handlers::editing::copy_selection(state);
            Ok(())
        }
        AppCommand::StartPastePreview => {
            handlers::editing::start_paste_preview(state);
            Ok(())
        }
        AppCommand::UpdatePastePreview { world_pos } => {
            handlers::editing::update_paste_preview(state, world_pos);
            Ok(())
        }
        AppCommand::ConfirmPaste => {
            handlers::editing::confirm_paste(state);
            Ok(())
        }
        AppCommand::CancelPastePreview => {
            handlers::editing::cancel_paste_preview(state);
            Ok(())
        }
        AppCommand::TraceAllFields {
            spacing,
            offset,
            tolerance,
            corner_angle,
            corner_rounding_radius,
            corner_rounding_max_angle_deg,
        } => {
            handlers::dialog::close_trace_all_fields_dialog(state);
            handlers::editing::trace_all_fields(
                state,
                spacing,
                offset,
                tolerance,
                corner_angle,
                corner_rounding_radius,
                corner_rounding_max_angle_deg,
            );
            Ok(())
        }
        AppCommand::ImportCurseplay { path } => {
            handlers::editing::import_curseplay_file(state, &path);
            Ok(())
        }
        AppCommand::ExportCurseplay { path } => {
            handlers::editing::export_curseplay_file(state, &path);
            Ok(())
        }
        other => anyhow::bail!("unerwarteter Editing-Command: {other:?}"),
    }
}
