//! Centerline-Berechnung: Polygon-basiert (Hauptalgorithmus) und Pixel-BFS (Fallback).

use super::farmland::FarmlandGrid;
use glam::Vec2;
use std::collections::{HashSet, VecDeque};

// ---------------------------------------------------------------------------
// VoronoiGrid
// ---------------------------------------------------------------------------

/// Ergebnis der Voronoi-BFS: Für jeden Void-Pixel die nächste Farmland-ID und Distanz.
pub struct VoronoiGrid {
    /// Nächste Farmland-ID pro Pixel (0 = nicht initialisiert)
    pub nearest_id: Vec<u8>,
    /// Distanz zum nächsten Farmland-Pixel (skaliert ×10; gerade=10, diagonal=14)
    pub distance: Vec<u16>,
    /// Rasterbreite in Pixeln
    pub width: u32,
    /// Rasterhöhe in Pixeln
    pub height: u32,
}

// ---------------------------------------------------------------------------
// Polygon-basierte Centerline (Hauptalgorithmus — kein Pixel-Grid noetig)
// ---------------------------------------------------------------------------

/// Berechnet die Mittellinie zwischen zwei Gruppen von Feld-Polygonen.
///
/// Rein polygon-basiert: funktioniert ohne Pixel-Grid, nur mit Weltkoordinaten.
/// Unterstuetzt mehrere Felder pro Seite (z.B. 2 Felder in Gruppe 1, 1 in Gruppe 2).
///
/// Algorithmus:
/// 1. Kanten beider Polygon-Gruppen dicht abtasten
/// 2. Fuer jeden Punkt auf Seite 1 den naechsten auf Seite 2 finden
/// 3. Paare auf den Korridor-Bereich filtern (nur die zueinander gewandten Kanten)
/// 4. Mittelpunkte berechnen und entlang der Hauptachse ordnen
pub fn compute_polygon_centerline(
    side1_polys: &[&[Vec2]],
    side2_polys: &[&[Vec2]],
    sample_spacing: f32,
) -> Vec<Vec2> {
    let samples1 = sample_multiple_polygon_edges(side1_polys, sample_spacing);
    let samples2 = sample_multiple_polygon_edges(side2_polys, sample_spacing);

    if samples1.is_empty() || samples2.is_empty() {
        return Vec::new();
    }

    // Fuer jeden Punkt auf Seite 1: naechsten auf Seite 2 und Distanz
    let mut pairs: Vec<(Vec2, Vec2, f32)> = samples1
        .iter()
        .map(|&p1| {
            let (p2, d) = nearest_in_set(p1, &samples2);
            (p1, p2, d)
        })
        .collect();

    if pairs.is_empty() {
        return Vec::new();
    }

    // Korridor-Filter: Nur die zueinander gewandten Kantenpaare behalten.
    // Heuristik: Paare sortieren, untere Haelfte der Distanzen = Korridor-Bereich.
    pairs.sort_by(|a, b| a.2.partial_cmp(&b.2).unwrap_or(std::cmp::Ordering::Equal));

    let min_dist = pairs[0].2;
    // Schwellwert: 3× die Minimaldistanz, mindestens 30m, maximal 50% der Paare
    let dist_threshold = (min_dist * 3.0).max(30.0);
    let max_count = (pairs.len() / 2).max(1);

    let corridor_pairs: Vec<(Vec2, Vec2)> = pairs
        .into_iter()
        .filter(|&(_, _, d)| d <= dist_threshold)
        .take(max_count)
        .map(|(p1, p2, _)| (p1, p2))
        .collect();

    if corridor_pairs.is_empty() {
        return Vec::new();
    }

    // Mittelpunkte berechnen
    let midpoints: Vec<Vec2> = corridor_pairs
        .iter()
        .map(|&(p1, p2)| (p1 + p2) * 0.5)
        .collect();

    // Entlang der Hauptachse ordnen (PCA-basiert)
    order_points_by_principal_axis(&midpoints)
}

/// Berechnet die Mittellinie zwischen zwei Gruppen von Grenz-Segmenten.
///
/// Rein geometrisch: funktioniert ohne Pixel-Grid.
/// Nutzt den gleichen Ansatz wie `compute_polygon_centerline`, aber mit offenen Polylines.
pub fn compute_segment_centerline(
    side1_segs: &[Vec<Vec2>],
    side2_segs: &[Vec<Vec2>],
    sample_spacing: f32,
) -> Vec<Vec2> {
    let samples1 = sample_multiple_polylines(side1_segs, sample_spacing);
    let samples2 = sample_multiple_polylines(side2_segs, sample_spacing);

    if samples1.is_empty() || samples2.is_empty() {
        return Vec::new();
    }

    // Alle Paare bilden (beide Seiten sind schon Korridor-Kanten)
    let midpoints: Vec<Vec2> = samples1
        .iter()
        .map(|&p1| {
            let (p2, _) = nearest_in_set(p1, &samples2);
            (p1 + p2) * 0.5
        })
        .collect();

    order_points_by_principal_axis(&midpoints)
}

// ---------------------------------------------------------------------------
// Pixel-basierte Centerline (Voronoi-BFS — benoetigt FarmlandGrid)
// ---------------------------------------------------------------------------

/// Berechnet Multi-Source BFS auf dem Farmland-Grid.
///
/// Alle Farmland-Pixel (`grid.ids[i] > 0`) sind Seeds mit Distanz 0.
/// Jeder Void-Pixel erhält die ID und Distanz des nächsten Farmland-Pixels.
/// 8-Konnektivität (diagonal ≈ 14, gerade = 10 für ganzzahlige Approximation).
pub fn compute_voronoi_bfs(grid: &FarmlandGrid) -> VoronoiGrid {
    let w = grid.width as usize;
    let h = grid.height as usize;
    let n = w * h;

    let mut nearest_id = vec![0u8; n];
    let mut distance = vec![u16::MAX; n];
    let mut queue: VecDeque<usize> = VecDeque::new();

    // Alle Farmland-Pixel als Multi-Source-Seeds initialisieren
    for (i, &id) in grid.ids.iter().enumerate() {
        if id > 0 {
            nearest_id[i] = id;
            distance[i] = 0;
            queue.push_back(i);
        }
    }

    // BFS mit 8 Nachbarn
    let offsets: [(i32, i32, u16); 8] = [
        (-1, 0, 10),
        (1, 0, 10),
        (0, -1, 10),
        (0, 1, 10), // gerade
        (-1, -1, 14),
        (1, -1, 14),
        (-1, 1, 14),
        (1, 1, 14), // diagonal
    ];

    while let Some(idx) = queue.pop_front() {
        let cx = (idx % w) as i32;
        let cy = (idx / w) as i32;
        let cur_dist = distance[idx];
        let cur_id = nearest_id[idx];

        for (dx, dy, step) in offsets {
            let nx = cx + dx;
            let ny = cy + dy;
            if nx < 0 || ny < 0 || nx >= w as i32 || ny >= h as i32 {
                continue;
            }
            let nidx = (ny as usize) * w + (nx as usize);
            let new_dist = cur_dist.saturating_add(step);
            if new_dist < distance[nidx] {
                distance[nidx] = new_dist;
                nearest_id[nidx] = cur_id;
                queue.push_back(nidx);
            }
        }
    }

    VoronoiGrid {
        nearest_id,
        distance,
        width: grid.width,
        height: grid.height,
    }
}

/// Extrahiert die Mittellinie eines Korridors zwischen zwei Feldgruppen.
///
/// - `voronoi`: Voronoi-BFS-Ergebnis
/// - `side1_ids`: Farmland-IDs der Seite 1
/// - `side2_ids`: Farmland-IDs der Seite 2
/// - `grid`: FarmlandGrid für Koordinatentransformation
///
/// Gibt die Mittellinie als Polyline in Welt-Koordinaten zurück.
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
            // Prüfen ob Nachbar aus anderer Seite kommt
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
/// - `grid`: FarmlandGrid für Koordinatentransformation
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

// ---------------------------------------------------------------------------
// Hilfsfunktionen: Polygon-Sampling und Punkt-Ordnung
// ---------------------------------------------------------------------------

/// Tastet die Kanten mehrerer Polygone gleichmaessig ab.
fn sample_multiple_polygon_edges(polys: &[&[Vec2]], spacing: f32) -> Vec<Vec2> {
    let mut all = Vec::new();
    for vertices in polys {
        all.extend(sample_closed_ring(vertices, spacing));
    }
    all
}

/// Tastet mehrere offene Polylines gleichmaessig ab.
fn sample_multiple_polylines(segs: &[Vec<Vec2>], spacing: f32) -> Vec<Vec2> {
    let mut all = Vec::new();
    for seg in segs {
        all.extend(sample_open_polyline(seg, spacing));
    }
    all
}

/// Tastet einen geschlossenen Polygon-Ring gleichmaessig ab.
fn sample_closed_ring(vertices: &[Vec2], spacing: f32) -> Vec<Vec2> {
    if vertices.len() < 2 {
        return vertices.to_vec();
    }
    let n = vertices.len();
    let mut samples = Vec::new();
    for i in 0..n {
        let a = vertices[i];
        let b = vertices[(i + 1) % n];
        let edge_len = a.distance(b);
        let steps = (edge_len / spacing).ceil().max(1.0) as usize;
        for s in 0..steps {
            let t = s as f32 / steps as f32;
            samples.push(a.lerp(b, t));
        }
    }
    samples
}

/// Tastet eine offene Polyline gleichmaessig ab.
fn sample_open_polyline(points: &[Vec2], spacing: f32) -> Vec<Vec2> {
    if points.len() < 2 {
        return points.to_vec();
    }
    let mut samples = Vec::new();
    for pair in points.windows(2) {
        let a = pair[0];
        let b = pair[1];
        let edge_len = a.distance(b);
        let steps = (edge_len / spacing).ceil().max(1.0) as usize;
        for s in 0..steps {
            let t = s as f32 / steps as f32;
            samples.push(a.lerp(b, t));
        }
    }
    // Letzten Punkt anfuegen
    if let Some(&last) = points.last() {
        samples.push(last);
    }
    samples
}

/// Findet den naechsten Punkt in einer Menge (Brute-Force).
fn nearest_in_set(query: Vec2, set: &[Vec2]) -> (Vec2, f32) {
    let mut best = set[0];
    let mut best_d = query.distance_squared(best);
    for &p in &set[1..] {
        let d = query.distance_squared(p);
        if d < best_d {
            best_d = d;
            best = p;
        }
    }
    (best, best_d.sqrt())
}

/// Ordnet Punkte entlang ihrer Hauptachse (PCA-basiert).
///
/// Berechnet die Richtung maximaler Varianz und projiziert alle Punkte darauf.
/// Dadurch entsteht eine sinnvolle Reihenfolge entlang des Korridors.
fn order_points_by_principal_axis(points: &[Vec2]) -> Vec<Vec2> {
    if points.len() < 2 {
        return points.to_vec();
    }

    let n = points.len() as f32;
    let centroid = points.iter().copied().sum::<Vec2>() / n;

    // 2×2 Kovarianzmatrix
    let mut cxx = 0.0f32;
    let mut cyy = 0.0f32;
    let mut cxy = 0.0f32;
    for &p in points {
        let d = p - centroid;
        cxx += d.x * d.x;
        cyy += d.y * d.y;
        cxy += d.x * d.y;
    }

    // Haupteigenvektor der 2×2 Kovarianzmatrix
    let angle = 0.5 * (2.0 * cxy).atan2(cxx - cyy);
    let axis = Vec2::new(angle.cos(), angle.sin());

    // Punkte auf Achse projizieren, nach Projektion sortieren, Duplikate entfernen
    let mut projected: Vec<(f32, Vec2)> = points
        .iter()
        .map(|&p| ((p - centroid).dot(axis), p))
        .collect();
    projected.sort_by(|a, b| a.0.partial_cmp(&b.0).unwrap_or(std::cmp::Ordering::Equal));

    // Eng beieinanderliegende Punkte deduplizieren (< 0.5m Abstand)
    let mut result = Vec::with_capacity(projected.len());
    for &(_, p) in &projected {
        if result.last().map_or(true, |&last: &Vec2| last.distance(p) > 0.5) {
            result.push(p);
        }
    }
    result
}

// ---------------------------------------------------------------------------
// Hilfsfunktionen: Pixel-Chaining und Rasterisierung
// ---------------------------------------------------------------------------

/// Verkettete Pixel-Liste aus unsortierten Kantenpixeln erstellen.
///
/// HashSet-basierter 8-Nachbar-Walk: O(n) im Normalfall statt O(n²) brute force.
/// Wächst vom Startpunkt aus in beide Richtungen.
fn chain_pixels(pixels: &[(u32, u32)]) -> Vec<(u32, u32)> {
    if pixels.is_empty() {
        return Vec::new();
    }

    let mut remaining: HashSet<(u32, u32)> = pixels.iter().copied().collect();

    // Startpunkt: minimales (x, y) für deterministisches Ergebnis
    let start = *pixels
        .iter()
        .min_by_key(|&&(x, y)| (x, y))
        .expect("pixels ist nicht leer");
    remaining.remove(&start);

    // Vorwärts-Walk
    let mut chain = vec![start];
    loop {
        let &(cx, cy) = chain.last().expect("chain ist nicht leer");
        match find_8neighbor(cx, cy, &remaining) {
            Some(n) => {
                remaining.remove(&n);
                chain.push(n);
            }
            None => break,
        }
    }

    // Rückwärts-Walk vom Startpunkt aus (verbleibende Pixel einsammeln)
    let mut backward: Vec<(u32, u32)> = Vec::new();
    let (mut cx, mut cy) = start;
    loop {
        match find_8neighbor(cx, cy, &remaining) {
            Some(n) => {
                remaining.remove(&n);
                backward.push(n);
                (cx, cy) = n;
            }
            None => break,
        }
    }

    // Ergebnis: rückwärts-Kette (umgekehrt) + vorwärts-Kette
    backward.reverse();
    backward.extend(chain);
    backward
}

/// Sucht den ersten 8-Nachbarn von `(cx, cy)` der in `remaining` liegt.
fn find_8neighbor(cx: u32, cy: u32, remaining: &HashSet<(u32, u32)>) -> Option<(u32, u32)> {
    const DIRS: [(i32, i32); 8] = [
        (-1, 0),
        (1, 0),
        (0, -1),
        (0, 1),
        (-1, -1),
        (1, -1),
        (-1, 1),
        (1, 1),
    ];
    for (dx, dy) in DIRS {
        let nx = cx as i32 + dx;
        let ny = cy as i32 + dy;
        if nx >= 0 && ny >= 0 {
            let candidate = (nx as u32, ny as u32);
            if remaining.contains(&candidate) {
                return Some(candidate);
            }
        }
    }
    None
}

/// Rasterisiert eine Liste von Polylines auf das Grid mit Bresenham-Linien.
///
/// Setzt jeden berührten Pixel auf die angegebene ID.
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

/// Bresenham-Linienalgorithmus — ruft `callback(x, y)` für jeden Pixel auf.
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

// ---------------------------------------------------------------------------
// Unit-Tests
// ---------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;
    use crate::core::farmland::FarmlandGrid;

    /// BFS auf einem kleinen Grid mit 2 Feldern — Voronoi-Grenze muss existieren.
    #[test]
    fn test_voronoi_bfs_basic() {
        // 6×1 Grid: [1,1,0,0,2,2]
        let ids = vec![1u8, 1, 0, 0, 2, 2];
        let grid = FarmlandGrid::new(ids, 6, 1, 6.0);
        let voronoi = compute_voronoi_bfs(&grid);

        // Linke Hälfte muss ID 1 haben
        assert_eq!(voronoi.nearest_id[0], 1);
        assert_eq!(voronoi.nearest_id[1], 1);
        // Rechte Hälfte muss ID 2 haben
        assert_eq!(voronoi.nearest_id[4], 2);
        assert_eq!(voronoi.nearest_id[5], 2);
        // Void-Pixel erhalten eine der beiden IDs
        assert!(voronoi.nearest_id[2] == 1 || voronoi.nearest_id[2] == 2);
        assert!(voronoi.nearest_id[3] == 1 || voronoi.nearest_id[3] == 2);
    }

    /// Centerline zwischen zwei benachbarten vertikalen Feldern.
    #[test]
    fn test_corridor_centerline_vertical() {
        // 5×3 Grid: Spalte 0-1 = ID 1, Spalte 3-4 = ID 2, Mitte = Void
        let w = 5u32;
        let h = 3u32;
        let mut ids = vec![0u8; (w * h) as usize];
        for y in 0..h {
            for x in 0..w {
                let i = (y * w + x) as usize;
                if x < 2 {
                    ids[i] = 1;
                } else if x >= 3 {
                    ids[i] = 2;
                }
            }
        }
        let grid = FarmlandGrid::new(ids, w, h, 5.0);
        let voronoi = compute_voronoi_bfs(&grid);
        let line = extract_corridor_centerline(&voronoi, &[1], &[2], &grid);
        // Mittellinie muss mindestens einen Punkt haben
        assert!(!line.is_empty());
    }

    /// chain_pixels mit einfacher 3-Pixel-Linie.
    #[test]
    fn test_chain_pixels_simple() {
        let pixels = vec![(0u32, 0u32), (2, 0), (1, 0)];
        let chained = chain_pixels(&pixels);
        assert_eq!(chained.len(), 3);
        // Erste pixel muss (0,0) sein (Startpunkt)
        assert_eq!(chained[0], (0, 0));
    }

    /// Leere Eingabe ergibt leere Ausgabe.
    #[test]
    fn test_chain_pixels_empty() {
        let chained = chain_pixels(&[]);
        assert!(chained.is_empty());
    }

    /// `extract_boundary_centerline` mit zwei parallelen vertikalen Linien.
    ///
    /// Die Mittellinie muss zwischen den beiden Seiten liegen (x ≈ 0).
    #[test]
    fn test_extract_boundary_centerline_basic() {
        let w = 20u32;
        let h = 20u32;
        let map_size = 20.0_f32;
        // scale = map_size / w = 1.0, half = 10.0
        // pixel x=5  → world x = 5*1 - 10 = -5.0
        // pixel x=15 → world x = 15*1 - 10 =  5.0
        let ids = vec![0u8; (w * h) as usize];
        let grid = FarmlandGrid::new(ids, w, h, map_size);

        let side1 = vec![Vec2::new(-5.0, -8.0), Vec2::new(-5.0, 8.0)];
        let side2 = vec![Vec2::new(5.0, -8.0), Vec2::new(5.0, 8.0)];

        let result = extract_boundary_centerline(&[side1], &[side2], &grid);

        assert!(!result.is_empty(), "Mittellinie darf nicht leer sein");
        for pt in &result {
            assert!(
                pt.x.abs() < 3.0,
                "Mittellinien-Punkt x={:.2} sollte nahe 0 liegen",
                pt.x
            );
        }
    }

    /// Polygon-basierte Centerline zwischen zwei parallelen Rechtecken.
    #[test]
    fn test_polygon_centerline_two_rects() {
        // Feld 1: Rechteck links (x = -20..-10, y = -30..30)
        let poly1 = vec![
            Vec2::new(-20.0, -30.0),
            Vec2::new(-10.0, -30.0),
            Vec2::new(-10.0, 30.0),
            Vec2::new(-20.0, 30.0),
        ];
        // Feld 2: Rechteck rechts (x = 10..20, y = -30..30)
        let poly2 = vec![
            Vec2::new(10.0, -30.0),
            Vec2::new(20.0, -30.0),
            Vec2::new(20.0, 30.0),
            Vec2::new(10.0, 30.0),
        ];

        let result = compute_polygon_centerline(
            &[poly1.as_slice()],
            &[poly2.as_slice()],
            2.0,
        );

        assert!(!result.is_empty(), "Mittellinie darf nicht leer sein");
        // Mittellinie muss zwischen x=-10 und x=10 liegen (nahe x=0)
        for pt in &result {
            assert!(
                pt.x.abs() < 5.0,
                "Mittellinien-Punkt x={:.2} sollte nahe 0 liegen",
                pt.x
            );
        }
    }

    /// Polygon-Centerline mit 2 Feldern auf Seite 1, 1 Feld auf Seite 2.
    #[test]
    fn test_polygon_centerline_two_vs_one() {
        // Seite 1: Zwei Felder nebeneinander (links oben + links unten)
        let poly1a = vec![
            Vec2::new(-20.0, 0.0),
            Vec2::new(-10.0, 0.0),
            Vec2::new(-10.0, 30.0),
            Vec2::new(-20.0, 30.0),
        ];
        let poly1b = vec![
            Vec2::new(-20.0, -30.0),
            Vec2::new(-10.0, -30.0),
            Vec2::new(-10.0, 0.0),
            Vec2::new(-20.0, 0.0),
        ];
        // Seite 2: Ein Feld rechts
        let poly2 = vec![
            Vec2::new(10.0, -30.0),
            Vec2::new(20.0, -30.0),
            Vec2::new(20.0, 30.0),
            Vec2::new(10.0, 30.0),
        ];

        let result = compute_polygon_centerline(
            &[poly1a.as_slice(), poly1b.as_slice()],
            &[poly2.as_slice()],
            2.0,
        );

        assert!(!result.is_empty(), "Mittellinie darf nicht leer sein");
        for pt in &result {
            assert!(
                pt.x.abs() < 5.0,
                "Mittellinien-Punkt x={:.2} sollte nahe 0 liegen",
                pt.x
            );
        }
    }

    /// Segment-basierte Centerline.
    #[test]
    fn test_segment_centerline() {
        let side1 = vec![Vec2::new(-5.0, -20.0), Vec2::new(-5.0, 20.0)];
        let side2 = vec![Vec2::new(5.0, -20.0), Vec2::new(5.0, 20.0)];

        let result = compute_segment_centerline(&[side1], &[side2], 2.0);

        assert!(!result.is_empty(), "Mittellinie darf nicht leer sein");
        for pt in &result {
            assert!(
                pt.x.abs() < 3.0,
                "Mittellinien-Punkt x={:.2} sollte nahe 0 liegen",
                pt.x
            );
        }
    }
}
