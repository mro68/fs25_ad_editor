//! Farb-Sampling und Masken-Erstellung fuer das ColorPathTool.
//!
//! Enthaelt die Pixel<->Welt-Umrechnung, das Lasso-basierte Farb-Sampling,
//! die Masken-Erstellung per Farb-Toleranz sowie morphologische Operationen.

use crate::core::point_in_polygon;
use glam::Vec2;
use image::{DynamicImage, GenericImageView};

// ---------------------------------------------------------------------------
// Koordinaten-Umrechnung
// ---------------------------------------------------------------------------

/// Konvertiert Weltkoordinaten in Pixel-Koordinaten des Hintergrundbildes.
///
/// Das Bild ist quadratisch und zentriert bei (0,0); `map_size` gibt
/// die Seitenlaenge in Metern an. `img_width` steuert den Massstab.
pub(crate) fn world_to_pixel(
    world: Vec2,
    map_size: f32,
    img_width: u32,
    img_height: u32,
) -> (u32, u32) {
    let scale = img_width as f32 / map_size;
    let px = ((world.x + map_size / 2.0) * scale).clamp(0.0, (img_width - 1) as f32) as u32;
    let py = ((world.y + map_size / 2.0) * scale).clamp(0.0, (img_height - 1) as f32) as u32;
    (px, py)
}

/// Konvertiert Pixel-Koordinaten in Weltkoordinaten.
///
/// Umkehrung von `world_to_pixel`. Fuer nicht-quadratische Bilder werden
/// X und Y je mit dem korrekten Skalierungsfaktor umgerechnet.
pub(crate) fn pixel_to_world(
    px: u32,
    py: u32,
    map_size: f32,
    img_width: u32,
    img_height: u32,
) -> Vec2 {
    let scale_x = map_size / img_width as f32;
    let scale_y = map_size / img_height as f32;
    Vec2::new(
        px as f32 * scale_x - map_size / 2.0,
        py as f32 * scale_y - map_size / 2.0,
    )
}

/// Konvertiert Sub-Pixel-Koordinaten (Fließkomma) in Weltkoordinaten.
///
/// Wird fuer die Medial-Axis-Korrektur verwendet, wo Pixel-Positionen
/// nicht ganzzahlig sind.
pub(crate) fn pixel_to_world_f32(
    px: f32,
    py: f32,
    map_size: f32,
    img_width: u32,
    img_height: u32,
) -> Vec2 {
    let scale_x = map_size / img_width as f32;
    let scale_y = map_size / img_height as f32;
    Vec2::new(px * scale_x - map_size / 2.0, py * scale_y - map_size / 2.0)
}

// ---------------------------------------------------------------------------
// Farb-Sampling
// ---------------------------------------------------------------------------

/// Sammelt alle Pixelfarben innerhalb eines Lasso-Polygons.
///
/// Berechnet die Bounding-Box des Polygons, prueft fuer jeden Pixel
/// per Ray-Casting ob er im Polygon liegt und liest die RGB-Werte
/// aus dem Hintergrundbild.
pub(crate) fn sample_colors_in_polygon(
    polygon: &[Vec2],
    image: &DynamicImage,
    map_size: f32,
) -> Vec<[u8; 3]> {
    if polygon.is_empty() {
        return Vec::new();
    }

    let img_w = image.width();
    let img_h = image.height();

    // Bounding-Box des Polygons in Weltkoords
    let mut min_x = f32::MAX;
    let mut max_x = f32::MIN;
    let mut min_y = f32::MAX;
    let mut max_y = f32::MIN;
    for p in polygon {
        min_x = min_x.min(p.x);
        max_x = max_x.max(p.x);
        min_y = min_y.min(p.y);
        max_y = max_y.max(p.y);
    }

    // BBox in Pixel-Koordinaten umrechnen
    let (px_min, py_min) = world_to_pixel(Vec2::new(min_x, min_y), map_size, img_w, img_h);
    let (px_max, py_max) = world_to_pixel(Vec2::new(max_x, max_y), map_size, img_w, img_h);

    let mut colors = Vec::new();
    for py in py_min..=py_max {
        for px in px_min..=px_max {
            // Mittelpunkt des Pixels in Weltkoords
            let world = pixel_to_world(px, py, map_size, img_w, img_h);
            if point_in_polygon(world, polygon) {
                let pixel = image.get_pixel(px, py);
                colors.push([pixel[0], pixel[1], pixel[2]]);
            }
        }
    }
    colors
}

// ---------------------------------------------------------------------------
// Farbmittelwert
// ---------------------------------------------------------------------------

/// Berechnet den RGB-Farbmittelwert aus gesammelten Farben.
///
/// Gibt `[0, 0, 0]` zurueck wenn die Eingabe leer ist.
pub(crate) fn compute_average_color(colors: &[[u8; 3]]) -> [u8; 3] {
    if colors.is_empty() {
        return [0, 0, 0];
    }
    let (sum_r, sum_g, sum_b) = colors.iter().fold((0u64, 0u64, 0u64), |acc, c| {
        (
            acc.0 + c[0] as u64,
            acc.1 + c[1] as u64,
            acc.2 + c[2] as u64,
        )
    });
    let n = colors.len() as u64;
    [(sum_r / n) as u8, (sum_g / n) as u8, (sum_b / n) as u8]
}

// ---------------------------------------------------------------------------
// Farbpalette
// ---------------------------------------------------------------------------

/// Quantisiert Rohfarben auf ein Raster und gibt die eindeutigen Farb-Buckets zurueck.
///
/// `bucket_size` bestimmt die Rasterung (z.B. 8 → Werte 0, 8, 16, …, 248).
/// Typisch 5–30 Eintraege bei realen Kartenausschnitten.
pub(crate) fn build_color_palette(raw_colors: &[[u8; 3]], bucket_size: u8) -> Vec<[u8; 3]> {
    use std::collections::HashSet;
    let mut seen = HashSet::new();
    for &c in raw_colors {
        let q = [
            (c[0] / bucket_size) * bucket_size,
            (c[1] / bucket_size) * bucket_size,
            (c[2] / bucket_size) * bucket_size,
        ];
        seen.insert(q);
    }
    seen.into_iter().collect()
}

// ---------------------------------------------------------------------------
// Bool-Maske
// ---------------------------------------------------------------------------

/// Erstellt eine Binaermaske per Flood-Fill ab einem Startpunkt.
///
/// Expandiert ab `start_pixel` zu allen 4-verbundenen Nachbarn,
/// deren Farbe innerhalb der Toleranz eines Palette-Eintrags liegt.
/// Ergibt immer genau einen zusammenhaengenden Bereich.
pub(crate) fn flood_fill_color_mask(
    image: &DynamicImage,
    palette: &[[u8; 3]],
    tolerance: f32,
    start_pixel: (u32, u32),
) -> (Vec<bool>, u32, u32) {
    let (width, height) = image.dimensions();
    let rgb = image.to_rgb8();
    let mut mask = vec![false; (width * height) as usize];

    let tolerance_sq = (tolerance as i32) * (tolerance as i32);

    // Prueft ob ein Pixel in der Palette liegt
    let pixel_matches = |x: u32, y: u32| -> bool {
        let px = rgb.get_pixel(x, y).0;
        palette.iter().any(|c| {
            let dr = px[0] as i32 - c[0] as i32;
            let dg = px[1] as i32 - c[1] as i32;
            let db = px[2] as i32 - c[2] as i32;
            dr * dr + dg * dg + db * db <= tolerance_sq
        })
    };

    // Startpunkt pruefen
    let (sx, sy) = start_pixel;
    if sx >= width || sy >= height || !pixel_matches(sx, sy) {
        return (mask, width, height);
    }

    // BFS Flood-Fill
    use std::collections::VecDeque;
    let mut queue = VecDeque::new();
    let start_idx = (sy * width + sx) as usize;
    mask[start_idx] = true;
    queue.push_back((sx, sy));

    while let Some((x, y)) = queue.pop_front() {
        for (nx, ny) in [
            (x.wrapping_sub(1), y),
            (x + 1, y),
            (x, y.wrapping_sub(1)),
            (x, y + 1),
        ] {
            if nx < width && ny < height {
                let idx = (ny * width + nx) as usize;
                if !mask[idx] && pixel_matches(nx, ny) {
                    mask[idx] = true;
                    queue.push_back((nx, ny));
                }
            }
        }
    }

    (mask, width, height)
}

/// Erstellt eine Bool-Maske aller Pixel deren Farbe mit einem Eintrag der Palette uebereinstimmt.
///
/// Maskengrösse = Bildgrösse (kein Downsampling). Das Ergebnis-Tuple enthaelt
/// `(maske, breite, hoehe)`. Optionale `bounds` (Weltkoords min/max) begrenzen
/// den berechneten Bereich auf eine Rect-Region.
/// Die Toleranzprüfung erfolgt als quadratische Distanz (kein sqrt nötig).
#[allow(dead_code)]
pub(crate) fn build_color_mask(
    image: &DynamicImage,
    palette: &[[u8; 3]],
    tolerance: f32,
    bounds: Option<(Vec2, Vec2)>,
    map_size: f32,
) -> (Vec<bool>, u32, u32) {
    let width = image.width();
    let height = image.height();

    // Pixelbereich bestimmen
    let (px_min, py_min, px_max, py_max) = if let Some((world_min, world_max)) = bounds {
        let (x0, y0) = world_to_pixel(world_min, map_size, width, height);
        let (x1, y1) = world_to_pixel(world_max, map_size, width, height);
        (x0.min(x1), y0.min(y1), x0.max(x1), y0.max(y1))
    } else {
        (0, 0, width - 1, height - 1)
    };

    // Maske der vollen Bildgrösse (für einfache Index-Berechnung)
    let mut mask = vec![false; (width * height) as usize];

    let tolerance_sq = (tolerance as i32) * (tolerance as i32);
    for py in py_min..=py_max {
        for px in px_min..=px_max {
            let pixel = image.get_pixel(px, py);
            let pr = pixel[0] as i32;
            let pg = pixel[1] as i32;
            let pb = pixel[2] as i32;
            let hit = palette.iter().any(|c| {
                let dr = pr - c[0] as i32;
                let dg = pg - c[1] as i32;
                let db = pb - c[2] as i32;
                dr * dr + dg * dg + db * db <= tolerance_sq
            });
            mask[(py * width + px) as usize] = hit;
        }
    }

    (mask, width, height)
}

// ---------------------------------------------------------------------------
// Morphologische Operationen
// ---------------------------------------------------------------------------

/// Erosion: Pixel wird false wenn weniger als 3 von 4 orthogonalen Nachbarn true sind.
///
/// Schwächere Majority-Bedingung (≥ 3 von 4) statt ALL-Bedingung verhindert,
/// dass dünne Verbindungen (1-3 px) durch Erosion komplett gelöscht werden.
/// Pixels am Bildrand gelten als nicht vorhanden (false).
pub(crate) fn erode(mask: &[bool], width: usize, height: usize) -> Vec<bool> {
    let mut result = vec![false; width * height];
    for y in 0..height {
        for x in 0..width {
            let idx = y * width + x;
            if !mask[idx] {
                // Bereits false — bleibt false
                continue;
            }
            // Pixel bleibt true nur wenn mindestens 3 von 4 Nachbarn true sind
            let left = x > 0 && mask[idx - 1];
            let right = x + 1 < width && mask[idx + 1];
            let up = y > 0 && mask[idx - width];
            let down = y + 1 < height && mask[idx + width];
            let count = [left, right, up, down].iter().filter(|&&v| v).count();
            result[idx] = count >= 3;
        }
    }
    result
}

/// Dilation: Pixel wird true wenn er selbst oder ein Nachbar (4-Connectivity) true ist.
///
/// Vergrossert Objekte um einen Pixel in alle vier Richtungen.
pub(crate) fn dilate(mask: &[bool], width: usize, height: usize) -> Vec<bool> {
    let mut result = vec![false; width * height];
    for y in 0..height {
        for x in 0..width {
            let idx = y * width + x;
            let center = mask[idx];
            let left = x > 0 && mask[idx - 1];
            let right = x + 1 < width && mask[idx + 1];
            let up = y > 0 && mask[idx - width];
            let down = y + 1 < height && mask[idx + width];
            result[idx] = center || left || right || up || down;
        }
    }
    result
}

/// Opening (Erosion + Dilation) — entfernt kleine Rausch-Inseln.
pub(crate) fn morphological_open(mask: &[bool], width: usize, height: usize) -> Vec<bool> {
    let eroded = erode(mask, width, height);
    dilate(&eroded, width, height)
}

/// Closing (Dilation + Erosion) — schliesst kleine Lücken.
pub(crate) fn morphological_close(mask: &[bool], width: usize, height: usize) -> Vec<bool> {
    let dilated = dilate(mask, width, height);
    erode(&dilated, width, height)
}

// ---------------------------------------------------------------------------
// Unit-Tests
// ---------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;
    use image::{DynamicImage, Rgb, RgbImage};

    // Hilfsfunktion: Erstellt ein 4x4-Bild mit zwei Farben (linke Hälfte rot, rechte grün)
    fn split_image_4x4() -> DynamicImage {
        let img = RgbImage::from_fn(4, 4, |x, _| {
            if x < 2 {
                Rgb([200u8, 0, 0])
            } else {
                Rgb([0u8, 200, 0])
            }
        });
        DynamicImage::ImageRgb8(img)
    }

    #[test]
    fn world_to_pixel_center_is_half_image() {
        // Weltmittelpunkt (0,0) muss auf Bildmitte zeigen
        let (px, py) = world_to_pixel(Vec2::ZERO, 2048.0, 512, 512);
        assert_eq!(px, 256);
        assert_eq!(py, 256);
    }

    #[test]
    fn pixel_to_world_roundtrip() {
        // Roundtrip: Welt→Pixel→Welt muss annaehernd urspruenglichen Wert ergeben
        let map_size = 2048.0;
        let (w, h) = (256u32, 256u32);
        let original = Vec2::new(100.0, -300.0);
        let (px, py) = world_to_pixel(original, map_size, w, h);
        let recovered = pixel_to_world(px, py, map_size, w, h);
        // Durch Quantisierung auf ganzzahlige Pixel: Toleranz 1 Skalierungseinheit
        let tolerance = map_size / w as f32;
        assert!(
            (recovered.x - original.x).abs() <= tolerance,
            "x: {} vs {}",
            recovered.x,
            original.x
        );
        assert!(
            (recovered.y - original.y).abs() <= tolerance,
            "y: {} vs {}",
            recovered.y,
            original.y
        );
    }

    #[test]
    fn compute_average_color_bekannte_werte() {
        // Drei bekannte Farben → erwarteter Mittelwert
        let colors: &[[u8; 3]] = &[[10, 20, 30], [20, 40, 60], [30, 60, 90]];
        let avg = compute_average_color(colors);
        assert_eq!(avg, [20, 40, 60]);
    }

    #[test]
    fn compute_average_color_leer_gibt_null() {
        let avg = compute_average_color(&[]);
        assert_eq!(avg, [0, 0, 0]);
    }

    #[test]
    fn build_color_mask_trifft_passende_pixel() {
        // 4x4-Bild: links rot [200,0,0], rechts gruen [0,200,0]
        let img = split_image_4x4();
        let avg = [200u8, 0, 0]; // Rot als Zielfarbe
        let tolerance = 30.0;
        let map_size = 4.0; // 1 Pixel = 1 Welteinheit

        let (mask, width, height) = build_color_mask(&img, &[avg], tolerance, None, map_size);
        assert_eq!(width, 4);
        assert_eq!(height, 4);

        // Linke Hälfte (x=0,1) muss true sein, rechte (x=2,3) false
        for y in 0..4usize {
            for x in 0..4usize {
                let expected = x < 2;
                assert_eq!(
                    mask[y * 4 + x],
                    expected,
                    "Pixel ({},{}) erwartet {}",
                    x,
                    y,
                    expected
                );
            }
        }
    }

    #[test]
    fn build_color_mask_mit_bounds_begrenzt_bereich() {
        // Nur die rechte Haelfte per bounds abfragen
        let img = split_image_4x4();
        let avg = [0u8, 200, 0]; // Gruen
        let tolerance = 30.0;
        let map_size = 4.0;

        // bounds = rechte Haelfte: x in [0..2) Welteinheiten (nach Mitte verschoben)
        // Bild 4x4, map_size=4: Weltkoords von -2.0 bis +2.0
        // Rechte Haelfte: x von 0.0 bis +2.0
        let bounds = Some((Vec2::new(0.0, -2.0), Vec2::new(2.0, 2.0)));
        let (mask, width, height) = build_color_mask(&img, &[avg], tolerance, bounds, map_size);
        assert_eq!(width, 4);
        assert_eq!(height, 4);

        // Pixel (0,0) und (1,0) sind ausserhalb bounds → false (rot, nicht gruen)
        // Pixel (2,0) und (3,0) sind gruen → true
        for y in 0..4usize {
            // x=2 und x=3 liegen im bounds-Bereich und haben gruene Farbe
            assert!(mask[y * 4 + 2], "Pixel (2,{}) sollte true sein", y);
            assert!(mask[y * 4 + 3], "Pixel (3,{}) sollte true sein", y);
        }
    }

    #[test]
    fn morphological_open_entfernt_einzelnen_pixel() {
        // 5x5-Maske: nur Pixel (2,2) ist true — Opening soll ihn entfernen
        let width = 5usize;
        let height = 5usize;
        let mut mask = vec![false; width * height];
        mask[2 * width + 2] = true; // Mittelpunkt

        let opened = morphological_open(&mask, width, height);
        // Nach Opening: der einzelne Pixel ist verschwunden
        assert!(
            !opened[2 * width + 2],
            "Einzelner Pixel sollte nach Opening entfernt werden"
        );
        // Alle anderen Pixel bleiben false
        assert!(opened.iter().all(|&v| !v));
    }

    #[test]
    fn morphological_close_schliesst_1px_luecke() {
        // 5x3-Maske: obere und untere Zeile vollstaendig gefuellt,
        // Mitte-Zeile (y=1) hat eine 1px-Luecke bei x=2.
        // Randpixel werden durch Erosion entfernt (Zero-Padding, erwartet).
        // Pixel (2,1) liegt innen → muss nach Closing true sein.
        let width = 5usize;
        let height = 3usize;
        let mask: Vec<bool> = vec![
            true, true, true, true, true, // Zeile 0
            true, true, false, true, true, // Zeile 1 — Luecke bei x=2
            true, true, true, true, true, // Zeile 2
        ];

        let closed = morphological_close(&mask, width, height);
        // Luecke bei (x=2, y=1) muss durch Closing geschlossen werden
        assert!(
            closed[width + 2],
            "1px-Luecke (2,1) muss nach Closing geschlossen sein"
        );
    }
}
