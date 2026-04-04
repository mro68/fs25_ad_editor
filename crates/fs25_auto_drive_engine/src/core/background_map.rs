//! Background-Map-Loader fuer Map-Hintergrund-Rendering.

use anyhow::{Context, Result};
use image::{DynamicImage, GenericImageView, ImageReader};
use std::io::{BufReader, Cursor, Read};
use std::sync::Arc;

use super::WorldBounds;

/// Bekannte Bild-Endungen fuer ZIP-Filterung
const IMAGE_EXTENSIONS: &[&str] = &["png", "jpg", "jpeg", "dds"];

/// Eintrag einer Bilddatei in einem ZIP-Archiv.
#[derive(Debug, Clone)]
pub struct ZipImageEntry {
    /// Dateiname im Archiv (inkl. Pfad)
    pub name: String,
    /// Unkomprimierte Dateigroesse in Bytes
    pub size: u64,
}

/// Background-Map fuer Map-Hintergrund-Rendering
pub struct BackgroundMap {
    /// Bilddaten (nach Center-Crop)
    image_data: Arc<DynamicImage>,
    /// Weltkoordinaten-Bereich
    world_bounds: WorldBounds,
    /// Opacity (0.0 = transparent, 1.0 = opak)
    opacity: f32,
}

impl BackgroundMap {
    /// Laedt eine Background-Map aus einer Datei
    ///
    /// Unterstuetzte Formate: PNG, JPG, JPEG, DDS
    ///
    /// # Parameter
    /// - `path`: Pfad zur Bilddatei
    /// - `crop_size`: Optionale Crop-Groesse (quadratisch). Falls angegeben, wird ein Center-Crop durchgefuehrt.
    pub fn load_from_file(path: &str, crop_size: Option<u32>) -> Result<Self> {
        // Zuerst versuchen wir die Erkennung anhand der Dateiendung.
        // Falls das fehlschlaegt (z.B. .dds-Datei die eigentlich PNG ist),
        // erkennen wir das Format anhand der Magic Bytes im Dateiinhalt.
        let image = match image::open(path) {
            Ok(img) => img,
            Err(ext_err) => {
                log::warn!(
                    "Format-Erkennung via Dateiendung fehlgeschlagen fuer '{}': {}. Versuche Erkennung via Dateiinhalt...",
                    path, ext_err
                );
                let file = std::fs::File::open(path)
                    .with_context(|| format!("Datei nicht gefunden: {}", path))?;
                let reader = ImageReader::new(BufReader::new(file))
                    .with_guessed_format()
                    .with_context(|| format!("Format-Erkennung fehlgeschlagen fuer: {}", path))?;
                if let Some(fmt) = reader.format() {
                    log::info!(
                        "Tatsaechliches Bildformat erkannt: {:?} fuer '{}'",
                        fmt,
                        path
                    );
                }
                reader.decode().with_context(|| {
                    format!("Fehler beim Dekodieren der Background-Map: {}", path)
                })?
            }
        };

        Self::from_image(image, path, crop_size)
    }

    /// Fuehrt Center-Crop auf ein Bild durch
    ///
    /// Schneidet das Bild auf die angegebene Zielgroesse zu, zentriert.
    /// Falls das Bild kleiner als die Zielgroesse ist, wird es ohne Crop zurueckgegeben.
    fn center_crop(image: DynamicImage, target_size: u32) -> Result<DynamicImage> {
        let (width, height) = image.dimensions();

        // Keine Crop noetig, wenn Bild kleiner als Ziel
        if width <= target_size && height <= target_size {
            log::warn!(
                "Bild ({}x{}) ist kleiner als Crop-Groesse ({}x{}), kein Crop durchgefuehrt",
                width,
                height,
                target_size,
                target_size
            );
            return Ok(image);
        }

        // Berechne Center-Crop-Koordinaten
        let crop_width = target_size.min(width);
        let crop_height = target_size.min(height);
        let x = (width.saturating_sub(crop_width)) / 2;
        let y = (height.saturating_sub(crop_height)) / 2;

        log::debug!(
            "Center-Crop: Original {}x{} -> Crop {}x{} bei ({}, {})",
            width,
            height,
            crop_width,
            crop_height,
            x,
            y
        );

        Ok(image.crop_imm(x, y, crop_width, crop_height))
    }

    /// Gibt die Bilddaten zurueck
    pub fn image_data(&self) -> &DynamicImage {
        self.image_data.as_ref()
    }

    /// Gibt die Bilddaten als `Arc` zurueck.
    pub fn image_arc(&self) -> Arc<DynamicImage> {
        Arc::clone(&self.image_data)
    }

    /// Gibt die Weltkoordinaten-Begrenzungen zurueck
    pub fn world_bounds(&self) -> &WorldBounds {
        &self.world_bounds
    }

    /// Gibt die aktuelle Opacity zurueck
    pub fn opacity(&self) -> f32 {
        self.opacity
    }

    /// Setzt die Opacity
    pub fn set_opacity(&mut self, opacity: f32) {
        self.opacity = opacity.clamp(0.0, 1.0);
    }

    /// Gibt die Dimensionen des Bildes zurueck
    pub fn dimensions(&self) -> (u32, u32) {
        self.image_data.dimensions()
    }

    /// Erstellt eine BackgroundMap aus einem bereits dekodierten Bild.
    ///
    /// Gemeinsame Logik fuer `load_from_file()`, `load_from_zip()` und
    /// `generate_overview_from_zip()`. Fuehrt optionalen Center-Crop durch,
    /// berechnet WorldBounds und loggt Dimensionen.
    pub fn from_image(
        image: DynamicImage,
        source_label: &str,
        crop_size: Option<u32>,
    ) -> Result<Self> {
        let (orig_width, orig_height) = image.dimensions();
        log::info!(
            "Background-Map geladen: {}x{} Pixel von '{}'",
            orig_width,
            orig_height,
            source_label
        );

        // Center-Crop durchfuehren, falls gewuenscht
        let image = if let Some(target_size) = crop_size {
            if orig_width != target_size || orig_height != target_size {
                let cropped = Self::center_crop(image, target_size)?;
                log::info!(
                    "Center-Crop auf {}x{} durchgefuehrt",
                    target_size,
                    target_size
                );
                cropped
            } else {
                image
            }
        } else {
            image
        };

        let (final_width, final_height) = image.dimensions();
        let map_size = final_width.min(final_height) as f32;
        let world_bounds = WorldBounds::from_map_size(map_size);

        log::info!(
            "Background-Map Weltkoordinaten: ({:.1}, {:.1}) bis ({:.1}, {:.1})",
            world_bounds.min_x,
            world_bounds.min_z,
            world_bounds.max_x,
            world_bounds.max_z
        );

        Ok(Self {
            image_data: Arc::new(image),
            world_bounds,
            opacity: 1.0,
        })
    }
}

/// Listet alle Bilddateien in einem ZIP-Archiv auf.
///
/// Gibt Eintraege mit Name und unkomprimierter Groesse zurueck,
/// standardmaessig absteigend nach Groesse sortiert.
pub fn list_images_in_zip(zip_path: &str) -> Result<Vec<ZipImageEntry>> {
    let file = std::fs::File::open(zip_path)
        .with_context(|| format!("ZIP-Datei nicht gefunden: {}", zip_path))?;
    let mut archive = zip::ZipArchive::new(BufReader::new(file))
        .with_context(|| format!("Ungueltiges ZIP-Archiv: {}", zip_path))?;

    let mut image_entries = Vec::new();
    for i in 0..archive.len() {
        let entry = archive.by_index(i)?;
        let name = entry.name().to_string();
        if entry.is_file() && is_image_filename(&name) {
            image_entries.push(ZipImageEntry {
                name,
                size: entry.size(),
            });
        }
    }

    // Groessste Dateien zuerst (typisch: overview.dds ist die groesste)
    image_entries.sort_by(|a, b| b.size.cmp(&a.size));
    log::info!(
        "ZIP '{}': {} Bilddateien gefunden",
        zip_path,
        image_entries.len()
    );
    Ok(image_entries)
}

/// Laedt eine Bilddatei aus einem ZIP-Archiv als BackgroundMap.
///
/// Die Datei wird komplett in-memory extrahiert und dann dekodiert.
pub fn load_from_zip(
    zip_path: &str,
    entry_name: &str,
    crop_size: Option<u32>,
) -> Result<BackgroundMap> {
    let file = std::fs::File::open(zip_path)
        .with_context(|| format!("ZIP-Datei nicht gefunden: {}", zip_path))?;
    let mut archive = zip::ZipArchive::new(BufReader::new(file))
        .with_context(|| format!("Ungueltiges ZIP-Archiv: {}", zip_path))?;

    let mut zip_entry = archive
        .by_name(entry_name)
        .with_context(|| format!("Eintrag '{}' nicht im ZIP gefunden", entry_name))?;

    // Komplett in Speicher lesen (noetig fuer Seek-Support bei DDS)
    let mut buffer = Vec::with_capacity(zip_entry.size() as usize);
    zip_entry
        .read_to_end(&mut buffer)
        .with_context(|| format!("Fehler beim Entpacken von '{}'", entry_name))?;

    log::info!(
        "ZIP-Eintrag '{}' entpackt: {:.1} MB",
        entry_name,
        buffer.len() as f64 / (1024.0 * 1024.0)
    );

    // Bild dekodieren (mit Format-Erkennung via Magic Bytes)
    let reader = ImageReader::new(Cursor::new(buffer))
        .with_guessed_format()
        .with_context(|| format!("Format-Erkennung fehlgeschlagen fuer: {}", entry_name))?;
    let image = reader
        .decode()
        .with_context(|| format!("Fehler beim Dekodieren von '{}' aus ZIP", entry_name))?;

    let source_label = format!("{}:{}", zip_path, entry_name);
    BackgroundMap::from_image(image, &source_label, crop_size)
}

/// Prueft ob ein Dateiname eine bekannte Bild-Endung hat.
fn is_image_filename(name: &str) -> bool {
    let lower = name.to_lowercase();
    IMAGE_EXTENSIONS
        .iter()
        .any(|ext| lower.ends_with(&format!(".{}", ext)))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_center_crop_coordinates() {
        // Simuliere 4096x4096 -> 2048x2048 Crop
        let width = 4096;
        let height = 4096;
        let target_size = 2048;

        let x = (width - target_size) / 2;
        let y = (height - target_size) / 2;

        assert_eq!(x, 1024);
        assert_eq!(y, 1024);
    }

    #[test]
    fn test_center_crop_non_square() {
        // Simuliere 8192x4096 -> 2048x2048 Crop
        let width = 8192;
        let height = 4096;
        let target_size = 2048;

        let crop_width = target_size.min(width);
        let crop_height = target_size.min(height);

        let x = (width - crop_width) / 2;
        let y = (height - crop_height) / 2;

        assert_eq!(crop_width, 2048);
        assert_eq!(crop_height, 2048);
        assert_eq!(x, 3072); // (8192 - 2048) / 2
        assert_eq!(y, 1024); // (4096 - 2048) / 2
    }

    #[test]
    fn test_opacity_clamping() {
        let mut map = BackgroundMap {
            image_data: Arc::new(DynamicImage::new_rgb8(1, 1)),
            world_bounds: WorldBounds::from_map_size(1.0),
            opacity: 1.0,
        };

        // Test Clamping auf [0.0, 1.0]
        map.set_opacity(-0.5);
        assert_eq!(map.opacity(), 0.0);

        map.set_opacity(1.5);
        assert_eq!(map.opacity(), 1.0);

        map.set_opacity(0.5);
        assert_eq!(map.opacity(), 0.5);
    }
}
