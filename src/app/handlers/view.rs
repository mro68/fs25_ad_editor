//! Handler für Kamera, Viewport und Background-Map.

use crate::app::use_cases;
use crate::app::AppState;
use crate::shared::RenderQuality;

pub fn reset_camera(state: &mut AppState) {
    use_cases::camera::reset_camera(state);
}

pub fn zoom_in(state: &mut AppState) {
    use_cases::camera::zoom_in(state);
}

pub fn zoom_out(state: &mut AppState) {
    use_cases::camera::zoom_out(state);
}

pub fn set_viewport_size(state: &mut AppState, size: [f32; 2]) {
    use_cases::viewport::resize(state, size);
}

pub fn pan(state: &mut AppState, delta: glam::Vec2) {
    use_cases::camera::pan(state, delta);
}

pub fn zoom_towards(state: &mut AppState, factor: f32, focus_world: Option<glam::Vec2>) {
    use_cases::camera::zoom_towards(state, factor, focus_world);
}

pub fn set_render_quality(state: &mut AppState, quality: RenderQuality) {
    use_cases::viewport::set_render_quality(state, quality);
}

pub fn load_background_map(state: &mut AppState, path: String, crop_size: Option<u32>) {
    if let Err(e) = use_cases::background_map::load_background_map(state, path, crop_size) {
        log::error!("Fehler beim Laden der Background-Map: {}", e);
    }
}

pub fn set_background_opacity(state: &mut AppState, opacity: f32) {
    use_cases::background_map::set_background_opacity(state, opacity);
}

pub fn toggle_background_visibility(state: &mut AppState) {
    use_cases::background_map::toggle_background_visibility(state);
}
