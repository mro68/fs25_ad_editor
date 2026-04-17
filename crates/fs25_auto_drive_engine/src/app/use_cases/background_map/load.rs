//! Use-Cases fuer das Laden von Background-Maps aus Dateien und ZIP-Archiven.

use super::field_polygons::{load_farmland_json, load_farmland_json_for_overview_dir};
use super::state_helpers::{apply_background_map, clear_background_assets};
use crate::app::state::ZipBrowserState;
use crate::app::ui_contract::{DialogRequest, DialogRequestKind};
use crate::app::AppState;
use crate::core;
use anyhow::Result;
use std::path::Path;

/// Oeffnet den Background-Map-Auswahl-Dialog.
pub fn request_background_map_dialog(state: &mut AppState) {
    state
        .ui
        .request_dialog(DialogRequest::pick_path(DialogRequestKind::BackgroundMap));
}

/// Laedt eine Background-Map von einem Dateipfad.
///
/// # Parameter
/// - `state`: Application State
/// - `path`: Pfad zur Bilddatei (PNG, JPG, JPEG, DDS)
/// - `crop_size`: Optionale Crop-Groesse (quadratisch, in Pixeln)
pub fn load_background_map(
    state: &mut AppState,
    path: String,
    crop_size: Option<u32>,
) -> Result<()> {
    log::info!("Lade Background-Map: {} (Crop: {:?})", path, crop_size);

    // Lade Background-Map
    let bg_map = core::BackgroundMap::load_from_file(&path, crop_size)?;

    // Log Informationen
    let (width, height) = bg_map.dimensions();
    log::info!(
        "Background-Map erfolgreich geladen: {}x{} Pixel, Weltkoordinaten: {:?}",
        width,
        height,
        bg_map.world_bounds()
    );

    // Speichere in State
    apply_background_map(state, bg_map);
    state.background_layers = None;
    state.pending_overview_bundle = None;

    let layer_bundle_loaded = Path::new(&path)
        .parent()
        .map(|dir| {
            match super::super::background_layers::try_load_background_layer_bundle_from_directory(
                state, dir,
            ) {
                Ok(true) => {
                    log::info!(
                        "Gespeichertes Background-Layer-Bundle erkannt und aktiviert: {}",
                        dir.display()
                    );
                    true
                }
                Ok(false) => false,
                Err(error) => {
                    log::warn!(
                        "Gespeichertes Background-Layer-Bundle konnte nicht geladen werden: {}",
                        error
                    );
                    false
                }
            }
        })
        .unwrap_or(false);

    // Farmland-Polygone aus begleitender JSON-Datei laden (falls vorhanden)
    if layer_bundle_loaded {
        if let Some(dir) = Path::new(&path).parent() {
            load_farmland_json_for_overview_dir(state, dir);
        }
    } else {
        load_farmland_json(state, &path);
    }

    Ok(())
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

/// Skaliert die Ausdehnung der Background-Map (relativ).
///
/// Multipliziert den aktuellen Skalierungsfaktor mit `factor`.
/// Begrenzt auf den Bereich 0.125 bis 8.0.
pub fn scale_background(state: &mut AppState, factor: f32) {
    let next_scale = (state.view.background_scale * factor).clamp(0.125, 8.0);
    if (next_scale - state.view.background_scale).abs() <= f32::EPSILON {
        return;
    }

    state.view.background_scale = next_scale;
    if state.view.background_map.is_some() {
        state.view.mark_background_transform_changed();
    }
    log::info!("Background-Scale: {:.3}", state.view.background_scale);
}

/// Entfernt die Background-Map.
pub fn clear_background_map(state: &mut AppState) {
    clear_background_assets(state);
    state.background_layers = None;
    state.pending_overview_bundle = None;
    log::info!("Background-Map entfernt");
}

/// Oeffnet den ZIP-Browser-Dialog: listet Bilddateien im Archiv auf.
///
/// Bei genau einem Treffer wird die Datei direkt geladen (kein Dialog).
pub fn browse_zip_background(state: &mut AppState, path: String) -> Result<()> {
    let entries = core::list_images_in_zip(&path)?;

    if entries.is_empty() {
        anyhow::bail!("Keine Bilddateien im ZIP-Archiv gefunden: {}", path);
    }

    // Bei genau einem Bild: direkt laden, ohne Browser-Dialog
    if entries.len() == 1 {
        let entry_name = entries.into_iter().next().unwrap().name;
        log::info!("ZIP enthaelt nur ein Bild — lade direkt: {}", entry_name);
        return load_background_from_zip(state, path, entry_name, None);
    }

    // Mehrere Bilder: Browser-Dialog oeffnen
    let has_overview = entries
        .iter()
        .any(|e| e.name.to_lowercase().contains("overview"));
    state.ui.zip_browser = Some(ZipBrowserState {
        zip_path: path,
        entries,
        selected: None,
        filter_overview: has_overview,
    });

    Ok(())
}

/// Laedt eine Bilddatei aus einem ZIP-Archiv als Background-Map.
pub fn load_background_from_zip(
    state: &mut AppState,
    zip_path: String,
    entry_name: String,
    crop_size: Option<u32>,
) -> Result<()> {
    log::info!(
        "Lade Background-Map aus ZIP: {}:{} (Crop: {:?})",
        zip_path,
        entry_name,
        crop_size
    );

    let bg_map = core::load_from_zip(&zip_path, &entry_name, crop_size)?;

    let (width, height) = bg_map.dimensions();
    log::info!(
        "Background-Map aus ZIP geladen: {}x{} Pixel, Weltkoordinaten: {:?}",
        width,
        height,
        bg_map.world_bounds()
    );

    apply_background_map(state, bg_map);
    state.background_layers = None;
    state.pending_overview_bundle = None;

    // ZIP-Browser schliessen (falls offen)
    state.ui.zip_browser = None;

    // Farmland-Polygone aus begleitender JSON-Datei neben dem ZIP laden (falls vorhanden)
    load_farmland_json(state, &zip_path);

    // Speichern als overview.png anbieten (falls XML geladen)
    super::generate::prompt_save_as_overview(state);

    Ok(())
}
