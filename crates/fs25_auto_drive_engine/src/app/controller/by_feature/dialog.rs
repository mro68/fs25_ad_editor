//! Command-Dispatch fuer Dialog- und Overlay-State.

use crate::app::handlers;
use crate::app::{AppCommand, AppState};

/// Fuehrt Dialog-Commands aus.
pub(super) fn handle(state: &mut AppState, command: AppCommand) -> anyhow::Result<()> {
    match command {
        AppCommand::RequestExit => {
            handlers::dialog::request_exit(state);
            Ok(())
        }
        AppCommand::RequestHeightmapDialog => {
            handlers::dialog::request_heightmap_dialog(state);
            Ok(())
        }
        AppCommand::RequestBackgroundMapDialog => {
            handlers::dialog::request_background_map_dialog(state);
            Ok(())
        }
        AppCommand::DismissHeightmapWarning => {
            handlers::dialog::dismiss_heightmap_warning(state);
            Ok(())
        }
        AppCommand::CloseMarkerDialog => {
            handlers::dialog::close_marker_dialog(state);
            Ok(())
        }
        AppCommand::OpenOptionsDialog => {
            handlers::dialog::open_options_dialog(state);
            Ok(())
        }
        AppCommand::CloseOptionsDialog => {
            handlers::dialog::close_options_dialog(state);
            Ok(())
        }
        AppCommand::ApplyOptions { options } => handlers::dialog::apply_options(state, *options),
        AppCommand::ResetOptions => handlers::dialog::reset_options(state),
        AppCommand::ToggleCommandPalette => {
            handlers::dialog::toggle_command_palette(state);
            Ok(())
        }
        AppCommand::DismissDeduplicateDialog => {
            handlers::dialog::dismiss_dedup_dialog(state);
            Ok(())
        }
        AppCommand::CloseZipBrowser => {
            handlers::dialog::close_zip_browser(state);
            Ok(())
        }
        AppCommand::RequestOverviewDialog => {
            handlers::dialog::request_overview_dialog(state);
            Ok(())
        }
        AppCommand::OpenOverviewOptionsDialog { path } => {
            handlers::dialog::open_overview_options_dialog(state, path);
            Ok(())
        }
        AppCommand::CloseOverviewOptionsDialog => {
            handlers::dialog::close_overview_options_dialog(state);
            Ok(())
        }
        AppCommand::DismissPostLoadDialog => {
            handlers::dialog::dismiss_post_load_dialog(state);
            Ok(())
        }
        AppCommand::DismissSaveOverviewDialog => {
            handlers::dialog::dismiss_save_overview_dialog(state);
            Ok(())
        }
        AppCommand::OpenTraceAllFieldsDialog => {
            handlers::dialog::open_trace_all_fields_dialog(state);
            Ok(())
        }
        AppCommand::CloseTraceAllFieldsDialog => {
            handlers::dialog::close_trace_all_fields_dialog(state);
            Ok(())
        }
        AppCommand::RequestCurseplayImportDialog => {
            handlers::dialog::request_curseplay_import_dialog(state);
            Ok(())
        }
        AppCommand::RequestCurseplayExportDialog => {
            handlers::dialog::request_curseplay_export_dialog(state);
            Ok(())
        }
        other => unreachable!("unerwarteter Dialog-Command: {other:?}"),
    }
}
