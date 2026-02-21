//! Handler für Datei-Operationen (Öffnen, Speichern, Heightmap).

use crate::app::use_cases;
use crate::app::AppState;

pub fn request_open(state: &mut AppState) {
    use_cases::file_io::request_open_file(state);
}

pub fn request_save(state: &mut AppState) {
    use_cases::file_io::request_save_file(state);
}

pub fn confirm_and_save(state: &mut AppState) -> anyhow::Result<()> {
    use_cases::file_io::confirm_and_save(state)
}

pub fn load(state: &mut AppState, path: String) -> anyhow::Result<()> {
    use_cases::file_io::load_selected_file(state, path)
}

pub fn save(state: &mut AppState, path: String) -> anyhow::Result<()> {
    use_cases::file_io::save_with_heightmap_check(state, path)
}

pub fn clear_heightmap(state: &mut AppState) {
    use_cases::heightmap::clear_heightmap(state);
}

pub fn set_heightmap(state: &mut AppState, path: String) {
    use_cases::heightmap::set_heightmap(state, path);
}

pub fn deduplicate(state: &mut AppState) {
    use_cases::file_io::deduplicate_loaded_roadmap(state);
}
