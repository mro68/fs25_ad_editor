//! Heightmap-Loader und Y-Koordinaten-Sampling.
//!
//! Erkennt automatisch die Bit-Tiefe (8-Bit oder 16-Bit) und normalisiert
//! die Pixelwerte entsprechend. Die Map-Größe wird aus den Pixel-Dimensionen
//! abgeleitet (FS25-Konvention: pixels = map_size + 1).

use anyhow::{Context, Result};
use image::{DynamicImage, GenericImageView};

/// Heightmap für Y-Koordinaten-Berechnung
pub struct Heightmap {
    /// Normalisierte Grauwerte [0.0, 1.0], zeilenweise gespeichert
    pixels: Vec<f32>,
    width: u32,
    height: u32,
    /// Weltkoordinaten-Bereich (min_x, min_z, max_x, max_z)
    world_bounds: WorldBounds,
    /// Erkannte Bit-Tiefe (8 oder 16)
    bit_depth: u8,
}

/// Weltkoordinaten-Begrenzungen der Heightmap
#[derive(Debug, Clone, Copy)]
pub struct WorldBounds {
    /// Minimale X-Koordinate (links)
    pub min_x: f32,
    /// Minimale Z-Koordinate (unten)
    pub min_z: f32,
    /// Maximale X-Koordinate (rechts)
    pub max_x: f32,
    /// Maximale Z-Koordinate (oben)
    pub max_z: f32,
}

impl WorldBounds {
    /// Erstellt Bounds aus Map-Größe (zentriert bei 0,0)
    pub fn from_map_size(size: f32) -> Self {
        let half = size / 2.0;
        Self {
            min_x: -half,
            min_z: -half,
            max_x: half,
            max_z: half,
        }
    }


}

impl Heightmap {
    /// Lädt eine Heightmap und erkennt Bit-Tiefe und Map-Größe automatisch.
    ///
    /// Die Map-Größe wird aus den Pixel-Dimensionen abgeleitet:
    /// FS25-Konvention: `map_size = max(width, height) - 1`
    /// (z.B. 4097×4097 Pixel → 4096m Map-Größe)
    pub fn load(path: &str) -> Result<Self> {
        let image = image::open(path)
            .with_context(|| format!("Fehler beim Laden der Heightmap: {}", path))?;

        let (width, height): (u32, u32) = image.dimensions();
        let map_size = (width.max(height) - 1) as f32;
        let world_bounds = WorldBounds::from_map_size(map_size);

        Self::from_image(image, world_bounds)
    }

    /// Lädt eine Heightmap mit expliziten World-Bounds.
    pub fn load_with_bounds(path: &str, world_bounds: WorldBounds) -> Result<Self> {
        let image = image::open(path)
            .with_context(|| format!("Fehler beim Laden der Heightmap: {}", path))?;

        Self::from_image(image, world_bounds)
    }

    /// Erstellt eine Heightmap aus einem geladenen Bild.
    /// Erkennt die Bit-Tiefe automatisch und konvertiert alle Pixel
    /// in normalisierte f32-Werte [0.0, 1.0].
    fn from_image(image: DynamicImage, world_bounds: WorldBounds) -> Result<Self> {
        let (width, height) = image.dimensions();

        // Bit-Tiefe aus dem Farbtyp erkennen
        let bit_depth = match image.color() {
            image::ColorType::L16
            | image::ColorType::La16
            | image::ColorType::Rgb16
            | image::ColorType::Rgba16 => 16u8,
            _ => 8u8,
        };

        // Pixel in normalisierte f32-Werte konvertieren
        let pixels: Vec<f32> = if bit_depth == 16 {
            let luma16 = image.into_luma16();
            luma16.pixels().map(|p| p[0] as f32 / 65535.0).collect()
        } else {
            let luma8 = image.into_luma8();
            luma8.pixels().map(|p| p[0] as f32 / 255.0).collect()
        };

        log::info!(
            "Heightmap geladen: {}x{} Pixel, {}-Bit, Map-Bereich: ({:.1}, {:.1}) bis ({:.1}, {:.1})",
            width,
            height,
            bit_depth,
            world_bounds.min_x,
            world_bounds.min_z,
            world_bounds.max_x,
            world_bounds.max_z
        );

        Ok(Self {
            pixels,
            width,
            height,
            world_bounds,
            bit_depth,
        })
    }

    /// Berechnet Y-Koordinate (Höhe) für eine gegebene X/Z-Position.
    ///
    /// Verwendet bikubische Interpolation (4×4 Nachbarpixel) für präzise, glatte Höhenwerte.
    /// Die Formel ist: `Y_meter = normalized_pixel × height_scale`
    ///
    /// Für Standard-FS25-Maps gilt `height_scale = 255.0` (maximale Terrainhöhe).
    /// Bei 16-Bit-Heightmaps ergibt das eine Auflösung von ~0.004m pro Stufe.
    pub fn sample_height(&self, x: f32, z: f32, height_scale: f32) -> f32 {
        // Normalisiere Weltkoordinaten auf [0, 1]
        let nx =
            (x - self.world_bounds.min_x) / (self.world_bounds.max_x - self.world_bounds.min_x);
        let nz =
            (z - self.world_bounds.min_z) / (self.world_bounds.max_z - self.world_bounds.min_z);

        // Clampe auf gültigen Bereich
        let nx = nx.clamp(0.0, 1.0);
        let nz = nz.clamp(0.0, 1.0);

        // Konvertiere zu Pixel-Koordinaten
        let px = nx * (self.width - 1) as f32;
        let pz = nz * (self.height - 1) as f32;

        // Debug-Logging für Diagnose
        log::trace!(
            "Sample at world ({:.3}, {:.3}) -> normalized ({:.6}, {:.6}) -> pixel ({:.3}, {:.3})",
            x,
            z,
            nx,
            nz,
            px,
            pz
        );

        // Bikubische Interpolation (nutzt 16 Nachbarpixel für beste Qualität)
        let height = self.sample_bicubic(px, pz);

        // Debug: Zeige Interpolationsergebnis
        log::trace!(
            "  Interpolated grayscale: {:.6} -> height: {:.3}m",
            height,
            height * height_scale
        );

        // Skaliere auf Höhenwert
        height * height_scale
    }

    /// Bikubische Interpolation für glatte Höhenwerte
    /// Nutzt 4x4 Grid von Pixeln um den Sample-Punkt
    fn sample_bicubic(&self, px: f32, pz: f32) -> f32 {
        let x = px.floor() as i32;
        let z = pz.floor() as i32;

        let fx = px - px.floor();
        let fz = pz - pz.floor();

        // Debug: Zeige Interpolationsfaktoren
        log::trace!("  Bicubic: x={}, z={}, fx={:.6}, fz={:.6}", x, z, fx, fz);

        // Sample 4x4 Grid von Pixeln
        let mut values = [[0.0f32; 4]; 4];
        for (j, row) in values.iter_mut().enumerate() {
            for (i, cell) in row.iter_mut().enumerate() {
                let sample_x = (x + i as i32 - 1).clamp(0, self.width as i32 - 1) as u32;
                let sample_z = (z + j as i32 - 1).clamp(0, self.height as i32 - 1) as u32;
                *cell = self.get_grayscale(sample_x, sample_z);
            }
        }

        // Debug: Zeige das zentrale 2x2 Grid
        log::trace!(
            "  Center 2x2 grid: [{:.3}, {:.3} | {:.3}, {:.3}]",
            values[1][1],
            values[1][2],
            values[2][1],
            values[2][2]
        );

        // Bikubische Interpolation in X-Richtung
        let mut col_values = [0.0f32; 4];
        for j in 0..4 {
            col_values[j] =
                Self::cubic_interpolate(values[j][0], values[j][1], values[j][2], values[j][3], fx);
        }

        // Bikubische Interpolation in Z-Richtung
        let result = Self::cubic_interpolate(
            col_values[0],
            col_values[1],
            col_values[2],
            col_values[3],
            fz,
        );

        log::trace!("  Bicubic result: {:.6}", result);

        result
    }

    /// Kubische Interpolation zwischen 4 Werten
    /// Nutzt Catmull-Rom Spline für glatte Kurven
    fn cubic_interpolate(p0: f32, p1: f32, p2: f32, p3: f32, t: f32) -> f32 {
        let t2 = t * t;
        let t3 = t2 * t;

        // Catmull-Rom Spline Koeffizienten
        let a = -0.5 * p0 + 1.5 * p1 - 1.5 * p2 + 0.5 * p3;
        let b = p0 - 2.5 * p1 + 2.0 * p2 - 0.5 * p3;
        let c = -0.5 * p0 + 0.5 * p2;
        let d = p1;

        a * t3 + b * t2 + c * t + d
    }

    /// Holt normalisierten Grauwert eines Pixels (0.0 = schwarz, 1.0 = weiß).
    /// Liest aus dem vorberechneten f32-Array (bereits korrekt normalisiert für 8/16-Bit).
    fn get_grayscale(&self, x: u32, y: u32) -> f32 {
        self.pixels[(y * self.width + x) as usize]
    }

    /// Gibt die Dimensionen der Heightmap zurück
    pub fn dimensions(&self) -> (u32, u32) {
        (self.width, self.height)
    }

    /// Gibt die erkannte Bit-Tiefe zurück (8 oder 16)
    pub fn bit_depth(&self) -> u8 {
        self.bit_depth
    }

    /// Gibt die verwendeten World-Bounds zurück
    pub fn world_bounds(&self) -> &WorldBounds {
        &self.world_bounds
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_world_bounds_from_size() {
        let bounds = WorldBounds::from_map_size(4096.0);
        assert_eq!(bounds.min_x, -2048.0);
        assert_eq!(bounds.min_z, -2048.0);
        assert_eq!(bounds.max_x, 2048.0);
        assert_eq!(bounds.max_z, 2048.0);

        let small = WorldBounds::from_map_size(1024.0);
        assert_eq!(small.min_x, -512.0);
        assert_eq!(small.max_x, 512.0);
    }

    #[test]
    fn test_cubic_interpolation() {
        // Test Catmull-Rom Interpolation
        // Bei t=0 sollte p1 zurückgegeben werden
        let result = Heightmap::cubic_interpolate(0.0, 0.5, 1.0, 1.5, 0.0);
        assert!((result - 0.5).abs() < 0.001);

        // Bei t=1 sollte p2 zurückgegeben werden
        let result = Heightmap::cubic_interpolate(0.0, 0.5, 1.0, 1.5, 1.0);
        assert!((result - 1.0).abs() < 0.001);

        // Bei t=0.5 sollte ein Wert zwischen p1 und p2 zurückgegeben werden
        let result = Heightmap::cubic_interpolate(0.0, 0.5, 1.0, 1.5, 0.5);
        assert!(result > 0.5 && result < 1.0);
    }

    #[test]
    fn test_cubic_interpolation_non_integer() {
        // Test mit Werten wie sie von Heightmap-Pixeln kommen würden
        // Simuliere Höhen 35m, 36m, 37m, 38m als normalisierte Werte
        let p0 = 35.0 / 255.0; // 0.137254...
        let p1 = 36.0 / 255.0; // 0.141176...
        let p2 = 37.0 / 255.0; // 0.145098...
        let p3 = 38.0 / 255.0; // 0.149019...

        // Bei t=0.5 zwischen p1 und p2 sollte ein Wert ~36.5/255 rauskommen
        let result = Heightmap::cubic_interpolate(p0, p1, p2, p3, 0.5);
        let height_meters = result * 255.0;

        // Sollte zwischen 36.0 und 37.0 liegen, aber nicht exakt 36.5
        // (wegen der Catmull-Rom Spline-Kurve)
        assert!(height_meters > 36.0 && height_meters < 37.0);

        // Sollte definitiv Nachkommastellen haben
        assert!((height_meters - height_meters.round()).abs() > 0.001);
    }
}
