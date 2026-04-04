//! Viewport-Culling fuer Connection-Segmente.

use glam::Vec2;

/// Prueft ob ein Punkt innerhalb eines AABB-Rechtecks liegt (inklusiv).
pub(super) fn point_in_rect(point: Vec2, min: Vec2, max: Vec2) -> bool {
    point.x >= min.x && point.x <= max.x && point.y >= min.y && point.y <= max.y
}

// Die non-cached Variante wurde entfernt zugunsten der `_cached`-Version,
// die bereits berechnete Rechtecks-Ecken erwartet und im Hot-Path verwendet wird.

/// Variante der Funktion, die bereits berechnete Rechteck-Ecken erwartet.
/// Dadurch vermeiden wir pro-Connection Allocs/Vec2-Konstruktionen in Hot-Path.
pub(super) fn segment_intersects_rect_cached(
    start: Vec2,
    end: Vec2,
    bottom_left: Vec2,
    bottom_right: Vec2,
    top_right: Vec2,
    top_left: Vec2,
) -> bool {
    if point_in_rect(start, bottom_left, top_right) || point_in_rect(end, bottom_left, top_right) {
        return true;
    }

    segments_intersect(start, end, bottom_left, bottom_right)
        || segments_intersect(start, end, bottom_right, top_right)
        || segments_intersect(start, end, top_right, top_left)
        || segments_intersect(start, end, top_left, bottom_left)
}

fn segments_intersect(a1: Vec2, a2: Vec2, b1: Vec2, b2: Vec2) -> bool {
    let o1 = orientation(a1, a2, b1);
    let o2 = orientation(a1, a2, b2);
    let o3 = orientation(b1, b2, a1);
    let o4 = orientation(b1, b2, a2);

    if o1 * o2 < 0.0 && o3 * o4 < 0.0 {
        return true;
    }

    const EPS: f32 = 1e-6;
    (o1.abs() <= EPS && point_on_segment(b1, a1, a2))
        || (o2.abs() <= EPS && point_on_segment(b2, a1, a2))
        || (o3.abs() <= EPS && point_on_segment(a1, b1, b2))
        || (o4.abs() <= EPS && point_on_segment(a2, b1, b2))
}

fn orientation(a: Vec2, b: Vec2, c: Vec2) -> f32 {
    (b.x - a.x) * (c.y - a.y) - (b.y - a.y) * (c.x - a.x)
}

fn point_on_segment(point: Vec2, seg_start: Vec2, seg_end: Vec2) -> bool {
    const EPS: f32 = 1e-6;
    point.x >= seg_start.x.min(seg_end.x) - EPS
        && point.x <= seg_start.x.max(seg_end.x) + EPS
        && point.y >= seg_start.y.min(seg_end.y) - EPS
        && point.y <= seg_start.y.max(seg_end.y) + EPS
}

#[cfg(test)]
mod tests {
    use super::{point_in_rect, segment_intersects_rect_cached};
    use glam::Vec2;

    #[test]
    fn test_point_in_rect_inclusive_edges() {
        let min = Vec2::new(-1.0, -1.0);
        let max = Vec2::new(1.0, 1.0);

        assert!(point_in_rect(Vec2::new(0.0, 0.0), min, max));
        assert!(point_in_rect(Vec2::new(1.0, 1.0), min, max));
        assert!(!point_in_rect(Vec2::new(1.1, 1.0), min, max));
    }

    #[test]
    fn test_segment_intersects_rect_when_crossing_view() {
        let min = Vec2::new(-1.0, -1.0);
        let max = Vec2::new(1.0, 1.0);

        let bottom_left = Vec2::new(min.x, min.y);
        let bottom_right = Vec2::new(max.x, min.y);
        let top_right = Vec2::new(max.x, max.y);
        let top_left = Vec2::new(min.x, max.y);

        let start = Vec2::new(-2.0, 0.0);
        let end = Vec2::new(2.0, 0.0);
        assert!(segment_intersects_rect_cached(
            start,
            end,
            bottom_left,
            bottom_right,
            top_right,
            top_left
        ));
    }

    #[test]
    fn test_segment_does_not_intersect_rect_when_fully_outside() {
        let min = Vec2::new(-1.0, -1.0);
        let max = Vec2::new(1.0, 1.0);

        let bottom_left = Vec2::new(min.x, min.y);
        let bottom_right = Vec2::new(max.x, min.y);
        let top_right = Vec2::new(max.x, max.y);
        let top_left = Vec2::new(min.x, max.y);

        let start = Vec2::new(2.0, 2.0);
        let end = Vec2::new(3.0, 3.0);
        assert!(!segment_intersects_rect_cached(
            start,
            end,
            bottom_left,
            bottom_right,
            top_right,
            top_left
        ));
    }

    /// Hilfsfunktion: Erzeugt die vier Rect-Ecken aus min/max.
    fn rect_corners(min: Vec2, max: Vec2) -> (Vec2, Vec2, Vec2, Vec2) {
        (
            Vec2::new(min.x, min.y), // bottom_left
            Vec2::new(max.x, min.y), // bottom_right
            Vec2::new(max.x, max.y), // top_right
            Vec2::new(min.x, max.y), // top_left
        )
    }

    #[test]
    fn test_punkt_exakt_auf_ecke_liegt_im_rect() {
        // Punkt auf der Ecke muss als INNERHALB gelten (inklusive Semantik).
        let min = Vec2::new(0.0, 0.0);
        let max = Vec2::new(10.0, 10.0);
        assert!(
            point_in_rect(Vec2::new(0.0, 0.0), min, max),
            "untere-linke Ecke"
        );
        assert!(
            point_in_rect(Vec2::new(10.0, 10.0), min, max),
            "obere-rechte Ecke"
        );
        assert!(
            point_in_rect(Vec2::new(10.0, 0.0), min, max),
            "untere-rechte Ecke"
        );
        assert!(
            point_in_rect(Vec2::new(0.0, 10.0), min, max),
            "obere-linke Ecke"
        );
    }

    #[test]
    fn test_segment_komplett_innerhalb_schneidet_rect() {
        // Segment komplett im Inneren muss als Schnitt gelten (beide Endpunkte drin).
        let min = Vec2::new(-5.0, -5.0);
        let max = Vec2::new(5.0, 5.0);
        let (bl, br, tr, tl) = rect_corners(min, max);
        assert!(segment_intersects_rect_cached(
            Vec2::new(-1.0, -1.0),
            Vec2::new(1.0, 1.0),
            bl,
            br,
            tr,
            tl,
        ));
    }

    #[test]
    fn test_segment_parallel_unter_rect_schneidet_nicht() {
        // Waagerechtes Segment unterhalb des Rects darf nicht als Schnitt gelten.
        let min = Vec2::new(-1.0, -1.0);
        let max = Vec2::new(1.0, 1.0);
        let (bl, br, tr, tl) = rect_corners(min, max);
        assert!(!segment_intersects_rect_cached(
            Vec2::new(-3.0, -2.0),
            Vec2::new(3.0, -2.0),
            bl,
            br,
            tr,
            tl,
        ));
    }

    #[test]
    fn test_segment_beruehrt_ecke_des_rects() {
        // Segment von außen, das exakt durch die untere-linke Ecke laeuft, zaehlt als Schnitt.
        let min = Vec2::new(-1.0, -1.0);
        let max = Vec2::new(1.0, 1.0);
        let (bl, br, tr, tl) = rect_corners(min, max);

        // Segment von (-2, 0) nach (0, -2) laeuft durch Punkt (-1, -1) = untere-linke Ecke
        assert!(segment_intersects_rect_cached(
            Vec2::new(-2.0, 0.0),
            Vec2::new(0.0, -2.0),
            bl,
            br,
            tr,
            tl,
        ));
    }

    #[test]
    fn test_segment_entlang_kante_gilt_als_schnitt() {
        // Segment kollinear auf einer Kante des Rects muss als Schnitt gelten.
        let min = Vec2::new(-1.0, -1.0);
        let max = Vec2::new(1.0, 1.0);
        let (bl, br, tr, tl) = rect_corners(min, max);

        // Entlang der unteren Kante (y = -1)
        assert!(segment_intersects_rect_cached(
            Vec2::new(-2.0, -1.0),
            Vec2::new(2.0, -1.0),
            bl,
            br,
            tr,
            tl,
        ));
    }
}
