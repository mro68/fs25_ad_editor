//! Use-Case-Funktionen für Background-Map-Verwaltung.

use crate::app::state::ZipBrowserState;
use crate::app::AppState;
use crate::core::{self, BackgroundMap};
use anyhow::Result;
use std::path::Path;
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
    state.view.background_scale = 1.0;
    state.view.background_dirty = true;

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
    state.view.background_scale = (state.view.background_scale * factor).clamp(0.125, 8.0);
    state.view.background_dirty = true;
    log::info!("Background-Scale: {:.3}", state.view.background_scale);
}

/// Entfernt die Background-Map.
pub fn clear_background_map(state: &mut AppState) {
    state.view.background_map = None;
    state.view.background_dirty = true;
    log::info!("Background-Map entfernt");
}

/// Öffnet den ZIP-Browser-Dialog: listet Bilddateien im Archiv auf.
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
        log::info!("ZIP enthält nur ein Bild — lade direkt: {}", entry_name);
        return load_background_from_zip(state, path, entry_name, None);
    }

    // Mehrere Bilder: Browser-Dialog öffnen
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

/// Lädt eine Bilddatei aus einem ZIP-Archiv als Background-Map.
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

    state.view.background_map = Some(Arc::new(bg_map));
    state.view.background_scale = 1.0;
    state.view.background_dirty = true;

    // ZIP-Browser schließen (falls offen)
    state.ui.zip_browser = None;

    // Speichern als overview.jpg anbieten (falls XML geladen)
    prompt_save_as_overview(state);

    Ok(())
}

/// Generiert eine Übersichtskarte mit den Optionen aus dem Dialog und lädt sie als Background.
///
/// Liest ZIP-Pfad und Layer-Optionen aus dem `OverviewOptionsDialogState`,
/// persistiert die Layer-Einstellungen in den `EditorOptions` und generiert
/// die Karte mit `fs25_map_overview`.
pub fn generate_overview_with_options(state: &mut AppState) -> Result<()> {
    let zip_path = state.ui.overview_options_dialog.zip_path.clone();
    let layers = state.ui.overview_options_dialog.layers.clone();

    log::info!("Generiere Übersichtskarte aus: {}", zip_path);

    // Layer-Optionen persistent speichern
    state.options.overview_layers = layers.clone();
    let config_path = crate::shared::EditorOptions::config_path();
    let _ = state.options.save_to_file(&config_path);

    let options = fs25_map_overview::OverviewOptions {
        hillshade: layers.hillshade,
        farmlands: layers.farmlands,
        farmland_ids: layers.farmland_ids,
        pois: layers.pois,
        legend: layers.legend,
    };

    let rgb_image = fs25_map_overview::generate_overview_from_zip(&zip_path, &options)?;

    let (width, height) = (rgb_image.width(), rgb_image.height());
    log::info!("Übersichtskarte generiert: {}x{} Pixel", width, height);

    let dynamic_image = image::DynamicImage::ImageRgb8(rgb_image);
    let bg_map = BackgroundMap::from_image(dynamic_image, &zip_path, None)?;

    state.view.background_map = Some(Arc::new(bg_map));
    state.view.background_scale = 1.0;
    state.view.background_dirty = true;

    // Dialog schließen
    state.ui.overview_options_dialog.visible = false;

    // Speichern als overview.jpg anbieten (falls XML geladen)
    prompt_save_as_overview(state);

    Ok(())
}

/// Prüft ob dem User das Speichern als overview.jpg angeboten werden soll.
///
/// Zeigt Dialog immer an. Falls overview.jpg bereits existiert, wird der User
/// gefragt ob er die bestehende Datei überschreiben möchte.
fn prompt_save_as_overview(state: &mut AppState) {
    let Some(ref xml_path) = state.ui.current_file_path else {
        return;
    };
    let Some(dir) = Path::new(xml_path).parent() else {
        return;
    };
    let target = dir.join("overview.jpg");
    let is_overwrite = target.exists();
    state.ui.save_overview_dialog.visible = true;
    state.ui.save_overview_dialog.target_path = target.to_string_lossy().into_owned();
    state.ui.save_overview_dialog.is_overwrite = is_overwrite;
    log::info!(
        "Angebot: Hintergrund als overview.jpg speichern in {} (überschreiben: {})",
        dir.display(),
        is_overwrite,
    );
}

/// Speichert die aktuelle Background-Map als overview.jpg (maximale Qualität).
pub fn save_background_as_overview(state: &mut AppState, path: String) -> Result<()> {
    let bg_map = state
        .view
        .background_map
        .as_ref()
        .ok_or_else(|| anyhow::anyhow!("Keine Background-Map geladen"))?;

    let rgb_image = bg_map.image_data().to_rgb8();
    let file = std::fs::File::create(&path)?;
    let encoder = image::codecs::jpeg::JpegEncoder::new_with_quality(file, 90);
    use image::ImageEncoder;
    encoder.write_image(
        rgb_image.as_raw(),
        rgb_image.width(),
        rgb_image.height(),
        image::ExtendedColorType::Rgb8,
    )?;

    log::info!("Background-Map als overview.jpg gespeichert: {}", path);
    Ok(())
}
