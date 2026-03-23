//! Geometrie-Berechnungen fuer das FieldBoundaryTool.
//!
//! Enthaelt Eckenerkennung, Kreisbogenverrundung und Ring-Resampling.

use crate::shared::spline_geometry::resample_by_distance;
use glam::Vec2;

/// Klassifizierung eines Ring-Knotens nach seiner geometrischen Funktion.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum RingNodeKind {
    /// Normaler Punkt des resamplten Rings (zwischen Ecken).
    Regular,
    /// Eckpunkt (Anker) – erkannte scharfe Ecke oder Eintrittspunkt eines Bogens.
    Corner,
    /// Verrundungspunkt – liegt auf einem Kreisbogen an einer verrundeten Ecke.
    RoundedCorner,
}

/// Erkennt Eckpunkte eines Polygons anhand des Ablenkungswinkels.
///
/// Ein Vertex gilt als Ecke, wenn der Ablenkungswinkel >= `angle_threshold_rad`.
/// Kleinerer Schwellwert → mehr Ecken erkannt.
///
/// Gibt sortierte Indizes der erkannten Eckpunkte zurueck.
pub(super) fn detect_corners(vertices: &[Vec2], angle_threshold_rad: f32) -> Vec<usize> {
    let n = vertices.len();
    if n < 3 {
        return Vec::new();
    }
    let mut corners = Vec::new();
    for i in 0..n {
        let prev = vertices[(i + n - 1) % n];
        let curr = vertices[i];
        let next = vertices[(i + 1) % n];
        let seg_a = (curr - prev).normalize_or_zero();
        let seg_b = (next - curr).normalize_or_zero();
        if seg_a == Vec2::ZERO || seg_b == Vec2::ZERO {
            continue;
        }
        // Ablenkungswinkel als Bogenwinkel (0 = Gerade, PI = U-Turn)
        let cos_angle = seg_a.dot(seg_b).clamp(-1.0, 1.0);
        let deflection = cos_angle.acos();
        if deflection >= angle_threshold_rad {
            corners.push(i);
        }
    }
    corners
}

/// Erzeugt einen Kreisbogen zwischen den Tangentenpunkten einer konvexen Ecke.
///
/// - `prev`: Vertex vor der Ecke
/// - `corner`: Eck-Vertex
/// - `next`: Vertex nach der Ecke
/// - `radius`: Verrundungsradius in Metern
/// - `max_angle_deg`: Maximale Winkelabweichung zwischen benachbarten Bogenpunkten in Grad
///
/// Gibt alle Bogenpunkte von t1 bis t2 (inkl.) zurueck.
/// Bei konkaven Ecken (Cross-Product <= 0, CCW-Konvention) oder degenerierten
/// Faellen wird `vec![corner]` zurueckgegeben (scharfe Ecke beibehalten).
pub(super) fn round_corner(
    prev: Vec2,
    corner: Vec2,
    next: Vec2,
    radius: f32,
    max_angle_deg: f32,
) -> Vec<Vec2> {
    let dir_in = (corner - prev).normalize_or_zero();
    let dir_out = (next - corner).normalize_or_zero();
    if dir_in == Vec2::ZERO || dir_out == Vec2::ZERO {
        return vec![corner];
    }

    // Konkave Ecke: Cross-Product <= 0 bei CCW-Polygon → keine Verrundung
    // cross = dir_in.x * dir_out.y - dir_in.y * dir_out.x
    let cross = dir_in.x * dir_out.y - dir_in.y * dir_out.x;
    if cross <= 0.0 {
        return vec![corner];
    }

    // Tangentenpunkte begrenzen auf 40 % der jeweiligen Kantenlaenge
    let max_in = (corner - prev).length() * 0.4;
    let max_out = (next - corner).length() * 0.4;
    let r = radius.min(max_in).min(max_out);
    if r < 0.1 {
        return vec![corner]; // Radius zu klein fuer sinnvollen Bogen
    }

    let t1 = corner - dir_in * r;
    let t2 = corner + dir_out * r;

    // Bogenmittelpunkt: Winkelhalbierender von -dir_in und dir_out
    let bisector = (-dir_in + dir_out).normalize_or_zero();
    if bisector == Vec2::ZERO {
        return vec![corner]; // 180°-Ecke, kein Bogen moeglich
    }

    let half_angle = dir_in.dot(dir_out).clamp(-1.0, 1.0).acos() / 2.0;
    let sin_half = half_angle.sin();
    if sin_half < 1e-6 {
        return vec![corner]; // degenerierter Winkel
    }

    let center_dist = r / sin_half;
    let center = corner + bisector * center_dist;

    // Winkel von Mittelpunkt zu t1 und t2
    let a1 = (t1 - center).to_angle();
    let a2 = (t2 - center).to_angle();

    // Bogenpunkte gleichmaessig ueber den Bogenwinkel verteilen
    let arc_angle_rad = 2.0 * half_angle;
    let n_points = ((arc_angle_rad / max_angle_deg.to_radians()).ceil() as usize).max(2);

    // Kuerzestes Winkelintervall (Gegenuhrzeigersinn bei CCW-Polygon)
    use std::f32::consts::PI;
    let mut delta = a2 - a1;
    if delta > PI {
        delta -= 2.0 * PI;
    }
    if delta < -PI {
        delta += 2.0 * PI;
    }

    let mut points = Vec::with_capacity(n_points + 1);
    for i in 0..=n_points {
        let t = i as f32 / n_points as f32;
        let angle = a1 + delta * t;
        points.push(center + Vec2::from_angle(angle) * r);
    }
    points
}

/// Resampled einen Polygon-Ring segmentweise mit Eckpunkten als festen Ankerpunkten.
///
/// - `simplified`: Vereinfachtes Polygon (nicht geschlossen, ohne letzten==ersten Punkt)
/// - `corner_indices`: Sortierte Indizes der Eckpunkte
/// - `spacing`: Maximaler Segment-Abstand beim Resampling der geraden Segmente
/// - `rounding_radius`: Wenn angegeben, werden konvexe Ecken mit Kreisbogen verrundet
/// - `max_angle_deg`: Maximale Winkelabweichung zwischen Bogenpunkten in Grad
///
/// Ruckgabe: Resamplter Ring als `(Position, RingNodeKind)` pro Punkt.
pub(super) fn resample_ring_with_corners(
    simplified: &[Vec2],
    corner_indices: &[usize],
    spacing: f32,
    rounding_radius: Option<f32>,
    max_angle_deg: f32,
) -> Vec<(Vec2, RingNodeKind)> {
    if corner_indices.is_empty() {
        // Keine Ecken: gesamten Ring normal resampling
        let mut closed = simplified.to_vec();
        closed.push(simplified[0]);
        let mut r = resample_by_distance(&closed, spacing.max(0.1));
        if r.len() > 1 {
            r.pop();
        }
        return r.into_iter().map(|p| (p, RingNodeKind::Regular)).collect();
    }

    let nc = corner_indices.len();
    let n = simplified.len();
    let sp = spacing.max(0.1);

    // Bogen-Daten fuer jede Ecke vorberechnen
    struct ArcEntry {
        t1: Vec2,
        inner: Vec<Vec2>, // Innere Bogenpunkte (ohne t1 und t2)
        t2: Vec2,
        rounded: bool,
    }

    let arcs: Vec<ArcEntry> = corner_indices
        .iter()
        .map(|&ci| {
            if let Some(r) = rounding_radius {
                let prev = simplified[(ci + n - 1) % n];
                let curr = simplified[ci];
                let nxt = simplified[(ci + 1) % n];
                let pts = round_corner(prev, curr, nxt, r, max_angle_deg);
                if pts.len() > 1 {
                    return ArcEntry {
                        t1: pts[0],
                        inner: pts[1..pts.len() - 1].to_vec(),
                        t2: *pts.last().expect("Bogenpunkte haben mindestens 2 Elemente"),
                        rounded: true,
                    };
                }
            }
            let pos = simplified[ci];
            ArcEntry {
                t1: pos,
                inner: vec![],
                t2: pos,
                rounded: false,
            }
        })
        .collect();

    let mut result = Vec::new();

    for c in 0..nc {
        let arc_c = &arcs[c];
        let arc_next = &arcs[(c + 1) % nc];
        let c_start = corner_indices[c];
        let c_end = corner_indices[(c + 1) % nc];

        // 1. Aktuelle Ecke emittieren:
        //    - t1 als Corner (Ankerpunkt der Ecke)
        //    - innere Bogenpunkte als RoundedCorner
        //    - t2 als RoundedCorner (letzter Bogenpunkt, nur bei Verrundung)
        result.push((arc_c.t1, RingNodeKind::Corner));
        for &p in &arc_c.inner {
            result.push((p, RingNodeKind::RoundedCorner));
        }
        if arc_c.rounded {
            result.push((arc_c.t2, RingNodeKind::RoundedCorner));
        }
        // Bei scharfer Ecke: t2 == t1, kein zweiter Punkt noetig

        // 2. Gerades Segment von t2 (aktuell) bis t1 (naechste Ecke)
        //    Polygon-Vertices zwischen den Ecken (exklusiv beide Ecken) einschliessen,
        //    um die tatsaechliche Polygonform zu erhalten.
        let mut seg = vec![arc_c.t2];
        let mut idx = (c_start + 1) % n;
        while idx != c_end {
            seg.push(simplified[idx]);
            idx = (idx + 1) % n;
        }
        seg.push(arc_next.t1);

        let resampled = resample_by_distance(&seg, sp);
        // Ersten Punkt weglassen (= arc_c.t2, bereits emittiert)
        // Letzten Punkt weglassen (= arc_next.t1, wird in naechster Iteration als Corner emittiert)
        let end_idx = if resampled.len() > 1 {
            resampled.len() - 1
        } else {
            resampled.len()
        };
        for &p in &resampled[1..end_idx] {
            result.push((p, RingNodeKind::Regular));
        }
    }

    result
}

#[cfg(test)]
mod tests {
    use super::*;

    /// Hilfsfunktion: Rechteck-Vertices aufbauen
    fn rectangle_vertices() -> Vec<Vec2> {
        vec![
            Vec2::new(0.0, 0.0),
            Vec2::new(100.0, 0.0),
            Vec2::new(100.0, 50.0),
            Vec2::new(0.0, 50.0),
        ]
    }

    #[test]
    fn test_detect_corners_rechteck_vier_ecken() {
        // Rechteck hat 4 rechte Winkel — alle sollen bei 80° Schwellwert erkannt werden
        let verts = rectangle_vertices();
        let threshold_rad = 80_f32.to_radians();
        let corners = detect_corners(&verts, threshold_rad);
        assert_eq!(
            corners.len(),
            4,
            "Rechteck sollte 4 Ecken haben, bekam: {:?}",
            corners
        );
    }

    #[test]
    fn test_detect_corners_kein_ergebnis_bei_hohem_schwellwert() {
        // Rechteck hat ~90° Ecken — bei 150° Schwellwert sollen keine erkannt werden
        let verts = rectangle_vertices();
        let threshold_rad = 150_f32.to_radians();
        let corners = detect_corners(&verts, threshold_rad);
        assert!(
            corners.is_empty(),
            "Bei 150° Schwellwert keine Ecken erwartet, bekam: {:?}",
            corners
        );
    }

    #[test]
    fn test_round_corner_konvexe_ecke_erzeugt_bogen() {
        // Rechtwinklige CCW-Ecke: links-unten → rechts-unten → rechts-oben
        // dir_in = (1,0), dir_out = (0,1) → Cross > 0 → konvex → Bogen erwartet
        let prev = Vec2::new(0.0, 0.0);
        let corner = Vec2::new(100.0, 0.0);
        let next = Vec2::new(100.0, 100.0);
        let pts = round_corner(prev, corner, next, 5.0, 15.0);
        assert!(pts.len() > 1, "Konvexe Ecke sollte Bogenpunkte erzeugen");
        // Erster Punkt ist t1 (auf Eingangs-Kante)
        let t1 = pts[0];
        assert!(
            (t1 - Vec2::new(95.0, 0.0)).length() < 0.1,
            "t1 sollte bei (95,0) liegen, ist: {:?}",
            t1
        );
        // Letzter Punkt ist t2 (auf Ausgangs-Kante)
        let t2 = *pts.last().expect("has last");
        assert!(
            (t2 - Vec2::new(100.0, 5.0)).length() < 0.1,
            "t2 sollte bei (100,5) liegen, ist: {:?}",
            t2
        );
    }

    #[test]
    fn test_round_corner_konkave_ecke_bleibt_scharf() {
        // Rechtwinklige CW-Ecke (konkav in CCW-Polygon): links-unten → rechts-unten → rechts-UNTEN
        // dir_in = (1,0), dir_out = (0,-1) → Cross < 0 → konkav → kein Bogen
        let prev = Vec2::new(0.0, 0.0);
        let corner = Vec2::new(100.0, 0.0);
        let next = Vec2::new(100.0, -100.0);
        let pts = round_corner(prev, corner, next, 5.0, 15.0);
        assert_eq!(
            pts.len(),
            1,
            "Konkave Ecke sollte nur den Eckpunkt zurueckgeben"
        );
        assert_eq!(pts[0], corner);
    }

    #[test]
    fn test_round_corner_gerade_linie() {
        // Gerade Linie: dir_in == dir_out → Cross-Product = 0 → keine Verrundung
        let prev = Vec2::new(0.0, 0.0);
        let corner = Vec2::new(50.0, 0.0);
        let next = Vec2::new(100.0, 0.0);
        let pts = round_corner(prev, corner, next, 5.0, 15.0);
        assert_eq!(
            pts.len(),
            1,
            "Gerade Linie sollte nur Eckpunkt zurueckgeben"
        );
        assert_eq!(pts[0], corner);
    }

    #[test]
    fn test_detect_corners_verschiedene_winkel() {
        // Rechteck: Ablenkungswinkel je 90° — Erkennungsschwelle variieren
        let verts = rectangle_vertices();

        // Schwelle 45°: 90° >= 45° → alle 4 Ecken erkannt
        let corners_45 = detect_corners(&verts, 45_f32.to_radians());
        assert_eq!(
            corners_45.len(),
            4,
            "Bei 45° Schwellwert alle 4 Ecken erwartet"
        );

        // Schwelle genau 90°: 90° >= 90° → alle 4 Ecken erkannt
        let corners_90 = detect_corners(&verts, 90_f32.to_radians());
        assert_eq!(
            corners_90.len(),
            4,
            "Bei 90° Schwellwert alle 4 Ecken erwartet"
        );

        // Schwelle knapp ueber 90°: 90° < 91° → keine Ecken erkannt
        let corners_91 = detect_corners(&verts, 91_f32.to_radians());
        assert!(
            corners_91.is_empty(),
            "Bei 91° Schwellwert keine Ecken erwartet, bekam: {:?}",
            corners_91
        );
    }

    #[test]
    fn test_resample_ring_mit_verrundung_ergibt_mehr_punkte() {
        // Rechteck mit Verrundungsradius 5m: mehr Punkte als ohne
        let verts = rectangle_vertices();
        let corners = detect_corners(&verts, 80_f32.to_radians());
        let ohne = resample_ring_with_corners(&verts, &corners, 10.0, None, 15.0);
        let mit = resample_ring_with_corners(&verts, &corners, 10.0, Some(5.0), 15.0);
        assert!(
            mit.len() >= ohne.len(),
            "Mit Verrundung sollte mindestens gleich viele Punkte geben"
        );
        // RoundedCorner-Punkte muessen im Ergebnis vorhanden sein
        let rounded_count = mit
            .iter()
            .filter(|(_, k)| *k == RingNodeKind::RoundedCorner)
            .count();
        assert!(
            rounded_count > 0,
            "Mindestens ein RoundedCorner-Punkt erwartet, bekam 0"
        );
    }
}
