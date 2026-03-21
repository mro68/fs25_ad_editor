//! Handler fuer Undo/Redo-Operationen.

use crate::app::history::Snapshot;
use crate::app::AppState;

/// Fuehrt einen Undo-Schritt aus, falls vorhanden.
///
/// Bei aktivem Group-Edit wird der Edit-Modus zuerst implizit abgebrochen
/// (Guard wird zurueckgesetzt), bevor Undo ausgefuehrt wird.
pub fn undo(state: &mut AppState) {
    // Group-Edit implizit abbrechen bei manuellem Undo (Inkonsistenz-Schutz)
    if state.group_editing.is_some() {
        state.group_editing = None;
        state.segment_registry.set_edit_guard(None);
        log::debug!("Undo: Group-Edit implizit abgebrochen");
    }

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
