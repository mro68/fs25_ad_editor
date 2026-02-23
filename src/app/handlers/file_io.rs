//! Handler für Datei-Operationen (Öffnen, Speichern, Heightmap).

use crate::app::use_cases;
use crate::app::AppState;
use std::path::Path;

/// Öffnet den Datei-Öffnen-Dialog.
pub fn request_open(state: &mut AppState) {
    use_cases::file_io::request_open_file(state);
}

/// Öffnet den Datei-Speichern-Dialog.
pub fn request_save(state: &mut AppState) {
    use_cases::file_io::request_save_file(state);
}

/// Bestätigt Heightmap-Warnung und führt Speichern aus.
pub fn confirm_and_save(state: &mut AppState) -> anyhow::Result<()> {
    use_cases::file_io::confirm_and_save(state)
}

/// Lädt eine RoadMap aus dem übergebenen Pfad.
///
/// Nach dem Laden wird automatisch geprüft, ob eine Heightmap und/oder
/// ein passender Map-Mod-ZIP im Mods-Verzeichnis vorhanden sind.
pub fn load(state: &mut AppState, path: String) -> anyhow::Result<()> {
    use_cases::file_io::load_selected_file(state, path.clone())?;
    run_post_load_detection(state, &path);
    Ok(())
}

/// Führt die automatische Erkennung von Heightmap und Map-Mod-ZIP durch.
fn run_post_load_detection(state: &mut AppState, xml_path: &str) {
    let xml_path = Path::new(xml_path);
    let map_name = state
        .road_map
        .as_ref()
        .and_then(|rm| rm.map_name.as_deref());

    let result = use_cases::auto_detect::detect_post_load(xml_path, map_name);

    let heightmap_set = result.heightmap_path.is_some();
    let heightmap_display = result
        .heightmap_path
        .as_ref()
        .and_then(|p| p.to_str())
        .map(String::from);

    // Heightmap lautlos setzen (non-destructive)
    if let Some(ref hm_path) = result.heightmap_path {
        if let Some(hm_str) = hm_path.to_str() {
            state.ui.heightmap_path = Some(hm_str.to_string());
            log::info!("Heightmap auto-detected: {}", hm_str);
        }
    }

    // Dialog nur anzeigen wenn etwas erkannt wurde
    let has_zips = !result.matching_zips.is_empty();
    if heightmap_set || has_zips {
        state.ui.post_load_dialog.visible = true;
        state.ui.post_load_dialog.heightmap_set = heightmap_set;
        state.ui.post_load_dialog.heightmap_path = heightmap_display;
        state.ui.post_load_dialog.matching_zips = result.matching_zips;
        state.ui.post_load_dialog.selected_zip_index = 0;
        state.ui.post_load_dialog.map_name = map_name.unwrap_or("").to_string();
    }
}

/// Speichert die RoadMap unter dem übergebenen Pfad (inkl. Heightmap-Check).
///
/// `None` speichert unter dem aktuell bekannten Pfad (oder öffnet den Dialog).
/// `Some(p)` speichert explizit unter dem neuen Pfad `p`.
pub fn save(state: &mut AppState, path: Option<String>) -> anyhow::Result<()> {
    use_cases::file_io::save_with_heightmap_check(state, path)
}

/// Entfernt die aktuell gesetzte Heightmap.
pub fn clear_heightmap(state: &mut AppState) {
    use_cases::heightmap::clear_heightmap(state);
}

/// Setzt eine Heightmap aus einem Dateipfad.
pub fn set_heightmap(state: &mut AppState, path: String) {
    use_cases::heightmap::set_heightmap(state, path);
}

/// Führt die Duplikat-Bereinigung auf der geladenen RoadMap aus.
pub fn deduplicate(state: &mut AppState) {
    use_cases::file_io::deduplicate_loaded_roadmap(state);
}
