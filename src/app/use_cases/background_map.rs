//! Use-Case-Funktionen fuer Background-Map-Verwaltung.

use crate::app::state::ZipBrowserState;
use crate::app::AppState;
use crate::core::{self, BackgroundMap, FieldPolygon};
use anyhow::Result;
use glam::Vec2;
use image::GenericImageView;
use std::path::Path;
use std::sync::Arc;

/// Berechnet den JSON-Pfad fuer Farmland-Polygone parallel zur Bilddatei.
///
/// Ersetzt die Dateiendung durch `.json` (z.B. `overview.jpg` → `overview.json`).
fn json_path_for(image_path: &str) -> String {
    let p = Path::new(image_path);
    p.with_extension("json").to_string_lossy().into_owned()
}

/// Oeffnet den Background-Map-Auswahl-Dialog.
pub fn request_background_map_dialog(state: &mut AppState) {
    state.ui.show_background_map_dialog = true;
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

    // Farmland-Polygone aus begleitender JSON-Datei laden (falls vorhanden)
    load_farmland_json(state, &path);

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

    state.view.background_map = Some(Arc::new(bg_map));
    state.view.background_scale = 1.0;
    state.view.background_dirty = true;

    // ZIP-Browser schliessen (falls offen)
    state.ui.zip_browser = None;

    // Farmland-Polygone aus begleitender JSON-Datei neben dem ZIP laden (falls vorhanden)
    load_farmland_json(state, &zip_path);

    // Speichern als overview.jpg anbieten (falls XML geladen)
    prompt_save_as_overview(state);

    Ok(())
}

/// Generiert eine Uebersichtskarte mit den Optionen aus dem Dialog und laedt sie als Background.
///
/// Liest ZIP-Pfad und Layer-Optionen aus dem `OverviewOptionsDialogState`,
/// persistiert die Layer-Einstellungen in den `EditorOptions` und generiert
/// die Karte mit `fs25_map_overview`.
pub fn generate_overview_with_options(state: &mut AppState) -> Result<()> {
    let zip_path = state.ui.overview_options_dialog.zip_path.clone();
    let layers = state.ui.overview_options_dialog.layers.clone();

    log::info!("Generiere Uebersichtskarte aus: {}", zip_path);

    // Layer-Optionen persistent speichern
    state.options.overview_layers = layers.clone();
    state.refresh_options_arc();
    let config_path = crate::shared::EditorOptions::config_path();
    let _ = state.options.save_to_file(&config_path);

    let options = fs25_map_overview::OverviewOptions {
        hillshade: layers.hillshade,
        farmlands: layers.farmlands,
        farmland_ids: layers.farmland_ids,
        pois: layers.pois,
        legend: layers.legend,
    };

    let overview = fs25_map_overview::generate_overview_result_from_zip(&zip_path, &options)?;

    let (width, height) = overview.image.dimensions();
    log::info!("Uebersichtskarte generiert: {}x{} Pixel", width, height);

    // Feldpolygone: Prioritaet 1 – infoLayer_fieldType.grle aus Savegame-Ordner;
    // Fallback: Farmland-Polygone aus dem Map-ZIP.
    let field_type_source = state
        .ui
        .current_file_path
        .as_ref()
        .and_then(|xml_path| Path::new(xml_path.as_str()).parent().map(|p| p.to_path_buf()))
        .and_then(|savegame_dir| {
            let grle_path = savegame_dir.join("infoLayer_fieldType.grle");
            if grle_path.is_file() {
                log::info!(
                    "Savegame-FieldType-GRLE gefunden: {}",
                    grle_path.display()
                );
                fs25_map_overview::try_extract_polygons_from_field_type_grle(&grle_path)
            } else {
                log::info!(
                    "infoLayer_fieldType.grle nicht vorhanden – verwende Farmland-Polygone aus ZIP"
                );
                None
            }
        });

    // Rohe Polygone und Rasterdimensionen ermitteln (aus FieldType oder Farmland-ZIP)
    let (raw_polygons, grle_w, grle_h) = match field_type_source {
        Some((polygons, w, h)) => (polygons, w, h),
        None => (
            overview.farmland_polygons,
            overview.grle_width,
            overview.grle_height,
        ),
    };

    // Pixel → Weltkoordinaten: world = pixel * (map_size / grle_size) - map_size / 2
    if !raw_polygons.is_empty() {
        let scale_x = overview.map_size / grle_w.max(1) as f32;
        let scale_y = overview.map_size / grle_h.max(1) as f32;
        let half = overview.map_size / 2.0;

        let field_polygons: Vec<FieldPolygon> = raw_polygons
            .into_iter()
            .map(|fp| FieldPolygon {
                id: fp.id,
                vertices: fp
                    .vertices
                    .into_iter()
                    .map(|(px, py)| Vec2::new(px * scale_x - half, py * scale_y - half))
                    .collect(),
            })
            .collect();

        log::info!(
            "Feldpolygone in Weltkoordinaten umgerechnet: {} Felder",
            field_polygons.len()
        );
        state.farmland_polygons = Some(Arc::new(field_polygons));
    } else {
        state.farmland_polygons = None;
    }

    let bg_map = BackgroundMap::from_image(overview.image, &zip_path, None)?;

    state.view.background_map = Some(Arc::new(bg_map));
    state.view.background_scale = 1.0;
    state.view.background_dirty = true;

    // Dialog schliessen
    state.ui.overview_options_dialog.visible = false;

    // Speichern als overview.jpg anbieten (falls XML geladen)
    prompt_save_as_overview(state);

    Ok(())
}

/// Prueft ob dem User das Speichern als overview.jpg angeboten werden soll.
///
/// Zeigt Dialog immer an. Falls overview.jpg bereits existiert, wird der User
/// gefragt ob er die bestehende Datei ueberschreiben moechte.
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
        "Angebot: Hintergrund als overview.jpg speichern in {} (ueberschreiben: {})",
        dir.display(),
        is_overwrite,
    );
}

/// Speichert die aktuelle Background-Map als overview.jpg (maximale Qualitaet).
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

    // Farmland-Polygone als JSON parallel zur Bilddatei speichern
    save_farmland_json(state, &path);

    Ok(())
}

/// Speichert Farmland-Polygone als JSON-Datei neben der Bilddatei.
///
/// Pfad wird aus dem Bildpfad abgeleitet (z.B. `overview.jpg` → `overview.json`).
/// Falls keine Polygone vorhanden sind, wird nichts geschrieben.
fn save_farmland_json(state: &AppState, image_path: &str) {
    let Some(ref polygons) = state.farmland_polygons else {
        return;
    };
    let json_path = json_path_for(image_path);
    match serde_json::to_string(polygons.as_ref()) {
        Ok(json) => match std::fs::write(&json_path, json) {
            Ok(()) => log::info!(
                "Farmland-Polygone gespeichert: {} ({} Felder)",
                json_path,
                polygons.len()
            ),
            Err(e) => log::warn!("Farmland-JSON konnte nicht geschrieben werden: {}", e),
        },
        Err(e) => log::warn!("Farmland-Polygone konnten nicht serialisiert werden: {}", e),
    }
}

/// Laedt Farmland-Polygone aus einer JSON-Datei neben der Bilddatei.
///
/// Prueft ob eine `.json`-Datei neben dem Bildpfad existiert und liest
/// die Polygon-Daten ein. Wird beim Auto-Load der overview.jpg aufgerufen.
pub fn load_farmland_json(state: &mut AppState, image_path: &str) {
    let json_path = json_path_for(image_path);
    let path = Path::new(&json_path);
    if !path.is_file() {
        return;
    }
    match std::fs::read_to_string(path) {
        Ok(content) => match serde_json::from_str::<Vec<FieldPolygon>>(&content) {
            Ok(polygons) => {
                log::info!(
                    "Farmland-Polygone geladen: {} ({} Felder)",
                    json_path,
                    polygons.len()
                );
                state.farmland_polygons = Some(Arc::new(polygons));
            }
            Err(e) => log::warn!("Farmland-JSON konnte nicht deserialisiert werden: {}", e),
        },
        Err(e) => log::warn!("Farmland-JSON konnte nicht gelesen werden: {}", e),
    }
}
