//! Intent-Mapping fuer Editing, Marker, Copy/Paste und Editing-Extras.

use crate::app::{AppCommand, AppIntent, AppState};

/// Mappt Editing-Intents auf Commands.
pub(super) fn map(state: &AppState, intent: AppIntent) -> Vec<AppCommand> {
    match intent {
        AppIntent::SetEditorToolRequested { tool } => vec![AppCommand::SetEditorTool { tool }],
        AppIntent::AddNodeRequested { world_pos } => {
            vec![AppCommand::AddNodeAtPosition { world_pos }]
        }
        AppIntent::DeleteSelectedRequested => vec![AppCommand::DeleteSelectedNodes],
        AppIntent::ConnectToolNodeClicked { world_pos } => vec![AppCommand::ConnectToolPickNode {
            world_pos,
            max_distance: state.options.hitbox_radius(),
        }],
        AppIntent::AddConnectionRequested {
            from_id,
            to_id,
            direction,
            priority,
        } => vec![AppCommand::AddConnection {
            from_id,
            to_id,
            direction,
            priority,
        }],
        AppIntent::RemoveConnectionBetweenRequested { node_a, node_b } => {
            vec![AppCommand::RemoveConnectionBetween { node_a, node_b }]
        }
        AppIntent::SetConnectionDirectionRequested {
            start_id,
            end_id,
            direction,
        } => vec![AppCommand::SetConnectionDirection {
            start_id,
            end_id,
            direction,
        }],
        AppIntent::SetConnectionPriorityRequested {
            start_id,
            end_id,
            priority,
        } => vec![AppCommand::SetConnectionPriority {
            start_id,
            end_id,
            priority,
        }],
        AppIntent::NodeFlagChangeRequested { node_id, flag } => {
            vec![AppCommand::SetNodeFlag { node_id, flag }]
        }
        AppIntent::SetDefaultDirectionRequested { direction } => {
            vec![AppCommand::SetDefaultDirection { direction }]
        }
        AppIntent::SetDefaultPriorityRequested { priority } => {
            vec![AppCommand::SetDefaultPriority { priority }]
        }
        AppIntent::SetAllConnectionsDirectionBetweenSelectedRequested { direction } => {
            vec![AppCommand::SetAllConnectionsDirectionBetweenSelected { direction }]
        }
        AppIntent::RemoveAllConnectionsBetweenSelectedRequested => {
            vec![AppCommand::RemoveAllConnectionsBetweenSelected]
        }
        AppIntent::InvertAllConnectionsBetweenSelectedRequested => {
            vec![AppCommand::InvertAllConnectionsBetweenSelected]
        }
        AppIntent::SetAllConnectionsPriorityBetweenSelectedRequested { priority } => {
            vec![AppCommand::SetAllConnectionsPriorityBetweenSelected { priority }]
        }
        AppIntent::ConnectSelectedNodesRequested => vec![AppCommand::ConnectSelectedNodes],
        AppIntent::CreateMarkerRequested { node_id } => vec![AppCommand::OpenMarkerDialog {
            node_id,
            is_new: true,
        }],
        AppIntent::RemoveMarkerRequested { node_id } => vec![AppCommand::RemoveMarker { node_id }],
        AppIntent::EditMarkerRequested { node_id } => vec![AppCommand::OpenMarkerDialog {
            node_id,
            is_new: false,
        }],
        AppIntent::MarkerDialogConfirmed {
            node_id,
            name,
            group,
            is_new,
        } => {
            if is_new {
                vec![
                    AppCommand::CreateMarker {
                        node_id,
                        name,
                        group,
                    },
                    AppCommand::CloseMarkerDialog,
                ]
            } else {
                vec![
                    AppCommand::UpdateMarker {
                        node_id,
                        name,
                        group,
                    },
                    AppCommand::CloseMarkerDialog,
                ]
            }
        }
        AppIntent::MarkerDialogCancelled => vec![AppCommand::CloseMarkerDialog],
        AppIntent::ResamplePathRequested => vec![AppCommand::ResamplePath],
        AppIntent::StreckenteilungAktivieren => vec![AppCommand::StreckenteilungAktivieren],
        AppIntent::CopySelectionRequested => vec![AppCommand::CopySelection],
        AppIntent::PasteStartRequested => vec![AppCommand::StartPastePreview],
        AppIntent::PastePreviewMoved { world_pos } => {
            vec![AppCommand::UpdatePastePreview { world_pos }]
        }
        AppIntent::PasteConfirmRequested => vec![AppCommand::ConfirmPaste],
        AppIntent::PasteCancelled => vec![AppCommand::CancelPastePreview],
        AppIntent::OpenTraceAllFieldsDialogRequested => vec![AppCommand::OpenTraceAllFieldsDialog],
        AppIntent::TraceAllFieldsConfirmed {
            spacing,
            offset,
            tolerance,
            corner_angle,
            corner_rounding_radius,
            corner_rounding_max_angle_deg,
        } => vec![AppCommand::TraceAllFields {
            spacing,
            offset,
            tolerance,
            corner_angle,
            corner_rounding_radius,
            corner_rounding_max_angle_deg,
        }],
        AppIntent::TraceAllFieldsCancelled => vec![AppCommand::CloseTraceAllFieldsDialog],
        AppIntent::CurseplayImportRequested => vec![AppCommand::RequestCurseplayImportDialog],
        AppIntent::CurseplayExportRequested => vec![AppCommand::RequestCurseplayExportDialog],
        AppIntent::CurseplayFileSelected { path } => vec![AppCommand::ImportCurseplay { path }],
        AppIntent::CurseplayExportPathSelected { path } => {
            vec![AppCommand::ExportCurseplay { path }]
        }
        other => unreachable!("unerwarteter Editing-Intent: {other:?}"),
    }
}
