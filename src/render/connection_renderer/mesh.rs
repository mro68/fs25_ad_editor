//! Vertex-Generierung fuer Connection-Linien und Pfeilspitzen.

use super::super::types::ConnectionVertex;
use crate::shared::{EditorOptions, RenderConnectionDirection, RenderConnectionPriority};
use glam::Vec2;

/// Bestimmt die Farbe einer Verbindung anhand von Richtung und Prioritaet.
pub(super) fn connection_color(
    direction: RenderConnectionDirection,
    priority: RenderConnectionPriority,
    options: &EditorOptions,
) -> [f32; 4] {
    let base = match direction {
        RenderConnectionDirection::Regular => options.connection_color_regular,
        RenderConnectionDirection::Dual => options.connection_color_dual,
        RenderConnectionDirection::Reverse => options.connection_color_reverse,
    };

    match priority {
        RenderConnectionPriority::Regular => base,
        RenderConnectionPriority::SubPriority => [
            (base[0] + 1.0) * 0.5,
            (base[1] + 1.0) * 0.5,
            (base[2] + 1.0) * 0.5,
            base[3],
        ],
    }
}

/// Erzeugt ein Quad (2 Dreiecke) fuer ein Liniensegment mit gegebener Breite.
pub(super) fn push_line_quad(
    vertices: &mut Vec<ConnectionVertex>,
    start: Vec2,
    end: Vec2,
    thickness: f32,
    color: [f32; 4],
) {
    let dir = (end - start).normalize();
    let perp = Vec2::new(-dir.y, dir.x) * (thickness * 0.5);

    let v0 = start + perp;
    let v1 = start - perp;
    let v2 = end + perp;
    let v3 = end - perp;

    vertices.push(ConnectionVertex::new([v0.x, v0.y], color));
    vertices.push(ConnectionVertex::new([v1.x, v1.y], color));
    vertices.push(ConnectionVertex::new([v2.x, v2.y], color));

    vertices.push(ConnectionVertex::new([v2.x, v2.y], color));
    vertices.push(ConnectionVertex::new([v1.x, v1.y], color));
    vertices.push(ConnectionVertex::new([v3.x, v3.y], color));
}

/// Erzeugt ein Dreieck als Richtungspfeil an der gegebenen Position.
///
/// Der geometrische Schwerpunkt (Zentroid) des Dreiecks liegt exakt bei `center`.
/// Fuer ein Dreieck gilt: Schwerpunkt = (Spitze + links + rechts) / 3.
/// Mit tip = center + dir * 2l/3 und base = center - dir * l/3
/// ergibt sich: (center + 2l/3 + center - l/3 + center - l/3) / 3 = center ✓
pub(super) fn push_arrow(
    vertices: &mut Vec<ConnectionVertex>,
    center: Vec2,
    direction: Vec2,
    length: f32,
    width: f32,
    color: [f32; 4],
) {
    let dir = direction.normalize();
    let perp = Vec2::new(-dir.y, dir.x);

    // Schwerpunkt-zentriert: Spitze 2/3 vor center, Basis 1/3 hinter center
    let tip = center + dir * (length * 2.0 / 3.0);
    let base = center - dir * (length / 3.0);
    let left = base + perp * (width * 0.5);
    let right = base - perp * (width * 0.5);

    vertices.push(ConnectionVertex::new([tip.x, tip.y], color));
    vertices.push(ConnectionVertex::new([left.x, left.y], color));
    vertices.push(ConnectionVertex::new([right.x, right.y], color));
}

#[cfg(test)]
mod tests {
    use super::{connection_color, push_arrow, push_line_quad};
    use crate::shared::{EditorOptions, RenderConnectionDirection, RenderConnectionPriority};
    use glam::Vec2;

    /// Maximale akzeptable Float-Abweichung fuer Vertex-Positionen.
    const F32_EPS: f32 = 1e-5;

    #[test]
    fn push_arrow_schwerpunkt_liegt_bei_center() {
        // Der geometrische Schwerpunkt der Pfeilspitze muss exakt bei `center` liegen.
        let mut vertices = Vec::new();
        let center = Vec2::new(3.0, 7.0);
        let direction = Vec2::new(1.0, 0.0);
        let color = [1.0, 0.0, 0.0, 1.0];

        push_arrow(&mut vertices, center, direction, 3.0, 2.0, color);

        assert_eq!(vertices.len(), 3, "Pfeil besteht aus genau 3 Vertices");

        let cx =
            (vertices[0].position[0] + vertices[1].position[0] + vertices[2].position[0]) / 3.0;
        let cy =
            (vertices[0].position[1] + vertices[1].position[1] + vertices[2].position[1]) / 3.0;

        assert!(
            (cx - center.x).abs() < F32_EPS,
            "Schwerpunkt-X muss bei center.x={} liegen, ist {cx}",
            center.x
        );
        assert!(
            (cy - center.y).abs() < F32_EPS,
            "Schwerpunkt-Y muss bei center.y={} liegen, ist {cy}",
            center.y
        );
    }

    #[test]
    fn push_arrow_spitze_zeigt_in_richtung_des_direction_vektors() {
        // Die Pfeilspitze (erster Vertex) liegt in Richtung des normalisierten Richtungsvektors.
        let mut vertices = Vec::new();
        let center = Vec2::ZERO;
        let direction = Vec2::new(0.0, 1.0); // nach oben
        push_arrow(&mut vertices, center, direction, 6.0, 2.0, [1.0; 4]);

        let tip = Vec2::new(vertices[0].position[0], vertices[0].position[1]);
        // Spitze = center + dir * (length * 2/3) = (0,0) + (0,1) * 4 = (0,4)
        assert!((tip.x).abs() < F32_EPS, "Spitze-X muss 0 sein");
        assert!((tip.y - 4.0).abs() < F32_EPS, "Spitze-Y muss 4.0 sein");
    }

    #[test]
    fn push_line_quad_erzeugt_sechs_vertices() {
        // Ein Liniensegment wird als 2 Dreiecke (6 Vertices) erzeugt.
        let mut vertices = Vec::new();
        push_line_quad(
            &mut vertices,
            Vec2::new(0.0, 0.0),
            Vec2::new(4.0, 0.0),
            2.0,
            [0.5, 0.5, 0.5, 1.0],
        );
        assert_eq!(vertices.len(), 6);
    }

    #[test]
    fn push_line_quad_vertices_korrekt_senkrecht_zur_linie() {
        // Fuer ein horizontales Segment von (0,0) nach (4,0) mit Dicke 2:
        // Alle Vertices muessen y=+1 oder y=-1 haben (halbe Dicke als senkrechter Abstand).
        let mut vertices = Vec::new();
        push_line_quad(
            &mut vertices,
            Vec2::new(0.0, 0.0),
            Vec2::new(4.0, 0.0),
            2.0,
            [1.0; 4],
        );
        for v in &vertices {
            assert!(
                (v.position[1].abs() - 1.0).abs() < F32_EPS,
                "Y-Abstand muss 1.0 (halbe Dicke) sein, ist {}",
                v.position[1]
            );
        }
    }

    #[test]
    fn connection_color_regular_richtung_liefert_regular_farbe() {
        // Regular-Richtung + Regular-Prioritaet muss die unveraenderte Regular-Farbe liefern.
        let opts = EditorOptions::default();
        let color = connection_color(
            RenderConnectionDirection::Regular,
            RenderConnectionPriority::Regular,
            &opts,
        );
        assert_eq!(color, opts.connection_color_regular);
    }

    #[test]
    fn connection_color_dual_richtung_liefert_dual_farbe() {
        // Dual-Richtung muss die Dual-Farbe aus den Optionen liefern.
        let opts = EditorOptions::default();
        let color = connection_color(
            RenderConnectionDirection::Dual,
            RenderConnectionPriority::Regular,
            &opts,
        );
        assert_eq!(color, opts.connection_color_dual);
    }

    #[test]
    fn connection_color_reverse_richtung_liefert_reverse_farbe() {
        let opts = EditorOptions::default();
        let color = connection_color(
            RenderConnectionDirection::Reverse,
            RenderConnectionPriority::Regular,
            &opts,
        );
        assert_eq!(color, opts.connection_color_reverse);
    }

    #[test]
    fn connection_color_subprio_hellt_farbe_auf() {
        // SubPriority hellt RGB-Kanaele auf; Alpha bleibt unveraendert.
        let opts = EditorOptions::default();
        let base = opts.connection_color_regular;
        let subprio = connection_color(
            RenderConnectionDirection::Regular,
            RenderConnectionPriority::SubPriority,
            &opts,
        );

        let expected_r = (base[0] + 1.0) * 0.5;
        let expected_g = (base[1] + 1.0) * 0.5;
        let expected_b = (base[2] + 1.0) * 0.5;

        assert!((subprio[0] - expected_r).abs() < F32_EPS, "R-Kanal");
        assert!((subprio[1] - expected_g).abs() < F32_EPS, "G-Kanal");
        assert!((subprio[2] - expected_b).abs() < F32_EPS, "B-Kanal");
        assert!((subprio[3] - base[3]).abs() < F32_EPS, "Alpha unveraendert");
    }
}
