//! Use-Case-Funktionen fuer Background-Map-Verwaltung.

use crate::app::state::ZipBrowserState;
use crate::app::ui_contract::{DialogRequest, DialogRequestKind};
use crate::app::AppState;
use crate::core::{self, BackgroundMap, FarmlandGrid, FieldPolygon};
use crate::shared::OverviewFieldDetectionSource;
use anyhow::Result;
use glam::Vec2;
use image::GenericImageView;
use std::path::Path;
use std::sync::Arc;

fn apply_background_map(state: &mut AppState, bg_map: BackgroundMap) {
    let image_arc = bg_map.image_arc();
    state.view.background_map = Some(Arc::new(bg_map));
    state.view.background_scale = 1.0;
    state.view.mark_background_asset_changed();
    state.background_image = Some(image_arc);
}

fn clear_background_assets(state: &mut AppState) {
    let had_background = state.view.background_map.is_some() || state.background_image.is_some();
    state.view.background_map = None;
    if had_background {
        state.view.mark_background_asset_changed();
    }
    state.background_image = None;
}

fn persist_overview_defaults(state: &mut AppState) {
    if let Err(error) = super::options::save_editor_options(&state.options) {
        let message = format!(
            "Uebersichtskarten-Voreinstellungen konnten nicht gespeichert werden: {}",
            error
        );
        log::warn!("{}", message);
        state.ui.status_message = Some(message);
    }
}

fn map_overview_field_detection_source(
    source: OverviewFieldDetectionSource,
) -> fs25_map_overview::FieldDetectionSource {
    match source {
        OverviewFieldDetectionSource::FromZip => fs25_map_overview::FieldDetectionSource::FromZip,
        OverviewFieldDetectionSource::ZipGroundGdm => {
            fs25_map_overview::FieldDetectionSource::ZipGroundGdm
        }
        OverviewFieldDetectionSource::FieldTypeGrle => {
            fs25_map_overview::FieldDetectionSource::FieldTypeGrle
        }
        OverviewFieldDetectionSource::GroundGdm => {
            fs25_map_overview::FieldDetectionSource::GroundGdm
        }
        OverviewFieldDetectionSource::FruitsGdm => {
            fs25_map_overview::FieldDetectionSource::FruitsGdm
        }
    }
}

fn extract_field_polygons_from_source(
    zip_path: &str,
    savegame_dir: Option<&Path>,
    field_source: OverviewFieldDetectionSource,
) -> Option<(Vec<fs25_map_overview::FarmlandPolygon>, u32, u32)> {
    match map_overview_field_detection_source(field_source) {
        fs25_map_overview::FieldDetectionSource::FromZip => {
            log::info!("Feldpolygone: Quelle = infoLayer_farmlands (Map-ZIP)");
            None
        }
        fs25_map_overview::FieldDetectionSource::ZipGroundGdm => {
            log::info!("Feldpolygone: Quelle = densityMap_ground.gdm (Map-ZIP)");
            fs25_map_overview::try_extract_polygons_from_zip_ground_gdm(zip_path)
        }
        fs25_map_overview::FieldDetectionSource::FieldTypeGrle => savegame_dir.and_then(|dir| {
            let path = dir.join("infoLayer_fieldType.grle");
            log::info!("Feldpolygone: Quelle = {}", path.display());
            fs25_map_overview::try_extract_polygons_from_field_type_grle(&path)
        }),
        fs25_map_overview::FieldDetectionSource::GroundGdm => savegame_dir.and_then(|dir| {
            let path = dir.join("densityMap_ground.gdm");
            log::info!("Feldpolygone: Quelle = {}", path.display());
            fs25_map_overview::try_extract_polygons_from_ground_gdm(&path)
        }),
        fs25_map_overview::FieldDetectionSource::FruitsGdm => savegame_dir.and_then(|dir| {
            let path = dir.join("densityMap_fruits.gdm");
            log::info!("Feldpolygone: Quelle = {}", path.display());
            fs25_map_overview::try_extract_polygons_from_fruits_gdm(&path)
        }),
    }
}

/// Berechnet den JSON-Pfad fuer Farmland-Polygone parallel zur Bilddatei.
///
/// Ersetzt die Dateiendung durch `.json` (z.B. `overview.png` → `overview.json`).
fn json_path_for(image_path: &str) -> String {
    let p = Path::new(image_path);
    p.with_extension("json").to_string_lossy().into_owned()
}

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
    apply_background_map(state, bg_map);

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

    // ZIP-Browser schliessen (falls offen)
    state.ui.zip_browser = None;

    // Farmland-Polygone aus begleitender JSON-Datei neben dem ZIP laden (falls vorhanden)
    load_farmland_json(state, &zip_path);

    // Speichern als overview.png anbieten (falls XML geladen)
    prompt_save_as_overview(state);

    Ok(())
}

/// Generiert eine Uebersichtskarte mit den Optionen aus dem Dialog und laedt sie als Background.
///
/// Liest ZIP-Pfad, Layer-Optionen und die gewaehlte Feldpolygon-Quelle aus dem
/// `OverviewOptionsDialogState`, persistiert die Layer-Einstellungen in den
/// `EditorOptions` und generiert die Karte mit `fs25_map_overview`.
pub fn generate_overview_with_options(state: &mut AppState) -> Result<()> {
    let zip_path = state.ui.overview_options_dialog.zip_path.clone();
    let layers = state.ui.overview_options_dialog.layers.clone();
    let field_source = state.ui.overview_options_dialog.field_detection_source;

    log::info!("Generiere Uebersichtskarte aus: {}", zip_path);

    // Layer-Optionen persistent speichern
    state.options.overview_layers = layers.clone();
    state.options.overview_field_detection_source = field_source;
    state.refresh_options_arc();
    persist_overview_defaults(state);

    let options = fs25_map_overview::OverviewOptions {
        terrain: layers.terrain,
        hillshade: layers.hillshade,
        farmlands: layers.farmlands,
        farmland_ids: layers.farmland_ids,
        pois: layers.pois,
        legend: layers.legend,
    };

    let overview = fs25_map_overview::generate_overview_result_from_zip(&zip_path, &options)?;

    let (width, height) = overview.image.dimensions();
    log::info!("Uebersichtskarte generiert: {}x{} Pixel", width, height);

    // Savegame-Verzeichnis (Elternordner der aktuell geladenen Config)
    let savegame_dir = state.ui.current_file_path.as_ref().and_then(|xml_path| {
        Path::new(xml_path.as_str())
            .parent()
            .map(|p| p.to_path_buf())
    });

    // Feldpolygone gemaess gewaehlter Quelle extrahieren
    let extracted =
        extract_field_polygons_from_source(&zip_path, savegame_dir.as_deref(), field_source);

    // Rohe Polygone und Rasterdimensionen ermitteln
    let (raw_polygons, grle_w, grle_h) = match extracted {
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

    // FarmlandGrid aus rohen GRLE/PNG-IDs aufbauen (falls vorhanden)
    if let Some(ids) = overview.farmland_ids {
        state.farmland_grid = Some(Arc::new(FarmlandGrid::new(
            ids,
            grle_w.max(1),
            grle_h.max(1),
            overview.map_size,
        )));
        log::info!("FarmlandGrid gespeichert: {}x{} Pixel", grle_w, grle_h);
    } else {
        state.farmland_grid = None;
    }

    let bg_map = BackgroundMap::from_image(overview.image, &zip_path, None)?;
    apply_background_map(state, bg_map);

    // Dialog schliessen
    state.ui.overview_options_dialog.visible = false;

    // Speichern als overview.png anbieten (falls XML geladen)
    prompt_save_as_overview(state);

    Ok(())
}

/// Prueft ob dem User das Speichern als overview.png angeboten werden soll.
///
/// Zeigt Dialog immer an. Falls overview.png bereits existiert, wird der User
/// gefragt ob er die bestehende Datei ueberschreiben moechte.
fn prompt_save_as_overview(state: &mut AppState) {
    let Some(ref xml_path) = state.ui.current_file_path else {
        return;
    };
    let Some(dir) = Path::new(xml_path).parent() else {
        return;
    };
    let target = dir.join("overview.png");
    let is_overwrite = target.exists();
    state.ui.save_overview_dialog.visible = true;
    state.ui.save_overview_dialog.target_path = target.to_string_lossy().into_owned();
    state.ui.save_overview_dialog.is_overwrite = is_overwrite;
    log::info!(
        "Angebot: Hintergrund als overview.png speichern in {} (ueberschreiben: {})",
        dir.display(),
        is_overwrite,
    );
}

/// Speichert die aktuelle Background-Map als overview.png (verlustfreies PNG).
pub fn save_background_as_overview(state: &mut AppState, path: String) -> Result<()> {
    let bg_map = state
        .view
        .background_map
        .as_ref()
        .ok_or_else(|| anyhow::anyhow!("Keine Background-Map geladen"))?;

    let rgb_image = bg_map.image_data().to_rgb8();
    rgb_image.save(&path)?;

    log::info!("Background-Map als overview.png gespeichert: {}", path);

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

#[cfg(test)]
mod tests {
    use super::{browse_zip_background, load_background_from_zip, persist_overview_defaults};
    use crate::app::state::ZipBrowserState;
    use crate::app::AppState;
    use image::{DynamicImage, ImageFormat};
    use std::io::{Cursor, Write};
    use std::path::{Path, PathBuf};
    use std::time::{SystemTime, UNIX_EPOCH};

    struct TempDirGuard {
        path: PathBuf,
    }

    impl TempDirGuard {
        fn new(prefix: &str) -> Self {
            let timestamp = SystemTime::now()
                .duration_since(UNIX_EPOCH)
                .expect("Systemzeit muss nach Unix-Epoche liegen")
                .as_nanos();
            let path = std::env::temp_dir().join(format!(
                "fs25_auto_drive_engine_{}_{}_{}",
                prefix,
                std::process::id(),
                timestamp
            ));
            std::fs::create_dir_all(&path).expect("Temp-Verzeichnis muss erstellt werden");
            Self { path }
        }

        fn path(&self) -> &Path {
            &self.path
        }
    }

    impl Drop for TempDirGuard {
        fn drop(&mut self) {
            let _ = std::fs::remove_dir_all(&self.path);
        }
    }

    fn rgba_png_bytes(width: u32, height: u32, rgba: [u8; 4]) -> Vec<u8> {
        let image = DynamicImage::ImageRgba8(image::RgbaImage::from_pixel(
            width,
            height,
            image::Rgba(rgba),
        ));
        let mut cursor = Cursor::new(Vec::new());
        image
            .write_to(&mut cursor, ImageFormat::Png)
            .expect("PNG muss erzeugt werden");
        cursor.into_inner()
    }

    fn write_zip(path: &Path, entries: Vec<(&str, Vec<u8>)>) {
        let file = std::fs::File::create(path).expect("ZIP-Datei muss erstellt werden");
        let mut writer = zip::ZipWriter::new(file);

        for (name, bytes) in entries {
            writer
                .start_file(name, zip::write::SimpleFileOptions::default())
                .expect("ZIP-Eintrag muss angelegt werden");
            writer
                .write_all(&bytes)
                .expect("ZIP-Eintrag muss geschrieben werden");
        }

        writer.finish().expect("ZIP muss finalisiert werden");
    }

    #[test]
    fn persist_overview_layer_defaults_surfaces_save_errors() {
        let mut state = AppState::new();
        state.options.camera_zoom_min = state.options.camera_zoom_max;

        persist_overview_defaults(&mut state);

        let message = state
            .ui
            .status_message
            .as_deref()
            .expect("Persistenzfehler muss sichtbar gemacht werden");
        assert!(
            message.contains("Uebersichtskarten-Voreinstellungen konnten nicht gespeichert werden"),
            "Unerwartete Statusmeldung: {message}"
        );
    }

    #[test]
    fn browse_zip_background_loads_single_image_without_dialog() {
        let temp_dir = TempDirGuard::new("zip_single");
        let zip_path = temp_dir.path().join("single_background.zip");
        let zip_path_string = zip_path.to_string_lossy().into_owned();

        write_zip(
            &zip_path,
            vec![(
                "maps/overview.png",
                rgba_png_bytes(6, 4, [32, 96, 160, 255]),
            )],
        );

        let mut state = AppState::new();
        browse_zip_background(&mut state, zip_path_string)
            .expect("Single-Image-ZIP muss direkt geladen werden");

        assert!(state.ui.zip_browser.is_none());
        assert!(state.background_image.is_some());
        let background = state
            .view
            .background_map
            .as_ref()
            .expect("Background-Map muss geladen sein");
        assert_eq!(background.dimensions(), (6, 4));
    }

    #[test]
    fn browse_zip_background_opens_browser_for_multiple_images() {
        let temp_dir = TempDirGuard::new("zip_browser");
        let zip_path = temp_dir.path().join("multi_background.zip");
        let zip_path_string = zip_path.to_string_lossy().into_owned();

        write_zip(
            &zip_path,
            vec![
                ("maps/detail.png", vec![1; 8]),
                ("maps/overview.png", vec![2; 64]),
                ("maps/readme.txt", vec![3; 128]),
            ],
        );

        let mut state = AppState::new();
        browse_zip_background(&mut state, zip_path_string.clone())
            .expect("Multi-Image-ZIP muss den Browser oeffnen");

        let browser = state
            .ui
            .zip_browser
            .as_ref()
            .expect("ZIP-Browser muss geoeffnet werden");
        assert_eq!(browser.zip_path, zip_path_string);
        assert_eq!(browser.entries.len(), 2);
        assert_eq!(browser.entries[0].name, "maps/overview.png");
        assert_eq!(browser.entries[1].name, "maps/detail.png");
        assert!(browser.filter_overview);
        assert!(state.view.background_map.is_none());
        assert!(state.background_image.is_none());
    }

    #[test]
    fn load_background_from_zip_applies_crop_and_clears_browser() {
        let temp_dir = TempDirGuard::new("zip_crop");
        let zip_path = temp_dir.path().join("cropped_background.zip");
        let zip_path_string = zip_path.to_string_lossy().into_owned();

        write_zip(
            &zip_path,
            vec![("maps/detail.png", rgba_png_bytes(6, 4, [8, 24, 48, 255]))],
        );

        let mut state = AppState::new();
        state.ui.zip_browser = Some(ZipBrowserState {
            zip_path: zip_path_string.clone(),
            entries: Vec::new(),
            selected: Some(0),
            filter_overview: false,
        });

        load_background_from_zip(
            &mut state,
            zip_path_string,
            "maps/detail.png".to_string(),
            Some(2),
        )
        .expect("ZIP-Bild muss mit Crop geladen werden");

        assert!(state.ui.zip_browser.is_none());
        assert_eq!(state.view.background_asset_revision, 1);
        let background = state
            .view
            .background_map
            .as_ref()
            .expect("Background-Map muss geladen sein");
        assert_eq!(background.dimensions(), (2, 2));
        assert_eq!(
            state.background_image.as_ref().map(|image| image.width()),
            Some(2)
        );
    }
}
