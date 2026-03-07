//! Handler fuer Kamera, Viewport und Background-Map.

use crate::app::use_cases;
use crate::app::AppState;
use crate::shared::RenderQuality;

/// Setzt die Kamera auf den Standardzustand zurueck.
pub fn reset_camera(state: &mut AppState) {
    use_cases::camera::reset_camera(state);
}

/// Zoomt stufenweise hinein.
pub fn zoom_in(state: &mut AppState) {
    use_cases::camera::zoom_in(state);
}

/// Zoomt stufenweise heraus.
pub fn zoom_out(state: &mut AppState) {
    use_cases::camera::zoom_out(state);
}

/// Aktualisiert die Viewport-Groesse im State.
pub fn set_viewport_size(state: &mut AppState, size: [f32; 2]) {
    use_cases::viewport::resize(state, size);
}

/// Verschiebt die Kamera um ein Weltkoordinaten-Delta.
pub fn pan(state: &mut AppState, delta: glam::Vec2) {
    use_cases::camera::pan(state, delta);
}

/// Zoomt mit optionalem Fokuspunkt im Weltkoordinatensystem.
pub fn zoom_towards(state: &mut AppState, factor: f32, focus_world: Option<glam::Vec2>) {
    use_cases::camera::zoom_towards(state, factor, focus_world);
}

/// Setzt die Render-Qualitaetsstufe.
pub fn set_render_quality(state: &mut AppState, quality: RenderQuality) {
    use_cases::viewport::set_render_quality(state, quality);
}

/// Laedt eine Background-Map und propagiert Fehler an den Aufrufer.
pub fn load_background_map(
    state: &mut AppState,
    path: String,
    crop_size: Option<u32>,
) -> anyhow::Result<()> {
    use_cases::background_map::load_background_map(state, path, crop_size)
}

/// Schaltet die Sichtbarkeit der Background-Map um.
pub fn toggle_background_visibility(state: &mut AppState) {
    use_cases::background_map::toggle_background_visibility(state);
}

/// Skaliert die Ausdehnung der Background-Map (relativ).
pub fn scale_background(state: &mut AppState, factor: f32) {
    use_cases::background_map::scale_background(state, factor);
}

/// Oeffnet den ZIP-Browser-Dialog fuer die gewaehlte ZIP-Datei.
pub fn browse_zip_background(state: &mut AppState, path: String) -> anyhow::Result<()> {
    use_cases::background_map::browse_zip_background(state, path)
}

/// Laedt eine Bilddatei aus einem ZIP-Archiv als Background-Map.
pub fn load_background_from_zip(
    state: &mut AppState,
    zip_path: String,
    entry_name: String,
    crop_size: Option<u32>,
) -> anyhow::Result<()> {
    use_cases::background_map::load_background_from_zip(state, zip_path, entry_name, crop_size)
}

/// Generiert eine Uebersichtskarte mit den Optionen aus dem Dialog.
pub fn generate_overview_with_options(state: &mut AppState) -> anyhow::Result<()> {
    use_cases::background_map::generate_overview_with_options(state)
}

/// Speichert die aktuelle Background-Map als overview.jpg.
pub fn save_background_as_overview(state: &mut AppState, path: String) -> anyhow::Result<()> {
    use_cases::background_map::save_background_as_overview(state, path)
}

/// Schaltet das Stra\u00dfenoverlay auf der Hintergrundkarte ein oder aus.
///
/// Setzt `background_dirty = true`, damit die GPU-Textur
/// beim naechsten Frame neu aufgebaut wird.
pub fn toggle_road_overlay(state: &mut AppState) {
    state.view.show_road_overlay = !state.view.show_road_overlay;
    state.view.background_dirty = true;
    log::info!(
        "Strassenoverlay: {}",
        if state.view.show_road_overlay {
            "an"
        } else {
            "aus"
        }
    );
}
