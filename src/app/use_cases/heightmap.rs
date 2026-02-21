//! Use-Case-Funktionen für Heightmap-Verwaltung.

use crate::app::AppState;

/// Öffnet den Heightmap-Auswahl-Dialog.
pub fn request_heightmap_dialog(state: &mut AppState) {
    state.ui.show_heightmap_dialog = true;
}

/// Entfernt die ausgewählte Heightmap.
pub fn clear_heightmap(state: &mut AppState) {
    state.ui.heightmap_path = None;
    log::info!("Heightmap gelöscht");
}

/// Setzt die Heightmap auf den angegebenen Pfad.
pub fn set_heightmap(state: &mut AppState, path: String) {
    state.ui.heightmap_path = Some(path.clone());
    log::info!("Heightmap ausgewählt: {}", path);
}

/// Blendet die Heightmap-Warnung aus und setzt den ausstehenden Speicherpfad zurück.
pub fn dismiss_heightmap_warning(state: &mut AppState) {
    state.ui.show_heightmap_warning = false;
    state.ui.pending_save_path = None;
}
