//! Feldgrenz-Polygone in Weltkoordinaten.
//!
//! Enthalten geordnete Umriss-Vertices eines Farmland-Feldes, umgerechnet
//! aus den GRLE-Pixel-Koordinaten in das Weltkoordinatensystem des Editors.
//!
//! Stellt ausserdem Core-Geometrie-Algorithmen bereit:
//! - Point-in-Polygon (Ray-Casting)
//! - Douglas-Peucker-Vereinfachung fuer geschlossene Ringe
//! - Normalen-basiertes Polygon-Offset

use glam::Vec2;
use serde::{Deserialize, Serialize};

/// Ein geordnetes Feldgrenz-Polygon in Weltkoordinaten (x/z-Ebene).
///
/// Die Vertices stammen aus dem GRLE-Farmland-Raster und wurden
/// per `world = pixel * (map_size / grle_width) - map_size / 2`
/// in Weltkoordinaten umgerechnet.
#[derive(Clone, Serialize, Deserialize)]
pub struct FieldPolygon {
    /// Farmland-ID (1–255, 0 = kein Feld)
    pub id: u32,
    /// Geordnete Rand-Vertices in Weltkoordinaten (x, z)
    pub vertices: Vec<Vec2>,
}

/// Prueft ob ein Punkt innerhalb eines Polygons liegt (Ray-Casting-Algorithmus).
///
/// Schiesst einen Strahl vom Punkt nach rechts und zaehlt Schnittpunkte
/// mit den Kanten. Eine ungerade Anzahl bedeutet: Punkt liegt innen.
/// Weniger als 3 Vertices → immer `false`.
pub fn point_in_polygon(point: Vec2, polygon: &[Vec2]) -> bool {
    let n = polygon.len();
    if n < 3 {
        return false;
    }
    let (px, py) = (point.x, point.y);
    let mut inside = false;
    let mut j = n - 1;
    for i in 0..n {
        let (xi, yi) = (polygon[i].x, polygon[i].y);
        let (xj, yj) = (polygon[j].x, polygon[j].y);
        if ((yi > py) != (yj > py)) && (px < (xj - xi) * (py - yi) / (yj - yi) + xi) {
            inside = !inside;
        }
        j = i;
    }
    inside
}

/// Findet das erste `FieldPolygon`, das den gegebenen Weltkoordinaten-Punkt enthaelt.
///
/// Gibt `None` zurueck wenn kein Polygon den Punkt enthaelt.
pub fn find_polygon_at(point: Vec2, polygons: &[FieldPolygon]) -> Option<&FieldPolygon> {
    polygons
        .iter()
        .find(|fp| point_in_polygon(point, &fp.vertices))
}

// ---------------------------------------------------------------------------
// Interne Hilfsfunktionen fuer Geometrie-Algorithmen
// ---------------------------------------------------------------------------

/// Senkrechter Abstand eines Punktes von einer Geraden (start→end).
fn perpendicular_distance(pt: Vec2, line_start: Vec2, line_end: Vec2) -> f32 {
    let delta = line_end - line_start;
    let len = delta.length();
    if len < f32::EPSILON {
        return pt.distance(line_start);
    }
    let d = delta.x * (line_start.y - pt.y) - (line_start.x - pt.x) * delta.y;
    d.abs() / len
}

/// Douglas-Peucker fuer offene Polylinien (internals, kein pub).
///
/// Gibt eine vereinfachte Teilmenge der Eingabepunkte zurueck, beginnend
/// mit dem ersten und endend mit dem letzten Punkt.
fn dp_open(points: &[Vec2], tolerance: f32) -> Vec<Vec2> {
    if points.len() <= 2 {
        return points.to_vec();
    }
    let first = points[0];
    let last = *points
        .last()
        .expect("dp_open: Eingabe-Slice muss mindestens einen Punkt enthalten");

    // Finde den Punkt mit maximalem senkrechten Abstand von der Geraden first→last
    let (max_idx, max_dist) = points[1..points.len() - 1]
        .iter()
        .enumerate()
        .map(|(i, &p)| (i + 1, perpendicular_distance(p, first, last)))
        .fold(
            (0, 0.0_f32),
            |(mi, md), (i, d)| {
                if d > md {
                    (i, d)
                } else {
                    (mi, md)
                }
            },
        );

    if max_dist > tolerance {
        // Teile am Maximum und vereinfache rekursiv
        let mut left = dp_open(&points[..=max_idx], tolerance);
        let right = dp_open(&points[max_idx..], tolerance);
        left.pop(); // Duplikat des Teilungspunktes entfernen
        left.extend(right);
        left
    } else {
        // Alle Zwischenpunkte liegen innerhalb der Toleranz
        vec![first, last]
    }
}

/// Berechnet die vorzeichenbehaftete Flaeche eines Polygons (Shoelace-Formel).
///
/// Positiv = gegen den Uhrzeigersinn (CCW), negativ = im Uhrzeigersinn (CW).
fn polygon_area_signed(vertices: &[Vec2]) -> f32 {
    let n = vertices.len();
    let mut area = 0.0_f32;
    for i in 0..n {
        let j = (i + 1) % n;
        area += vertices[i].x * vertices[j].y;
        area -= vertices[j].x * vertices[i].y;
    }
    area / 2.0
}

// ---------------------------------------------------------------------------
// Oeffentliche Geometrie-Algorithmen
// ---------------------------------------------------------------------------

/// Vereinfacht ein geschlossenes Polygon durch Entfernen von Punkten, die
/// weniger als `tolerance` von der Verbindungslinie abweichen
/// (Douglas-Peucker-Algorithmus fuer geschlossene Ringe).
///
/// - `tolerance = 0.0` → keine Vereinfachung, Original wird zurueckgegeben.
/// - Mindestens 3 Punkte werden immer behalten. Wenn die Vereinfachung zu
///   weniger als 3 Punkten fuehren wuerde, wird das Original zurueckgegeben.
pub fn simplify_polygon(vertices: &[Vec2], tolerance: f32) -> Vec<Vec2> {
    if vertices.len() < 3 {
        return vertices.to_vec();
    }
    if tolerance <= 0.0 {
        return vertices.to_vec();
    }

    let n = vertices.len();

    // Teilungspunkt: Vertex mit groesstem quadratischen Abstand von vertex[0]
    // Teilt den Ring in zwei offene Haelften auf
    let split_idx = (1..n)
        .max_by(|&a, &b| {
            vertices[a]
                .distance_squared(vertices[0])
                .partial_cmp(&vertices[b].distance_squared(vertices[0]))
                .unwrap_or(std::cmp::Ordering::Equal)
        })
        .unwrap_or(n / 2);

    // Erste Haelfte: vertex[0] → vertex[split_idx] (inklusive)
    let arc1 = &vertices[..=split_idx];
    // Zweite Haelfte: vertex[split_idx] → vertex[n-1] → vertex[0]
    let mut arc2: Vec<Vec2> = vertices[split_idx..].to_vec();
    arc2.push(vertices[0]);

    let simplified1 = dp_open(arc1, tolerance);
    let simplified2 = dp_open(&arc2, tolerance);

    // Kombiniere: letzter Punkt von simplified1 = split_idx-Duplikat → entfernen;
    // letzter Punkt von simplified2 = vertex[0]-Duplikat → ebenfalls entfernen
    let mut result = simplified1;
    result.pop();
    let keep = simplified2.len().saturating_sub(1);
    result.extend_from_slice(&simplified2[..keep]);

    // Sicherheits-Fallback: mindestens 3 Punkte behalten
    if result.len() < 3 {
        return vertices.to_vec();
    }

    result
}

/// Verschiebt ein Polygon um `offset` Meter nach innen (negativ) oder aussen (positiv).
///
/// Nutzt Normalen-basiertes Vertex-Offset: jeder Vertex wird entlang des
/// gemittelten Aussenormalenvektors der anliegenden Kanten verschoben.
///
/// Fuer CCW-Polygone (positive Flaeche) zeigen die rechten Kantennormalen
/// nach aussen; fuer CW-Polygone entsprechend die linken.
///
/// **Fallback:** Wenn das Ergebnis degeneriert ist (weniger als 3 Punkte,
/// Flaeche = 0 oder Orientierungswechsel), wird das Original zurueckgegeben.
pub fn offset_polygon(vertices: &[Vec2], offset: f32) -> Vec<Vec2> {
    let n = vertices.len();
    if n < 3 {
        return vertices.to_vec();
    }
    if offset == 0.0 {
        return vertices.to_vec();
    }

    let signed_area = polygon_area_signed(vertices);
    // CCW (positive Flaeche): rechte Kantennormale = Aussenormale → Vec2(edge.y, -edge.x)
    // CW (negative Flaeche): linke Kantennormale = Aussenormale → Vec2(-edge.y, edge.x)
    // Vereinheitlicht: Vorzeichen der Flaeche bestimmt die Richtung
    let orientation_sign = if signed_area >= 0.0 {
        1.0_f32
    } else {
        -1.0_f32
    };

    // Normierte Aussenormale fuer jede Kante
    let edge_normals: Vec<Vec2> = (0..n)
        .map(|i| {
            let j = (i + 1) % n;
            let edge = vertices[j] - vertices[i];
            let len = edge.length();
            if len < f32::EPSILON {
                Vec2::ZERO
            } else {
                Vec2::new(edge.y, -edge.x) * orientation_sign / len
            }
        })
        .collect();

    // Verschiebe jeden Vertex entlang der gemittelten Normalen der anliegenden Kanten
    let result: Vec<Vec2> = (0..n)
        .map(|i| {
            let prev = (i + n - 1) % n;
            let avg_normal = (edge_normals[prev] + edge_normals[i]).normalize_or_zero();
            vertices[i] + avg_normal * offset
        })
        .collect();

    // Degenerations-Fallback: zu wenige Punkte, Flaeche null, Orientierungswechsel
    // oder Miter-Overshoot (Shrink-Offset macht das Polygon groesser)
    if result.len() < 3 {
        return vertices.to_vec();
    }
    let result_area_signed = polygon_area_signed(&result);
    let orig_area = signed_area.abs();
    let res_area = result_area_signed.abs();
    let orientation_flipped =
        result_area_signed == 0.0 || result_area_signed.signum() != signed_area.signum();
    let overshoot =
        (offset < 0.0 && res_area > orig_area) || (offset > 0.0 && res_area < orig_area);
    if orientation_flipped || overshoot {
        return vertices.to_vec();
    }

    result
}

// ---------------------------------------------------------------------------
// Unit-Tests
// ---------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;
    use glam::Vec2;

    /// Erstellt ein achsenparalleles Quadrat (CCW, 4×4) fuer Tests.
    fn square_ccw() -> Vec<Vec2> {
        vec![
            Vec2::new(0.0, 0.0),
            Vec2::new(4.0, 0.0),
            Vec2::new(4.0, 4.0),
            Vec2::new(0.0, 4.0),
        ]
    }

    // --- point_in_polygon ---

    #[test]
    fn test_pip_rechteck_innen() {
        let poly = square_ccw();
        assert!(point_in_polygon(Vec2::new(2.0, 2.0), &poly));
    }

    #[test]
    fn test_pip_rechteck_aussen() {
        let poly = square_ccw();
        assert!(!point_in_polygon(Vec2::new(5.0, 5.0), &poly));
        assert!(!point_in_polygon(Vec2::new(-1.0, 2.0), &poly));
    }

    #[test]
    fn test_pip_rechteck_nahe_kante_innen() {
        let poly = square_ccw();
        // Knapp innerhalb der rechten Kante
        assert!(point_in_polygon(Vec2::new(3.9, 2.0), &poly));
        // Klar ausserhalb
        assert!(!point_in_polygon(Vec2::new(4.1, 2.0), &poly));
    }

    #[test]
    fn test_pip_l_form() {
        // L-foermiges Polygon (CCW)
        let l_poly = vec![
            Vec2::new(0.0, 0.0),
            Vec2::new(2.0, 0.0),
            Vec2::new(2.0, 1.0),
            Vec2::new(1.0, 1.0),
            Vec2::new(1.0, 2.0),
            Vec2::new(0.0, 2.0),
        ];
        // Unterer Bereich des L → innen
        assert!(point_in_polygon(Vec2::new(0.5, 0.5), &l_poly));
        // Linker oberer Bereich des L → innen
        assert!(point_in_polygon(Vec2::new(0.5, 1.5), &l_poly));
        // Fehlendes Eck oben rechts → aussen
        assert!(!point_in_polygon(Vec2::new(1.5, 1.5), &l_poly));
    }

    // --- simplify_polygon ---

    #[test]
    fn test_simplify_tolerance_null_keine_aenderung() {
        let poly = square_ccw();
        let result = simplify_polygon(&poly, 0.0);
        assert_eq!(result.len(), poly.len());
        for (a, b) in result.iter().zip(poly.iter()) {
            assert!((a.x - b.x).abs() < f32::EPSILON);
            assert!((a.y - b.y).abs() < f32::EPSILON);
        }
    }

    #[test]
    fn test_simplify_entfernt_fast_kollinearen_punkt() {
        // Pentagon mit einem Punkt nahe der Verbindungslinie
        let poly = vec![
            Vec2::new(0.0, 0.0),
            Vec2::new(5.0, 0.001), // Abstand 0.001 von der Linie (0,0)→(10,0)
            Vec2::new(10.0, 0.0),
            Vec2::new(10.0, 10.0),
            Vec2::new(0.0, 10.0),
        ];
        // tolerance 0.01 > 0.001 → Mittelpunkt wird entfernt
        let result = simplify_polygon(&poly, 0.01);
        assert!(
            result.len() < poly.len(),
            "Kollinearer Punkt sollte entfernt werden"
        );
        assert!(result.len() >= 3, "Mindestens 3 Punkte erwartet");
    }

    #[test]
    fn test_simplify_behaelt_punkte_mit_grosser_abweichung() {
        // Pentagon mit einem Punkt weit von der Verbindungslinie
        let poly = vec![
            Vec2::new(0.0, 0.0),
            Vec2::new(5.0, 3.0), // Grosse Abweichung → bleibt
            Vec2::new(10.0, 0.0),
            Vec2::new(10.0, 10.0),
            Vec2::new(0.0, 10.0),
        ];
        let result = simplify_polygon(&poly, 0.01);
        assert_eq!(
            result.len(),
            poly.len(),
            "Kein Punkt sollte bei kleiner Toleranz entfernt werden"
        );
    }

    // --- offset_polygon ---

    #[test]
    fn test_offset_null_keine_aenderung() {
        let poly = square_ccw();
        let result = offset_polygon(&poly, 0.0);
        assert_eq!(result, poly);
    }

    #[test]
    fn test_offset_positiv_vergroessert_flaeche() {
        let poly = square_ccw();
        let original_area = polygon_area_signed(&poly).abs();
        let result = offset_polygon(&poly, 0.5);
        let result_area = polygon_area_signed(&result).abs();
        assert_eq!(result.len(), poly.len());
        assert!(
            result_area > original_area,
            "Positiver Offset muss Flaeche vergroessern: {result_area} > {original_area}"
        );
    }

    #[test]
    fn test_offset_negativ_verkleinert_flaeche() {
        let poly = square_ccw();
        let original_area = polygon_area_signed(&poly).abs();
        let result = offset_polygon(&poly, -0.5);
        let result_area = polygon_area_signed(&result).abs();
        assert_eq!(result.len(), poly.len());
        assert!(
            result_area < original_area,
            "Negativer Offset muss Flaeche verkleinern: {result_area} < {original_area}"
        );
    }

    #[test]
    fn test_offset_fallback_bei_degeneration() {
        // Rechtwinkliges Dreieck – bei sehr grossem negativem Offset
        // kollabiert die Orientierung (Vorzeichen der Flaeche flippt) → Fallback
        let tri = vec![
            Vec2::new(0.0, 0.0),
            Vec2::new(2.0, 0.0),
            Vec2::new(0.0, 2.0),
        ];
        let result = offset_polygon(&tri, -100.0);
        assert_eq!(
            result, tri,
            "Fallback zum Original bei degeneriertem Offset erwartet"
        );
    }
}
