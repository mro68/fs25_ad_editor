//! Farmland-Polygon-Extraktion via Moore-Neighbor-Boundary-Tracing.
//!
//! Extrahiert fuer jede Farmland-ID einen geordneten Umriss-Polygon
//! aus rohen GRLE-Daten. Die Pixel-Koordinaten werden im Raster-Raum
//! zurueckgegeben; die Umrechnung in Weltkoordinaten erfolgt in der aufrufenden Schicht.

use std::collections::HashMap;

use anyhow::Result;

use crate::grle;

/// Clockwise-Neighbor-Offsets fuer Moore-Boundary-Tracing (W, NW, N, NE, E, SE, S, SW).
///
/// In Bildkoordinaten (Y nach unten) entspricht diese Reihenfolge dem Uhrzeigersinn.
const CLOCKWISE: [(i32, i32); 8] = [
    (-1, 0),  // 0: W
    (-1, -1), // 1: NW
    (0, -1),  // 2: N
    (1, -1),  // 3: NE
    (1, 0),   // 4: E
    (1, 1),   // 5: SE
    (0, 1),   // 6: S
    (-1, 1),  // 7: SW
];

/// Ein geordnetes Umriss-Polygon fuer ein einzelnes Farmland-Feld.
///
/// Die Vertices sind Pixel-Koordinaten im GRLE-Raster. Fuer Weltkoordinaten
/// muss der Aufrufer `(pixel_x * scale, pixel_y * scale)` berechnen.
pub struct FarmlandPolygon {
    /// Farmland-ID (1–255, 0 = kein Feld)
    pub id: u32,
    /// Geordnete Rand-Pixel als (x, y) in Pixel-Koordinaten
    pub vertices: Vec<(f32, f32)>,
}

/// Extrahiert Farmland-Polygone aus GRLE-Rohdaten per Moore-Neighbor-Boundary-Tracing.
///
/// Gibt alle gefundenen Polygone sowie die Raster-Dimensionen (width, height)
/// zurueck. Die Koordinaten der Polygone liegen im GRLE-Pixel-Raum.
///
/// # Fehler
/// Schlaegt fehl wenn die GRLE-Daten ungueltiges Format haben.
pub fn extract_farmland_polygons(grle_data: &[u8]) -> Result<(Vec<FarmlandPolygon>, u32, u32)> {
    let decoded = grle::decode_grle(grle_data)?;
    let width = decoded.width;
    let height = decoded.height;
    let pixels = &decoded.pixels;

    // Ersten Vorkommen jeder ID in Scan-Reihenfolge (top-left) sammeln
    let mut start_pixels: HashMap<u8, (i32, i32)> = HashMap::new();
    for y in 0..height {
        for x in 0..width {
            let id = pixels[y * width + x];
            if id != 0 {
                start_pixels.entry(id).or_insert((x as i32, y as i32));
            }
        }
    }

    let mut polygons = Vec::with_capacity(start_pixels.len());

    for (&id, &start) in &start_pixels {
        let raw_contour = trace_moore_contour(pixels, width, height, id, start);

        // Aufeinander folgende Duplikate entfernen (entstehen bei sehr duennen Regionen)
        let vertices = dedup_consecutive(raw_contour);

        if vertices.len() >= 2 {
            polygons.push(FarmlandPolygon {
                id: id as u32,
                vertices,
            });
        }
    }

    log::debug!(
        "Farmland-Polygone extrahiert: {} Felder aus {}x{} Raster",
        polygons.len(),
        width,
        height
    );

    Ok((polygons, width as u32, height as u32))
}

// ---------------------------------------------------------------------------
// Interne Hilfsfunktionen
// ---------------------------------------------------------------------------

/// Liest den Pixelwert an (x, y); gibt 0 zurueck wenn ausserhalb des Rasters.
fn get_pixel(pixels: &[u8], width: usize, height: usize, x: i32, y: i32) -> u8 {
    if x < 0 || y < 0 || x >= width as i32 || y >= height as i32 {
        0
    } else {
        pixels[y as usize * width + x as usize]
    }
}

/// Berechnet den Startindex fuer die Clockwise-Suche aus der Richtung current → prev.
///
/// Gibt den CLOCKWISE-Index zurueck, der der Richtung von `current` nach `prev` entspricht.
fn clockwise_start_index(current: (i32, i32), prev: (i32, i32)) -> usize {
    let dx = prev.0 - current.0;
    let dy = prev.1 - current.1;
    CLOCKWISE
        .iter()
        .position(|&(odx, ody)| odx == dx && ody == dy)
        .unwrap_or(0)
}

/// Moore-Neighbor-Boundary-Tracing mit Jacob's Stopping-Criterion.
///
/// Tracer den aeusseren Rand der Komponente mit ID `target_id`.
/// `start` muss der top-leftmost Pixel der Komponente sein
/// (garantiert durch die Scan-Reihenfolge in `extract_farmland_polygons`).
fn trace_moore_contour(
    pixels: &[u8],
    width: usize,
    height: usize,
    target_id: u8,
    start: (i32, i32),
) -> Vec<(f32, f32)> {
    // Initialer Backtrack: Pixel westlich vom Start (immer Hintergrund,
    // da Start der leftmost-Pixel in der topmost-Zeile der Komponente ist).
    let initial_b = (start.0 - 1, start.1);

    let mut contour = vec![(start.0 as f32, start.1 as f32)];
    let mut current = start;
    let mut b = initial_b;

    // Jacob's Stopping-Criterion: Wir stoppen, wenn wir zum Start zurueckkehren
    // und dabei genau denselben Backtrack wie beim ersten Besuch des Starts haben.
    let mut b_at_first_return: Option<(i32, i32)> = None;

    // Sicherheitslimit: maximal 4 × Rasterpixel Schritte
    let max_steps = width * height * 4;

    for _ in 0..max_steps {
        // Naechsten Vordergrund-Pixel im Uhrzeigersinn ab Backtrack suchen
        let start_idx = clockwise_start_index(current, b);
        let mut found_next: Option<(i32, i32)> = None;
        let mut new_b = b;

        for i in 0..8_usize {
            let idx = (start_idx + i) % 8;
            let (dx, dy) = CLOCKWISE[idx];
            let nx = current.0 + dx;
            let ny = current.1 + dy;
            if get_pixel(pixels, width, height, nx, ny) == target_id {
                found_next = Some((nx, ny));
                break;
            } else {
                // Letzten Hintergrund-Nachbar merken
                new_b = (nx, ny);
            }
        }

        let next = match found_next {
            Some(p) => p,
            // Isolierter Einzelpixel – kein weiterer Nachbar
            None => break,
        };

        b = new_b;
        current = next;

        // Jacob's Stopping-Criterion pruefen
        if current == start {
            match b_at_first_return {
                None => {
                    // Erster Besuch beim Start: Backtrack-Zustand merken
                    b_at_first_return = Some(b);
                    // Start nicht doppelt in den Contour einfuegen
                }
                Some(initial_b_on_return) => {
                    if b == initial_b_on_return {
                        // Zweiter Besuch mit gleichem Backtrack → vollstaendig
                        break;
                    }
                    // Grenzfall: anderer Backtrack, weiterlaufen ohne zu pushen
                }
            }
        } else {
            contour.push((current.0 as f32, current.1 as f32));
        }
    }

    contour
}

/// Entfernt aufeinander folgende doppelte Vertices aus dem Contour.
fn dedup_consecutive(mut vertices: Vec<(f32, f32)>) -> Vec<(f32, f32)> {
    vertices.dedup();
    vertices
}
