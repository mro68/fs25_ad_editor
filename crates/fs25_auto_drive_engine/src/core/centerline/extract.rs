use super::helpers::chain_pixels;
use super::voronoi::{compute_voronoi_bfs, VoronoiGrid};
use crate::core::farmland::FarmlandGrid;
use glam::Vec2;

/// Extrahiert die Mittellinie eines Korridors zwischen zwei Feldgruppen.
///
/// - `voronoi`: Voronoi-BFS-Ergebnis
/// - `side1_ids`: Farmland-IDs der Seite 1
/// - `side2_ids`: Farmland-IDs der Seite 2
/// - `grid`: FarmlandGrid fuer Koordinatentransformation
///
/// Gibt die Mittellinie als Polyline in Welt-Koordinaten zurueck.
pub fn extract_corridor_centerline(
    voronoi: &VoronoiGrid,
    side1_ids: &[u8],
    side2_ids: &[u8],
    grid: &FarmlandGrid,
) -> Vec<Vec2> {
    let w = voronoi.width as usize;
    let h = voronoi.height as usize;

    let in_side1 = |id: u8| side1_ids.contains(&id);
    let in_side2 = |id: u8| side2_ids.contains(&id);

    // Voronoi-Kantenpixel zwischen den beiden Seiten sammeln
    let mut edge_pixels: Vec<(u32, u32)> = Vec::new();
    let offsets4: [(i32, i32); 4] = [(-1, 0), (1, 0), (0, -1), (0, 1)];

    for y in 0..h {
        for x in 0..w {
            let idx = y * w + x;
            let id = voronoi.nearest_id[idx];
            if id == 0 {
                continue;
            }

            // Pruefen ob Nachbar aus anderer Seite kommt
            for (dx, dy) in offsets4 {
                let nx = x as i32 + dx;
                let ny = y as i32 + dy;
                if nx < 0 || ny < 0 || nx >= w as i32 || ny >= h as i32 {
                    continue;
                }
                let nid = voronoi.nearest_id[(ny as usize) * w + (nx as usize)];
                if (in_side1(id) && in_side2(nid)) || (in_side2(id) && in_side1(nid)) {
                    edge_pixels.push((x as u32, y as u32));
                    break;
                }
            }
        }
    }

    if edge_pixels.is_empty() {
        return Vec::new();
    }

    // Pixel zu Nearest-Neighbor-Kette zusammensetzen
    let chained = chain_pixels(&edge_pixels);

    // Pixel → Weltkoordinaten
    chained
        .into_iter()
        .map(|(px, py)| grid.pixel_to_world(px, py))
        .collect()
}

/// Extrahiert die Mittellinie zwischen zwei Gruppen von Feldgrenzen-Segmenten.
///
/// - `segments_side1`: Polyline-Segmente der Seite 1 (Welt-Koordinaten)
/// - `segments_side2`: Polyline-Segmente der Seite 2 (Welt-Koordinaten)
/// - `grid`: FarmlandGrid fuer Koordinatentransformation
///
/// Rasterisiert die Segmente als Pseudo-Felder (ID 254 / 255), dann BFS-Centerline.
pub fn extract_boundary_centerline(
    segments_side1: &[Vec<Vec2>],
    segments_side2: &[Vec<Vec2>],
    grid: &FarmlandGrid,
) -> Vec<Vec2> {
    let w = grid.width as usize;
    let h = grid.height as usize;
    let mut ids = vec![0u8; w * h];

    // Seite 1 auf ID 254 rasterisieren
    rasterize_segments(segments_side1, &mut ids, grid, 254);
    // Seite 2 auf ID 255 rasterisieren
    rasterize_segments(segments_side2, &mut ids, grid, 255);

    let pseudo_grid = FarmlandGrid::new(ids, grid.width, grid.height, grid.map_size);
    let voronoi = compute_voronoi_bfs(&pseudo_grid);

    extract_corridor_centerline(&voronoi, &[254], &[255], &pseudo_grid)
}

/// Rasterisiert eine Liste von Polylines auf das Grid mit Bresenham-Linien.
///
/// Setzt jeden beruehrten Pixel auf die angegebene ID.
fn rasterize_segments(segments: &[Vec<Vec2>], ids: &mut [u8], grid: &FarmlandGrid, id: u8) {
    for segment in segments {
        for pair in segment.windows(2) {
            let (x0, y0) = grid.world_to_pixel(pair[0]);
            let (x1, y1) = grid.world_to_pixel(pair[1]);
            bresenham(x0 as i32, y0 as i32, x1 as i32, y1 as i32, |x, y| {
                if x >= 0 && y >= 0 && (x as u32) < grid.width && (y as u32) < grid.height {
                    let idx = (y as u32 * grid.width + x as u32) as usize;
                    ids[idx] = id;
                }
            });
        }
    }
}

/// Bresenham-Linienalgorithmus — ruft `callback(x, y)` fuer jeden Pixel auf.
fn bresenham<F: FnMut(i32, i32)>(x0: i32, y0: i32, x1: i32, y1: i32, mut callback: F) {
    let dx = (x1 - x0).abs();
    let dy = (y1 - y0).abs();
    let sx = if x0 < x1 { 1 } else { -1 };
    let sy = if y0 < y1 { 1 } else { -1 };
    let mut x = x0;
    let mut y = y0;
    let mut err = dx - dy;

    loop {
        callback(x, y);
        if x == x1 && y == y1 {
            break;
        }
        let e2 = 2 * err;
        if e2 > -dy {
            err -= dy;
            x += sx;
        }
        if e2 < dx {
            err += dx;
            y += sy;
        }
    }
}
