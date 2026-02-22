use super::super::{RouteTool, ToolAction, ToolAnchor};
use super::geometry::{compute_curve_positions, cubic_bezier, quadratic_bezier};
use super::state::{CurveDegree, CurveTool, Phase};
use crate::core::RoadMap;
use glam::Vec2;

// ── Quadratische Bézier ──

#[test]
fn test_quadratic_bezier_endpoints() {
    let p0 = Vec2::new(0.0, 0.0);
    let p1 = Vec2::new(5.0, 10.0);
    let p2 = Vec2::new(10.0, 0.0);

    let start = quadratic_bezier(p0, p1, p2, 0.0);
    let end = quadratic_bezier(p0, p1, p2, 1.0);
    let mid = quadratic_bezier(p0, p1, p2, 0.5);

    assert!((start - p0).length() < 0.001);
    assert!((end - p2).length() < 0.001);
    assert!((mid - Vec2::new(5.0, 5.0)).length() < 0.001);
}

#[test]
fn test_quadratic_curve_positions_count() {
    let start = Vec2::new(0.0, 0.0);
    let control = Vec2::new(5.0, 10.0);
    let end = Vec2::new(10.0, 0.0);

    let positions = compute_curve_positions(|t| quadratic_bezier(start, control, end, t), 2.0);
    assert!(positions.len() >= 3);
    assert!((positions[0] - start).length() < 0.01);
    assert!((*positions.last().unwrap() - end).length() < 0.01);
}

#[test]
fn test_quadratic_curve_spacing() {
    let start = Vec2::new(0.0, 0.0);
    let control = Vec2::new(5.0, 10.0);
    let end = Vec2::new(10.0, 0.0);

    let positions = compute_curve_positions(|t| quadratic_bezier(start, control, end, t), 2.0);
    for i in 0..positions.len() - 1 {
        let dist = positions[i].distance(positions[i + 1]);
        assert!(dist < 2.5, "Segment {} hat Abstand {:.2}m", i, dist);
    }
}

// ── Kubische Bézier ──

#[test]
fn test_cubic_bezier_endpoints() {
    let p0 = Vec2::new(0.0, 0.0);
    let p1 = Vec2::new(3.0, 10.0);
    let p2 = Vec2::new(7.0, 10.0);
    let p3 = Vec2::new(10.0, 0.0);

    let start = cubic_bezier(p0, p1, p2, p3, 0.0);
    let end = cubic_bezier(p0, p1, p2, p3, 1.0);

    assert!((start - p0).length() < 0.001);
    assert!((end - p3).length() < 0.001);
}

#[test]
fn test_cubic_bezier_symmetry() {
    // Symmetrische S-Kurve → Mittelpunkt bei (5, 5)
    let p0 = Vec2::new(0.0, 0.0);
    let p1 = Vec2::new(0.0, 10.0);
    let p2 = Vec2::new(10.0, 0.0);
    let p3 = Vec2::new(10.0, 10.0);

    let mid = cubic_bezier(p0, p1, p2, p3, 0.5);
    // B(0.5) = 0.125*P0 + 0.375*P1 + 0.375*P2 + 0.125*P3 = (5, 5)
    assert!((mid - Vec2::new(5.0, 5.0)).length() < 0.001);
}

#[test]
fn test_cubic_curve_positions_count() {
    let start = Vec2::new(0.0, 0.0);
    let cp1 = Vec2::new(3.0, 10.0);
    let cp2 = Vec2::new(7.0, 10.0);
    let end = Vec2::new(10.0, 0.0);

    let positions = compute_curve_positions(|t| cubic_bezier(start, cp1, cp2, end, t), 2.0);
    assert!(positions.len() >= 3);
    assert!((positions[0] - start).length() < 0.01);
    assert!((*positions.last().unwrap() - end).length() < 0.01);
}

// ── Tool-Flow quadratisch ──

#[test]
fn test_tool_quadratic_click_flow() {
    let mut tool = CurveTool::new();
    tool.degree = CurveDegree::Quadratic;
    let road_map = RoadMap::new(3);

    assert!(!tool.is_ready());
    assert_eq!(tool.status_text(), "Startpunkt klicken");

    let action = tool.on_click(Vec2::ZERO, &road_map, false);
    assert_eq!(action, ToolAction::Continue);

    let action = tool.on_click(Vec2::new(10.0, 0.0), &road_map, false);
    assert_eq!(action, ToolAction::Continue);
    assert!(tool.status_text().contains("Steuerpunkt"));

    let action = tool.on_click(Vec2::new(5.0, 8.0), &road_map, false);
    assert_eq!(action, ToolAction::UpdatePreview);
    assert!(tool.is_ready());

    // Erneuter Klick ignoriert (Drag übernimmt)
    let action = tool.on_click(Vec2::new(5.0, 12.0), &road_map, false);
    assert_eq!(action, ToolAction::UpdatePreview);
    // CP1 bleibt beim ersten Wert
    assert_eq!(tool.control_point1, Some(Vec2::new(5.0, 8.0)));
}

// ── Tool-Flow kubisch ──

#[test]
fn test_tool_cubic_click_flow() {
    let mut tool = CurveTool::new();
    tool.degree = CurveDegree::Cubic;
    let road_map = RoadMap::new(3);

    // Start
    tool.on_click(Vec2::ZERO, &road_map, false);
    // End
    tool.on_click(Vec2::new(10.0, 0.0), &road_map, false);
    // CP1 per normalem Klick
    let action = tool.on_click(Vec2::new(3.0, 8.0), &road_map, false);
    assert_eq!(action, ToolAction::UpdatePreview);
    assert!(!tool.is_ready()); // CP2 fehlt noch

    // CP2 per zweitem Klick (kein Ctrl nötig)
    let action = tool.on_click(Vec2::new(7.0, 8.0), &road_map, false);
    assert_eq!(action, ToolAction::UpdatePreview);
    assert!(tool.is_ready());
    assert_eq!(tool.control_point1, Some(Vec2::new(3.0, 8.0)));
    assert_eq!(tool.control_point2, Some(Vec2::new(7.0, 8.0)));
}

#[test]
fn test_tool_cubic_drag_repositions() {
    let mut tool = CurveTool::new();
    tool.degree = CurveDegree::Cubic;
    let road_map = RoadMap::new(3);

    tool.on_click(Vec2::ZERO, &road_map, false);
    tool.on_click(Vec2::new(10.0, 0.0), &road_map, false);
    tool.on_click(Vec2::new(3.0, 8.0), &road_map, false); // CP1
    tool.on_click(Vec2::new(7.0, 8.0), &road_map, false); // CP2
    assert!(tool.is_ready());

    // Drag-Targets sind verfügbar
    let targets = tool.drag_targets();
    assert_eq!(targets.len(), 4); // Start, End, CP1, CP2

    // CP1 per Drag verschieben
    let grabbed = tool.on_drag_start(Vec2::new(3.0, 8.0), &road_map, 2.0);
    assert!(grabbed);
    tool.on_drag_update(Vec2::new(2.0, 6.0));
    assert_eq!(tool.control_point1, Some(Vec2::new(2.0, 6.0)));
    tool.on_drag_end(&road_map);

    // CP2 per Drag verschieben
    let grabbed = tool.on_drag_start(Vec2::new(7.0, 8.0), &road_map, 2.0);
    assert!(grabbed);
    tool.on_drag_update(Vec2::new(8.0, 6.0));
    assert_eq!(tool.control_point2, Some(Vec2::new(8.0, 6.0)));
    tool.on_drag_end(&road_map);
}

#[test]
fn test_tool_cubic_execute() {
    let mut tool = CurveTool::new();
    tool.degree = CurveDegree::Cubic;
    tool.seg.max_segment_length = 2.0;
    let road_map = RoadMap::new(3);

    tool.on_click(Vec2::ZERO, &road_map, false);
    tool.on_click(Vec2::new(10.0, 0.0), &road_map, false);
    tool.on_click(Vec2::new(3.0, 8.0), &road_map, false);
    tool.on_click(Vec2::new(7.0, 8.0), &road_map, false);

    let result = tool.execute(&road_map).expect("Ergebnis erwartet");
    assert!(result.new_nodes.len() >= 3);
    assert_eq!(
        result.internal_connections.len(),
        result.new_nodes.len() - 1,
    );
}

#[test]
fn test_tool_execute_quadratic() {
    let mut tool = CurveTool::new();
    tool.seg.max_segment_length = 2.0;
    let road_map = RoadMap::new(3);

    tool.on_click(Vec2::ZERO, &road_map, false);
    tool.on_click(Vec2::new(10.0, 0.0), &road_map, false);
    tool.on_click(Vec2::new(5.0, 8.0), &road_map, false);

    let result = tool.execute(&road_map).expect("Ergebnis erwartet");
    assert!(result.new_nodes.len() >= 3);
}

#[test]
fn test_tool_reset() {
    let mut tool = CurveTool::new();
    tool.degree = CurveDegree::Cubic;
    let road_map = RoadMap::new(3);

    tool.on_click(Vec2::ZERO, &road_map, false);
    tool.on_click(Vec2::new(10.0, 0.0), &road_map, false);
    tool.on_click(Vec2::new(3.0, 8.0), &road_map, false);
    tool.on_click(Vec2::new(7.0, 8.0), &road_map, false);
    assert!(tool.is_ready());

    tool.reset();
    assert!(!tool.is_ready());
    assert_eq!(tool.phase, Phase::Start);
    assert!(tool.control_point1.is_none());
    assert!(tool.control_point2.is_none());
}

#[test]
fn test_chaining_uses_last_end_as_start() {
    let mut tool = CurveTool::new();
    let road_map = RoadMap::new(3);

    tool.on_click(Vec2::ZERO, &road_map, false);
    tool.on_click(Vec2::new(10.0, 0.0), &road_map, false);
    tool.on_click(Vec2::new(5.0, 8.0), &road_map, false);
    tool.set_last_created(vec![100, 101, 102], &road_map);
    tool.reset();

    let action = tool.on_click(Vec2::new(20.0, 0.0), &road_map, false);
    assert_eq!(action, ToolAction::Continue);
    assert!(tool.start.is_some());
    assert!(tool.end.is_some());
    assert_eq!(tool.phase, Phase::Control);
}

#[test]
fn test_execute_from_anchors() {
    let mut tool = CurveTool::new();
    tool.seg.max_segment_length = 2.0;
    let road_map = RoadMap::new(3);

    tool.on_click(Vec2::ZERO, &road_map, false);
    tool.on_click(Vec2::new(10.0, 0.0), &road_map, false);
    tool.on_click(Vec2::new(5.0, 8.0), &road_map, false);
    let original = tool.execute(&road_map).unwrap();
    tool.set_last_created(vec![1, 2, 3, 4, 5], &road_map);
    tool.reset();

    tool.seg.max_segment_length = 5.0;
    let result = tool
        .execute_from_anchors(&road_map)
        .expect("Ergebnis erwartet");
    assert!(result.new_nodes.len() < original.new_nodes.len());
}

#[test]
fn test_approx_length_straight_line() {
    let length = CurveTool::approx_length(|t| Vec2::new(t * 10.0, 0.0), 128);
    assert!((length - 10.0).abs() < 0.1);
}

#[test]
fn test_straight_control_point_gives_straight_line() {
    let start = Vec2::ZERO;
    let end = Vec2::new(10.0, 0.0);
    let control = Vec2::new(5.0, 0.0);

    let positions = compute_curve_positions(|t| quadratic_bezier(start, control, end, t), 2.0);
    for (i, pos) in positions.iter().enumerate() {
        assert!(
            pos.y.abs() < 0.01,
            "Node {} hat y={:.3}, erwartet 0",
            i,
            pos.y
        );
    }
}

// ── Drag-Tests ──

#[test]
fn test_drag_targets_empty_before_controls_complete() {
    let mut tool = CurveTool::new();
    tool.degree = CurveDegree::Quadratic;
    let road_map = RoadMap::new(3);

    assert!(tool.drag_targets().is_empty());
    tool.on_click(Vec2::ZERO, &road_map, false);
    assert!(tool.drag_targets().is_empty());
    tool.on_click(Vec2::new(10.0, 0.0), &road_map, false);
    assert!(tool.drag_targets().is_empty());
}

#[test]
fn test_drag_targets_available_after_controls_complete() {
    let mut tool = CurveTool::new();
    tool.degree = CurveDegree::Quadratic;
    let road_map = RoadMap::new(3);

    tool.on_click(Vec2::ZERO, &road_map, false);
    tool.on_click(Vec2::new(10.0, 0.0), &road_map, false);
    tool.on_click(Vec2::new(5.0, 8.0), &road_map, false);

    let targets = tool.drag_targets();
    assert_eq!(targets.len(), 3); // Start, End, CP1
}

#[test]
fn test_drag_start_returns_false_outside_radius() {
    let mut tool = CurveTool::new();
    let road_map = RoadMap::new(3);

    tool.on_click(Vec2::ZERO, &road_map, false);
    tool.on_click(Vec2::new(10.0, 0.0), &road_map, false);
    tool.on_click(Vec2::new(5.0, 8.0), &road_map, false);

    // Weit weg von allen Punkten
    let grabbed = tool.on_drag_start(Vec2::new(50.0, 50.0), &road_map, 2.0);
    assert!(!grabbed);
    assert!(tool.dragging.is_none());
}

#[test]
fn test_drag_start_end_resnap() {
    let mut tool = CurveTool::new();
    let road_map = RoadMap::new(3);

    tool.on_click(Vec2::ZERO, &road_map, false);
    tool.on_click(Vec2::new(10.0, 0.0), &road_map, false);
    tool.on_click(Vec2::new(5.0, 8.0), &road_map, false);

    // Start draggen
    let grabbed = tool.on_drag_start(Vec2::new(0.0, 0.0), &road_map, 2.0);
    assert!(grabbed);
    tool.on_drag_update(Vec2::new(1.0, 1.0));
    // Während Drag: NewPosition
    match &tool.start {
        Some(ToolAnchor::NewPosition(pos)) => {
            assert!((pos.x - 1.0).abs() < 0.01);
        }
        _ => panic!("Start sollte NewPosition sein während Drag"),
    }
    tool.on_drag_end(&road_map);
    // Nach Drag: Re-Snap (kein Node in der Nähe → bleibt NewPosition)
    assert!(tool.dragging.is_none());
}

#[test]
fn test_drag_quadratic_cp1() {
    let mut tool = CurveTool::new();
    tool.degree = CurveDegree::Quadratic;
    let road_map = RoadMap::new(3);

    tool.on_click(Vec2::ZERO, &road_map, false);
    tool.on_click(Vec2::new(10.0, 0.0), &road_map, false);
    tool.on_click(Vec2::new(5.0, 8.0), &road_map, false);

    // CP1 draggen
    let grabbed = tool.on_drag_start(Vec2::new(5.0, 8.0), &road_map, 2.0);
    assert!(grabbed);
    tool.on_drag_update(Vec2::new(3.0, 12.0));
    assert_eq!(tool.control_point1, Some(Vec2::new(3.0, 12.0)));
    tool.on_drag_end(&road_map);
    assert!(tool.dragging.is_none());
    assert_eq!(tool.control_point1, Some(Vec2::new(3.0, 12.0)));
}
