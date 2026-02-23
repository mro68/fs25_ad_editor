//! Handler für Kamera, Viewport und Background-Map.

use crate::app::use_cases;
use crate::app::AppState;
use crate::shared::RenderQuality;

/// Setzt die Kamera auf den Standardzustand zurück.
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

/// Aktualisiert die Viewport-Größe im State.
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

/// Setzt die Render-Qualitätsstufe.
pub fn set_render_quality(state: &mut AppState, quality: RenderQuality) {
    use_cases::viewport::set_render_quality(state, quality);
}

/// Lädt eine Background-Map und propagiert Fehler an den Aufrufer.
pub fn load_background_map(
    state: &mut AppState,
    path: String,
    crop_size: Option<u32>,
) -> anyhow::Result<()> {
    use_cases::background_map::load_background_map(state, path, crop_size)
}

/// Setzt die Transparenz der Background-Map.
pub fn set_background_opacity(state: &mut AppState, opacity: f32) {
    use_cases::background_map::set_background_opacity(state, opacity);
}

/// Schaltet die Sichtbarkeit der Background-Map um.
pub fn toggle_background_visibility(state: &mut AppState) {
    use_cases::background_map::toggle_background_visibility(state);
}

/// Öffnet den ZIP-Browser-Dialog für die gewählte ZIP-Datei.
pub fn browse_zip_background(state: &mut AppState, path: String) -> anyhow::Result<()> {
    use_cases::background_map::browse_zip_background(state, path)
}

/// Lädt eine Bilddatei aus einem ZIP-Archiv als Background-Map.
pub fn load_background_from_zip(
    state: &mut AppState,
    zip_path: String,
    entry_name: String,
    crop_size: Option<u32>,
) -> anyhow::Result<()> {
    use_cases::background_map::load_background_from_zip(state, zip_path, entry_name, crop_size)
}
