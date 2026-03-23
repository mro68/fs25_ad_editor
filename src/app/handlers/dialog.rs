//! Handler fuer Dialog-State und Anwendungssteuerung.

use crate::app::use_cases;
use crate::app::AppState;
use crate::shared::EditorOptions;
use fs25_map_overview::FieldDetectionSource;
use std::path::Path;

/// Markiert die Anwendung zum Beenden im naechsten Frame.
pub fn request_exit(state: &mut AppState) {
    state.should_exit = true;
}

/// Oeffnet den Heightmap-Dateidialog.
pub fn request_heightmap_dialog(state: &mut AppState) {
    use_cases::heightmap::request_heightmap_dialog(state);
}

/// Oeffnet den Background-Map-Dateidialog.
pub fn request_background_map_dialog(state: &mut AppState) {
    use_cases::background_map::request_background_map_dialog(state);
}

/// Schliesst die Heightmap-Warnung.
pub fn dismiss_heightmap_warning(state: &mut AppState) {
    use_cases::heightmap::dismiss_heightmap_warning(state);
}

/// Schliesst den Marker-Dialog und raeumt dessen Auswahlzustand auf.
pub fn close_marker_dialog(state: &mut AppState) {
    state.ui.marker_dialog.visible = false;
    state.ui.marker_dialog.node_id = None;
}

/// Oeffnet den Optionen-Dialog.
pub fn open_options_dialog(state: &mut AppState) {
    state.show_options_dialog = true;
}

/// Schliesst den Optionen-Dialog.
pub fn close_options_dialog(state: &mut AppState) {
    state.show_options_dialog = false;
}

/// Uebernimmt neue Optionen und persistiert sie in der Konfigurationsdatei.
pub fn apply_options(state: &mut AppState, options: EditorOptions) -> anyhow::Result<()> {
    // Erst validieren, damit keine inkonsistenten Werte temporaer in den State gelangen.
    options.validate()?;
    state.set_options(options);
    let path = EditorOptions::config_path();
    state.options.save_to_file(&path)
}

/// Setzt Optionen auf Standardwerte zurueck und persistiert sie.
pub fn reset_options(state: &mut AppState) -> anyhow::Result<()> {
    state.set_options(EditorOptions::default());
    let path = EditorOptions::config_path();
    state.options.save_to_file(&path)
}

/// Schaltet die Sichtbarkeit der Command-Palette um.
pub fn toggle_command_palette(state: &mut AppState) {
    state.ui.show_command_palette = !state.ui.show_command_palette;
}

/// Schliesst den Duplikat-Dialog und entfernt die Statusmeldung.
pub fn dismiss_dedup_dialog(state: &mut AppState) {
    state.ui.dedup_dialog.visible = false;
    state.ui.status_message = None;
}

/// Schliesst den ZIP-Browser-Dialog.
pub fn close_zip_browser(state: &mut AppState) {
    state.ui.zip_browser = None;
}

/// Oeffnet den Uebersichtskarten-ZIP-Auswahl-Dialog.
pub fn request_overview_dialog(state: &mut AppState) {
    state.ui.show_overview_dialog = true;
}

/// Oeffnet den Uebersichtskarten-Options-Dialog mit dem gewaehlten ZIP-Pfad.
///
/// Prueft welche Savegame-Dateien im Elternordner der aktuell geladenen
/// Config-Datei vorhanden sind und befuellt die verfuegbaren Quellen.
pub fn open_overview_options_dialog(state: &mut AppState, zip_path: String) {
    state.ui.show_overview_dialog = false;
    state.ui.overview_options_dialog.visible = true;
    state.ui.overview_options_dialog.zip_path = zip_path;
    state.ui.overview_options_dialog.layers = state.options.overview_layers.clone();

    // Verfuegbare Quellen bestimmen
    let mut available = vec![FieldDetectionSource::FromZip];
    if let Some(xml_path) = state.ui.current_file_path.as_ref() {
        if let Some(savegame_dir) = Path::new(xml_path.as_str()).parent() {
            if savegame_dir.join("infoLayer_fieldType.grle").is_file() {
                available.push(FieldDetectionSource::FieldTypeGrle);
            }
            if savegame_dir.join("densityMap_ground.gdm").is_file() {
                available.push(FieldDetectionSource::GroundGdm);
            }
            if savegame_dir.join("densityMap_fruits.gdm").is_file() {
                available.push(FieldDetectionSource::FruitsGdm);
            }
        }
    }
    // Aktuelle Auswahl auf verfuegbare Quelle korrigieren
    if !available.contains(&state.ui.overview_options_dialog.field_detection_source) {
        state.ui.overview_options_dialog.field_detection_source = FieldDetectionSource::FromZip;
    }
    state.ui.overview_options_dialog.available_sources = available;
}

/// Schliesst den Uebersichtskarten-Options-Dialog.
pub fn close_overview_options_dialog(state: &mut AppState) {
    state.ui.overview_options_dialog.visible = false;
}

/// Schliesst den Post-Load-Dialog (Heightmap/ZIP-Erkennung).
pub fn dismiss_post_load_dialog(state: &mut AppState) {
    state.ui.post_load_dialog = Default::default();
}

/// Schliesst den "Als overview.png speichern"-Dialog.
pub fn dismiss_save_overview_dialog(state: &mut AppState) {
    state.ui.save_overview_dialog = Default::default();
}
