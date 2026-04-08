//! Use-Case-Funktionen fuer Heightmap-Verwaltung.

use crate::app::ui_contract::{DialogRequest, DialogRequestKind};
use crate::app::AppState;

/// Oeffnet den Heightmap-Auswahl-Dialog.
pub fn request_heightmap_dialog(state: &mut AppState) {
    state
        .ui
        .request_dialog(DialogRequest::pick_path(DialogRequestKind::Heightmap));
}

/// Entfernt die ausgewaehlte Heightmap.
pub fn clear_heightmap(state: &mut AppState) {
    state.ui.heightmap_path = None;
    log::info!("Heightmap geloescht");
}

/// Setzt die Heightmap auf den angegebenen Pfad.
pub fn set_heightmap(state: &mut AppState, path: String) {
    state.ui.heightmap_path = Some(path.clone());
    log::info!("Heightmap ausgewaehlt: {}", path);
}

/// Blendet die Heightmap-Warnung aus und setzt den ausstehenden Speicherpfad zurueck.
pub fn dismiss_heightmap_warning(state: &mut AppState) {
    state
        .ui
        .request_dialog(DialogRequest::DismissHeightmapWarning);
    state.ui.pending_save_path = None;
}
