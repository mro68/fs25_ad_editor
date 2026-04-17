use crate::app::use_cases;
use crate::app::AppState;

/// Erstellt einen Map-Marker fuer einen Node.
pub fn create_marker(state: &mut AppState, node_id: u64, name: &str, group: &str) {
    use_cases::editing::create_marker(state, node_id, name, group);
}

/// Entfernt den Map-Marker eines Nodes.
pub fn remove_marker(state: &mut AppState, node_id: u64) {
    use_cases::editing::remove_marker(state, node_id);
}

/// Oeffnet den Marker-Dialog zum Erstellen oder Bearbeiten.
pub fn open_marker_dialog(state: &mut AppState, node_id: u64, is_new: bool) {
    use_cases::editing::open_marker_dialog(state, node_id, is_new);
}

/// Aktualisiert Name/Gruppe eines bestehenden Markers.
pub fn update_marker(state: &mut AppState, node_id: u64, name: &str, group: &str) {
    use_cases::editing::update_marker(state, node_id, name, group);
}
