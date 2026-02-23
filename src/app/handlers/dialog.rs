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

/// Schließt den ZIP-Browser-Dialog.
pub fn close_zip_browser(state: &mut AppState) {
    state.ui.zip_browser = None;
}

/// Öffnet den Übersichtskarten-ZIP-Auswahl-Dialog.
pub fn request_overview_dialog(state: &mut AppState) {
    state.ui.show_overview_dialog = true;
}

/// Öffnet den Übersichtskarten-Options-Dialog mit dem gewählten ZIP-Pfad.
pub fn open_overview_options_dialog(state: &mut AppState, zip_path: String) {
    state.ui.show_overview_dialog = false;
    state.ui.overview_options_dialog.visible = true;
    state.ui.overview_options_dialog.zip_path = zip_path;
    state.ui.overview_options_dialog.layers = state.options.overview_layers.clone();
}

/// Schließt den Übersichtskarten-Options-Dialog.
pub fn close_overview_options_dialog(state: &mut AppState) {
    state.ui.overview_options_dialog.visible = false;
}

/// Schließt den Post-Load-Dialog (Heightmap/ZIP-Erkennung).
pub fn dismiss_post_load_dialog(state: &mut AppState) {
    state.ui.post_load_dialog = Default::default();
}
