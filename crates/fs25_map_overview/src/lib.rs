//! `fs25_map_overview` — Overview-Map-Generator für FS25 Map-Mods.
//!
//! Generiert aus einem FS25 Map-Mod-ZIP eine detaillierte Übersichtskarte:
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

/// Generiert eine Overview-Map aus einem FS25 Map-Mod-ZIP.
///
/// # Parameter
/// - `zip_path`: Pfad zum ZIP-Archiv
/// - `options`: Steuerung welche Layer gezeichnet werden
///
/// # Rückgabe
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
/// Nützlich wenn das ZIP bereits entpackt vorliegt oder
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
        // Fallback: einheitliches Grün
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
            log::info!("Kein DEM gefunden – Hillshade übersprungen");
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
    if options.pois {
        if let Some(placeables_path) = &map_info.placeables_path {
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

/// Extrahiert alle Dateien aus einem ZIP-Archiv in eine HashMap.
fn extract_zip(zip_path: &str) -> Result<HashMap<String, Vec<u8>>> {
    let file = std::fs::File::open(zip_path)
        .with_context(|| format!("ZIP-Datei nicht gefunden: {}", zip_path))?;
    let mut archive = zip::ZipArchive::new(BufReader::new(file))
        .with_context(|| format!("Ungültiges ZIP-Archiv: {}", zip_path))?;

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
