//! Command-Dispatch fuer Datei- und XML-I/O.

use crate::app::handlers;
use crate::app::{AppCommand, AppState};

/// Fuehrt Datei-I/O-Commands aus.
pub(super) fn handle(state: &mut AppState, command: AppCommand) -> anyhow::Result<()> {
    match command {
        AppCommand::RequestOpenFileDialog => {
            handlers::file_io::request_open(state);
            Ok(())
        }
        AppCommand::RequestSaveFileDialog => {
            handlers::file_io::request_save(state);
            Ok(())
        }
        AppCommand::ConfirmAndSaveFile => handlers::file_io::confirm_and_save(state),
        AppCommand::LoadFile { path } => handlers::file_io::load(state, path),
        AppCommand::SaveFile { path } => handlers::file_io::save(state, path),
        AppCommand::ClearHeightmap => {
            handlers::file_io::clear_heightmap(state);
            Ok(())
        }
        AppCommand::SetHeightmap { path } => {
            handlers::file_io::set_heightmap(state, path);
            Ok(())
        }
        AppCommand::DeduplicateNodes => {
            handlers::file_io::deduplicate(state);
            Ok(())
        }
        other => unreachable!("unerwarteter FileIo-Command: {other:?}"),
    }
}
