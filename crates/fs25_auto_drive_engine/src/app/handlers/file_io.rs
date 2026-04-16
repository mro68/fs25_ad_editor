//! Handler fuer Datei-Operationen (Oeffnen, Speichern, Heightmap).

use super::dialog;
use crate::app::use_cases;
use crate::app::AppState;
use crate::app::BackgroundLayerFiles;
use std::path::Path;

/// Oeffnet den Datei-Oeffnen-Dialog.
pub fn request_open(state: &mut AppState) {
    use_cases::file_io::request_open_file(state);
}

/// Oeffnet den Datei-Speichern-Dialog.
pub fn request_save(state: &mut AppState) {
    use_cases::file_io::request_save_file(state);
}

/// Bestaetigt Heightmap-Warnung und fuehrt Speichern aus.
pub fn confirm_and_save(state: &mut AppState) -> anyhow::Result<()> {
    use_cases::file_io::confirm_and_save(state)
}

/// Laedt eine RoadMap aus dem uebergebenen Pfad.
///
/// Nach dem Laden wird automatisch geprueft, ob eine Heightmap und/oder
/// gespeicherte Background-Layer, ein Legacy-Overview-Bild und/oder ein
/// passender Map-Mod-ZIP im Mods-Verzeichnis vorhanden sind.
pub fn load(state: &mut AppState, path: String) -> anyhow::Result<()> {
    use_cases::file_io::load_selected_file(state, path.clone())?;
    run_post_load_detection(state, &path);
    Ok(())
}

/// Fuehrt die automatische Erkennung von Heightmap, gespeicherten Overview-Layern,
/// Legacy-Overview-Bildern und Map-Mod-ZIPs durch.
fn run_post_load_detection(state: &mut AppState, xml_path: &str) {
    let xml_path = Path::new(xml_path);
    let map_name = state.road_map.as_ref().and_then(|rm| rm.map_name.clone());

    let result = use_cases::auto_detect::detect_post_load(xml_path, map_name.as_deref());
    let background_layer_files = result.background_layer_files;
    let overview_path = result.overview_path;
    let matching_zips = result.matching_zips;

    let heightmap_set = result.heightmap_path.is_some();
    let heightmap_display = result
        .heightmap_path
        .as_ref()
        .and_then(|p| p.to_str())
        .map(String::from);

    // Heightmap lautlos setzen (non-destructive)
    if let Some(ref hm_path) = result.heightmap_path
        && let Some(hm_str) = hm_path.to_str()
    {
        state.ui.heightmap_path = Some(hm_str.to_string());
        log::info!("Heightmap auto-detected: {}", hm_str);
    }

    // Hintergrundbild automatisch laden: gespeichertes Layer-Bundle hat Vorrang,
    // danach Legacy overview.png und zuletzt overview.jpg.
    let overview_loaded = if let Some(files) = background_layer_files {
        if load_detected_background_layers(state, files) {
            true
        } else {
            load_detected_legacy_overview(state, overview_path.as_deref())
        }
    } else {
        load_detected_legacy_overview(state, overview_path.as_deref())
    };

    // Dialog nur anzeigen wenn etwas erkannt wurde
    let has_zips = !matching_zips.is_empty();
    if heightmap_set || has_zips || overview_loaded {
        dialog::open_detected_overview_source_dialog(
            state,
            heightmap_set,
            heightmap_display,
            overview_loaded,
            matching_zips,
            map_name.unwrap_or_default(),
        );
    }
}

fn load_detected_background_layers(state: &mut AppState, files: BackgroundLayerFiles) -> bool {
    let overview_path = files.directory.join("overview.png");
    let overview_path = overview_path.to_string_lossy().into_owned();

    match use_cases::background_layers::load_background_layer_catalog_into_state(state, files) {
        Ok(()) => {
            log::info!(
                "Gespeichertes Background-Layer-Bundle automatisch geladen: {}",
                overview_path
            );
            use_cases::background_map::load_farmland_json(state, &overview_path);
            true
        }
        Err(error) => {
            log::warn!(
                "Gespeichertes Background-Layer-Bundle konnte nicht geladen werden: {}",
                error
            );
            false
        }
    }
}

fn load_detected_legacy_overview(
    state: &mut AppState,
    overview_path: Option<&std::path::Path>,
) -> bool {
    if let Some(overview_path) = overview_path
        && let Some(overview_path) = overview_path.to_str()
    {
        match use_cases::background_map::load_background_map(state, overview_path.to_string(), None)
        {
            Ok(()) => {
                log::info!("Overview automatisch geladen: {}", overview_path);
                true
            }
            Err(error) => {
                log::warn!("Overview konnte nicht geladen werden: {}", error);
                false
            }
        }
    } else {
        false
    }
}

/// Speichert die RoadMap unter dem uebergebenen Pfad (inkl. Heightmap-Check).
///
/// `None` speichert unter dem aktuell bekannten Pfad (oder oeffnet den Dialog).
/// `Some(p)` speichert explizit unter dem neuen Pfad `p`.
pub fn save(state: &mut AppState, path: Option<String>) -> anyhow::Result<()> {
    use_cases::file_io::save_with_heightmap_check(state, path)
}

/// Entfernt die aktuell gesetzte Heightmap.
pub fn clear_heightmap(state: &mut AppState) {
    use_cases::heightmap::clear_heightmap(state);
}

/// Setzt eine Heightmap aus einem Dateipfad.
pub fn set_heightmap(state: &mut AppState, path: String) {
    use_cases::heightmap::set_heightmap(state, path);
}

/// Fuehrt die Duplikat-Bereinigung auf der geladenen RoadMap aus.
pub fn deduplicate(state: &mut AppState) {
    use_cases::file_io::deduplicate_loaded_roadmap(state);
}

#[cfg(test)]
mod tests {
    use super::run_post_load_detection;
    use crate::app::AppState;
    use crate::core::FieldPolygon;
    use crate::shared::OverviewLayerOptions;
    use glam::Vec2;
    use image::{DynamicImage, ImageFormat, Rgba, RgbaImage};
    use std::io::Cursor;
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
                "fs25_auto_drive_engine_file_io_{}_{}_{}",
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

    fn write_png(path: &Path, image: &RgbaImage) {
        let mut cursor = Cursor::new(Vec::new());
        DynamicImage::ImageRgba8(image.clone())
            .write_to(&mut cursor, ImageFormat::Png)
            .expect("PNG muss erzeugt werden");
        std::fs::write(path, cursor.into_inner()).expect("PNG muss geschrieben werden");
    }

    fn write_farmland_json(path: &Path) {
        let polygons = serde_json::to_string(&vec![FieldPolygon {
            id: 9,
            vertices: vec![Vec2::new(0.0, 0.0), Vec2::new(1.0, 1.0)],
        }])
        .expect("Farmland-JSON muss serialisierbar sein");
        std::fs::write(path, polygons).expect("Farmland-JSON muss geschrieben werden");
    }

    #[test]
    fn run_post_load_detection_prefers_saved_layer_bundle_over_legacy_overview() {
        let temp_dir = TempDirGuard::new("post_load_bundle_priority");
        let xml_path = temp_dir.path().join("AutoDrive_config.xml");
        std::fs::write(&xml_path, b"<xml/>").expect("XML muss geschrieben werden");

        write_png(
            &temp_dir.path().join("overview.png"),
            &RgbaImage::from_pixel(2, 2, Rgba([1, 2, 3, 255])),
        );
        write_png(
            &temp_dir.path().join("overview_terrain.png"),
            &RgbaImage::from_pixel(2, 2, Rgba([20, 40, 60, 255])),
        );
        write_png(
            &temp_dir.path().join("overview_hillshade.png"),
            &RgbaImage::from_pixel(2, 2, Rgba([220, 0, 0, 128])),
        );
        write_farmland_json(&temp_dir.path().join("overview.json"));

        let mut state = AppState::new();
        state.options.overview_layers = OverviewLayerOptions {
            terrain: true,
            hillshade: false,
            farmlands: false,
            farmland_ids: false,
            pois: false,
            legend: false,
        };

        run_post_load_detection(&mut state, &xml_path.to_string_lossy());

        let background = state
            .view
            .background_map
            .as_ref()
            .expect("Background-Map muss geladen sein");
        assert_eq!(
            background.image_data().to_rgba8().get_pixel(0, 0).0,
            [20, 40, 60, 255]
        );
        assert!(state.background_layers.is_some());
        assert_eq!(
            state
                .farmland_polygons
                .as_ref()
                .map(|polygons| polygons.len()),
            Some(1)
        );
        assert!(state.ui.post_load_dialog.visible);
        assert!(state.ui.post_load_dialog.overview_loaded);
    }

    #[test]
    fn run_post_load_detection_falls_back_to_legacy_overview_without_terrain_base() {
        let temp_dir = TempDirGuard::new("post_load_legacy_fallback");
        let xml_path = temp_dir.path().join("AutoDrive_config.xml");
        std::fs::write(&xml_path, b"<xml/>").expect("XML muss geschrieben werden");

        write_png(
            &temp_dir.path().join("overview.png"),
            &RgbaImage::from_pixel(2, 2, Rgba([11, 22, 33, 255])),
        );
        write_png(
            &temp_dir.path().join("overview_hillshade.png"),
            &RgbaImage::from_pixel(2, 2, Rgba([220, 0, 0, 128])),
        );
        write_farmland_json(&temp_dir.path().join("overview.json"));

        let mut state = AppState::new();
        run_post_load_detection(&mut state, &xml_path.to_string_lossy());

        let background = state
            .view
            .background_map
            .as_ref()
            .expect("Legacy-Overview muss geladen sein");
        assert_eq!(
            background.image_data().to_rgba8().get_pixel(0, 0).0,
            [11, 22, 33, 255]
        );
        assert!(state.background_layers.is_none());
        assert_eq!(
            state
                .farmland_polygons
                .as_ref()
                .map(|polygons| polygons.len()),
            Some(1)
        );
        assert!(state.ui.post_load_dialog.visible);
        assert!(state.ui.post_load_dialog.overview_loaded);
    }
}
