//! Reine Geometrie-Funktionen für Catmull-Rom-Splines.
//!
//! Layer-neutral: kann von `tools`, `use_cases` und anderen Layer-übergreifenden
//! Modulen importiert werden ohne Zirkel-Abhängigkeiten zu erzeugen.

use glam::Vec2;

/// Berechnet einen Punkt auf einem Catmull-Rom-Segment (t ∈ [0, 1]).
///
/// p0, p1, p2, p3: vier aufeinanderfolgende Kontrollpunkte.
/// Die Kurve verläuft von p1 nach p2.
pub fn catmull_rom_point(p0: Vec2, p1: Vec2, p2: Vec2, p3: Vec2, t: f32) -> Vec2 {
    let t2 = t * t;
    let t3 = t2 * t;
    0.5 * ((2.0 * p1)
        + (-p0 + p2) * t
        + (2.0 * p0 - 5.0 * p1 + 4.0 * p2 - p3) * t2
        + (-p0 + 3.0 * p1 - 3.0 * p2 + p3) * t3)
}

/// Berechnet eine dichte Punktliste entlang einer Catmull-Rom-Spline durch `points`.
///
/// Für Rand-Segmente werden Phantom-Punkte gespiegelt, damit die Kurve
/// natürlich durch den ersten und letzten Punkt läuft.
/// Wenn `start_phantom`/`end_phantom` gesetzt, werden die Phantom-Punkte überschrieben.
///
/// `samples_per_segment`: Anzahl der Zwischenpunkte pro Segment (ohne Endpunkt).
pub fn catmull_rom_chain_with_tangents(
    points: &[Vec2],
    samples_per_segment: usize,
    start_phantom: Option<Vec2>,
    end_phantom: Option<Vec2>,
) -> Vec<Vec2> {
    if points.len() < 2 {
        return points.to_vec();
    }
    if points.len() == 2 {
        // Gerade Linie — kein Spline nötig
        let mut result = Vec::with_capacity(samples_per_segment + 1);
        for i in 0..=samples_per_segment {
            let t = i as f32 / samples_per_segment as f32;
            result.push(points[0].lerp(points[1], t));
        }
        return result;
    }

    let n = points.len();
    let mut result = Vec::with_capacity((n - 1) * samples_per_segment + 1);

    for seg in 0..(n - 1) {
        // Phantom-Punkte an den Rändern (ggf. durch Tangente überschrieben)
        let p0 = if seg == 0 {
            start_phantom.unwrap_or_else(|| 2.0 * points[0] - points[1])
        } else {
            points[seg - 1]
        };
        let p1 = points[seg];
        let p2 = points[seg + 1];
        let p3 = if seg + 2 < n {
            points[seg + 2]
        } else {
            end_phantom.unwrap_or_else(|| 2.0 * points[n - 1] - points[n - 2])
        };

        let steps = if seg == n - 2 {
            samples_per_segment + 1 // letztes Segment: Endpunkt einschließen
        } else {
            samples_per_segment
        };

        for i in 0..steps {
            let t = i as f32 / samples_per_segment as f32;
            result.push(catmull_rom_point(p0, p1, p2, p3, t));
        }
    }

    result
}

/// Approximierte Länge einer Polyline.
pub fn polyline_length(points: &[Vec2]) -> f32 {
    points.windows(2).map(|w| w[0].distance(w[1])).sum()
}

/// Verteilt Punkte gleichmäßig (Arc-Length) entlang einer Polyline.
pub fn resample_by_distance(polyline: &[Vec2], max_segment_length: f32) -> Vec<Vec2> {
    if polyline.len() < 2 {
        return polyline.to_vec();
    }

    let total = polyline_length(polyline);
    if total < f32::EPSILON {
        return vec![polyline[0]];
    }

    let segment_count = (total / max_segment_length).ceil().max(1.0) as usize;
    let spacing = total / segment_count as f32;

    let mut result = Vec::with_capacity(segment_count + 1);
    result.push(polyline[0]);

    let mut poly_idx = 0;
    let mut remainder = 0.0f32; // Rest-Distanz im aktuellen Polyline-Segment

    for _ in 1..segment_count {
        let mut needed = spacing;

        loop {
            if poly_idx + 1 >= polyline.len() {
                break;
            }
            let seg_len = polyline[poly_idx].distance(polyline[poly_idx + 1]);
            let available = seg_len - remainder;

            if available >= needed {
                remainder += needed;
                let t = remainder / seg_len;
                result.push(polyline[poly_idx].lerp(polyline[poly_idx + 1], t));
                break;
            } else {
                needed -= available;
                remainder = 0.0;
                poly_idx += 1;
            }
        }
    }

    // Endpunkt immer exakt übernehmen
    result.push(*polyline.last().unwrap());
    result
}
