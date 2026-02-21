//! Background-Map-Loader für Map-Hintergrund-Rendering.

use anyhow::{Context, Result};
use image::{DynamicImage, GenericImageView, ImageReader};
use std::io::BufReader;

use super::WorldBounds;

/// Background-Map für Map-Hintergrund-Rendering
pub struct BackgroundMap {
    /// Bilddaten (nach Center-Crop)
    image_data: DynamicImage,
    /// Weltkoordinaten-Bereich
    world_bounds: WorldBounds,
    /// Opacity (0.0 = transparent, 1.0 = opak)
    opacity: f32,
}

impl BackgroundMap {
    /// Lädt eine Background-Map aus einer Datei
    ///
    /// Unterstützte Formate: PNG, JPG, JPEG, DDS
    ///
    /// # Parameter
    /// - `path`: Pfad zur Bilddatei
    /// - `crop_size`: Optionale Crop-Größe (quadratisch). Falls angegeben, wird ein Center-Crop durchgeführt.
    pub fn load_from_file(path: &str, crop_size: Option<u32>) -> Result<Self> {
        // Zuerst versuchen wir die Erkennung anhand der Dateiendung.
        // Falls das fehlschlägt (z.B. .dds-Datei die eigentlich PNG ist),
        // erkennen wir das Format anhand der Magic Bytes im Dateiinhalt.
        let mut image = match image::open(path) {
            Ok(img) => img,
            Err(ext_err) => {
                log::warn!(
                    "Format-Erkennung via Dateiendung fehlgeschlagen für '{}': {}. Versuche Erkennung via Dateiinhalt...",
                    path, ext_err
                );
                let file = std::fs::File::open(path)
                    .with_context(|| format!("Datei nicht gefunden: {}", path))?;
                let reader = ImageReader::new(BufReader::new(file))
                    .with_guessed_format()
                    .with_context(|| format!("Format-Erkennung fehlgeschlagen für: {}", path))?;
                if let Some(fmt) = reader.format() {
                    log::info!("Tatsächliches Bildformat erkannt: {:?} für '{}'", fmt, path);
                }
                reader.decode().with_context(|| {
                    format!("Fehler beim Dekodieren der Background-Map: {}", path)
                })?
            }
        };

        let (orig_width, orig_height) = image.dimensions();
        log::info!(
            "Background-Map geladen: {}x{} Pixel von '{}'",
            orig_width,
            orig_height,
            path
        );

        // Center-Crop durchführen, falls gewünscht
        if let Some(target_size) = crop_size {
            if orig_width != target_size || orig_height != target_size {
                image = Self::center_crop(image, target_size)?;
                log::info!(
                    "Center-Crop auf {}x{} durchgeführt",
                    target_size,
                    target_size
                );
            }
        }

        let (final_width, final_height) = image.dimensions();

        // Berechne Weltkoordinaten-Bereich (zentriert, nimmt kleinere Dimension)
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
            image_data: image,
            world_bounds,
            opacity: 1.0, // Default: voll opak
        })
    }

    /// Führt Center-Crop auf ein Bild durch
    ///
    /// Schneidet das Bild auf die angegebene Zielgröße zu, zentriert.
    /// Falls das Bild kleiner als die Zielgröße ist, wird es ohne Crop zurückgegeben.
    fn center_crop(image: DynamicImage, target_size: u32) -> Result<DynamicImage> {
        let (width, height) = image.dimensions();

        // Keine Crop nötig, wenn Bild kleiner als Ziel
        if width <= target_size && height <= target_size {
            log::warn!(
                "Bild ({}x{}) ist kleiner als Crop-Größe ({}x{}), kein Crop durchgeführt",
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

    /// Gibt die Bilddaten zurück
    pub fn image_data(&self) -> &DynamicImage {
        &self.image_data
    }

    /// Gibt die Weltkoordinaten-Begrenzungen zurück
    pub fn world_bounds(&self) -> &WorldBounds {
        &self.world_bounds
    }

    /// Gibt die aktuelle Opacity zurück
    pub fn opacity(&self) -> f32 {
        self.opacity
    }

    /// Setzt die Opacity
    pub fn set_opacity(&mut self, opacity: f32) {
        self.opacity = opacity.clamp(0.0, 1.0);
    }

    /// Gibt die Dimensionen des Bildes zurück
    pub fn dimensions(&self) -> (u32, u32) {
        self.image_data.dimensions()
    }
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
            image_data: DynamicImage::new_rgb8(1, 1),
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
