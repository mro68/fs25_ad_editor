//! Handler für Undo/Redo-Operationen.

use crate::app::history::Snapshot;
use crate::app::AppState;

/// Führt einen Undo-Schritt aus, falls vorhanden.
pub fn undo(state: &mut AppState) {
    let current = Snapshot::from_state(state);
    if let Some(prev) = state.history.pop_undo_with_current(current) {
        prev.apply_to(state);
        log::info!("Undo ausgeführt");
    } else {
        log::debug!("Undo: nichts zu tun");
    }
}

/// Führt einen Redo-Schritt aus, falls vorhanden.
pub fn redo(state: &mut AppState) {
    let current = Snapshot::from_state(state);
    if let Some(next) = state.history.pop_redo_with_current(current) {
        next.apply_to(state);
        log::info!("Redo ausgeführt");
    } else {
        log::debug!("Redo: nichts zu tun");
    }
}
