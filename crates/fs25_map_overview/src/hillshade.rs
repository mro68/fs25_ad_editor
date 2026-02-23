//! Hillshade-Berechnung aus DEM-Daten.
//!
//! Erzeugt eine 3D-Reliefschattierung basierend auf dem
//! Digital Elevation Model (DEM) der Karte.

use anyhow::Result;
use image::GrayImage;

/// Parameter für die Hillshade-Berechnung.
pub struct HillshadeParams {
    /// Azimut der Lichtquelle in Grad (Standard: 315° = Nordwest)
    pub azimuth_deg: f32,
    /// Höhenwinkel der Lichtquelle in Grad (Standard: 45°)
    pub altitude_deg: f32,
    /// Mischfaktor: 0.0 = kein Hillshade, 1.0 = voller Effekt
    pub blend_factor: f32,
}

impl Default for HillshadeParams {
    fn default() -> Self {
        Self {
            azimuth_deg: 315.0,
            altitude_deg: 45.0,
            blend_factor: 0.45,
        }
    }
}

/// Berechnet eine Hillshade-Map aus einem DEM-Grayscale-Bild.
///
/// Gibt ein f32-Array zurück mit Werten zwischen 0.0 und 1.0
/// (0.0 = voller Schatten, 1.0 = voll beleuchtet).
///
/// # Parameter
/// - `dem`: Grayscale-DEM-Bild
/// - `params`: Beleuchtungsparameter
pub fn compute_hillshade(dem: &GrayImage, params: &HillshadeParams) -> Result<Vec<f32>> {
    let width = dem.width() as usize;
    let height = dem.height() as usize;

    let azimuth = params.azimuth_deg.to_radians();
    let altitude = params.altitude_deg.to_radians();

    let sin_alt = altitude.sin();
    let cos_alt = altitude.cos();

    let pixels = dem.as_raw();
    let mut hillshade = vec![0.5f32; width * height];

    // Gradient (Sobel-artiger Ansatz mit numpy-ähnlichem gradient())
    for y in 0..height {
        for x in 0..width {
            // dz/dx
            let dx = if x == 0 {
                pixels[y * width + 1] as f32 - pixels[y * width] as f32
            } else if x == width - 1 {
                pixels[y * width + x] as f32 - pixels[y * width + x - 1] as f32
            } else {
                (pixels[y * width + x + 1] as f32 - pixels[y * width + x - 1] as f32) / 2.0
            };

            // dz/dy
            let dy = if y == 0 {
                pixels[(y + 1) * width + x] as f32 - pixels[y * width + x] as f32
            } else if y == height - 1 {
                pixels[y * width + x] as f32 - pixels[(y - 1) * width + x] as f32
            } else {
                (pixels[(y + 1) * width + x] as f32 - pixels[(y - 1) * width + x] as f32) / 2.0
            };

            let slope = (dx * dx + dy * dy).sqrt();
            let aspect = (-dy).atan2(dx);

            let hs = sin_alt * slope.atan().cos()
                + cos_alt * slope.atan().sin() * (azimuth - aspect).cos();

            hillshade[y * width + x] = hs.clamp(0.0, 1.0);
        }
    }

    Ok(hillshade)
}

/// Wendet Hillshade auf ein RGB-Bild an (in-place).
///
/// Moduliert die Helligkeit jedes Pixels basierend auf dem Hillshade-Wert.
///
/// # Parameter
/// - `rgb_data`: RGB-Pixeldaten (3 Bytes pro Pixel)
/// - `hillshade`: Hillshade-Werte (0.0–1.0), gleiche Dimension
/// - `blend`: Mischfaktor (0.0 = kein Effekt, 1.0 = voller Effekt)
pub fn apply_hillshade(rgb_data: &mut [u8], hillshade: &[f32], blend: f32) {
    let base = 1.0 - blend;
    for (i, &hs) in hillshade.iter().enumerate() {
        let offset = i * 3;
        if offset + 2 < rgb_data.len() {
            let factor = base + blend * hs;
            rgb_data[offset] = (rgb_data[offset] as f32 * factor).clamp(0.0, 255.0) as u8;
            rgb_data[offset + 1] = (rgb_data[offset + 1] as f32 * factor).clamp(0.0, 255.0) as u8;
            rgb_data[offset + 2] = (rgb_data[offset + 2] as f32 * factor).clamp(0.0, 255.0) as u8;
        }
    }
}
