//! `fs25_map_overview` — Overview-Map-Generator fuer FS25 Map-Mods.
//!
//! Generiert aus einem FS25 Map-Mod-ZIP eine detaillierte Uebersichtskarte:
//! - Terrain-Compositing aus Weight-Maps
//! - Hillshade aus DEM (Digital Elevation Model)
//! - Farmland-Grenzen und ID-Labels
//! - POI-Marker mit Beschriftung
//! - Legende und Titel-Bar
//!
//! # Beispiel
//! ```no_run
//! use fs25_map_overview::{generate_overview_from_zip, OverviewOptions};
//!
//! let image = generate_overview_from_zip("path/to/map.zip", &OverviewOptions::default())?;
//! image.save("overview.png")?;
//! # Ok::<(), anyhow::Error>(())
//! ```
//!
//! GRLE/GDM-Decoder basieren auf grleconvert von Kim Brandwijk (MIT-Lizenz).
//! <https://github.com/Paint-a-Farm/grleconvert>

pub mod composite;
pub mod discovery;
pub mod farmland;
pub mod gdm;
pub mod grle;
pub mod hillshade;
pub mod palette;
pub mod terrain;
pub mod text;

use anyhow::{Context, Result};
use image::{DynamicImage, RgbImage};
use std::collections::HashMap;
use std::io::{BufReader, Read};
use std::path::Path;

pub use composite::{OverviewOptions, Poi};
pub use discovery::MapInfo;
pub use farmland::{
    extract_farmland_polygons, extract_farmland_polygons_from_ids, extract_field_polygons_by_ccl,
    extract_field_type_polygons_from_ids, FarmlandPolygon,
};

/// Quelle fuer die Feldpolygon-Erkennung beim Generieren der Uebersichtskarte.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum FieldDetectionSource {
    /// Aus infoLayer_farmlands (Map-ZIP) — bisherige Methode
    FromZip,
    /// Aus densityMap_ground.gdm (Map-ZIP)
    #[default]
    ZipGroundGdm,
    /// Aus infoLayer_fieldType.grle (Savegame)
    FieldTypeGrle,
    /// Aus densityMap_ground.gdm (Savegame)
    GroundGdm,
    /// Aus densityMap_fruits.gdm (Savegame)
    FruitsGdm,
}

/// Ergebnis der Overview-Generierung mit optionalen Farmland-Polygonen.
pub struct OverviewResult {
    /// Generiertes Uebersichtsbild als dynamisches Bild
    pub image: DynamicImage,
    /// Extrahierte Farmland-Polygone im Pixel-Koordinatenraum des GRLE-Rasters
    pub farmland_polygons: Vec<FarmlandPolygon>,
    /// Kartengroesse des GRLE-Rasters in Pixeln (= Weltgroesse in Metern)
    pub grle_width: u32,
    /// GRLE-Rasterhoehe in Pixeln (= Weltgroesse in Metern)
    pub grle_height: u32,
    /// Weltgroesse in Metern (aus MapInfo)
    pub map_size: f32,
    /// Rohes Farmland-ID-Raster (1 Byte pro Pixel, 0 = kein Feld).
    /// Fuer Pixel-basierte Analysen im Editor (z.B. Feldweg-Erkennung).
    pub farmland_ids: Option<Vec<u8>>,
}

/// Generiert eine Overview-Map aus einem FS25 Map-Mod-ZIP.
///
/// # Parameter
/// - `zip_path`: Pfad zum ZIP-Archiv
/// - `options`: Steuerung welche Layer gezeichnet werden
///
/// # Rueckgabe
/// RGB-Bild als `image::RgbImage`
pub fn generate_overview_from_zip(zip_path: &str, options: &OverviewOptions) -> Result<RgbImage> {
    log::info!("Lade Map-Mod-ZIP: {}", zip_path);

    // 1. ZIP entpacken → HashMap<Pfad, Bytes>
    let files = extract_zip(zip_path)?;

    // 2. Kartenstruktur erkennen
    let map_info = discovery::discover_map(&files)?;
    log::info!(
        "Map: '{}', {}x{}",
        map_info.title,
        map_info.map_size,
        map_info.map_size
    );

    generate_overview(&files, &map_info, options)
}

/// Generiert eine Overview-Map aus bereits extrahierten Dateien.
///
/// Nuetzlich wenn das ZIP bereits entpackt vorliegt oder
/// die MapInfo anderweitig ermittelt wurde.
pub fn generate_overview(
    files: &HashMap<String, Vec<u8>>,
    map_info: &MapInfo,
    options: &OverviewOptions,
) -> Result<RgbImage> {
    let map_size = map_info.map_size;

    // 3. Weight-Maps laden und compositen
    let weight_maps = discovery::find_weight_maps(files, &map_info.data_dir);
    let weight_images: Vec<(String, DynamicImage)> = weight_maps
        .iter()
        .filter_map(|(path, data)| {
            let img = image::load_from_memory(data).ok()?;
            let name = Path::new(path).file_name()?.to_str()?.to_string();
            Some((name, img))
        })
        .collect();

    log::info!("{} Weight-Maps geladen", weight_images.len());

    let mut image = if weight_images.is_empty() {
        // Fallback: einheitliches Gruen
        RgbImage::from_pixel(map_size, map_size, image::Rgb([80, 100, 60]))
    } else {
        terrain::composite_terrain_from_images(&weight_images, map_size)?
    };

    // 4. Hillshade
    if options.hillshade {
        if let Some(dem_data) = discovery::find_dem(files, &map_info.data_dir) {
            match image::load_from_memory(dem_data) {
                Ok(dem_img) => {
                    let dem_gray = dem_img.to_luma8();
                    let dem_resized =
                        if dem_gray.width() != map_size || dem_gray.height() != map_size {
                            image::imageops::resize(
                                &dem_gray,
                                map_size,
                                map_size,
                                image::imageops::FilterType::Lanczos3,
                            )
                        } else {
                            dem_gray
                        };
                    let params = hillshade::HillshadeParams::default();
                    match hillshade::compute_hillshade(&dem_resized, &params) {
                        Ok(hs) => {
                            hillshade::apply_hillshade(image.as_mut(), &hs, params.blend_factor);
                            log::info!("Hillshade angewendet");
                        }
                        Err(e) => log::warn!("Hillshade-Berechnung fehlgeschlagen: {}", e),
                    }
                }
                Err(e) => log::warn!("DEM konnte nicht geladen werden: {}", e),
            }
        } else {
            log::info!("Kein DEM gefunden – Hillshade uebersprungen");
        }
    }

    // 5. Farmland-Grenzen + IDs
    if options.farmlands || options.farmland_ids {
        if let Some((path, data)) = discovery::find_farmlands(files, &map_info.data_dir) {
            let is_grle = path.ends_with(".grle");
            if is_grle {
                match composite::extract_farmland_boundaries(data, map_size) {
                    Ok(farmlands) => {
                        if options.farmlands {
                            composite::draw_farmland_boundaries(&mut image, &farmlands);
                            log::info!("Farmland-Grenzen gezeichnet");
                        }
                        if options.farmland_ids {
                            composite::draw_farmland_ids(&mut image, &farmlands);
                            log::info!("Farmland-IDs gezeichnet");
                        }
                    }
                    Err(e) => log::warn!("Farmland-Verarbeitung fehlgeschlagen: {}", e),
                }
            } else {
                // PNG-basierte Farmlands (bereits dekodiert)
                log::info!("Farmlands als PNG gefunden – grenzlose Variante nicht implementiert");
            }
        } else {
            log::info!("Keine Farmland-Daten gefunden");
        }
    }

    // 6. POIs
    if options.pois
        && let Some(placeables_path) = &map_info.placeables_path
    {
        if let Some(xml_data) = files.get(placeables_path.as_str()) {
            let pois = composite::extract_pois(xml_data, map_size);
            if !pois.is_empty() {
                composite::draw_pois_with_labels(&mut image, &pois);
                log::info!("{} POIs gezeichnet", pois.len());
            }
        } else {
            log::info!("placeables.xml nicht gefunden: {}", placeables_path);
        }
    }

    // 7. Legende
    if options.legend {
        composite::draw_legend(&mut image, options);
        log::info!("Legende gezeichnet");
    }

    // 8. Titel-Bar
    composite::draw_title_bar(&mut image, &map_info.title);

    Ok(image)
}

/// Versucht Feldpolygone aus einer `infoLayer_fieldType.grle`-Datei zu lesen.
///
/// Liegt die Datei im Savegame-Ordner (neben `AutoDrive_config.xml`), liefert
/// diese Funktion Polygone fuer alle Pixel-IDs ≥ 1 (Frucht-ID). Pixelwert 255
/// ist hier ein **gueltiger** Feldtyp und wird nicht herausgefiltert.
///
/// Rueckgabe: `Some((polygons, grle_width, grle_height))` oder `None` bei
/// fehlender Datei oder Dekodierungsfehler.
pub fn try_extract_polygons_from_field_type_grle(
    path: &Path,
) -> Option<(Vec<FarmlandPolygon>, u32, u32)> {
    let data = std::fs::read(path)
        .map_err(|e| {
            log::warn!(
                "FieldType-GRLE lesen fehlgeschlagen ({}): {}",
                path.display(),
                e
            )
        })
        .ok()?;
    let decoded = grle::decode_grle(&data)
        .map_err(|e| log::warn!("FieldType-GRLE Dekodierung fehlgeschlagen: {}", e))
        .ok()?;
    let w = decoded.width;
    let h = decoded.height;
    let polygons = farmland::extract_field_polygons_by_ccl(&decoded.pixels, w, h);
    if polygons.is_empty() {
        log::info!("Keine Feldpolygone in FieldType-GRLE gefunden");
        return None;
    }
    log::info!(
        "FieldType-Polygone extrahiert: {} Felder aus {}x{} Raster",
        polygons.len(),
        w,
        h
    );
    Some((polygons, w as u32, h as u32))
}

/// Versucht Feldpolygone aus einer `densityMap_ground.gdm`-Datei zu lesen.
///
/// Dekodiert die GDM-Datei (16 Kanaele, RGB-Encoding) und extrahiert
/// den Feld-Status aus dem unteren Nibble des R-Kanals (Bits 0–3).
/// Pixel mit Wert != 0 werden als Feld gewertet.
///
/// Rueckgabe: `Some((polygons, width, height))` oder `None` bei
/// fehlender Datei oder Dekodierungsfehler.
pub fn try_extract_polygons_from_ground_gdm(
    path: &Path,
) -> Option<(Vec<FarmlandPolygon>, u32, u32)> {
    let data = std::fs::read(path)
        .map_err(|e| {
            log::warn!(
                "Ground-GDM lesen fehlgeschlagen ({}): {}",
                path.display(),
                e
            )
        })
        .ok()?;
    try_extract_polygons_from_ground_gdm_bytes(&data)
}

/// Versucht Feldpolygone aus `densityMap_ground.gdm` innerhalb eines Map-ZIPs zu lesen.
pub fn try_extract_polygons_from_zip_ground_gdm(
    zip_path: &str,
) -> Option<(Vec<FarmlandPolygon>, u32, u32)> {
    let files = extract_zip(zip_path)
        .map_err(|e| {
            log::warn!(
                "ZIP-Extraktion fuer Ground-GDM fehlgeschlagen ({}): {}",
                zip_path,
                e
            )
        })
        .ok()?;
    let map_info = discovery::discover_map(&files)
        .map_err(|e| {
            log::warn!(
                "Map-Discovery fuer Ground-GDM fehlgeschlagen ({}): {}",
                zip_path,
                e
            )
        })
        .ok()?;
    let (path, data) = discovery::find_ground_gdm(&files, &map_info.data_dir)?;
    log::info!("Ground-GDM im ZIP gefunden: {}", path);
    try_extract_polygons_from_ground_gdm_bytes(data)
}

fn try_extract_polygons_from_ground_gdm_bytes(
    data: &[u8],
) -> Option<(Vec<FarmlandPolygon>, u32, u32)> {
    let img = gdm::decode_gdm(&data)
        .map_err(|e| log::warn!("Ground-GDM Dekodierung fehlgeschlagen: {}", e))
        .ok()?;
    let dim = img.dimension;
    // RGB-Encoding: R-Kanal enthaelt Bits 0–7; unteres Nibble = Feld-/Bodentyp
    let converted: Vec<u8> = if img.is_rgb {
        img.pixels.chunks(3).map(|rgb| rgb[0] & 0x0F).collect()
    } else {
        img.pixels.iter().map(|&b| b & 0x0F).collect()
    };
    let polygons = farmland::extract_field_polygons_by_ccl(&converted, dim, dim);
    if polygons.is_empty() {
        log::info!("Keine Feldpolygone in Ground-GDM gefunden");
        return None;
    }
    log::info!(
        "Ground-GDM-Polygone extrahiert: {} Felder aus {}x{} Raster",
        polygons.len(),
        dim,
        dim
    );
    Some((polygons, dim as u32, dim as u32))
}

/// Versucht Feldpolygone aus einer `densityMap_fruits.gdm`-Datei zu lesen.
///
/// Dekodiert die GDM-Datei (12 Kanaele, RGB-Encoding) und extrahiert
/// die Frucht-ID aus den unteren 6 Bits des R-Kanals (Bits 0–5).
/// Pixel mit Frucht-ID != 0 werden als Feld gewertet.
///
/// Rueckgabe: `Some((polygons, width, height))` oder `None` bei
/// fehlender Datei oder Dekodierungsfehler.
pub fn try_extract_polygons_from_fruits_gdm(
    path: &Path,
) -> Option<(Vec<FarmlandPolygon>, u32, u32)> {
    let data = std::fs::read(path)
        .map_err(|e| {
            log::warn!(
                "Fruits-GDM lesen fehlgeschlagen ({}): {}",
                path.display(),
                e
            )
        })
        .ok()?;
    let img = gdm::decode_gdm(&data)
        .map_err(|e| log::warn!("Fruits-GDM Dekodierung fehlgeschlagen: {}", e))
        .ok()?;
    let dim = img.dimension;
    // RGB-Encoding: R-Kanal enthaelt Bits 0–7; untere 6 Bit = Frucht-ID
    let converted: Vec<u8> = if img.is_rgb {
        img.pixels.chunks(3).map(|rgb| rgb[0] & 0x3F).collect()
    } else {
        img.pixels.iter().map(|&b| b & 0x3F).collect()
    };
    let polygons = farmland::extract_field_polygons_by_ccl(&converted, dim, dim);
    if polygons.is_empty() {
        log::info!("Keine Feldpolygone in Fruits-GDM gefunden");
        return None;
    }
    log::info!(
        "Fruits-GDM-Polygone extrahiert: {} Felder aus {}x{} Raster",
        polygons.len(),
        dim,
        dim
    );
    Some((polygons, dim as u32, dim as u32))
}

/// Extrahiert alle Dateien aus einem ZIP-Archiv in eine HashMap.
fn extract_zip(zip_path: &str) -> Result<HashMap<String, Vec<u8>>> {
    let file = std::fs::File::open(zip_path)
        .with_context(|| format!("ZIP-Datei nicht gefunden: {}", zip_path))?;
    let mut archive = zip::ZipArchive::new(BufReader::new(file))
        .with_context(|| format!("Ungueltiges ZIP-Archiv: {}", zip_path))?;

    let mut files = HashMap::new();
    for i in 0..archive.len() {
        let mut entry = archive.by_index(i)?;
        if entry.is_file() {
            let name = entry.name().to_string();
            let mut buffer = Vec::with_capacity(entry.size() as usize);
            entry.read_to_end(&mut buffer)?;
            files.insert(name, buffer);
        }
    }

    log::info!("ZIP entpackt: {} Dateien", files.len());
    Ok(files)
}

/// Generiert eine Overview-Map und extrahiert gleichzeitig Farmland-Polygone.
///
/// Gibt ein [`OverviewResult`] zurueck, das das Bild sowie die Roh-Polygone
/// aus dem GRLE-Farmland-Layer enthaelt. Die Polygon-Koordinaten liegen
/// im Pixel-Raum des GRLE-Rasters; der Aufrufer muss sie in Weltkoordinaten
/// umrechnen (`world = pixel * (map_size / grle_width)`).
///
/// Die bestehenden Optionen steuern weiterhin den Bild-Inhalt (Hillshade,
/// Farmland-Grenzen usw.). Die Polygon-Extraktion findet unabhaengig statt.
pub fn generate_overview_result_from_zip(
    zip_path: &str,
    options: &OverviewOptions,
) -> Result<OverviewResult> {
    log::info!(
        "Generiere Overview + Farmland-Polygone aus ZIP: {}",
        zip_path
    );

    let files = extract_zip(zip_path)?;
    let map_info = discovery::discover_map(&files)?;

    let rgb_image = generate_overview(&files, &map_info, options)?;

    let (farmland_polygons, grle_width, grle_height, farmland_ids) =
        try_extract_polygons_from_files(&files, &map_info);

    Ok(OverviewResult {
        image: DynamicImage::ImageRgb8(rgb_image),
        farmland_polygons,
        grle_width,
        grle_height,
        map_size: map_info.map_size as f32,
        farmland_ids,
    })
}

/// Versucht Farmland-Polygone und rohe ID-Pixeldaten aus den ZIP-Dateien zu extrahieren.
///
/// Schlaegt bei fehlenden oder fehlerhaften Daten still fehl (leere Liste, None).
/// Gibt `(polygons, width, height, raw_ids)` zurueck.
fn try_extract_polygons_from_files(
    files: &HashMap<String, Vec<u8>>,
    map_info: &discovery::MapInfo,
) -> (Vec<FarmlandPolygon>, u32, u32, Option<Vec<u8>>) {
    let Some((path, data)) = discovery::find_farmlands(files, &map_info.data_dir) else {
        log::info!("Keine Farmland-Daten gefunden – Polygone werden nicht extrahiert");
        return (Vec::new(), map_info.map_size, map_info.map_size, None);
    };

    if path.ends_with(".grle") {
        match crate::grle::decode_grle(data) {
            Ok(decoded) => {
                let w = decoded.width;
                let h = decoded.height;
                let polygons = farmland::extract_farmland_polygons_from_ids(&decoded.pixels, w, h);
                log::info!(
                    "Farmland-Polygone extrahiert: {} Felder aus {}x{} Raster",
                    polygons.len(),
                    w,
                    h
                );
                (polygons, w as u32, h as u32, Some(decoded.pixels))
            }
            Err(e) => {
                log::warn!("Farmland-GRLE-Dekodierung fehlgeschlagen: {}", e);
                (Vec::new(), map_info.map_size, map_info.map_size, None)
            }
        }
    } else if path.ends_with(".png") {
        // PNG-Farmland: Bild zu Graustufen decodieren, Pixelwerte als Farmland-IDs nutzen
        match image::load_from_memory(data) {
            Ok(img) => {
                let luma = img.to_luma8();
                let w = luma.width() as usize;
                let h = luma.height() as usize;
                let raw_ids = luma.as_raw().clone();
                let polygons = farmland::extract_farmland_polygons_from_ids(&raw_ids, w, h);
                log::info!(
                    "Farmland-Polygone aus PNG extrahiert: {} Felder aus {}x{} Raster",
                    polygons.len(),
                    w,
                    h
                );
                (polygons, w as u32, h as u32, Some(raw_ids))
            }
            Err(e) => {
                log::warn!("PNG-Farmland konnte nicht dekodiert werden: {}", e);
                (Vec::new(), map_info.map_size, map_info.map_size, None)
            }
        }
    } else {
        log::info!(
            "Farmland-Datei in unbekanntem Format ({}), Polygon-Extraktion uebersprungen",
            path
        );
        (Vec::new(), map_info.map_size, map_info.map_size, None)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use image::{DynamicImage, GrayImage, ImageFormat};
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
    fn generate_overview_result_from_zip_reads_nested_archive_entries() {
        let temp_dir = TempDirGuard::new("nested_zip");
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
            hillshade: false,
            farmlands: false,
            farmland_ids: false,
            pois: false,
            legend: false,
        };

        let result = generate_overview_result_from_zip(
            zip_path.to_str().expect("Temp-ZIP-Pfad muss UTF-8 sein"),
            &options,
        )
        .expect("Overview aus Test-ZIP muss erzeugt werden");

        assert_eq!(result.image.width(), 32);
        assert_eq!(result.image.height(), 32);
        assert_eq!(result.map_size, 32.0);
        assert_eq!(result.grle_width, 2);
        assert_eq!(result.grle_height, 2);
        assert_eq!(result.farmland_ids, Some(vec![0, 1, 1, 0]));
    }

    #[test]
    fn field_detection_source_defaults_to_zip_ground_gdm() {
        assert_eq!(
            FieldDetectionSource::default(),
            FieldDetectionSource::ZipGroundGdm
        );
    }
}
