//! Handler für Dialog-State und Anwendungssteuerung.

use crate::app::use_cases;
use crate::app::AppState;
use crate::shared::EditorOptions;

pub fn request_exit(state: &mut AppState) {
    state.should_exit = true;
}

pub fn request_heightmap_dialog(state: &mut AppState) {
    use_cases::heightmap::request_heightmap_dialog(state);
}

pub fn request_background_map_dialog(state: &mut AppState) {
    use_cases::background_map::request_background_map_dialog(state);
}

pub fn dismiss_heightmap_warning(state: &mut AppState) {
    use_cases::heightmap::dismiss_heightmap_warning(state);
}

pub fn close_marker_dialog(state: &mut AppState) {
    state.ui.show_marker_dialog = false;
    state.ui.marker_dialog_node_id = None;
}

pub fn open_options_dialog(state: &mut AppState) {
    state.show_options_dialog = true;
}

pub fn close_options_dialog(state: &mut AppState) {
    state.show_options_dialog = false;
}

pub fn apply_options(state: &mut AppState, options: EditorOptions) {
    state.options = options;
    let path = EditorOptions::config_path();
    if let Err(e) = state.options.save_to_file(&path) {
        log::error!("Optionen konnten nicht gespeichert werden: {}", e);
    }
}

pub fn reset_options(state: &mut AppState) {
    state.options = EditorOptions::default();
    let path = EditorOptions::config_path();
    if let Err(e) = state.options.save_to_file(&path) {
        log::error!("Optionen konnten nicht gespeichert werden: {}", e);
    }
}

pub fn dismiss_dedup_dialog(state: &mut AppState) {
    state.ui.show_dedup_dialog = false;
    state.ui.status_message = None;
}
