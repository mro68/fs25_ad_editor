//! Handler fuer Undo/Redo-Operationen.

use crate::app::history::Snapshot;
use crate::app::AppState;

/// Fuehrt einen Undo-Schritt aus, falls vorhanden.
///
/// Aktive Group- und Tool-Edits werden dabei als transiente Restore-Flows
/// behandelt: Der Zwischenzustand wird verworfen und erzeugt bewusst keinen
/// Redo-Eintrag.
pub fn undo(state: &mut AppState) {
    if state.active_tool_edit_session.is_some() {
        crate::app::tool_editing::cancel_active_edit(state);
        return;
    }

    if super::group::abort_active_group_edit(state) {
        if restore_last_snapshot_without_redo(state) {
            log::info!("Undo: aktiver Group-Edit verworfen");
        } else {
            log::debug!("Undo: kein Snapshot fuer aktiven Group-Edit vorhanden");
        }
        return;
    }

    if apply_undo_with_redo(state) {
        log::info!("Undo ausgefuehrt");
    } else {
        log::debug!("Undo: nichts zu tun");
    }
}

pub(crate) fn restore_last_snapshot_without_redo(state: &mut AppState) -> bool {
    match state.history.pop_undo_discard_current() {
        Some(prev) => {
            prev.apply_to(state);
            true
        }
        None => false,
    }
}

/// Fuehrt einen Redo-Schritt aus, falls vorhanden.
pub fn redo(state: &mut AppState) {
    let current = Snapshot::from_state(state);
    match state.history.pop_redo_with_current(current) {
        Some(next) => {
            next.apply_to(state);
            log::info!("Redo ausgefuehrt");
        }
        None => {
            log::debug!("Redo: nichts zu tun");
        }
    }
}

fn apply_undo_with_redo(state: &mut AppState) -> bool {
    let current = Snapshot::from_state(state);
    match state.history.pop_undo_with_current(current) {
        Some(prev) => {
            prev.apply_to(state);
            true
        }
        None => false,
    }
}
