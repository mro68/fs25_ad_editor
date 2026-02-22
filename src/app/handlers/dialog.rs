//! Handler für Dialog-State und Anwendungssteuerung.

use crate::app::use_cases;
use crate::app::AppState;
use crate::shared::EditorOptions;

/// Markiert die Anwendung zum Beenden im nächsten Frame.
pub fn request_exit(state: &mut AppState) {
    state.should_exit = true;
}

/// Öffnet den Heightmap-Dateidialog.
pub fn request_heightmap_dialog(state: &mut AppState) {
    use_cases::heightmap::request_heightmap_dialog(state);
}

/// Öffnet den Background-Map-Dateidialog.
pub fn request_background_map_dialog(state: &mut AppState) {
    use_cases::background_map::request_background_map_dialog(state);
}

/// Schließt die Heightmap-Warnung.
pub fn dismiss_heightmap_warning(state: &mut AppState) {
    use_cases::heightmap::dismiss_heightmap_warning(state);
}

/// Schließt den Marker-Dialog und räumt dessen Auswahlzustand auf.
pub fn close_marker_dialog(state: &mut AppState) {
    state.ui.marker_dialog.visible = false;
    state.ui.marker_dialog.node_id = None;
}

/// Öffnet den Optionen-Dialog.
pub fn open_options_dialog(state: &mut AppState) {
    state.show_options_dialog = true;
}

/// Schließt den Optionen-Dialog.
pub fn close_options_dialog(state: &mut AppState) {
    state.show_options_dialog = false;
}

/// Übernimmt neue Optionen und persistiert sie in der Konfigurationsdatei.
pub fn apply_options(state: &mut AppState, options: EditorOptions) -> anyhow::Result<()> {
    state.options = options;
    let path = EditorOptions::config_path();
    state.options.save_to_file(&path)
}

/// Setzt Optionen auf Standardwerte zurück und persistiert sie.
pub fn reset_options(state: &mut AppState) -> anyhow::Result<()> {
    state.options = EditorOptions::default();
    let path = EditorOptions::config_path();
    state.options.save_to_file(&path)
}

/// Schließt den Duplikat-Dialog und entfernt die Statusmeldung.
pub fn dismiss_dedup_dialog(state: &mut AppState) {
    state.ui.dedup_dialog.visible = false;
    state.ui.status_message = None;
}
