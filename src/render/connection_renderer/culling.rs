//! Viewport-Culling für Connection-Segmente.

use glam::Vec2;

/// Prüft ob ein Punkt innerhalb eines AABB-Rechtecks liegt (inklusiv).
pub(super) fn point_in_rect(point: Vec2, min: Vec2, max: Vec2) -> bool {
    point.x >= min.x && point.x <= max.x && point.y >= min.y && point.y <= max.y
}

/// Prüft ob ein Liniensegment ein AABB-Rechteck schneidet oder darin liegt.
pub(super) fn segment_intersects_rect(start: Vec2, end: Vec2, min: Vec2, max: Vec2) -> bool {
    if point_in_rect(start, min, max) || point_in_rect(end, min, max) {
        return true;
    }

    let bottom_left = Vec2::new(min.x, min.y);
    let bottom_right = Vec2::new(max.x, min.y);
    let top_right = Vec2::new(max.x, max.y);
    let top_left = Vec2::new(min.x, max.y);

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
    use super::{point_in_rect, segment_intersects_rect};
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

        let start = Vec2::new(-2.0, 0.0);
        let end = Vec2::new(2.0, 0.0);
        assert!(segment_intersects_rect(start, end, min, max));
    }

    #[test]
    fn test_segment_does_not_intersect_rect_when_fully_outside() {
        let min = Vec2::new(-1.0, -1.0);
        let max = Vec2::new(1.0, 1.0);

        let start = Vec2::new(2.0, 2.0);
        let end = Vec2::new(3.0, 3.0);
        assert!(!segment_intersects_rect(start, end, min, max));
    }
}
