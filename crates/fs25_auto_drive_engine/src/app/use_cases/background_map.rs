//! Use-Case-Funktionen fuer Background-Map-Verwaltung.

use crate::app::state::{PendingOverviewBundle, ZipBrowserState};
use crate::app::ui_contract::{DialogRequest, DialogRequestKind};
use crate::app::AppState;
use crate::core::{self, BackgroundMap, FarmlandGrid, FieldPolygon};
use crate::shared::{BackgroundLayerKind, OverviewFieldDetectionSource};
use anyhow::{Context, Result};
use glam::Vec2;
use image::DynamicImage;
use std::path::Path;
use std::sync::Arc;

fn apply_background_map(state: &mut AppState, bg_map: BackgroundMap) {
    apply_background_map_with_scale(state, bg_map, 1.0);
}

fn apply_background_map_with_scale(state: &mut AppState, bg_map: BackgroundMap, scale: f32) {
    let image_arc = bg_map.image_arc();
    state.view.background_map = Some(Arc::new(bg_map));
    state.view.background_scale = scale;
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

fn load_farmland_json_for_overview_dir(state: &mut AppState, dir: &Path) {
    let overview_path = dir.join("overview.png");
    let overview_path = overview_path.to_string_lossy().into_owned();
    load_farmland_json(state, &overview_path);
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
    state.background_layers = None;
    state.pending_overview_bundle = None;

    let layer_bundle_loaded = Path::new(&path)
        .parent()
        .map(
            |dir| match super::background_layers::try_load_background_layer_bundle_from_directory(
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
            },
        )
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
    prompt_save_as_overview(state);

    Ok(())
}

/// Generiert eine Uebersichtskarte mit den Optionen aus dem Dialog und laedt sie als Background.
///
/// Liest ZIP-Pfad, Layer-Optionen und die gewaehlte Feldpolygon-Quelle aus dem
/// `OverviewOptionsDialogState`, persistiert die Layer-Einstellungen in den
/// `EditorOptions` und generiert die Karte mit `fs25_map_overview`.
/// Die einzelnen Layer-PNGs werden sofort persistiert; im State bleiben danach
/// nur das Preview-Bild, der Layer-Katalog und ein Pending-Marker aktiv.
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

    let bundle = fs25_map_overview::generate_overview_layer_bundle_from_zip(&zip_path, &options)?;

    let (width, height) = bundle.combined.dimensions();
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
    let (field_polygons, grle_w, grle_h) = match extracted {
        Some((polygons, w, h)) => {
            let scale_x = bundle.map_size / w.max(1) as f32;
            let scale_y = bundle.map_size / h.max(1) as f32;
            let half = bundle.map_size / 2.0;
            let polygons: Vec<FieldPolygon> = polygons
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
            (polygons, w, h)
        }
        None => {
            let scale_x = bundle.map_size / bundle.grle_width.max(1) as f32;
            let scale_y = bundle.map_size / bundle.grle_height.max(1) as f32;
            let half = bundle.map_size / 2.0;
            let polygons: Vec<FieldPolygon> = bundle
                .farmland_polygons
                .iter()
                .map(|fp| FieldPolygon {
                    id: fp.id,
                    vertices: fp
                        .vertices
                        .iter()
                        .map(|(px, py)| Vec2::new(*px * scale_x - half, *py * scale_y - half))
                        .collect(),
                })
                .collect();
            (polygons, bundle.grle_width, bundle.grle_height)
        }
    };

    if !field_polygons.is_empty() {
        log::info!(
            "Feldpolygone in Weltkoordinaten umgerechnet: {} Felder",
            field_polygons.len()
        );
        state.farmland_polygons = Some(Arc::new(field_polygons));
    } else {
        state.farmland_polygons = None;
    }

    // FarmlandGrid aus rohen GRLE/PNG-IDs aufbauen (falls vorhanden)
    if let Some(ids) = bundle.farmland_ids_raw.clone() {
        state.farmland_grid = Some(Arc::new(FarmlandGrid::new(
            ids,
            grle_w.max(1),
            grle_h.max(1),
            bundle.map_size,
        )));
        log::info!("FarmlandGrid gespeichert: {}x{} Pixel", grle_w, grle_h);
    } else {
        state.farmland_grid = None;
    }

    let target_dir = savegame_dir.unwrap_or_else(|| {
        Path::new(&zip_path)
            .parent()
            .unwrap_or_else(|| Path::new("."))
            .to_path_buf()
    });

    std::fs::create_dir_all(&target_dir).with_context(|| {
        format!(
            "Overview-Verzeichnis konnte nicht erstellt werden: {}",
            target_dir.display()
        )
    })?;
    write_layer_pngs_to_directory(&bundle, &target_dir)?;

    let bg_map = BackgroundMap::from_image(
        DynamicImage::ImageRgba8(bundle.combined.clone()),
        &zip_path,
        None,
    )?;
    drop(bundle);

    let files = super::background_layers::discover_background_layer_files(&target_dir);
    let catalog = super::background_layers::load_background_layer_catalog(files, &layers)?;

    apply_background_map(state, bg_map);
    state.background_layers = Some(catalog);
    log::info!(
        "Layer-PNGs gespeichert und Katalog aktiviert: {}",
        target_dir.display()
    );
    state.pending_overview_bundle = Some(PendingOverviewBundle { target_dir });

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
///
/// Bei einem Pending-Marker sind die kanonischen Layer-Dateien bereits geschrieben.
/// Der Save-Schritt aktualisiert dann nur noch `overview.png` und `overview.json`.
pub fn save_background_as_overview(state: &mut AppState, path: String) -> Result<()> {
    if let Some(pending) = state.pending_overview_bundle.as_ref() {
        let target_dir = Path::new(&path)
            .parent()
            .map(|dir| dir.to_path_buf())
            .unwrap_or_else(|| pending.target_dir.clone());
        std::fs::create_dir_all(&target_dir).with_context(|| {
            format!(
                "Overview-Verzeichnis konnte nicht erstellt werden: {}",
                target_dir.display()
            )
        })?;
    }

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

    if state.pending_overview_bundle.is_some() {
        state.pending_overview_bundle = None;
        log::info!(
            "Pending-Overview bestaetigt; Layer-Katalog bleibt aktiv: {}",
            path
        );
    }

    Ok(())
}

fn write_layer_pngs_to_directory(
    bundle: &fs25_map_overview::OverviewLayerBundle,
    target_dir: &Path,
) -> Result<()> {
    for (kind, image) in [
        (BackgroundLayerKind::Terrain, &bundle.terrain),
        (BackgroundLayerKind::Hillshade, &bundle.hillshade),
        (
            BackgroundLayerKind::FarmlandBorders,
            &bundle.farmland_borders,
        ),
        (BackgroundLayerKind::FarmlandIds, &bundle.farmland_ids),
        (BackgroundLayerKind::PoiMarkers, &bundle.poi_markers),
        (BackgroundLayerKind::Legend, &bundle.legend),
    ] {
        let layer_path = target_dir.join(kind.file_name());
        image.save(&layer_path).with_context(|| {
            format!(
                "Overview-Layer konnte nicht gespeichert werden: {}",
                layer_path.display()
            )
        })?;
    }

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
    use super::{
        browse_zip_background, clear_background_map, generate_overview_with_options,
        load_background_from_zip, load_background_map, persist_overview_defaults,
        save_background_as_overview, write_layer_pngs_to_directory,
    };
    use crate::app::state::{
        BackgroundLayerCatalog, BackgroundLayerFiles, PendingOverviewBundle, StoredBackgroundLayer,
        ZipBrowserState,
    };
    use crate::app::AppState;
    use crate::core::{BackgroundMap, FieldPolygon};
    use crate::shared::{BackgroundLayerKind, OverviewFieldDetectionSource, OverviewLayerOptions};
    use glam::Vec2;
    use image::{DynamicImage, ImageFormat, Rgba, RgbaImage};
    use std::io::{Cursor, Write};
    use std::path::{Path, PathBuf};
    use std::sync::Arc;
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

    fn sample_overview_bundle() -> fs25_map_overview::OverviewLayerBundle {
        let terrain = RgbaImage::from_pixel(2, 2, Rgba([20, 40, 60, 255]));
        let hillshade = RgbaImage::from_pixel(2, 2, Rgba([200, 0, 0, 128]));
        let farmland_borders = RgbaImage::from_pixel(2, 2, Rgba([0, 200, 0, 96]));
        let farmland_ids = RgbaImage::from_pixel(2, 2, Rgba([0, 0, 0, 0]));
        let poi_markers = RgbaImage::from_pixel(2, 2, Rgba([0, 0, 200, 128]));
        let legend = RgbaImage::from_pixel(2, 2, Rgba([255, 255, 255, 64]));
        let combined = fs25_map_overview::compose_layers(
            &terrain,
            &[
                (true, &hillshade),
                (true, &farmland_borders),
                (false, &farmland_ids),
                (false, &poi_markers),
                (false, &legend),
            ],
        );

        fs25_map_overview::OverviewLayerBundle {
            terrain,
            hillshade,
            farmland_borders,
            farmland_ids,
            poi_markers,
            legend,
            combined,
            farmland_polygons: Vec::new(),
            grle_width: 2,
            grle_height: 2,
            map_size: 2.0,
            farmland_ids_raw: Some(vec![1, 2, 3, 4]),
        }
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

    #[test]
    fn load_background_map_discovers_layer_bundle_in_same_directory() {
        let temp_dir = TempDirGuard::new("manual_layer_bundle_load");
        let overview_path = temp_dir.path().join("overview.png");
        let overview_path_string = overview_path.to_string_lossy().into_owned();

        std::fs::write(&overview_path, rgba_png_bytes(2, 2, [1, 2, 3, 255]))
            .expect("Legacy-overview muss geschrieben werden");
        std::fs::write(
            temp_dir.path().join("overview_terrain.png"),
            rgba_png_bytes(2, 2, [20, 40, 60, 255]),
        )
        .expect("Terrain-PNG muss geschrieben werden");
        std::fs::write(
            temp_dir.path().join("overview_hillshade.png"),
            rgba_png_bytes(2, 2, [220, 0, 0, 128]),
        )
        .expect("Hillshade-PNG muss geschrieben werden");

        let polygons = serde_json::to_string(&vec![FieldPolygon {
            id: 5,
            vertices: vec![Vec2::new(0.0, 0.0), Vec2::new(1.0, 1.0)],
        }])
        .expect("Farmland-JSON muss serialisierbar sein");
        std::fs::write(temp_dir.path().join("overview.json"), polygons)
            .expect("Farmland-JSON muss geschrieben werden");

        let mut state = AppState::new();
        state.options.overview_layers = OverviewLayerOptions {
            terrain: true,
            hillshade: false,
            farmlands: false,
            farmland_ids: false,
            pois: false,
            legend: false,
        };

        load_background_map(&mut state, overview_path_string, None)
            .expect("Manuelles Laden muss Layer-Bundle aktivieren");

        let background = state
            .view
            .background_map
            .as_ref()
            .expect("Background-Map muss gesetzt sein");
        assert_eq!(
            background.image_data().to_rgba8().get_pixel(0, 0).0,
            [20, 40, 60, 255]
        );

        let catalog = state
            .background_layers
            .as_ref()
            .expect("Layer-Katalog muss erkannt werden");
        assert_eq!(catalog.layers.len(), 2);
        assert_eq!(catalog.layers[0].kind, BackgroundLayerKind::Terrain);
        assert!(catalog.layers[0].path.ends_with("overview_terrain.png"));
        assert_eq!(catalog.layers[1].kind, BackgroundLayerKind::Hillshade);
        assert!(catalog.layers[1].path.ends_with("overview_hillshade.png"));
        assert!(catalog.visible.terrain);
        assert!(!catalog.visible.hillshade);
        assert_eq!(
            state
                .farmland_polygons
                .as_ref()
                .map(|polygons| polygons.len()),
            Some(1)
        );
    }

    #[test]
    fn load_background_map_keeps_legacy_overview_without_terrain_base() {
        let temp_dir = TempDirGuard::new("manual_legacy_background_load");
        let overview_path = temp_dir.path().join("overview.png");
        let overview_path_string = overview_path.to_string_lossy().into_owned();

        std::fs::write(&overview_path, rgba_png_bytes(2, 2, [11, 22, 33, 255]))
            .expect("Legacy-overview muss geschrieben werden");
        std::fs::write(
            temp_dir.path().join("overview_hillshade.png"),
            rgba_png_bytes(2, 2, [220, 0, 0, 128]),
        )
        .expect("Overlay-PNG muss geschrieben werden");

        let mut state = AppState::new();
        load_background_map(&mut state, overview_path_string, None)
            .expect("Legacy-Overview ohne Terrain muss ladbar bleiben");

        let background = state
            .view
            .background_map
            .as_ref()
            .expect("Background-Map muss gesetzt sein");
        assert_eq!(
            background.image_data().to_rgba8().get_pixel(0, 0).0,
            [11, 22, 33, 255]
        );
        assert!(state.background_layers.is_none());
    }

    #[test]
    fn save_background_as_overview_persists_pending_preview_and_keeps_catalog() {
        let temp_dir = TempDirGuard::new("overview_bundle_save");
        let target_path = temp_dir.path().join("overview.png");
        let target_path_string = target_path.to_string_lossy().into_owned();

        let bundle = sample_overview_bundle();
        write_layer_pngs_to_directory(&bundle, temp_dir.path())
            .expect("Layer-PNGs fuer Save-Test muessen geschrieben werden");
        let background = BackgroundMap::from_image(
            DynamicImage::ImageRgba8(bundle.combined.clone()),
            "test-bundle",
            None,
        )
        .expect("Background-Map muss aus Combined-Bild erstellt werden");

        let mut state = AppState::new();
        state.view.background_map = Some(Arc::new(background));
        state.background_image = state
            .view
            .background_map
            .as_ref()
            .map(|background| background.image_arc());
        state.options.overview_layers = OverviewLayerOptions {
            terrain: true,
            hillshade: true,
            farmlands: true,
            farmland_ids: false,
            pois: false,
            legend: false,
        };
        let files =
            super::super::background_layers::discover_background_layer_files(temp_dir.path());
        let catalog = super::super::background_layers::load_background_layer_catalog(
            files,
            &state.options.overview_layers,
        )
        .expect("Layer-Katalog muss vor Save aktivierbar sein");
        state.background_layers = Some(catalog);
        state.farmland_polygons = Some(Arc::new(vec![FieldPolygon {
            id: 7,
            vertices: vec![Vec2::new(1.0, 2.0), Vec2::new(3.0, 4.0)],
        }]));
        state.pending_overview_bundle = Some(PendingOverviewBundle {
            target_dir: temp_dir.path().to_path_buf(),
        });

        save_background_as_overview(&mut state, target_path_string)
            .expect("Pending-Overview muss speicherbar sein");

        assert!(target_path.is_file());
        assert!(temp_dir.path().join("overview.json").is_file());
        for file_name in [
            "overview_terrain.png",
            "overview_hillshade.png",
            "overview_farmland_borders.png",
            "overview_farmland_ids.png",
            "overview_poi_markers.png",
            "overview_legend.png",
        ] {
            assert!(
                temp_dir.path().join(file_name).is_file(),
                "{} muss geschrieben werden",
                file_name
            );
        }

        assert!(state.pending_overview_bundle.is_none());
        let catalog = state
            .background_layers
            .as_ref()
            .expect("Aktiver Katalog muss erhalten bleiben");
        assert_eq!(catalog.layers.len(), 6);
        assert!(catalog.visible.terrain);
        assert!(catalog.visible.hillshade);
        assert!(catalog.visible.farmlands);
        assert!(!catalog.visible.farmland_ids);
        assert!(catalog.layers.iter().all(|layer| layer.path.is_file()));
        assert_eq!(state.view.background_scale, 1.0);
    }

    #[test]
    fn generate_overview_with_options_keeps_preview_and_pending_marker_before_save() {
        let temp_dir = TempDirGuard::new("overview_generate_preview_only");
        let zip_path = temp_dir.path().join("test_map.zip");
        let zip_path_string = zip_path.to_string_lossy().into_owned();
        let xml_path = temp_dir.path().join("AutoDrive_config.xml");
        let xml_path_string = xml_path.to_string_lossy().into_owned();

        std::fs::write(&xml_path, b"<xml/>").expect("Test-XML muss geschrieben werden");
        write_zip(
            &zip_path,
            vec![
                (
                    "TestMap/modDesc.xml",
                    br#"<?xml version="1.0" encoding="utf-8"?>
<modDesc>
  <title><en>Test Map</en></title>
  <map configFilename="maps/config/map.xml" />
</modDesc>"#
                        .to_vec(),
                ),
                (
                    "TestMap/maps/config/map.xml",
                    br#"<?xml version="1.0" encoding="utf-8"?>
<map width="32" height="32" />"#
                        .to_vec(),
                ),
                (
                    "TestMap/maps/data/dem.png",
                    rgba_png_bytes(1, 1, [0, 0, 0, 255]),
                ),
                ("TestMap/maps/data/infoLayer_farmlands.png", {
                    let image = image::DynamicImage::ImageLuma8(
                        image::GrayImage::from_vec(2, 2, vec![0, 1, 1, 0])
                            .expect("GrayImage fuer Test-ZIP muss erzeugbar sein"),
                    );
                    let mut cursor = Cursor::new(Vec::new());
                    image
                        .write_to(&mut cursor, ImageFormat::Png)
                        .expect("Farmland-PNG muss erzeugt werden");
                    cursor.into_inner()
                }),
            ],
        );

        let mut state = AppState::new();
        state.ui.current_file_path = Some(xml_path_string.clone());
        state.ui.overview_options_dialog.visible = true;
        state.ui.overview_options_dialog.zip_path = zip_path_string;
        state.ui.overview_options_dialog.layers = OverviewLayerOptions {
            terrain: true,
            hillshade: true,
            farmlands: false,
            farmland_ids: false,
            pois: false,
            legend: false,
        };
        state.ui.overview_options_dialog.field_detection_source =
            OverviewFieldDetectionSource::FromZip;

        generate_overview_with_options(&mut state)
            .expect("Overview-Generierung muss ein Preview-Bild erzeugen");

        let background = state
            .view
            .background_map
            .as_ref()
            .expect("Preview-Background muss gesetzt sein");
        assert_eq!(background.dimensions(), (32, 32));
        let catalog = state
            .background_layers
            .as_ref()
            .expect("Layer-Katalog muss direkt nach der Generierung aktiv sein");
        assert_eq!(catalog.layers.len(), 6);
        assert!(catalog.visible.terrain);
        assert!(catalog.visible.hillshade);
        assert!(!catalog.visible.farmlands);
        assert!(!catalog.visible.farmland_ids);
        assert!(!catalog.visible.pois);
        assert!(!catalog.visible.legend);
        for file_name in [
            "overview_terrain.png",
            "overview_hillshade.png",
            "overview_farmland_borders.png",
            "overview_farmland_ids.png",
            "overview_poi_markers.png",
            "overview_legend.png",
        ] {
            assert!(
                temp_dir.path().join(file_name).is_file(),
                "{} muss direkt nach der Generierung existieren",
                file_name
            );
        }
        let pending = state
            .pending_overview_bundle
            .as_ref()
            .expect("Pending-Marker muss bis zum Save erhalten bleiben");
        assert_eq!(pending.target_dir, temp_dir.path().to_path_buf());
        assert!(state.ui.save_overview_dialog.visible);
        assert_eq!(
            state.ui.save_overview_dialog.target_path,
            temp_dir.path().join("overview.png").to_string_lossy()
        );
        assert!(!state.ui.overview_options_dialog.visible);
    }

    #[test]
    fn clear_background_map_resets_layer_catalog_and_pending_bundle() {
        let mut state = AppState::new();
        state.background_layers = Some(BackgroundLayerCatalog {
            files: BackgroundLayerFiles {
                directory: PathBuf::from("/tmp/overview"),
                terrain: Some(PathBuf::from("/tmp/overview/overview_terrain.png")),
                hillshade: None,
                farmland_borders: None,
                farmland_ids: None,
                poi_markers: None,
                legend: None,
            },
            layers: vec![StoredBackgroundLayer {
                kind: BackgroundLayerKind::Terrain,
                path: PathBuf::from("/tmp/overview/overview_terrain.png"),
            }],
            visible: OverviewLayerOptions::default(),
        });
        state.pending_overview_bundle = Some(PendingOverviewBundle {
            target_dir: PathBuf::from("/tmp/overview"),
        });

        clear_background_map(&mut state);

        assert!(state.background_layers.is_none());
        assert!(state.pending_overview_bundle.is_none());
    }
}
