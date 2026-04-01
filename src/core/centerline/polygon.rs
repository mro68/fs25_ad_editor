use super::helpers::{
    nearest_in_set, order_points_by_principal_axis, sample_multiple_polygon_edges,
};
use glam::Vec2;

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
