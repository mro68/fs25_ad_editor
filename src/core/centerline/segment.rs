use super::{
    helpers::{order_points_by_principal_axis, sample_multiple_polylines},
    search::SampleSearchIndex,
};
use glam::Vec2;

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

    let samples2_index = SampleSearchIndex::from_points(samples2);

    // Alle Paare bilden (beide Seiten sind schon Korridor-Kanten)
    let midpoints: Vec<Vec2> = samples1
        .iter()
        .map(|&p1| {
            let (p2, _) = samples2_index
                .nearest(p1)
                .expect("samples2_index enthaelt mindestens einen Punkt");
            (p1 + p2) * 0.5
        })
        .collect();

    order_points_by_principal_axis(&midpoints)
}
