//! Handler für Datei-Operationen (Öffnen, Speichern, Heightmap).

use crate::app::use_cases;
use crate::app::AppState;

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
pub fn load(state: &mut AppState, path: String) -> anyhow::Result<()> {
    use_cases::file_io::load_selected_file(state, path)
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
