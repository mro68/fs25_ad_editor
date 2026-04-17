//! Use-Cases fuer Background-Map-Verwaltung.
//!
//! Das Modul ist in folgende Teilbereiche aufgeteilt:
//! - `state_helpers`: Interne State-Mutations-Hilfsfunktionen
//! - `field_polygons`: Farmland-Polygon-Verwaltung und JSON-I/O
//! - `load`: Laden aus Datei und ZIP-Archiven
//! - `generate`: Uebersichtskarten-Generierung und -Speicherung

mod field_polygons;
mod generate;
mod load;
mod state_helpers;

pub use field_polygons::load_farmland_json;
pub use generate::{generate_overview_with_options, save_background_as_overview};
pub use load::{
    browse_zip_background, clear_background_map, load_background_from_zip, load_background_map,
    request_background_map_dialog, scale_background, toggle_background_visibility,
};

#[cfg(test)]
mod tests {
    use super::generate::{
        generate_overview_with_options, save_background_as_overview, write_layer_pngs_to_directory,
    };
    use super::load::{
        browse_zip_background, clear_background_map, load_background_from_zip, load_background_map,
    };
    use super::state_helpers::persist_overview_defaults;
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
