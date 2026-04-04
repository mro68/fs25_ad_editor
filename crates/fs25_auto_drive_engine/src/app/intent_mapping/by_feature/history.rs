//! Intent-Mapping fuer Undo/Redo.

use crate::app::{AppCommand, AppIntent, AppState};

/// Mappt History-Intents auf Commands.
pub(super) fn map(_state: &AppState, intent: AppIntent) -> Vec<AppCommand> {
    match intent {
        AppIntent::UndoRequested => vec![AppCommand::Undo],
        AppIntent::RedoRequested => vec![AppCommand::Redo],
        other => unreachable!("unerwarteter History-Intent: {other:?}"),
    }
}
