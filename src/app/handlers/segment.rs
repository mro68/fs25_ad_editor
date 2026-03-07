//! Handler fuer Segment-Operationen (Lock-Toggle).

use crate::app::AppState;

/// Schaltet den Lock-Zustand eines Segments um.
///
/// Ist das Segment gesperrt, werden beim Verschieben eines zugehoerigen Nodes
/// alle Segment-Nodes mitbewegt. Unbekannte IDs werden ignoriert.
pub fn toggle_lock(state: &mut AppState, segment_id: u64) {
    state.segment_registry.toggle_lock(segment_id);
}

/// Loest ein Segment auf, indem nur der Segment-Record entfernt wird.
///
/// Die zugehoerigen Nodes und Verbindungen in der RoadMap bleiben unveraendert.
/// Unbekannte IDs werden ignoriert.
pub fn dissolve(state: &mut AppState, segment_id: u64) {
    state.segment_registry.remove(segment_id);
}
