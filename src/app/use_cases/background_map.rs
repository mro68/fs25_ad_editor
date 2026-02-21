//! Use-Case-Funktionen für Background-Map-Verwaltung.

use crate::app::AppState;
use crate::core::BackgroundMap;
use anyhow::Result;
use std::sync::Arc;

/// Öffnet den Background-Map-Auswahl-Dialog.
pub fn request_background_map_dialog(state: &mut AppState) {
    state.ui.show_background_map_dialog = true;
}

/// Lädt eine Background-Map von einem Dateipfad.
///
/// # Parameter
/// - `state`: Application State
/// - `path`: Pfad zur Bilddatei (PNG, JPG, JPEG, DDS)
/// - `crop_size`: Optionale Crop-Größe (quadratisch, in Pixeln)
pub fn load_background_map(
    state: &mut AppState,
    path: String,
    crop_size: Option<u32>,
) -> Result<()> {
    log::info!("Lade Background-Map: {} (Crop: {:?})", path, crop_size);

    // Lade Background-Map
    let bg_map = BackgroundMap::load_from_file(&path, crop_size)?;

    // Log Informationen
    let (width, height) = bg_map.dimensions();
    log::info!(
        "Background-Map erfolgreich geladen: {}x{} Pixel, Weltkoordinaten: {:?}",
        width,
        height,
        bg_map.world_bounds()
    );

    // Speichere in State
    state.view.background_map = Some(Arc::new(bg_map));
    state.view.background_dirty = true;

    Ok(())
}

/// Setzt die Opacity der Background-Map.
pub fn set_background_opacity(state: &mut AppState, opacity: f32) {
    state.view.background_opacity = opacity.clamp(0.0, 1.0);
    log::debug!(
        "Background-Opacity gesetzt: {:.2}",
        state.view.background_opacity
    );
}

/// Schaltet die Sichtbarkeit der Background-Map um.
pub fn toggle_background_visibility(state: &mut AppState) {
    state.view.background_visible = !state.view.background_visible;
    log::info!(
        "Background-Sichtbarkeit: {}",
        if state.view.background_visible {
            "an"
        } else {
            "aus"
        }
    );
}

/// Entfernt die Background-Map.
pub fn clear_background_map(state: &mut AppState) {
    state.view.background_map = None;
    state.view.background_dirty = true;
    log::info!("Background-Map entfernt");
}
