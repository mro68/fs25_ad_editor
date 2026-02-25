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
        super::intent_mapping::map_intent_to_commands(state, intent)
    }

    /// Führt mutierende Commands auf dem AppState aus.
    /// Dispatcht an Feature-Handler in `handlers/`.
    pub fn handle_command(
        &mut self,
        state: &mut AppState,
        command: AppCommand,
    ) -> anyhow::Result<()> {
        state.command_log.record(&command);
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
            AppCommand::ToggleBackgroundVisibility => {
                handlers::view::toggle_background_visibility(state)
            }
            AppCommand::ScaleBackground { factor } => {
                handlers::view::scale_background(state, factor)
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
            AppCommand::RouteToolDragStart { world_pos } => {
                handlers::route_tool::drag_start(state, world_pos)
            }
            AppCommand::RouteToolDragUpdate { world_pos } => {
                handlers::route_tool::drag_update(state, world_pos)
            }
            AppCommand::RouteToolDragEnd => handlers::route_tool::drag_end(state),
            AppCommand::EditSegment { record_id } => {
                handlers::editing::edit_segment(state, record_id)
            }

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
            AppCommand::ApplyOptions { options } => {
                handlers::dialog::apply_options(state, options)?
            }
            AppCommand::ResetOptions => handlers::dialog::reset_options(state)?,
            AppCommand::DismissDeduplicateDialog => handlers::dialog::dismiss_dedup_dialog(state),

            // === History ===
            AppCommand::Undo => handlers::history::undo(state),
            AppCommand::Redo => handlers::history::redo(state),

            // === ZIP-Background ===
            AppCommand::BrowseZipBackground { path } => {
                handlers::view::browse_zip_background(state, path)?
            }
            AppCommand::LoadBackgroundFromZip {
                zip_path,
                entry_name,
                crop_size,
            } => handlers::view::load_background_from_zip(state, zip_path, entry_name, crop_size)?,
            AppCommand::CloseZipBrowser => handlers::dialog::close_zip_browser(state),

            // === Overview-Map ===
            AppCommand::RequestOverviewDialog => handlers::dialog::request_overview_dialog(state),
            AppCommand::OpenOverviewOptionsDialog { path } => {
                handlers::dialog::open_overview_options_dialog(state, path)
            }
            AppCommand::GenerateOverviewWithOptions => {
                handlers::view::generate_overview_with_options(state)?
            }
            AppCommand::CloseOverviewOptionsDialog => {
                handlers::dialog::close_overview_options_dialog(state)
            }

            // === Post-Load-Dialog ===
            AppCommand::DismissPostLoadDialog => handlers::dialog::dismiss_post_load_dialog(state),

            // === overview.jpg speichern ===
            AppCommand::SaveBackgroundAsOverview { path } => {
                handlers::view::save_background_as_overview(state, path)?
            }
            AppCommand::DismissSaveOverviewDialog => {
                handlers::dialog::dismiss_save_overview_dialog(state)
            }

            // === Distanzen ===
            AppCommand::ResamplePath => handlers::editing::resample_path(state),
        }

        Ok(())
    }

    /// Baut die Render-Szene aus dem aktuellen AppState.
    pub fn build_render_scene(&self, state: &AppState, viewport_size: [f32; 2]) -> RenderScene {
        render_scene::build(state, viewport_size)
    }
}
