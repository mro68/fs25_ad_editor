//! Handler fuer Dialog-State und Anwendungssteuerung.

use crate::app::ui_contract::{DialogRequest, DialogRequestKind};
use crate::app::use_cases;
use crate::app::AppState;
use crate::shared::{
    EditorOptions, OverviewFieldDetectionSource, OverviewSourceContext, PostLoadDialogState,
};
use std::path::{Path, PathBuf};

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

/// Oeffnet den Dialog fuer das Nachzeichnen aller Felder.
pub fn open_trace_all_fields_dialog(state: &mut AppState) {
    state.ui.trace_all_fields_dialog.visible = true;
}

/// Schliesst den Dialog fuer das Nachzeichnen aller Felder.
pub fn close_trace_all_fields_dialog(state: &mut AppState) {
    state.ui.trace_all_fields_dialog.visible = false;
}

/// Oeffnet den Curseplay-Import-Dateidialog.
pub fn request_curseplay_import_dialog(state: &mut AppState) {
    state
        .ui
        .request_dialog(DialogRequest::pick_path(DialogRequestKind::CurseplayImport));
}

/// Oeffnet den Curseplay-Export-Dateidialog.
pub fn request_curseplay_export_dialog(state: &mut AppState) {
    state
        .ui
        .request_dialog(DialogRequest::pick_path(DialogRequestKind::CurseplayExport));
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
    state.ui.request_dialog(DialogRequest::OpenOptionsDialog);
}

/// Schliesst den Optionen-Dialog.
pub fn close_options_dialog(state: &mut AppState) {
    state.ui.request_dialog(DialogRequest::CloseOptionsDialog);
}

/// Uebernimmt neue Optionen und persistiert sie in der Konfigurationsdatei.
pub fn apply_options(state: &mut AppState, options: EditorOptions) -> anyhow::Result<()> {
    // Erst validieren, damit keine inkonsistenten Werte temporaer in den State gelangen.
    options.validate()?;
    state.set_options(options);
    use_cases::options::save_editor_options(&state.options)
}

/// Setzt Optionen auf Standardwerte zurueck und persistiert sie.
pub fn reset_options(state: &mut AppState) -> anyhow::Result<()> {
    state.set_options(EditorOptions::default());
    use_cases::options::save_editor_options(&state.options)
}

/// Schaltet die Sichtbarkeit der Command-Palette um.
pub fn toggle_command_palette(state: &mut AppState) {
    state.ui.request_dialog(DialogRequest::ToggleCommandPalette);
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

fn build_overview_source_dialog_state(
    context: OverviewSourceContext,
    heightmap_set: bool,
    heightmap_path: Option<String>,
    overview_loaded: bool,
    matching_zips: Vec<PathBuf>,
    map_name: String,
) -> PostLoadDialogState {
    PostLoadDialogState {
        visible: true,
        context,
        heightmap_set,
        heightmap_path,
        overview_loaded,
        matching_zips,
        selected_zip_index: 0,
        map_name,
    }
}

fn collect_available_overview_field_detection_sources(
    current_file_path: Option<&str>,
) -> Vec<OverviewFieldDetectionSource> {
    let mut available = vec![
        OverviewFieldDetectionSource::FromZip,
        OverviewFieldDetectionSource::ZipGroundGdm,
    ];

    if let Some(savegame_dir) = current_file_path.and_then(|xml_path| Path::new(xml_path).parent())
    {
        if savegame_dir.join("infoLayer_fieldType.grle").is_file() {
            available.push(OverviewFieldDetectionSource::FieldTypeGrle);
        }
        if savegame_dir.join("densityMap_ground.gdm").is_file() {
            available.push(OverviewFieldDetectionSource::GroundGdm);
        }
    }

    available
}

/// Oeffnet den wiederverwendbaren Overview-Source-Dialog fuer den Menue-Einstieg.
pub fn open_overview_source_dialog(state: &mut AppState) {
    state.ui.post_load_dialog = build_overview_source_dialog_state(
        OverviewSourceContext::ManualMenu,
        false,
        None,
        false,
        Vec::new(),
        String::new(),
    );
}

/// Oeffnet den Overview-Source-Dialog mit Auto-Detection-Ergebnissen nach dem Laden.
pub fn open_detected_overview_source_dialog(
    state: &mut AppState,
    heightmap_set: bool,
    heightmap_path: Option<String>,
    overview_loaded: bool,
    matching_zips: Vec<PathBuf>,
    map_name: String,
) {
    state.ui.post_load_dialog = build_overview_source_dialog_state(
        OverviewSourceContext::PostLoadDetected,
        heightmap_set,
        heightmap_path,
        overview_loaded,
        matching_zips,
        map_name,
    );
}

/// Oeffnet den nativen Uebersichtskarten-ZIP-Auswahl-Dialog.
pub fn request_overview_dialog(state: &mut AppState) {
    state
        .ui
        .request_dialog(DialogRequest::pick_path(DialogRequestKind::OverviewZip));
}

/// Oeffnet den Uebersichtskarten-Options-Dialog mit dem gewaehlten ZIP-Pfad.
///
/// Befuellt ZIP- und Savegame-basierte Feldquellen und lädt die persistierten
/// Standardwerte aus den Editor-Optionen.
pub fn open_overview_options_dialog(state: &mut AppState, zip_path: String) {
    state.ui.overview_options_dialog.visible = true;
    state.ui.overview_options_dialog.zip_path = zip_path;
    state.ui.overview_options_dialog.layers = state.options.overview_layers.clone();
    state.ui.overview_options_dialog.field_detection_source =
        state.options.overview_field_detection_source;

    let available =
        collect_available_overview_field_detection_sources(state.ui.current_file_path.as_deref());
    if !available.contains(&state.ui.overview_options_dialog.field_detection_source) {
        state.ui.overview_options_dialog.field_detection_source =
            OverviewFieldDetectionSource::default();
    }
    if !available.contains(&state.ui.overview_options_dialog.field_detection_source) {
        state.ui.overview_options_dialog.field_detection_source = available
            .first()
            .copied()
            .unwrap_or_else(OverviewFieldDetectionSource::default);
    }
    state.ui.overview_options_dialog.available_sources = available;
}

/// Schliesst den Uebersichtskarten-Options-Dialog.
pub fn close_overview_options_dialog(state: &mut AppState) {
    state.ui.overview_options_dialog.visible = false;
}

/// Schliesst den wiederverwendbaren Overview-Source-Dialog.
pub fn dismiss_post_load_dialog(state: &mut AppState) {
    state.ui.post_load_dialog = Default::default();
}

/// Schliesst den "Als overview.png speichern"-Dialog.
pub fn dismiss_save_overview_dialog(state: &mut AppState) {
    state.ui.save_overview_dialog = Default::default();
}
