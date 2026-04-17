use crate::app::AppState;

/// Laedt ein gespeichertes Segment zur nachtraeglichen Bearbeitung.
///
/// Loescht die zugehoerigen Nodes aus der RoadMap, aktiviert das passende
/// Route-Tool und befuellt es mit den gespeicherten Parametern.
pub fn edit_group(state: &mut AppState, record_id: u64) {
    crate::app::tool_editing::begin_edit(state, record_id);
}
