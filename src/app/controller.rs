//! Application Controller für zentrale Event-Verarbeitung.

use super::render_scene;
use super::{AppCommand, AppIntent, AppState};
use crate::shared::RenderScene;

/// Orchestriert UI-Events und Use-Cases auf den AppState.
#[derive(Default)]
pub struct AppController;

impl AppController {
    /// Erstellt einen neuen Controller.
    pub fn new() -> Self {
        Self
    }

    /// Verarbeitet einen Intent über Intent->Command Mapping.
    pub fn handle_intent(&mut self, state: &mut AppState, intent: AppIntent) -> anyhow::Result<()> {
        let commands = self.map_intent_to_commands(state, intent);
        for command in commands {
            self.handle_command(state, command)?;
        }

        Ok(())
    }

    fn map_intent_to_commands(&self, state: &AppState, intent: AppIntent) -> Vec<AppCommand> {
        match intent {
            AppIntent::OpenFileRequested => vec![AppCommand::RequestOpenFileDialog],
            AppIntent::SaveRequested => {
                // Save nutzt aktuellen Pfad oder öffnet Dialog
                vec![AppCommand::SaveFile {
                    path: String::new(),
                }]
            }
            AppIntent::SaveAsRequested => vec![AppCommand::RequestSaveFileDialog],
            AppIntent::ExitRequested => vec![AppCommand::RequestExit],
            AppIntent::HeightmapSelectionRequested => vec![
                AppCommand::DismissHeightmapWarning,
                AppCommand::RequestHeightmapDialog,
            ],
            AppIntent::BackgroundMapSelectionRequested => {
                vec![AppCommand::RequestBackgroundMapDialog]
            }
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
                let base_max_distance = state.view.camera.pick_radius_world_scaled(
                    state.view.viewport_size[1],
                    state.options.selection_pick_radius_px,
                );

                // Für bereits selektierte Nodes erweitern wir das Click‑Fenster
                let increased_max_distance =
                    base_max_distance * state.options.selection_size_factor;

                // Verwende erhöhten Schwellwert *nur*, wenn sich ein selektierter Node innerhalb
                // dieses erweiterten Abstands befindet (vermeidet globale Änderung des Picks).
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
                let max_distance = state.view.camera.pick_radius_world_scaled(
                    state.view.viewport_size[1],
                    state.options.selection_pick_radius_px,
                );

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
            AppIntent::SaveFilePathSelected { path } => vec![AppCommand::SaveFile { path }],
            AppIntent::UndoRequested => vec![AppCommand::Undo],
            AppIntent::RedoRequested => vec![AppCommand::Redo],
            AppIntent::SetEditorToolRequested { tool } => vec![AppCommand::SetEditorTool { tool }],
            AppIntent::AddNodeRequested { world_pos } => {
                vec![AppCommand::AddNodeAtPosition { world_pos }]
            }
            AppIntent::DeleteSelectedRequested => vec![AppCommand::DeleteSelectedNodes],
            AppIntent::ConnectToolNodeClicked { world_pos } => {
                let max_distance = state.view.camera.pick_radius_world_scaled(
                    state.view.viewport_size[1],
                    state.options.selection_pick_radius_px,
                );
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
            } => {
                vec![AppCommand::AddConnection {
                    from_id,
                    to_id,
                    direction,
                    priority,
                }]
            }
            AppIntent::RemoveConnectionBetweenRequested { node_a, node_b } => {
                vec![AppCommand::RemoveConnectionBetween { node_a, node_b }]
            }
            AppIntent::SetConnectionDirectionRequested {
                start_id,
                end_id,
                direction,
            } => {
                vec![AppCommand::SetConnectionDirection {
                    start_id,
                    end_id,
                    direction,
                }]
            }
            AppIntent::SetConnectionPriorityRequested {
                start_id,
                end_id,
                priority,
            } => {
                vec![AppCommand::SetConnectionPriority {
                    start_id,
                    end_id,
                    priority,
                }]
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
            AppIntent::ConnectSelectedNodesRequested => {
                vec![AppCommand::ConnectSelectedNodes]
            }
            AppIntent::BackgroundMapSelected { path, crop_size } => {
                vec![AppCommand::LoadBackgroundMap { path, crop_size }]
            }
            AppIntent::SetBackgroundOpacity { opacity } => {
                vec![AppCommand::UpdateBackgroundOpacity { opacity }]
            }
            AppIntent::ToggleBackgroundVisibility => {
                vec![AppCommand::ToggleBackgroundVisibility]
            }
            AppIntent::CreateMarkerRequested { node_id } => {
                vec![AppCommand::OpenMarkerDialog {
                    node_id,
                    is_new: true,
                }]
            }
            AppIntent::RemoveMarkerRequested { node_id } => {
                vec![AppCommand::RemoveMarker { node_id }]
            }
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
            AppIntent::MarkerDialogCancelled => {
                vec![AppCommand::CloseMarkerDialog]
            }
            AppIntent::OpenOptionsDialogRequested => {
                vec![AppCommand::OpenOptionsDialog]
            }
            AppIntent::CloseOptionsDialogRequested => {
                vec![AppCommand::CloseOptionsDialog]
            }
            AppIntent::OptionsChanged { options } => {
                vec![AppCommand::ApplyOptions { options }]
            }
            AppIntent::ResetOptionsRequested => {
                vec![AppCommand::ResetOptions]
            }
            AppIntent::ClearSelectionRequested => {
                vec![AppCommand::ClearSelection]
            }
            AppIntent::SelectAllRequested => {
                vec![AppCommand::SelectAllNodes]
            }
            AppIntent::DeduplicateConfirmed => {
                vec![AppCommand::DeduplicateNodes]
            }
            AppIntent::DeduplicateCancelled => {
                vec![AppCommand::DismissDeduplicateDialog]
            }
            AppIntent::RouteToolClicked { world_pos, ctrl } => {
                vec![AppCommand::RouteToolClick { world_pos, ctrl }]
            }
            AppIntent::RouteToolExecuteRequested => {
                vec![AppCommand::RouteToolExecute]
            }
            AppIntent::RouteToolCancelled => {
                vec![AppCommand::RouteToolCancel]
            }
            AppIntent::SelectRouteToolRequested { index } => {
                vec![AppCommand::SelectRouteTool { index }]
            }
            AppIntent::RouteToolConfigChanged => {
                vec![AppCommand::RouteToolRecreate]
            }
        }
    }

    /// Führt mutierende Commands auf dem AppState aus.
    /// Dispatcht an Feature-Handler in `handlers/`.
    pub fn handle_command(
        &mut self,
        state: &mut AppState,
        command: AppCommand,
    ) -> anyhow::Result<()> {
        let executed_command = command.clone();
        use super::handlers;

        match command {
            // === Datei-I/O ===
            AppCommand::RequestOpenFileDialog => handlers::file_io::request_open(state),
            AppCommand::RequestSaveFileDialog => handlers::file_io::request_save(state),
            AppCommand::ConfirmAndSaveFile => handlers::file_io::confirm_and_save(state)?,
            AppCommand::LoadFile { path } => handlers::file_io::load(state, path)?,
            AppCommand::SaveFile { path } => handlers::file_io::save(state, path)?,
            AppCommand::ClearHeightmap => handlers::file_io::clear_heightmap(state),
            AppCommand::SetHeightmap { path } => handlers::file_io::set_heightmap(state, path),
            AppCommand::DeduplicateNodes => handlers::file_io::deduplicate(state),

            // === Kamera & Viewport ===
            AppCommand::ResetCamera => handlers::view::reset_camera(state),
            AppCommand::ZoomIn => handlers::view::zoom_in(state),
            AppCommand::ZoomOut => handlers::view::zoom_out(state),
            AppCommand::SetViewportSize { size } => handlers::view::set_viewport_size(state, size),
            AppCommand::PanCamera { delta } => handlers::view::pan(state, delta),
            AppCommand::ZoomCamera {
                factor,
                focus_world,
            } => handlers::view::zoom_towards(state, factor, focus_world),
            AppCommand::SetRenderQuality { quality } => {
                handlers::view::set_render_quality(state, quality)
            }
            AppCommand::LoadBackgroundMap { path, crop_size } => {
                handlers::view::load_background_map(state, path, crop_size)?
            }
            AppCommand::UpdateBackgroundOpacity { opacity } => {
                handlers::view::set_background_opacity(state, opacity)
            }
            AppCommand::ToggleBackgroundVisibility => {
                handlers::view::toggle_background_visibility(state)
            }

            // === Selektion ===
            AppCommand::SelectNearestNode {
                world_pos,
                max_distance,
                additive,
                extend_path,
            } => handlers::selection::select_nearest_node(
                state,
                world_pos,
                max_distance,
                additive,
                extend_path,
            ),
            AppCommand::SelectSegmentBetweenNearestIntersections {
                world_pos,
                max_distance,
                additive,
            } => handlers::selection::select_segment(state, world_pos, max_distance, additive),
            AppCommand::SelectNodesInRect { min, max, additive } => {
                handlers::selection::select_in_rect(state, min, max, additive)
            }
            AppCommand::SelectNodesInLasso { polygon, additive } => {
                handlers::selection::select_in_lasso(state, &polygon, additive)
            }
            AppCommand::MoveSelectedNodes { delta_world } => {
                handlers::selection::move_selected(state, delta_world)
            }
            AppCommand::BeginMoveSelectedNodes => handlers::selection::begin_move(state),
            AppCommand::EndMoveSelectedNodes => { /* No-op: Move-Lifecycle Ende */ }
            AppCommand::ClearSelection => handlers::selection::clear(state),
            AppCommand::SelectAllNodes => handlers::selection::select_all(state),

            // === Editing ===
            AppCommand::SetEditorTool { tool } => handlers::editing::set_editor_tool(state, tool),
            AppCommand::AddNodeAtPosition { world_pos } => {
                handlers::editing::add_node(state, world_pos)
            }
            AppCommand::DeleteSelectedNodes => handlers::editing::delete_selected(state),
            AppCommand::ConnectToolPickNode {
                world_pos,
                max_distance,
            } => handlers::editing::connect_tool_pick(state, world_pos, max_distance),
            AppCommand::AddConnection {
                from_id,
                to_id,
                direction,
                priority,
            } => handlers::editing::add_connection(state, from_id, to_id, direction, priority),
            AppCommand::RemoveConnectionBetween { node_a, node_b } => {
                handlers::editing::remove_connection_between(state, node_a, node_b)
            }
            AppCommand::SetConnectionDirection {
                start_id,
                end_id,
                direction,
            } => handlers::editing::set_connection_direction(state, start_id, end_id, direction),
            AppCommand::SetConnectionPriority {
                start_id,
                end_id,
                priority,
            } => handlers::editing::set_connection_priority(state, start_id, end_id, priority),
            AppCommand::SetDefaultDirection { direction } => {
                handlers::editing::set_default_direction(state, direction)
            }
            AppCommand::SetDefaultPriority { priority } => {
                handlers::editing::set_default_priority(state, priority)
            }
            AppCommand::SetAllConnectionsDirectionBetweenSelected { direction } => {
                handlers::editing::set_all_directions_between_selected(state, direction)
            }
            AppCommand::RemoveAllConnectionsBetweenSelected => {
                handlers::editing::remove_all_between_selected(state)
            }
            AppCommand::InvertAllConnectionsBetweenSelected => {
                handlers::editing::invert_all_between_selected(state)
            }
            AppCommand::SetAllConnectionsPriorityBetweenSelected { priority } => {
                handlers::editing::set_all_priorities_between_selected(state, priority)
            }
            AppCommand::ConnectSelectedNodes => handlers::editing::connect_selected(state),
            AppCommand::CreateMarker {
                node_id,
                name,
                group,
            } => handlers::editing::create_marker(state, node_id, &name, &group),
            AppCommand::RemoveMarker { node_id } => {
                handlers::editing::remove_marker(state, node_id)
            }
            AppCommand::OpenMarkerDialog { node_id, is_new } => {
                handlers::editing::open_marker_dialog(state, node_id, is_new)
            }
            AppCommand::UpdateMarker {
                node_id,
                name,
                group,
            } => handlers::editing::update_marker(state, node_id, &name, &group),

            // === Route-Tool ===
            AppCommand::RouteToolClick { world_pos, ctrl } => {
                handlers::route_tool::click(state, world_pos, ctrl)
            }
            AppCommand::RouteToolExecute => handlers::route_tool::execute(state),
            AppCommand::RouteToolCancel => handlers::route_tool::cancel(state),
            AppCommand::SelectRouteTool { index } => handlers::route_tool::select(state, index),
            AppCommand::RouteToolRecreate => handlers::route_tool::recreate(state),

            // === Dialoge & Anwendungssteuerung ===
            AppCommand::RequestExit => handlers::dialog::request_exit(state),
            AppCommand::RequestHeightmapDialog => handlers::dialog::request_heightmap_dialog(state),
            AppCommand::RequestBackgroundMapDialog => {
                handlers::dialog::request_background_map_dialog(state)
            }
            AppCommand::DismissHeightmapWarning => {
                handlers::dialog::dismiss_heightmap_warning(state)
            }
            AppCommand::CloseMarkerDialog => handlers::dialog::close_marker_dialog(state),
            AppCommand::OpenOptionsDialog => handlers::dialog::open_options_dialog(state),
            AppCommand::CloseOptionsDialog => handlers::dialog::close_options_dialog(state),
            AppCommand::ApplyOptions { options } => handlers::dialog::apply_options(state, options),
            AppCommand::ResetOptions => handlers::dialog::reset_options(state),
            AppCommand::DismissDeduplicateDialog => handlers::dialog::dismiss_dedup_dialog(state),

            // === History ===
            AppCommand::Undo => handlers::history::undo(state),
            AppCommand::Redo => handlers::history::redo(state),
        }

        state.command_log.record(executed_command);

        Ok(())
    }

    /// Baut die Render-Szene aus dem aktuellen AppState.
    pub fn build_render_scene(&self, state: &AppState, viewport_size: [f32; 2]) -> RenderScene {
        render_scene::build(state, viewport_size)
    }
}
