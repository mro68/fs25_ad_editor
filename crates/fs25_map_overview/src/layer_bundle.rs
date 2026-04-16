//! Layer-Bundle-API fuer separat speicherbare Overview-PNGs.

use std::collections::HashMap;

use anyhow::Result;
use image::{GrayImage, Rgb, RgbImage, Rgba, RgbaImage};

use crate::composite::{self, FarmlandData, OverviewOptions, Poi};
use crate::discovery::MapInfo;
use crate::hillshade::{self, HillshadeParams};
use crate::{terrain, FarmlandPolygon};

/// Separat generierte Bild-Layer einer Uebersichtskarte.
///
/// `terrain` enthaelt das opake Terrain-Basisbild inklusive Title-Bar.
/// Alle weiteren Bildfelder sind transparente RGBA-Overlays, die per
/// [`compose_layers`] wieder zu `combined` zusammengesetzt werden koennen.
pub struct OverviewLayerBundle {
    /// Opakes Terrain-Basisbild inklusive Title-Bar.
    pub terrain: RgbaImage,
    /// Transparente Hillshade-Schattierung.
    pub hillshade: RgbaImage,
    /// Transparente Farmland-Grenzen.
    pub farmland_borders: RgbaImage,
    /// Transparente Farmland-ID-Beschriftungen.
    pub farmland_ids: RgbaImage,
    /// Transparente POI-Marker und Labels.
    pub poi_markers: RgbaImage,
    /// Transparente Legende.
    pub legend: RgbaImage,
    /// Aus den sichtbaren Layern zusammengesetztes Ergebnisbild.
    pub combined: RgbaImage,
    /// Extrahierte Farmland-Polygone im Pixel-Koordinatenraum.
    pub farmland_polygons: Vec<FarmlandPolygon>,
    /// Rasterbreite der Farmland-Daten in Pixeln.
    pub grle_width: u32,
    /// Rasterhoehe der Farmland-Daten in Pixeln.
    pub grle_height: u32,
    /// Weltgroesse der Karte in Metern.
    pub map_size: f32,
    /// Rohe Farmland-ID-Pixel fuer spaetere Editor-Analysen.
    pub farmland_ids_raw: Option<Vec<u8>>,
}

/// Setzt Terrain-Basisbild und aktive transparente Layer zu einem Gesamtbild zusammen.
pub fn compose_layers(terrain: &RgbaImage, layers: &[(bool, &RgbaImage)]) -> RgbaImage {
    let mut combined = terrain.clone();

    for (is_visible, layer) in layers {
        if !*is_visible {
            continue;
        }

        debug_assert_eq!(combined.dimensions(), layer.dimensions());

        for (dst_pixel, src_pixel) in combined.pixels_mut().zip(layer.pixels()) {
            *dst_pixel = blend_pixel(*dst_pixel, *src_pixel);
        }
    }

    combined
}

/// Generiert alle Overview-Layer aus bereits extrahierten Dateien.
///
/// Die `options` steuern nur, welche Layer initial in `combined` sichtbar sind.
/// Alle verfuegbaren Einzel-Layer werden unabhaengig davon erzeugt.
pub fn generate_overview_layer_bundle(
    files: &HashMap<String, Vec<u8>>,
    map_info: &MapInfo,
    options: &OverviewOptions,
) -> Result<OverviewLayerBundle> {
    let terrain_base = render_terrain_base(files, map_info)?;
    let terrain_rgb = render_terrain_with_title(&terrain_base, &map_info.title);
    let terrain = rgb_to_opaque_rgba(&terrain_rgb);

    let hillshade = if let Some(dem) = load_resized_dem(files, map_info) {
        render_hillshade_layer(&terrain_base, &terrain_rgb, &map_info.title, &dem)
    } else {
        blank_layer(map_info.map_size, map_info.map_size)
    };

    let farmland_data = load_farmland_data(files, map_info);
    let farmland_borders = farmland_data
        .as_ref()
        .map(|farmlands| {
            render_farmland_borders_layer(&terrain_base, &terrain_rgb, &map_info.title, farmlands)
        })
        .unwrap_or_else(|| blank_layer(map_info.map_size, map_info.map_size));
    let farmland_ids = farmland_data
        .as_ref()
        .map(|farmlands| {
            render_farmland_ids_layer(&terrain_base, &terrain_rgb, &map_info.title, farmlands)
        })
        .unwrap_or_else(|| blank_layer(map_info.map_size, map_info.map_size));

    let pois = load_pois(files, map_info);
    let poi_markers = if pois.is_empty() {
        blank_layer(map_info.map_size, map_info.map_size)
    } else {
        render_poi_markers_layer(&terrain_base, &terrain_rgb, &map_info.title, &pois)
    };

    let legend_options = OverviewOptions {
        terrain: true,
        hillshade: true,
        farmlands: farmland_data.is_some(),
        farmland_ids: true,
        pois: !pois.is_empty(),
        legend: true,
    };
    let legend = render_legend_layer(
        &terrain_base,
        &terrain_rgb,
        &map_info.title,
        &legend_options,
    );

    let combined_base = if options.terrain {
        terrain.clone()
    } else {
        blank_layer(terrain.width(), terrain.height())
    };
    let combined = compose_layers(
        &combined_base,
        &[
            (options.hillshade, &hillshade),
            (options.farmlands, &farmland_borders),
            (options.farmland_ids, &farmland_ids),
            (options.pois, &poi_markers),
            (options.legend, &legend),
        ],
    );

    let (farmland_polygons, grle_width, grle_height, farmland_ids_raw) =
        crate::try_extract_polygons_from_files(files, map_info);

    Ok(OverviewLayerBundle {
        terrain,
        hillshade,
        farmland_borders,
        farmland_ids,
        poi_markers,
        legend,
        combined,
        farmland_polygons,
        grle_width,
        grle_height,
        map_size: map_info.map_size as f32,
        farmland_ids_raw,
    })
}

fn render_terrain_base(files: &HashMap<String, Vec<u8>>, map_info: &MapInfo) -> Result<RgbImage> {
    let map_size = map_info.map_size;
    let weight_maps = crate::discovery::find_weight_maps(files, &map_info.data_dir);
    let weight_images: Vec<(String, image::DynamicImage)> = weight_maps
        .iter()
        .filter_map(|(path, data)| {
            let img = image::load_from_memory(data).ok()?;
            let name = std::path::Path::new(path)
                .file_name()?
                .to_str()?
                .to_string();
            Some((name, img))
        })
        .collect();

    log::info!("{} Weight-Maps geladen", weight_images.len());

    if weight_images.is_empty() {
        Ok(RgbImage::from_pixel(map_size, map_size, Rgb([80, 100, 60])))
    } else {
        terrain::composite_terrain_from_images(&weight_images, map_size)
    }
}

fn render_terrain_with_title(terrain_base: &RgbImage, title: &str) -> RgbImage {
    let mut terrain = terrain_base.clone();
    composite::draw_title_bar(&mut terrain, title);
    terrain
}

fn load_resized_dem(files: &HashMap<String, Vec<u8>>, map_info: &MapInfo) -> Option<GrayImage> {
    let dem_data = crate::discovery::find_dem(files, &map_info.data_dir)?;
    match image::load_from_memory(dem_data) {
        Ok(dem_img) => {
            let dem_gray = dem_img.to_luma8();
            let map_size = map_info.map_size;
            if dem_gray.width() != map_size || dem_gray.height() != map_size {
                Some(image::imageops::resize(
                    &dem_gray,
                    map_size,
                    map_size,
                    image::imageops::FilterType::Lanczos3,
                ))
            } else {
                Some(dem_gray)
            }
        }
        Err(error) => {
            log::warn!("DEM konnte nicht geladen werden: {}", error);
            None
        }
    }
}

fn load_farmland_data(
    files: &HashMap<String, Vec<u8>>,
    map_info: &MapInfo,
) -> Option<FarmlandData> {
    let (path, data) = crate::discovery::find_farmlands(files, &map_info.data_dir)?;
    if !path.ends_with(".grle") {
        log::info!("Farmlands als PNG gefunden – grenzlose Variante nicht implementiert");
        return None;
    }

    match composite::extract_farmland_boundaries(data, map_info.map_size) {
        Ok(farmlands) => Some(farmlands),
        Err(error) => {
            log::warn!("Farmland-Verarbeitung fehlgeschlagen: {}", error);
            None
        }
    }
}

fn load_pois(files: &HashMap<String, Vec<u8>>, map_info: &MapInfo) -> Vec<Poi> {
    let Some(placeables_path) = &map_info.placeables_path else {
        return Vec::new();
    };

    let Some(xml_data) = files.get(placeables_path.as_str()) else {
        log::info!("placeables.xml nicht gefunden: {}", placeables_path);
        return Vec::new();
    };

    composite::extract_pois(xml_data, map_info.map_size)
}

fn render_hillshade_layer(
    terrain_base: &RgbImage,
    terrain_with_title: &RgbImage,
    title: &str,
    dem: &GrayImage,
) -> RgbaImage {
    let params = HillshadeParams::default();
    match hillshade::compute_hillshade(dem, &params) {
        Ok(hillshade_values) => {
            render_layer_from_renderer(terrain_base, terrain_with_title, title, |image| {
                hillshade::apply_hillshade(image.as_mut(), &hillshade_values, params.blend_factor)
            })
        }
        Err(error) => {
            log::warn!("Hillshade-Berechnung fehlgeschlagen: {}", error);
            blank_layer(terrain_base.width(), terrain_base.height())
        }
    }
}

fn render_farmland_borders_layer(
    terrain_base: &RgbImage,
    terrain_with_title: &RgbImage,
    title: &str,
    farmlands: &FarmlandData,
) -> RgbaImage {
    render_layer_from_renderer(terrain_base, terrain_with_title, title, |image| {
        composite::draw_farmland_boundaries(image, farmlands);
    })
}

fn render_farmland_ids_layer(
    terrain_base: &RgbImage,
    terrain_with_title: &RgbImage,
    title: &str,
    farmlands: &FarmlandData,
) -> RgbaImage {
    render_layer_from_renderer(terrain_base, terrain_with_title, title, |image| {
        composite::draw_farmland_ids(image, farmlands);
    })
}

fn render_poi_markers_layer(
    terrain_base: &RgbImage,
    terrain_with_title: &RgbImage,
    title: &str,
    pois: &[Poi],
) -> RgbaImage {
    render_layer_from_renderer(terrain_base, terrain_with_title, title, |image| {
        composite::draw_pois_with_labels(image, pois);
    })
}

fn render_legend_layer(
    terrain_base: &RgbImage,
    terrain_with_title: &RgbImage,
    title: &str,
    options: &OverviewOptions,
) -> RgbaImage {
    render_layer_from_renderer(terrain_base, terrain_with_title, title, |image| {
        composite::draw_legend(image, options);
    })
}

fn render_layer_from_renderer(
    terrain_base: &RgbImage,
    terrain_with_title: &RgbImage,
    title: &str,
    render: impl FnOnce(&mut RgbImage),
) -> RgbaImage {
    let mut rendered = terrain_base.clone();
    render(&mut rendered);
    composite::draw_title_bar(&mut rendered, title);
    derive_overlay_from_base(terrain_with_title, &rendered)
}

fn derive_overlay_from_base(base: &RgbImage, rendered: &RgbImage) -> RgbaImage {
    debug_assert_eq!(base.dimensions(), rendered.dimensions());

    RgbaImage::from_fn(base.width(), base.height(), |x, y| {
        derive_overlay_pixel(base.get_pixel(x, y), rendered.get_pixel(x, y))
    })
}

fn derive_overlay_pixel(base: &Rgb<u8>, rendered: &Rgb<u8>) -> Rgba<u8> {
    if base == rendered {
        return Rgba([0, 0, 0, 0]);
    }

    let base_rgb = [
        base[0] as f32 / 255.0,
        base[1] as f32 / 255.0,
        base[2] as f32 / 255.0,
    ];
    let rendered_rgb = [
        rendered[0] as f32 / 255.0,
        rendered[1] as f32 / 255.0,
        rendered[2] as f32 / 255.0,
    ];

    let mut alpha = 0.0_f32;
    for channel in 0..3 {
        let base_value = base_rgb[channel];
        let rendered_value = rendered_rgb[channel];
        if rendered_value > base_value {
            let remaining = (1.0 - base_value).max(f32::EPSILON);
            alpha = alpha.max((rendered_value - base_value) / remaining);
        } else if rendered_value < base_value {
            let base_amount = base_value.max(f32::EPSILON);
            alpha = alpha.max((base_value - rendered_value) / base_amount);
        }
    }

    let alpha_u8 = (alpha.clamp(0.0, 1.0) * 255.0).ceil() as u8;
    if alpha_u8 == 0 {
        return Rgba([0, 0, 0, 0]);
    }

    let alpha_factor = alpha_u8 as f32 / 255.0;
    let mut overlay = [0u8; 4];
    for channel in 0..3 {
        let base_value = base_rgb[channel];
        let rendered_value = rendered_rgb[channel];
        let overlay_value =
            ((rendered_value - base_value * (1.0 - alpha_factor)) / alpha_factor) * 255.0;
        overlay[channel] = overlay_value.round().clamp(0.0, 255.0) as u8;
    }
    overlay[3] = alpha_u8;

    Rgba(overlay)
}

fn rgb_to_opaque_rgba(image: &RgbImage) -> RgbaImage {
    RgbaImage::from_fn(image.width(), image.height(), |x, y| {
        let pixel = image.get_pixel(x, y);
        Rgba([pixel[0], pixel[1], pixel[2], 255])
    })
}

fn blank_layer(width: u32, height: u32) -> RgbaImage {
    RgbaImage::new(width, height)
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

#[cfg(test)]
mod tests {
    use super::*;
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
                "fs25_map_overview_{}_{}_{}",
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
        let image = DynamicImage::ImageRgba8(RgbaImage::from_pixel(width, height, Rgba(rgba)));
        let mut cursor = Cursor::new(Vec::new());
        image
            .write_to(&mut cursor, ImageFormat::Png)
            .expect("PNG muss erzeugt werden");
        cursor.into_inner()
    }

    fn luma_png_bytes(width: u32, height: u32, pixels: Vec<u8>) -> Vec<u8> {
        let image = GrayImage::from_raw(width, height, pixels)
            .expect("Graustufenbild muss die erwarteten Dimensionen haben");
        let mut cursor = Cursor::new(Vec::new());
        DynamicImage::ImageLuma8(image)
            .write_to(&mut cursor, ImageFormat::Png)
            .expect("Graustufen-PNG muss erzeugt werden");
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
    fn compose_layers_blends_active_layers_in_order() {
        let terrain = RgbaImage::from_pixel(1, 1, Rgba([100, 120, 140, 255]));
        let red = RgbaImage::from_pixel(1, 1, Rgba([200, 0, 0, 128]));
        let blue = RgbaImage::from_pixel(1, 1, Rgba([0, 0, 200, 128]));

        let result = compose_layers(&terrain, &[(true, &red), (true, &blue)]);
        let expected = blend_pixel(
            blend_pixel(Rgba([100, 120, 140, 255]), Rgba([200, 0, 0, 128])),
            Rgba([0, 0, 200, 128]),
        );

        assert_eq!(*result.get_pixel(0, 0), expected);
    }

    #[test]
    fn generate_overview_layer_bundle_from_zip_returns_rgba_layers() {
        let temp_dir = TempDirGuard::new("layer_bundle");
        let zip_path = temp_dir.path().join("test_map.zip");

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
                (
                    "TestMap/maps/data/infoLayer_farmlands.png",
                    luma_png_bytes(2, 2, vec![0, 1, 1, 0]),
                ),
            ],
        );

        let options = OverviewOptions {
            terrain: true,
            hillshade: true,
            farmlands: false,
            farmland_ids: false,
            pois: false,
            legend: false,
        };

        let bundle = crate::generate_overview_layer_bundle_from_zip(
            zip_path.to_str().expect("Temp-ZIP-Pfad muss UTF-8 sein"),
            &options,
        )
        .expect("Layer-Bundle aus Test-ZIP muss erzeugt werden");

        assert_eq!(bundle.terrain.dimensions(), (32, 32));
        assert_eq!(bundle.hillshade.dimensions(), (32, 32));
        assert_eq!(bundle.farmland_borders.dimensions(), (32, 32));
        assert_eq!(bundle.farmland_ids.dimensions(), (32, 32));
        assert_eq!(bundle.poi_markers.dimensions(), (32, 32));
        assert_eq!(bundle.legend.dimensions(), (32, 32));
        assert_eq!(bundle.combined.dimensions(), (32, 32));
        assert!(bundle.terrain.pixels().all(|pixel| pixel[3] == 255));
        assert!(bundle.hillshade.pixels().any(|pixel| pixel[3] > 0));
        assert!(bundle.poi_markers.pixels().all(|pixel| pixel[3] == 0));
        assert_eq!(bundle.farmland_ids_raw, Some(vec![0, 1, 1, 0]));
    }
}
