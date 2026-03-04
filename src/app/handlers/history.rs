//! Handler fuer Undo/Redo-Operationen.

use crate::app::history::Snapshot;
use crate::app::AppState;

/// Fuehrt einen Undo-Schritt aus, falls vorhanden.
pub fn undo(state: &mut AppState) {
    let current = Snapshot::from_state(state);
    if let Some(prev) = state.history.pop_undo_with_current(current) {
        prev.apply_to(state);
        log::info!("Undo ausgefuehrt");
    } else {
        log::debug!("Undo: nichts zu tun");
    }
}

/// Fuehrt einen Redo-Schritt aus, falls vorhanden.
pub fn redo(state: &mut AppState) {
    let current = Snapshot::from_state(state);
    if let Some(next) = state.history.pop_redo_with_current(current) {
        next.apply_to(state);
        log::info!("Redo ausgefuehrt");
    } else {
        log::debug!("Redo: nichts zu tun");
    }
}
