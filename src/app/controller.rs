//! Application Controller für zentrale Event-Verarbeitung.

use super::render_scene;
use super::use_cases;
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
        }
    }

    /// Führt mutierende Commands auf dem AppState aus.
    pub fn handle_command(
        &mut self,
        state: &mut AppState,
        command: AppCommand,
    ) -> anyhow::Result<()> {
        let executed_command = command.clone();

        match command {
            AppCommand::RequestOpenFileDialog => {
                use_cases::file_io::request_open_file(state);
            }
            AppCommand::RequestSaveFileDialog => {
                use_cases::file_io::request_save_file(state);
            }
            AppCommand::RequestExit => {
                state.should_exit = true;
            }
            AppCommand::RequestHeightmapDialog => {
                use_cases::heightmap::request_heightmap_dialog(state);
            }
            AppCommand::RequestBackgroundMapDialog => {
                use_cases::background_map::request_background_map_dialog(state);
            }
            AppCommand::ClearHeightmap => {
                use_cases::heightmap::clear_heightmap(state);
            }
            AppCommand::SetHeightmap { path } => {
                use_cases::heightmap::set_heightmap(state, path);
            }
            AppCommand::DismissHeightmapWarning => {
                use_cases::heightmap::dismiss_heightmap_warning(state);
            }
            AppCommand::ConfirmAndSaveFile => {
                use_cases::file_io::confirm_and_save(state)?;
            }
            AppCommand::ResetCamera => {
                use_cases::camera::reset_camera(state);
            }
            AppCommand::ZoomIn => {
                use_cases::camera::zoom_in(state);
            }
            AppCommand::ZoomOut => {
                use_cases::camera::zoom_out(state);
            }
            AppCommand::SetViewportSize { size } => {
                use_cases::viewport::resize(state, size);
            }
            AppCommand::PanCamera { delta } => {
                use_cases::camera::pan(state, delta);
            }
            AppCommand::ZoomCamera {
                factor,
                focus_world,
            } => {
                use_cases::camera::zoom_towards(state, factor, focus_world);
            }
            AppCommand::SelectNearestNode {
                world_pos,
                max_distance,
                additive,
                extend_path,
            } => {
                use_cases::selection::select_nearest_node(
                    state,
                    world_pos,
                    max_distance,
                    additive,
                    extend_path,
                );
            }
            AppCommand::SelectSegmentBetweenNearestIntersections {
                world_pos,
                max_distance,
                additive,
            } => {
                use_cases::selection::select_segment_between_nearest_intersections(
                    state,
                    world_pos,
                    max_distance,
                    additive,
                );
            }
            AppCommand::MoveSelectedNodes { delta_world } => {
                use_cases::selection::move_selected_nodes(state, delta_world);
            }
            AppCommand::BeginMoveSelectedNodes => {
                state.record_undo_snapshot();
            }
            AppCommand::EndMoveSelectedNodes => {
                // No-op for now (move lifecycle end).
            }
            AppCommand::SelectNodesInRect { min, max, additive } => {
                use_cases::selection::select_nodes_in_rect(state, min, max, additive);
            }
            AppCommand::SelectNodesInLasso { polygon, additive } => {
                use_cases::selection::select_nodes_in_lasso(state, &polygon, additive);
            }
            AppCommand::SetRenderQuality { quality } => {
                use_cases::viewport::set_render_quality(state, quality);
            }
            AppCommand::LoadFile { path } => {
                use_cases::file_io::load_selected_file(state, path)?;
            }
            AppCommand::SaveFile { path } => {
                use_cases::file_io::save_with_heightmap_check(state, path)?;
            }
            AppCommand::Undo => {
                let current = super::history::Snapshot::from_state(state);
                if let Some(prev) = state.history.pop_undo_with_current(current) {
                    prev.apply_to(state);
                    log::info!("Undo ausgeführt");
                } else {
                    log::debug!("Undo: nichts zu tun");
                }
            }
            AppCommand::Redo => {
                let current = super::history::Snapshot::from_state(state);
                if let Some(next) = state.history.pop_redo_with_current(current) {
                    next.apply_to(state);
                    log::info!("Redo ausgeführt");
                } else {
                    log::debug!("Redo: nichts zu tun");
                }
            }
            AppCommand::SetEditorTool { tool } => {
                state.editor.active_tool = tool;
                state.editor.connect_source_node = None;
                log::info!("Editor-Werkzeug: {:?}", tool);
            }
            AppCommand::AddNodeAtPosition { world_pos } => {
                use_cases::editing::add_node_at_position(state, world_pos);
            }
            AppCommand::DeleteSelectedNodes => {
                use_cases::editing::delete_selected_nodes(state);
            }
            AppCommand::ConnectToolPickNode {
                world_pos,
                max_distance,
            } => {
                use_cases::editing::connect_tool_pick_node(state, world_pos, max_distance);
            }
            AppCommand::AddConnection {
                from_id,
                to_id,
                direction,
                priority,
            } => {
                use_cases::editing::add_connection(state, from_id, to_id, direction, priority);
            }
            AppCommand::RemoveConnectionBetween { node_a, node_b } => {
                use_cases::editing::remove_connection_between(state, node_a, node_b);
            }
            AppCommand::SetConnectionDirection {
                start_id,
                end_id,
                direction,
            } => {
                use_cases::editing::set_connection_direction(state, start_id, end_id, direction);
            }
            AppCommand::SetConnectionPriority {
                start_id,
                end_id,
                priority,
            } => {
                use_cases::editing::set_connection_priority(state, start_id, end_id, priority);
            }
            AppCommand::SetDefaultDirection { direction } => {
                state.editor.default_direction = direction;
                log::info!("Standard-Verbindungsrichtung: {:?}", direction);
            }
            AppCommand::SetDefaultPriority { priority } => {
                state.editor.default_priority = priority;
                log::info!("Standard-Straßenart: {:?}", priority);
            }
            AppCommand::SetAllConnectionsDirectionBetweenSelected { direction } => {
                use_cases::editing::set_all_connections_direction_between_selected(
                    state, direction,
                );
            }
            AppCommand::RemoveAllConnectionsBetweenSelected => {
                use_cases::editing::remove_all_connections_between_selected(state);
            }
            AppCommand::InvertAllConnectionsBetweenSelected => {
                use_cases::editing::invert_all_connections_between_selected(state);
            }
            AppCommand::SetAllConnectionsPriorityBetweenSelected { priority } => {
                use_cases::editing::set_all_connections_priority_between_selected(state, priority);
            }
            AppCommand::ConnectSelectedNodes => {
                // Verbinde die beiden selektierten Nodes mit Standard-Richtung/Priorität
                let ids: Vec<u64> = state.selection.selected_node_ids.iter().copied().collect();
                if ids.len() == 2 {
                    let direction = state.editor.default_direction;
                    let priority = state.editor.default_priority;
                    use_cases::editing::add_connection(state, ids[0], ids[1], direction, priority);
                }
            }
            AppCommand::LoadBackgroundMap { path, crop_size } => {
                if let Err(e) =
                    use_cases::background_map::load_background_map(state, path, crop_size)
                {
                    log::error!("Fehler beim Laden der Background-Map: {}", e);
                }
            }
            AppCommand::UpdateBackgroundOpacity { opacity } => {
                use_cases::background_map::set_background_opacity(state, opacity);
            }
            AppCommand::ToggleBackgroundVisibility => {
                use_cases::background_map::toggle_background_visibility(state);
            }
            AppCommand::CreateMarker {
                node_id,
                name,
                group,
            } => {
                use_cases::editing::create_marker(state, node_id, &name, &group);
            }
            AppCommand::RemoveMarker { node_id } => {
                use_cases::editing::remove_marker(state, node_id);
            }
            AppCommand::OpenMarkerDialog { node_id, is_new } => {
                use_cases::editing::open_marker_dialog(state, node_id, is_new);
            }
            AppCommand::UpdateMarker {
                node_id,
                name,
                group,
            } => {
                use_cases::editing::update_marker(state, node_id, &name, &group);
            }
            AppCommand::CloseMarkerDialog => {
                state.ui.show_marker_dialog = false;
                state.ui.marker_dialog_node_id = None;
            }
            AppCommand::OpenOptionsDialog => {
                state.show_options_dialog = true;
            }
            AppCommand::CloseOptionsDialog => {
                state.show_options_dialog = false;
            }
            AppCommand::ApplyOptions { options } => {
                state.options = options;
                // Sofort speichern
                let path = crate::shared::EditorOptions::config_path();
                if let Err(e) = state.options.save_to_file(&path) {
                    log::error!("Optionen konnten nicht gespeichert werden: {}", e);
                }
            }
            AppCommand::ResetOptions => {
                state.options = crate::shared::EditorOptions::default();
                let path = crate::shared::EditorOptions::config_path();
                if let Err(e) = state.options.save_to_file(&path) {
                    log::error!("Optionen konnten nicht gespeichert werden: {}", e);
                }
            }
            AppCommand::ClearSelection => {
                use_cases::selection::clear_selection(state);
            }
            AppCommand::SelectAllNodes => {
                if let Some(road_map) = state.road_map.as_deref() {
                    state.selection.selected_node_ids = road_map.nodes.keys().copied().collect();
                    state.selection.selection_anchor_node_id = None;
                    log::info!(
                        "Alle {} Nodes selektiert",
                        state.selection.selected_node_ids.len()
                    );
                }
            }
            AppCommand::DeduplicateNodes => {
                use_cases::file_io::deduplicate_loaded_roadmap(state);
            }
            AppCommand::DismissDeduplicateDialog => {
                state.ui.show_dedup_dialog = false;
                state.ui.status_message = None;
            }
        }

        state.command_log.record(executed_command);

        Ok(())
    }

    /// Baut die Render-Szene aus dem aktuellen AppState.
    pub fn build_render_scene(&self, state: &AppState, viewport_size: [f32; 2]) -> RenderScene {
        render_scene::build(state, viewport_size)
    }
}
