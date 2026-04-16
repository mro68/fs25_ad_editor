//! Use-Case-Funktionen fuer gespeicherte Overview-Layer-Dateien und CPU-Komposition.

use crate::app::state::{BackgroundLayerCatalog, BackgroundLayerFiles, StoredBackgroundLayer};
use crate::shared::{BackgroundLayerKind, OverviewLayerOptions};
use anyhow::{bail, Context, Result};
use image::{DynamicImage, GenericImageView, RgbaImage};
use std::path::{Path, PathBuf};
use std::sync::Arc;

/// Sucht im Verzeichnis nach bekannten Layer-Dateinamen.
pub fn discover_background_layer_files(dir: &Path) -> BackgroundLayerFiles {
    BackgroundLayerFiles {
        directory: dir.to_path_buf(),
        terrain: discovered_layer_path(dir, BackgroundLayerKind::Terrain),
        hillshade: discovered_layer_path(dir, BackgroundLayerKind::Hillshade),
        farmland_borders: discovered_layer_path(dir, BackgroundLayerKind::FarmlandBorders),
        farmland_ids: discovered_layer_path(dir, BackgroundLayerKind::FarmlandIds),
        poi_markers: discovered_layer_path(dir, BackgroundLayerKind::PoiMarkers),
        legend: discovered_layer_path(dir, BackgroundLayerKind::Legend),
    }
}

/// Laedt die gefundenen Layer-Dateien und erstellt einen Katalog.
pub fn load_background_layer_catalog(
    files: BackgroundLayerFiles,
    visible: &OverviewLayerOptions,
) -> Result<BackgroundLayerCatalog> {
    let terrain_path = files
        .terrain
        .as_ref()
        .context("Gespeicherter Terrain-Layer fehlt")?;
    let terrain = load_layer_image(BackgroundLayerKind::Terrain, terrain_path)?;
    let terrain_dimensions = terrain.image.dimensions();

    let mut layers = vec![terrain];
    for (kind, path) in [
        (BackgroundLayerKind::Hillshade, files.hillshade.as_ref()),
        (
            BackgroundLayerKind::FarmlandBorders,
            files.farmland_borders.as_ref(),
        ),
        (
            BackgroundLayerKind::FarmlandIds,
            files.farmland_ids.as_ref(),
        ),
        (BackgroundLayerKind::PoiMarkers, files.poi_markers.as_ref()),
        (BackgroundLayerKind::Legend, files.legend.as_ref()),
    ] {
        let Some(path) = path else {
            continue;
        };

        let layer = load_layer_image(kind, path)?;
        validate_dimensions(&layer, terrain_dimensions)?;
        layers.push(layer);
    }

    let mut runtime_visible = visible.clone();
    runtime_visible.terrain &= files.terrain.is_some();
    runtime_visible.hillshade &= files.hillshade.is_some();
    runtime_visible.farmlands &= files.farmland_borders.is_some();
    runtime_visible.farmland_ids &= files.farmland_ids.is_some();
    runtime_visible.pois &= files.poi_markers.is_some();
    runtime_visible.legend &= files.legend.is_some();

    Ok(BackgroundLayerCatalog {
        files,
        layers,
        visible: runtime_visible,
    })
}

/// Setzt aus den sichtbaren Layern ein kombiniertes Bild zusammen (CPU-Komposition).
pub fn compose_background_from_catalog(catalog: &BackgroundLayerCatalog) -> Result<DynamicImage> {
    let terrain = catalog_layer_image(catalog, BackgroundLayerKind::Terrain)
        .context("Terrain-Layer fuer Komposition fehlt")?;
    let (width, height) = terrain.dimensions();
    let base = if catalog.visible.terrain {
        terrain.to_rgba8()
    } else {
        RgbaImage::new(width, height)
    };

    let overlays: Vec<(bool, RgbaImage)> = [
        (catalog.visible.hillshade, BackgroundLayerKind::Hillshade),
        (
            catalog.visible.farmlands,
            BackgroundLayerKind::FarmlandBorders,
        ),
        (
            catalog.visible.farmland_ids,
            BackgroundLayerKind::FarmlandIds,
        ),
        (catalog.visible.pois, BackgroundLayerKind::PoiMarkers),
        (catalog.visible.legend, BackgroundLayerKind::Legend),
    ]
    .into_iter()
    .filter_map(|(is_visible, kind)| {
        catalog_layer_image(catalog, kind).map(|image| (is_visible, image.to_rgba8()))
    })
    .collect();
    let overlay_refs: Vec<(bool, &RgbaImage)> = overlays
        .iter()
        .map(|(is_visible, image)| (*is_visible, image))
        .collect();

    Ok(DynamicImage::ImageRgba8(fs25_map_overview::compose_layers(
        &base,
        &overlay_refs,
    )))
}

fn discovered_layer_path(dir: &Path, kind: BackgroundLayerKind) -> Option<PathBuf> {
    let path = dir.join(kind.file_name());
    path.is_file().then_some(path)
}

fn load_layer_image(kind: BackgroundLayerKind, path: &Path) -> Result<StoredBackgroundLayer> {
    let image = image::open(path)
        .with_context(|| format!("Layer-Bild konnte nicht geladen werden: {}", path.display()))?;
    Ok(StoredBackgroundLayer {
        kind,
        path: path.to_path_buf(),
        image: Arc::new(image),
    })
}

fn validate_dimensions(layer: &StoredBackgroundLayer, expected: (u32, u32)) -> Result<()> {
    let actual = layer.image.dimensions();
    if actual != expected {
        bail!(
            "Layer {} hat abweichende Dimensionen: erwartet {}x{}, erhalten {}x{}",
            layer.path.display(),
            expected.0,
            expected.1,
            actual.0,
            actual.1
        );
    }
    Ok(())
}

fn catalog_layer_image(
    catalog: &BackgroundLayerCatalog,
    kind: BackgroundLayerKind,
) -> Option<&DynamicImage> {
    catalog
        .layers
        .iter()
        .find(|layer| layer.kind == kind)
        .map(|layer| layer.image.as_ref())
}

#[cfg(test)]
mod tests {
    use super::{
        compose_background_from_catalog, discover_background_layer_files,
        load_background_layer_catalog,
    };
    use crate::shared::OverviewLayerOptions;
    use image::{DynamicImage, GenericImageView, ImageFormat, Rgba, RgbaImage};
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

    fn write_png(path: &Path, image: &RgbaImage) {
        let mut cursor = Cursor::new(Vec::new());
        DynamicImage::ImageRgba8(image.clone())
            .write_to(&mut cursor, ImageFormat::Png)
            .expect("PNG muss erzeugt werden");
        std::fs::write(path, cursor.into_inner()).expect("PNG muss geschrieben werden");
    }

    #[test]
    fn discover_and_load_background_layer_catalog_sanitizes_missing_layers() {
        let temp_dir = TempDirGuard::new("background_layers_catalog");
        let terrain = RgbaImage::from_pixel(2, 2, Rgba([20, 40, 60, 255]));
        let hillshade = RgbaImage::from_pixel(2, 2, Rgba([220, 0, 0, 128]));
        write_png(&temp_dir.path().join("overview_terrain.png"), &terrain);
        write_png(&temp_dir.path().join("overview_hillshade.png"), &hillshade);

        let files = discover_background_layer_files(temp_dir.path());
        assert!(files.terrain.is_some());
        assert!(files.hillshade.is_some());
        assert!(files.farmland_borders.is_none());

        let requested = OverviewLayerOptions {
            terrain: true,
            hillshade: true,
            farmlands: true,
            farmland_ids: true,
            pois: true,
            legend: true,
        };
        let catalog = load_background_layer_catalog(files, &requested)
            .expect("Katalog muss fuer Terrain + Hillshade ladbar sein");

        assert_eq!(catalog.layers.len(), 2);
        assert!(catalog.visible.terrain);
        assert!(catalog.visible.hillshade);
        assert!(!catalog.visible.farmlands);
        assert!(!catalog.visible.farmland_ids);
        assert!(!catalog.visible.pois);
        assert!(!catalog.visible.legend);

        let composed = compose_background_from_catalog(&catalog)
            .expect("Komposition aus Terrain + Hillshade muss funktionieren");
        assert_eq!(composed.dimensions(), (2, 2));

        let pixel = composed.to_rgba8().get_pixel(0, 0).0;
        assert_eq!(pixel, [120, 20, 30, 255]);
    }
}
