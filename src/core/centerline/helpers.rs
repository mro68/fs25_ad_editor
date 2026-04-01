use glam::Vec2;
use std::collections::HashSet;

/// Tastet die Kanten mehrerer Polygone gleichmaessig ab.
pub(super) fn sample_multiple_polygon_edges(polys: &[&[Vec2]], spacing: f32) -> Vec<Vec2> {
    let mut all = Vec::new();
    for vertices in polys {
        all.extend(sample_closed_ring(vertices, spacing));
    }
    all
}

/// Tastet mehrere offene Polylines gleichmaessig ab.
pub(super) fn sample_multiple_polylines(segs: &[Vec<Vec2>], spacing: f32) -> Vec<Vec2> {
    let mut all = Vec::new();
    for seg in segs {
        all.extend(sample_open_polyline(seg, spacing));
    }
    all
}

/// Findet den naechsten Punkt in einer Menge (Brute-Force).
pub(super) fn nearest_in_set(query: Vec2, set: &[Vec2]) -> (Vec2, f32) {
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
pub(super) fn order_points_by_principal_axis(points: &[Vec2]) -> Vec<Vec2> {
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
        if result
            .last()
            .is_none_or(|&last: &Vec2| last.distance(p) > 0.5)
        {
            result.push(p);
        }
    }
    result
}

/// Verkettete Pixel-Liste aus unsortierten Kantenpixeln erstellen.
///
/// HashSet-basierter 8-Nachbar-Walk: O(n) im Normalfall statt O(n²) brute force.
/// Wächst vom Startpunkt aus in beide Richtungen.
pub(super) fn chain_pixels(pixels: &[(u32, u32)]) -> Vec<(u32, u32)> {
    if pixels.is_empty() {
        return Vec::new();
    }

    let mut remaining: HashSet<(u32, u32)> = pixels.iter().copied().collect();

    // Startpunkt: minimales (x, y) fuer deterministisches Ergebnis
    let start = *pixels
        .iter()
        .min_by_key(|&&(x, y)| (x, y))
        .expect("pixels ist nicht leer");
    remaining.remove(&start);

    // Vorwaerts-Walk
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

    // Rueckwaerts-Walk vom Startpunkt aus (verbleibende Pixel einsammeln)
    let mut backward: Vec<(u32, u32)> = Vec::new();
    let (mut cx, mut cy) = start;
    while let Some(n) = find_8neighbor(cx, cy, &remaining) {
        remaining.remove(&n);
        backward.push(n);
        (cx, cy) = n;
    }

    // Ergebnis: rueckwaerts-Kette (umgekehrt) + vorwaerts-Kette
    backward.reverse();
    backward.extend(chain);
    backward
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
