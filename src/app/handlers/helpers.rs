//! Zentrale Helfer fuer Undo/Selection-Operationen in den Handlern.
//!
//! Ziel: Reduziere redundanten Code beim Aufnehmen von Undo-Snapshots
//! und minimiere unnötige Clones durch gezieltes Arc-Cloning der
//! `selected_node_ids`.

use crate::app::history::Snapshot;
use crate::app::state::SelectionState;
use crate::app::AppState;
use indexmap::IndexSet;
use std::sync::Arc;

/// Nimmt den aktuellen Selektionszustand (Arc-Clone O(1)) und gibt
/// `(selected_node_ids_arc, selection_anchor)` zurück.
#[inline]
pub fn capture_selection_snapshot(state: &AppState) -> (Arc<IndexSet<u64>>, Option<u64>) {
    (
        state.selection.selected_node_ids.clone(),
        state.selection.selection_anchor_node_id,
    )
}

/// Vergleicht den übergebenen alten Selektionszustand mit dem aktuellen
/// und legt bei Unterschied einen Undo-Snapshot mit der alten Selektion an.
///
/// Wichtig: `old_selected` sollte mittels `capture_selection_snapshot()`
/// erzeugt worden sein (O(1) Arc-Clone).
pub fn record_selection_if_changed(
    state: &mut AppState,
    old_selected: Arc<IndexSet<u64>>,
    old_anchor: Option<u64>,
) {
    let current_selected = &state.selection.selected_node_ids;
    let current_anchor = state.selection.selection_anchor_node_id;

    if !Arc::ptr_eq(&old_selected, current_selected) && *old_selected != **current_selected
        || old_anchor != current_anchor
    {
        let old_selection = SelectionState {
            selected_node_ids: old_selected,
            selection_anchor_node_id: old_anchor,
            generation: 0,
        };
        let snap = Snapshot {
            road_map: state.road_map.clone(),
            selection: old_selection,
        };
        state.history.record_snapshot(snap);
    }
}
