//! Command-Dispatch fuer Undo/Redo.

use crate::app::handlers;
use crate::app::{AppCommand, AppState};

/// Fuehrt History-Commands aus.
pub(super) fn handle(state: &mut AppState, command: AppCommand) -> anyhow::Result<()> {
    match command {
        AppCommand::Undo => {
            handlers::history::undo(state);
            Ok(())
        }
        AppCommand::Redo => {
            handlers::history::redo(state);
            Ok(())
        }
        other => unreachable!("unerwarteter History-Command: {other:?}"),
    }
}
