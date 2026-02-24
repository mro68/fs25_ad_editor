//! Mapping von UI-Intents auf mutierende App-Commands.

use super::{AppCommand, AppIntent, AppState};

/// Übersetzt einen `AppIntent` in eine Sequenz ausführbarer `AppCommand`s.
pub fn map_intent_to_commands(state: &AppState, intent: AppIntent) -> Vec<AppCommand> {
    match intent {
        AppIntent::OpenFileRequested => vec![AppCommand::RequestOpenFileDialog],
        AppIntent::SaveRequested => {
            vec![AppCommand::SaveFile { path: None }]
        }
        AppIntent::SaveAsRequested => vec![AppCommand::RequestSaveFileDialog],
        AppIntent::ExitRequested => vec![AppCommand::RequestExit],
        AppIntent::HeightmapSelectionRequested => vec![
            AppCommand::DismissHeightmapWarning,
            AppCommand::RequestHeightmapDialog,
        ],
        AppIntent::BackgroundMapSelectionRequested => vec![AppCommand::RequestBackgroundMapDialog],
        AppIntent::HeightmapCleared => vec![AppCommand::ClearHeightmap],
        AppIntent::HeightmapSelected { path } => vec![AppCommand::SetHeightmap { path }],
        AppIntent::HeightmapWarningConfirmed => vec![
            AppCommand::ConfirmAndSaveFile,
            AppCommand::DismissHeightmapWarning,
        ],
        AppIntent::HeightmapWarningCancelled => vec![AppCommand::DismissHeightmapWarning],
        AppIntent::ResetCameraRequested => vec![AppCommand::ResetCamera],
        AppIntent::ZoomInRequested => vec![AppCommand::ZoomIn],
        AppIntent::ZoomOutRequested => vec![AppCommand::ZoomOut],
        AppIntent::ViewportResized { size } => vec![AppCommand::SetViewportSize { size }],
        AppIntent::CameraPan { delta } => vec![AppCommand::PanCamera { delta }],
        AppIntent::CameraZoom {
            factor,
            focus_world,
        } => vec![AppCommand::ZoomCamera {
            factor,
            focus_world,
        }],
        AppIntent::NodePickRequested {
            world_pos,
            additive,
            extend_path,
        } => {
            let base_max_distance = state.options.hitbox_radius();

            let increased_max_distance = base_max_distance * state.options.selection_size_factor;

            let mut max_distance = base_max_distance;
            if let Some(rm) = state.road_map.as_ref() {
                for id in state.selection.selected_node_ids.iter() {
                    if let Some(node) = rm.nodes.get(id) {
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
        AppIntent::NodeSegmentBetweenIntersectionsRequested {
            world_pos,
            additive,
        } => {
            let max_distance = state.options.hitbox_radius();

            vec![AppCommand::SelectSegmentBetweenNearestIntersections {
                world_pos,
                max_distance,
                additive,
            }]
        }
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
        AppIntent::RenderQualityChanged { quality } => {
            vec![AppCommand::SetRenderQuality { quality }]
        }
        AppIntent::FileSelected { path } => vec![AppCommand::LoadFile { path }],
        AppIntent::SaveFilePathSelected { path } => vec![AppCommand::SaveFile { path: Some(path) }],
        AppIntent::UndoRequested => vec![AppCommand::Undo],
        AppIntent::RedoRequested => vec![AppCommand::Redo],
        AppIntent::SetEditorToolRequested { tool } => vec![AppCommand::SetEditorTool { tool }],
        AppIntent::AddNodeRequested { world_pos } => {
            vec![AppCommand::AddNodeAtPosition { world_pos }]
        }
        AppIntent::DeleteSelectedRequested => vec![AppCommand::DeleteSelectedNodes],
        AppIntent::ConnectToolNodeClicked { world_pos } => {
            let max_distance = state.options.hitbox_radius();
            vec![AppCommand::ConnectToolPickNode {
                world_pos,
                max_distance,
            }]
        }
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
        AppIntent::BackgroundMapSelected { path, crop_size } => {
            vec![AppCommand::LoadBackgroundMap { path, crop_size }]
        }
        AppIntent::SetBackgroundOpacity { opacity } => {
            vec![AppCommand::UpdateBackgroundOpacity { opacity }]
        }
        AppIntent::ToggleBackgroundVisibility => vec![AppCommand::ToggleBackgroundVisibility],
        AppIntent::ScaleBackground { factor } => vec![AppCommand::ScaleBackground { factor }],
        AppIntent::CreateMarkerRequested { node_id } => {
            vec![AppCommand::OpenMarkerDialog {
                node_id,
                is_new: true,
            }]
        }
        AppIntent::RemoveMarkerRequested { node_id } => vec![AppCommand::RemoveMarker { node_id }],
        AppIntent::EditMarkerRequested { node_id } => {
            vec![AppCommand::OpenMarkerDialog {
                node_id,
                is_new: false,
            }]
        }
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
        AppIntent::OpenOptionsDialogRequested => vec![AppCommand::OpenOptionsDialog],
        AppIntent::CloseOptionsDialogRequested => vec![AppCommand::CloseOptionsDialog],
        AppIntent::OptionsChanged { options } => vec![AppCommand::ApplyOptions { options }],
        AppIntent::ResetOptionsRequested => vec![AppCommand::ResetOptions],
        AppIntent::ClearSelectionRequested => vec![AppCommand::ClearSelection],
        AppIntent::SelectAllRequested => vec![AppCommand::SelectAllNodes],
        AppIntent::DeduplicateConfirmed => vec![AppCommand::DeduplicateNodes],
        AppIntent::DeduplicateCancelled => vec![AppCommand::DismissDeduplicateDialog],
        AppIntent::RouteToolClicked { world_pos, ctrl } => {
            vec![AppCommand::RouteToolClick { world_pos, ctrl }]
        }
        AppIntent::RouteToolExecuteRequested => vec![AppCommand::RouteToolExecute],
        AppIntent::RouteToolCancelled => vec![AppCommand::RouteToolCancel],
        AppIntent::SelectRouteToolRequested { index } => {
            vec![AppCommand::SelectRouteTool { index }]
        }
        AppIntent::RouteToolConfigChanged => vec![AppCommand::RouteToolRecreate],
        AppIntent::RouteToolDragStarted { world_pos } => {
            vec![AppCommand::RouteToolDragStart { world_pos }]
        }
        AppIntent::RouteToolDragUpdated { world_pos } => {
            vec![AppCommand::RouteToolDragUpdate { world_pos }]
        }
        AppIntent::RouteToolDragEnded => vec![AppCommand::RouteToolDragEnd],
        AppIntent::EditSegmentRequested { record_id } => {
            vec![AppCommand::EditSegment { record_id }]
        }
        AppIntent::ZipBackgroundBrowseRequested { path } => {
            vec![AppCommand::BrowseZipBackground { path }]
        }
        AppIntent::ZipBackgroundFileSelected {
            zip_path,
            entry_name,
        } => vec![AppCommand::LoadBackgroundFromZip {
            zip_path,
            entry_name,
            crop_size: None,
        }],
        AppIntent::ZipBrowserCancelled => vec![AppCommand::CloseZipBrowser],
        AppIntent::GenerateOverviewRequested => vec![AppCommand::RequestOverviewDialog],
        AppIntent::GenerateOverviewFromZip { path } => {
            vec![AppCommand::OpenOverviewOptionsDialog { path }]
        }
        AppIntent::OverviewOptionsConfirmed => vec![AppCommand::GenerateOverviewWithOptions],
        AppIntent::OverviewOptionsCancelled => vec![AppCommand::CloseOverviewOptionsDialog],
        AppIntent::PostLoadGenerateOverview { zip_path } => {
            vec![
                AppCommand::DismissPostLoadDialog,
                AppCommand::OpenOverviewOptionsDialog { path: zip_path },
            ]
        }
        AppIntent::PostLoadDialogDismissed => vec![AppCommand::DismissPostLoadDialog],
        AppIntent::ResamplePathRequested => vec![AppCommand::ResamplePath],
    }
}

#[cfg(test)]
mod tests;
