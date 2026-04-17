use crate::app::use_cases;
use crate::app::AppState;

/// Kopiert die Selektion (Nodes, Verbindungen, Marker) in die Zwischenablage.
pub fn copy_selection(state: &mut AppState) {
    use_cases::editing::copy_selected_to_clipboard(state);
}

/// Aktiviert den Einfuegen-Vorschau-Modus.
pub fn start_paste_preview(state: &mut AppState) {
    use_cases::editing::start_paste_preview(state);
}

/// Aktualisiert die Einfuegen-Vorschauposition.
pub fn update_paste_preview(state: &mut AppState, world_pos: glam::Vec2) {
    use_cases::editing::update_paste_preview(state, world_pos);
}

/// Bestaetigt das Einfuegen an der aktuellen Vorschauposition.
pub fn confirm_paste(state: &mut AppState) {
    use_cases::editing::confirm_paste(state);
}

/// Bricht die Einfuegen-Vorschau ab.
pub fn cancel_paste_preview(state: &mut AppState) {
    use_cases::editing::cancel_paste_preview(state);
}

/// Importiert eine Curseplay-XML-Datei und legt Nodes + Ring-Verbindungen an.
pub fn import_curseplay_file(state: &mut AppState, path: &str) {
    use_cases::editing::import_curseplay(state, path);
}

/// Exportiert die selektierten Nodes als Curseplay-XML-Datei.
pub fn export_curseplay_file(state: &AppState, path: &str) {
    use_cases::editing::export_curseplay(state, path);
}
