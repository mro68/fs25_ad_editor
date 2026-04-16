//! Command-Dispatch fuer Kamera, Viewport und Background-/Overview-Features.

use crate::app::handlers;
use crate::app::{AppCommand, AppState};

/// Fuehrt View-Commands aus.
pub(super) fn handle(state: &mut AppState, command: AppCommand) -> anyhow::Result<()> {
    match command {
        AppCommand::ResetCamera => {
            handlers::view::reset_camera(state);
            Ok(())
        }
        AppCommand::ZoomIn => {
            handlers::view::zoom_in(state);
            Ok(())
        }
        AppCommand::ZoomOut => {
            handlers::view::zoom_out(state);
            Ok(())
        }
        AppCommand::SetViewportSize { size } => {
            handlers::view::set_viewport_size(state, size);
            Ok(())
        }
        AppCommand::PanCamera { delta } => {
            handlers::view::pan(state, delta);
            Ok(())
        }
        AppCommand::ZoomCamera {
            factor,
            focus_world,
        } => {
            handlers::view::zoom_towards(state, factor, focus_world);
            Ok(())
        }
        AppCommand::CenterOnNode { node_id } => {
            handlers::view::center_on_node(state, node_id);
            Ok(())
        }
        AppCommand::SetRenderQuality { quality } => {
            handlers::view::set_render_quality(state, quality);
            Ok(())
        }
        AppCommand::LoadBackgroundMap { path, crop_size } => {
            handlers::view::load_background_map(state, path, crop_size)
        }
        AppCommand::ToggleBackgroundVisibility => {
            handlers::view::toggle_background_visibility(state);
            Ok(())
        }
        AppCommand::SetBackgroundLayerVisibility { layer, visible } => {
            handlers::view::set_background_layer_visibility(state, layer, visible)
        }
        AppCommand::ScaleBackground { factor } => {
            handlers::view::scale_background(state, factor);
            Ok(())
        }
        AppCommand::BrowseZipBackground { path } => {
            handlers::view::browse_zip_background(state, path)
        }
        AppCommand::LoadBackgroundFromZip {
            zip_path,
            entry_name,
            crop_size,
        } => handlers::view::load_background_from_zip(state, zip_path, entry_name, crop_size),
        AppCommand::GenerateOverviewWithOptions => {
            handlers::view::generate_overview_with_options(state)
        }
        AppCommand::SaveBackgroundAsOverview { path } => {
            handlers::view::save_background_as_overview(state, path)
        }
        AppCommand::ZoomToFit => {
            handlers::view::zoom_to_fit(state);
            Ok(())
        }
        AppCommand::ZoomToSelectionBounds => {
            handlers::view::zoom_to_selection_bounds(state);
            Ok(())
        }
        other => anyhow::bail!("unerwarteter View-Command: {other:?}"),
    }
}
