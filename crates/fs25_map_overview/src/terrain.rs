//! Terrain-Compositing aus Weight-Maps.
//!
//! Mischt alle Weight-Maps eines FS25-Map-Mods farblich
//! zu einem Terrain-Bild zusammen.

use anyhow::Result;
use image::{DynamicImage, GrayImage, RgbImage};

use crate::palette;

/// Hintergrundfarbe für Pixel ohne Weight-Map-Abdeckung.
const BACKGROUND_COLOR: [f64; 3] = [80.0, 100.0, 60.0];

/// Ein einzelnes Weight-Map-Layer mit zugehöriger Farbe.
pub struct WeightLayer {
    /// Name der Weight-Map (für Farbzuordnung)
    pub name: String,
    /// Grayscale-Gewichtsbild (0–255)
    pub weights: GrayImage,
}

/// Mischt Weight-Map-Layer zu einem RGB-Terrain-Bild.
///
/// Jedes Pixel wird als gewichteter Durchschnitt der Layer-Farben berechnet.
/// Pixel ohne Abdeckung erhalten die Hintergrundfarbe.
///
/// # Parameter
/// - `layers`: Weight-Map-Layer mit Name und Gewichtsbild
/// - `target_size`: Zielgröße (quadratisch), alle Layer werden darauf skaliert
pub fn composite_terrain(layers: &[WeightLayer], target_size: u32) -> Result<RgbImage> {
    let size = target_size as usize;
    let mut result_r = vec![0.0f64; size * size];
    let mut result_g = vec![0.0f64; size * size];
    let mut result_b = vec![0.0f64; size * size];
    let mut total_weight = vec![0.0f64; size * size];

    log::info!("{} Weight-Maps werden gemischt...", layers.len());

    for layer in layers {
        let color = palette::terrain_color(&layer.name);
        let color_r = color[0] as f64;
        let color_g = color[1] as f64;
        let color_b = color[2] as f64;

        // Weight-Map auf Zielgröße skalieren falls nötig
        let weights =
            if layer.weights.width() != target_size || layer.weights.height() != target_size {
                image::imageops::resize(
                    &layer.weights,
                    target_size,
                    target_size,
                    image::imageops::FilterType::Lanczos3,
                )
            } else {
                layer.weights.clone()
            };

        for (i, &w_byte) in weights.as_raw().iter().enumerate() {
            let w = w_byte as f64 / 255.0;
            result_r[i] += w * color_r;
            result_g[i] += w * color_g;
            result_b[i] += w * color_b;
            total_weight[i] += w;
        }
    }

    // Normalisieren + Hintergrundfarbe für unbedeckte Pixel
    let mut rgb_data = vec![0u8; size * size * 3];
    for i in 0..(size * size) {
        let (r, g, b) = if total_weight[i] > 0.0 {
            (
                result_r[i] / total_weight[i],
                result_g[i] / total_weight[i],
                result_b[i] / total_weight[i],
            )
        } else {
            (
                BACKGROUND_COLOR[0],
                BACKGROUND_COLOR[1],
                BACKGROUND_COLOR[2],
            )
        };
        rgb_data[i * 3] = r.clamp(0.0, 255.0) as u8;
        rgb_data[i * 3 + 1] = g.clamp(0.0, 255.0) as u8;
        rgb_data[i * 3 + 2] = b.clamp(0.0, 255.0) as u8;
    }

    RgbImage::from_raw(target_size, target_size, rgb_data)
        .ok_or_else(|| anyhow::anyhow!("Fehler beim Erstellen des Terrain-Bildes"))
}

/// Erstellt ein Terrain-Bild aus einer Liste von Weight-Map-Bildern.
///
/// Convenience-Funktion: akzeptiert (name, DynamicImage)-Paare.
pub fn composite_terrain_from_images(
    images: &[(String, DynamicImage)],
    target_size: u32,
) -> Result<RgbImage> {
    let layers: Vec<WeightLayer> = images
        .iter()
        .map(|(name, img)| WeightLayer {
            name: name.clone(),
            weights: img.to_luma8(),
        })
        .collect();

    composite_terrain(&layers, target_size)
}
