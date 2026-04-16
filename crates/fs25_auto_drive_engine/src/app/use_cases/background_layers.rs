//! Use-Case-Funktionen fuer gespeicherte Overview-Layer-Dateien und CPU-Komposition.

use crate::app::state::{BackgroundLayerCatalog, BackgroundLayerFiles, StoredBackgroundLayer};
use crate::app::AppState;
use crate::core::BackgroundMap;
use crate::shared::{BackgroundLayerKind, OverviewLayerOptions};
use anyhow::{bail, Context, Result};
use image::{DynamicImage, Rgba, RgbaImage};
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

/// Baut aus den gefundenen Dateien einen metadatenbasierten Runtime-Katalog.
pub fn load_background_layer_catalog(
    files: BackgroundLayerFiles,
    visible: &OverviewLayerOptions,
) -> Result<BackgroundLayerCatalog> {
    let terrain_path = files
        .terrain
        .as_ref()
        .context("Gespeicherter Terrain-Layer fehlt")?;

    let mut layers = vec![StoredBackgroundLayer {
        kind: BackgroundLayerKind::Terrain,
        path: terrain_path.clone(),
    }];
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
        if let Some(path) = path {
            layers.push(StoredBackgroundLayer {
                kind,
                path: path.clone(),
            });
        }
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

/// Setzt aus den sichtbaren Layern ein kombiniertes Bild zusammen.
///
/// Laedt sichtbare PNG-Dateien sequentiell von Platte und blendet sie direkt in
/// das Ergebnisbild, damit nie alle Layer gleichzeitig dekodiert im RAM liegen.
pub fn compose_background_from_catalog(catalog: &BackgroundLayerCatalog) -> Result<DynamicImage> {
    let terrain_entry = catalog_layer(catalog, BackgroundLayerKind::Terrain)
        .context("Terrain-Layer fuer Komposition fehlt")?;
    let terrain = load_layer_image(terrain_entry.kind, &terrain_entry.path)?;
    let terrain_dimensions = terrain.dimensions();
    let (width, height) = terrain_dimensions;
    let mut result = if catalog.visible.terrain {
        terrain
    } else {
        RgbaImage::new(width, height)
    };

    for kind in [
        BackgroundLayerKind::Hillshade,
        BackgroundLayerKind::FarmlandBorders,
        BackgroundLayerKind::FarmlandIds,
        BackgroundLayerKind::PoiMarkers,
        BackgroundLayerKind::Legend,
    ] {
        if !layer_visibility(&catalog.visible, kind) {
            continue;
        }

        let Some(layer) = catalog_layer(catalog, kind) else {
            continue;
        };

        let overlay = load_layer_image(layer.kind, &layer.path)?;
        validate_dimensions(
            layer.kind,
            &layer.path,
            overlay.dimensions(),
            terrain_dimensions,
        )?;
        blend_image(&mut result, &overlay);
    }

    Ok(DynamicImage::ImageRgba8(result))
}

/// Laedt einen gespeicherten Layer-Katalog in den State und setzt die CPU-komponierte
/// Hintergrundkarte als aktives Background-Asset.
pub(crate) fn load_background_layer_catalog_into_state(
    state: &mut AppState,
    files: BackgroundLayerFiles,
) -> Result<()> {
    let requested_visible = state.options.overview_layers.clone();
    let catalog = load_background_layer_catalog(files, &requested_visible)?;
    let source_label = catalog_source_label(&catalog);
    let composed_background = compose_background_from_catalog(&catalog)?;
    let background_map = BackgroundMap::from_image(composed_background, &source_label, None)?;

    apply_background_map_with_scale(state, background_map, 1.0);
    state.background_layers = Some(catalog);
    state.pending_overview_bundle = None;

    Ok(())
}

/// Versucht, im angegebenen Verzeichnis ein gespeichertes Layer-Bundle zu erkennen,
/// zu laden und als aktiven Hintergrund zu setzen.
pub(crate) fn try_load_background_layer_bundle_from_directory(
    state: &mut AppState,
    dir: &Path,
) -> Result<bool> {
    let files = discover_background_layer_files(dir);
    if files.terrain.is_none() {
        return Ok(false);
    }

    load_background_layer_catalog_into_state(state, files)?;
    Ok(true)
}

/// Schaltet die Sichtbarkeit eines einzelnen Hintergrund-Layers um und
/// setzt das Hintergrundbild aus den aktiven Layern neu zusammen.
///
/// Die Runtime-Sichtbarkeit wird erst nach erfolgreicher Re-Komposition
/// in den State uebernommen, damit Katalog und gerendertes Hintergrundbild
/// auch bei I/O- oder Dimensionsfehlern konsistent bleiben.
pub fn set_background_layer_visibility(
    state: &mut AppState,
    layer: BackgroundLayerKind,
    visible: bool,
) -> Result<()> {
    let background_scale = state.view.background_scale;
    let (composed_background, source_label, committed_visible) = {
        let catalog = state
            .background_layers
            .as_ref()
            .context("Keine gespeicherten Hintergrund-Layer geladen")?;

        if catalog_layer(catalog, layer).is_none() {
            bail!("Hintergrund-Layer {} ist nicht geladen", layer);
        }

        if layer_visibility(&catalog.visible, layer) == visible {
            return Ok(());
        }

        let mut recomposed_catalog = catalog.clone();
        set_layer_visibility(&mut recomposed_catalog.visible, layer, visible);
        let source_label = format!(
            "{}:{}",
            recomposed_catalog.files.directory.display(),
            layer.file_name()
        );
        let composed_background = compose_background_from_catalog(&recomposed_catalog)?;
        (
            composed_background,
            source_label,
            recomposed_catalog.visible,
        )
    };

    let background_map = BackgroundMap::from_image(composed_background, &source_label, None)?;
    apply_background_map_with_scale(state, background_map, background_scale);
    state
        .background_layers
        .as_mut()
        .context("Keine gespeicherten Hintergrund-Layer geladen")?
        .visible = committed_visible.clone();
    log::info!(
        "Hintergrund-Layer {}: {}",
        layer,
        if layer_visibility(&committed_visible, layer) {
            "an"
        } else {
            "aus"
        }
    );
    Ok(())
}

fn discovered_layer_path(dir: &Path, kind: BackgroundLayerKind) -> Option<PathBuf> {
    let path = dir.join(kind.file_name());
    path.is_file().then_some(path)
}

fn catalog_source_label(catalog: &BackgroundLayerCatalog) -> String {
    catalog
        .files
        .terrain
        .as_ref()
        .map(|path| path.to_string_lossy().into_owned())
        .unwrap_or_else(|| catalog.files.directory.display().to_string())
}

fn apply_background_map_with_scale(
    state: &mut AppState,
    background_map: BackgroundMap,
    scale: f32,
) {
    let image_arc = background_map.image_arc();
    state.view.background_map = Some(Arc::new(background_map));
    state.view.background_scale = scale;
    state.view.mark_background_asset_changed();
    state.background_image = Some(image_arc);
}

fn load_layer_image(kind: BackgroundLayerKind, path: &Path) -> Result<RgbaImage> {
    let image = image::open(path).with_context(|| {
        format!(
            "Layer-Bild fuer {} konnte nicht geladen werden: {}",
            kind,
            path.display()
        )
    })?;
    Ok(image.to_rgba8())
}

fn validate_dimensions(
    kind: BackgroundLayerKind,
    path: &Path,
    actual: (u32, u32),
    expected: (u32, u32),
) -> Result<()> {
    if actual != expected {
        bail!(
            "Layer {} ({}) hat abweichende Dimensionen: erwartet {}x{}, erhalten {}x{}",
            kind,
            path.display(),
            expected.0,
            expected.1,
            actual.0,
            actual.1
        );
    }
    Ok(())
}

fn catalog_layer(
    catalog: &BackgroundLayerCatalog,
    kind: BackgroundLayerKind,
) -> Option<&StoredBackgroundLayer> {
    catalog.layers.iter().find(|layer| layer.kind == kind)
}

fn blend_image(base: &mut RgbaImage, overlay: &RgbaImage) {
    debug_assert_eq!(base.dimensions(), overlay.dimensions());

    for (dst_pixel, src_pixel) in base.pixels_mut().zip(overlay.pixels()) {
        *dst_pixel = blend_pixel(*dst_pixel, *src_pixel);
    }
}

fn blend_pixel(dst: Rgba<u8>, src: Rgba<u8>) -> Rgba<u8> {
    let src_alpha = src[3] as f32 / 255.0;
    if src_alpha <= f32::EPSILON {
        return dst;
    }

    let dst_alpha = dst[3] as f32 / 255.0;
    let out_alpha = src_alpha + dst_alpha * (1.0 - src_alpha);
    if out_alpha <= f32::EPSILON {
        return Rgba([0, 0, 0, 0]);
    }

    let mut out = [0u8; 4];
    for channel in 0..3 {
        let src_value = src[channel] as f32 / 255.0;
        let dst_value = dst[channel] as f32 / 255.0;
        let out_value =
            (src_value * src_alpha + dst_value * dst_alpha * (1.0 - src_alpha)) / out_alpha;
        out[channel] = (out_value * 255.0).round().clamp(0.0, 255.0) as u8;
    }
    out[3] = (out_alpha * 255.0).round().clamp(0.0, 255.0) as u8;
    Rgba(out)
}

fn layer_visibility(visible: &OverviewLayerOptions, layer: BackgroundLayerKind) -> bool {
    match layer {
        BackgroundLayerKind::Terrain => visible.terrain,
        BackgroundLayerKind::Hillshade => visible.hillshade,
        BackgroundLayerKind::FarmlandBorders => visible.farmlands,
        BackgroundLayerKind::FarmlandIds => visible.farmland_ids,
        BackgroundLayerKind::PoiMarkers => visible.pois,
        BackgroundLayerKind::Legend => visible.legend,
    }
}

fn set_layer_visibility(
    visible: &mut OverviewLayerOptions,
    layer: BackgroundLayerKind,
    is_visible: bool,
) {
    match layer {
        BackgroundLayerKind::Terrain => visible.terrain = is_visible,
        BackgroundLayerKind::Hillshade => visible.hillshade = is_visible,
        BackgroundLayerKind::FarmlandBorders => visible.farmlands = is_visible,
        BackgroundLayerKind::FarmlandIds => visible.farmland_ids = is_visible,
        BackgroundLayerKind::PoiMarkers => visible.pois = is_visible,
        BackgroundLayerKind::Legend => visible.legend = is_visible,
    }
}

#[cfg(test)]
mod tests {
    use super::{
        compose_background_from_catalog, discover_background_layer_files,
        load_background_layer_catalog, set_background_layer_visibility,
        try_load_background_layer_bundle_from_directory,
    };
    use crate::app::AppState;
    use crate::core::BackgroundMap;
    use crate::shared::{BackgroundLayerKind, OverviewLayerOptions};
    use image::{DynamicImage, GenericImageView, ImageFormat, Rgba, RgbaImage};
    use std::io::Cursor;
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
        assert_eq!(catalog.layers[0].kind, BackgroundLayerKind::Terrain);
        assert!(catalog.layers[0].path.ends_with("overview_terrain.png"));
        assert_eq!(catalog.layers[1].kind, BackgroundLayerKind::Hillshade);
        assert!(catalog.layers[1].path.ends_with("overview_hillshade.png"));
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

    #[test]
    fn try_load_background_layer_bundle_requires_terrain_base() {
        let temp_dir = TempDirGuard::new("background_layers_missing_terrain");
        let hillshade = RgbaImage::from_pixel(2, 2, Rgba([220, 0, 0, 128]));
        write_png(&temp_dir.path().join("overview_hillshade.png"), &hillshade);

        let mut state = AppState::new();
        let loaded = try_load_background_layer_bundle_from_directory(&mut state, temp_dir.path())
            .expect("Discovery ohne Terrain darf nicht fehlschlagen");

        assert!(!loaded);
        assert!(state.background_layers.is_none());
        assert!(state.view.background_map.is_none());
    }

    #[test]
    fn set_background_layer_visibility_recomposes_image_and_preserves_scale() {
        let temp_dir = TempDirGuard::new("background_layers_toggle");
        let terrain = RgbaImage::from_pixel(2, 2, Rgba([20, 40, 60, 255]));
        let hillshade = RgbaImage::from_pixel(2, 2, Rgba([220, 0, 0, 128]));
        write_png(&temp_dir.path().join("overview_terrain.png"), &terrain);
        write_png(&temp_dir.path().join("overview_hillshade.png"), &hillshade);

        let files = discover_background_layer_files(temp_dir.path());
        let visible = OverviewLayerOptions {
            terrain: true,
            hillshade: true,
            farmlands: false,
            farmland_ids: false,
            pois: false,
            legend: false,
        };
        let catalog = load_background_layer_catalog(files, &visible)
            .expect("Katalog muss fuer Toggle-Test ladbar sein");
        let composed =
            compose_background_from_catalog(&catalog).expect("Ausgangsbild muss komponierbar sein");
        let background = BackgroundMap::from_image(composed, "toggle-test", None)
            .expect("Background-Map muss aus Ausgangsbild erzeugbar sein");

        let mut state = AppState::new();
        state.view.background_map = Some(Arc::new(background));
        state.background_image = state
            .view
            .background_map
            .as_ref()
            .map(|background| background.image_arc());
        state.view.background_scale = 1.75;
        state.background_layers = Some(catalog);

        set_background_layer_visibility(&mut state, BackgroundLayerKind::Hillshade, false)
            .expect("Layer-Toggle muss das Bild neu zusammensetzen");

        let catalog = state
            .background_layers
            .as_ref()
            .expect("Layer-Katalog muss im State erhalten bleiben");
        assert!(!catalog.visible.hillshade);
        assert_eq!(catalog.layers.len(), 2);
        assert_eq!(state.view.background_scale, 1.75);
        assert_eq!(state.view.background_asset_revision, 1);

        let pixel = state
            .background_image
            .as_ref()
            .expect("Komponiertes Bild muss im State liegen")
            .to_rgba8()
            .get_pixel(0, 0)
            .0;
        assert_eq!(pixel, [20, 40, 60, 255]);
    }

    #[test]
    fn set_background_layer_visibility_keeps_state_on_recompose_error() {
        let temp_dir = TempDirGuard::new("background_layers_toggle_error");
        let terrain = RgbaImage::from_pixel(2, 2, Rgba([20, 40, 60, 255]));
        let hillshade = RgbaImage::from_pixel(3, 3, Rgba([220, 0, 0, 128]));
        write_png(&temp_dir.path().join("overview_terrain.png"), &terrain);
        write_png(&temp_dir.path().join("overview_hillshade.png"), &hillshade);

        let files = discover_background_layer_files(temp_dir.path());
        let visible = OverviewLayerOptions {
            terrain: true,
            hillshade: false,
            farmlands: false,
            farmland_ids: false,
            pois: false,
            legend: false,
        };
        let catalog = load_background_layer_catalog(files, &visible)
            .expect("Katalog muss fuer Fehlerpfad-Test ladbar sein");
        let composed =
            compose_background_from_catalog(&catalog).expect("Ausgangsbild muss komponierbar sein");
        let background = BackgroundMap::from_image(composed, "toggle-error-test", None)
            .expect("Background-Map muss aus Ausgangsbild erzeugbar sein");

        let mut state = AppState::new();
        state.view.background_map = Some(Arc::new(background));
        state.background_image = state
            .view
            .background_map
            .as_ref()
            .map(|background| background.image_arc());
        state.view.background_scale = 1.75;
        state.background_layers = Some(catalog);

        let error =
            set_background_layer_visibility(&mut state, BackgroundLayerKind::Hillshade, true)
                .expect_err("Dimensionsfehler beim Re-Komponieren muss durchgereicht werden");

        assert!(
            error.to_string().contains("abweichende Dimensionen"),
            "Fehler muss den Dimensionskonflikt benennen: {error:#}"
        );

        let catalog = state
            .background_layers
            .as_ref()
            .expect("Layer-Katalog muss im State erhalten bleiben");
        assert!(!catalog.visible.hillshade);
        assert_eq!(catalog.layers.len(), 2);
        assert_eq!(state.view.background_scale, 1.75);
        assert_eq!(state.view.background_asset_revision, 0);

        let pixel = state
            .background_image
            .as_ref()
            .expect("Ausgangsbild muss im State erhalten bleiben")
            .to_rgba8()
            .get_pixel(0, 0)
            .0;
        assert_eq!(pixel, [20, 40, 60, 255]);
    }
}
