use super::super::common::{angle_to_compass, TangentSource};
use super::super::{RouteTool, ToolAction, ToolAnchor};
use super::geometry::{
    compute_curve_positions, compute_tangent_cp, cubic_bezier, project_onto_tangent_line,
    quadratic_bezier, solve_cps_from_apex_both_tangents,
};
use super::state::{CurveDegree, CurveTool, Phase};
use crate::core::{ConnectedNeighbor, RoadMap};
use glam::Vec2;

// ── angle_to_compass ──

#[test]
fn test_angle_to_compass_cardinal() {
    // Ost (0°)
    assert_eq!(angle_to_compass(0.0_f32.to_radians()), "O");
    // Süd (90° in FS25-Koordinatensystem)
    assert_eq!(angle_to_compass(90.0_f32.to_radians()), "S");
    // West (180°)
    assert_eq!(angle_to_compass(180.0_f32.to_radians()), "W");
    // Nord (270°)
    assert_eq!(angle_to_compass(270.0_f32.to_radians()), "N");
}

#[test]
fn test_angle_to_compass_negative_wraps() {
    // -90° = 270° → Nord
    assert_eq!(angle_to_compass(-90.0_f32.to_radians()), "N");
}

// ── compute_tangent_cp ──

#[test]
fn test_compute_tangent_cp_start() {
    // Tangente zeigt nach Osten (0°), CP1 soll nach Westen (weg vom Nachbar)
    let anchor = Vec2::new(0.0, 0.0);
    let other = Vec2::new(10.0, 0.0);
    let cp = compute_tangent_cp(anchor, 0.0, other, true);
    // chord_length = 10, cp_distance = 10/3 ≈ 3.33
    // Richtung = angle + PI = 180° → (-1, 0)
    assert!(
        cp.x < 0.0,
        "CP1 sollte links vom Startpunkt liegen, war: {:?}",
        cp
    );
    assert!((cp.y).abs() < 0.01, "CP1 sollte auf der x-Achse liegen");
    assert!((cp.x + 10.0 / 3.0).abs() < 0.01);
}

#[test]
fn test_compute_tangent_cp_end() {
    // Tangente zeigt nach Osten (0°), CP2 soll nach Osten (Richtung Nachbar)
    let anchor = Vec2::new(10.0, 0.0);
    let other = Vec2::new(0.0, 0.0);
    let cp = compute_tangent_cp(anchor, 0.0, other, false);
    // Richtung = angle direkt = 0° → (+1, 0)
    assert!(
        cp.x > 10.0,
        "CP2 sollte rechts vom Endpunkt liegen, war: {:?}",
        cp
    );
    assert!((cp.y).abs() < 0.01, "CP2 sollte auf der x-Achse liegen");
}

// ── apply_tangent_to_cp ──

#[test]
fn test_apply_tangent_to_cp_cubic() {
    let mut tool = CurveTool::new();
    tool.degree = CurveDegree::Cubic;
    let road_map = RoadMap::new(3);
    tool.on_click(Vec2::ZERO, &road_map, false);
    tool.on_click(Vec2::new(12.0, 0.0), &road_map, false);

    // Tangente nach Osten (0°) am Startpunkt
    tool.tangents.tangent_start = TangentSource::Connection {
        neighbor_id: 99,
        angle: 0.0,
    };
    // Tangente nach Westen (PI) am Endpunkt
    tool.tangents.tangent_end = TangentSource::Connection {
        neighbor_id: 98,
        angle: std::f32::consts::PI,
    };

    tool.apply_tangent_to_cp();

    // CP1 soll in negativer x-Richtung vom Start-Anker
    let cp1 = tool.control_point1.expect("CP1 sollte gesetzt sein");
    assert!(cp1.x < 0.0, "CP1 sollte links liegen, war: {:?}", cp1);

    // CP2 soll in positiver x-Richtung vom End-Anker (Winkel=PI → Richtung (-1,0), also links)
    let cp2 = tool.control_point2.expect("CP2 sollte gesetzt sein");
    // tangent_end angle=PI → direction = Vec2::from_angle(PI) = (-1, 0) → cp2.x = 12 - 4 = 8
    assert!(
        cp2.x < 12.0,
        "CP2 sollte links vom Endpunkt liegen, war: {:?}",
        cp2
    );
}

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

    assert!(!tool.is_ready());
    assert_eq!(tool.status_text(), "Startpunkt klicken");

    let action = tool.on_click(Vec2::ZERO, &road_map, false);
    assert_eq!(action, ToolAction::Continue);

    // Nach End-Klick: CP2 wird automatisch initialisiert (set_default_cp2_if_missing)
    let action = tool.on_click(Vec2::new(10.0, 0.0), &road_map, false);
    assert_eq!(action, ToolAction::Continue);
    assert!(
        tool.control_point2.is_some(),
        "CP2 sollte nach End-Klick auto-gesetzt sein"
    );
    assert!(!tool.is_ready(), "is_ready() ohne CP1 noch false");

    // CP1 per Klick setzen → jetzt sind beide CPs gesetzt
    let action = tool.on_click(Vec2::new(3.0, 8.0), &road_map, false);
    assert_eq!(action, ToolAction::UpdatePreview);
    assert!(
        tool.is_ready(),
        "Nach CP1-Klick und Auto-CP2 soll is_ready() true sein"
    );
    assert_eq!(tool.control_point1, Some(Vec2::new(3.0, 8.0)));
}

#[test]
fn test_tool_cubic_drag_repositions() {
    let mut tool = CurveTool::new();
    tool.degree = CurveDegree::Cubic;
    let road_map = RoadMap::new(3);

    tool.on_click(Vec2::ZERO, &road_map, false);
    tool.on_click(Vec2::new(10.0, 0.0), &road_map, false);
    tool.on_click(Vec2::new(3.0, 8.0), &road_map, false); // CP1
                                                          // CP2 wurde bereits beim End-Klick auto-gesetzt
    assert!(tool.is_ready());

    // Drag-Targets: Start, End, CP1, CP2, Apex (Cubic) = 5
    let targets = tool.drag_targets();
    assert_eq!(
        targets.len(),
        5,
        "Cubic: Start, End, CP1, CP2, Apex = 5 Targets"
    );

    // CP1 per Drag verschieben
    let grabbed = tool.on_drag_start(Vec2::new(3.0, 8.0), &road_map, 2.0);
    assert!(grabbed);
    tool.on_drag_update(Vec2::new(2.0, 6.0));
    assert_eq!(tool.control_point1, Some(Vec2::new(2.0, 6.0)));
    tool.on_drag_end(&road_map);

    // CP2 per Drag verschieben (auto-Position abfragen)
    let cp2_pos = tool.control_point2.expect("CP2 sollte gesetzt sein");
    let grabbed = tool.on_drag_start(cp2_pos, &road_map, 1.0);
    assert!(grabbed, "CP2 sollte greifbar sein an {:?}", cp2_pos);
    let new_cp2 = cp2_pos + Vec2::new(1.0, -2.0);
    tool.on_drag_update(new_cp2);
    tool.on_drag_end(&road_map);
    assert!(tool.dragging.is_none());
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

// ── project_onto_tangent_line ──

#[test]
fn test_project_onto_tangent_line_start_along_axis() {
    // Tangente zeigt nach Ost (0°), is_start=true → Projektion entlang Westrichtung
    let anchor = Vec2::new(0.0, 0.0);
    let angle = 0.0_f32; // Ost
                         // Cursor liegt irgendwo oben rechts
    let cursor = Vec2::new(5.0, 3.0);
    let result = project_onto_tangent_line(anchor, angle, cursor, true);
    // dir = Vec2::from_angle(PI) = (-1, 0), t = dot((5,3), (-1,0)) = -5
    // Projizierter Punkt: (0,0) + (-5)*(-1,0) = (5, 0) — nicht negativ, weil t negativ × neg dir
    // result.y soll ~ 0 sein (auf Tangenten-Linie)
    assert!(
        result.y.abs() < 1e-5,
        "Projektion sollte auf y=0 liegen, war {:?}",
        result
    );
}

#[test]
fn test_project_onto_tangent_line_end_perpendicular_is_zero() {
    // Tangente zeigt nach Ost (0°), is_start=false → dir = (1,0)
    // Cursor lotrecht zur Linie → Projektion ist der Fußpunkt
    let anchor = Vec2::new(5.0, 0.0);
    let cursor = Vec2::new(5.0, 10.0); // direkt lotrecht
    let result = project_onto_tangent_line(anchor, 0.0_f32, cursor, false);
    // dir = (1,0), t = dot((0,10), (1,0)) = 0 → result = anchor
    assert!((result - anchor).length() < 1e-5);
}

// ── solve_cps_from_apex_both_tangents ──

#[test]
fn test_solve_cps_symmetric_horizontal() {
    // P0=(0,0), P3=(10,0), dir1=(0,1) [Nord], dir2=(-1,0) [West]
    // dir1 und dir2 sind linear unabhängig (Kreuzprodukt ≠ 0)
    let p0 = Vec2::new(0.0, 0.0);
    let p3 = Vec2::new(10.0, 0.0);
    let dir1 = Vec2::new(0.0, 1.0); // Nord
    let dir2 = Vec2::new(-1.0, 0.0); // West
    let apex = Vec2::new(5.0, 5.0);

    let result = solve_cps_from_apex_both_tangents(p0, p3, dir1, dir2, apex);
    assert!(
        result.is_some(),
        "Sollte lösbar sein (nicht-parallele Tangenten)"
    );
    let (cp1, cp2) = result.unwrap();

    // CP1 liegt auf der Linie p0 + t*dir1 (Nord) → cp1.x ≈ 0
    assert!(cp1.x.abs() < 1e-3, "CP1.x sollte 0 sein, war {:?}", cp1);

    // B(0.5) soll ≈ apex sein
    let b_half = (p0 + 3.0 * cp1 + 3.0 * cp2 + p3) / 8.0;
    assert!(
        (b_half - apex).length() < 1e-3,
        "B(0.5)={:?}, erwartet {:?}",
        b_half,
        apex
    );
}

#[test]
fn test_solve_cps_parallel_tangents_returns_none() {
    // Parallele Tangenten → keine eindeutige Lösung
    let p0 = Vec2::new(0.0, 0.0);
    let p3 = Vec2::new(10.0, 0.0);
    let dir1 = Vec2::new(1.0, 0.0); // Ost
    let dir2 = Vec2::new(1.0, 0.0); // Ost (parallel zu dir1)
    let apex = Vec2::new(5.0, 5.0);

    let result = solve_cps_from_apex_both_tangents(p0, p3, dir1, dir2, apex);
    assert!(result.is_none(), "Parallele Tangenten sollten None ergeben");
}

#[test]
fn test_solve_cps_asymmetric_apex() {
    // Asymmetrische Kurve: dir1 zeigt nach Oben, dir2 nach Links
    let p0 = Vec2::new(0.0, 0.0);
    let p3 = Vec2::new(10.0, 10.0);
    let dir1 = Vec2::new(0.0, 1.0); // Nord
    let dir2 = Vec2::new(-1.0, 0.0); // West
    let apex = Vec2::new(2.0, 8.0);

    let result = solve_cps_from_apex_both_tangents(p0, p3, dir1, dir2, apex);
    assert!(result.is_some());
    let (cp1, cp2) = result.unwrap();

    // CP1 muss auf der Linie p0 + t*dir1 liegen → cp1.x ≈ 0
    assert!(
        cp1.x.abs() < 1e-3,
        "CP1.x sollte 0 sein (dir1 = Nord), war {:?}",
        cp1
    );
    // CP2 muss auf der Linie p3 + t*dir2 liegen → cp2.y ≈ 10
    assert!(
        (cp2.y - 10.0).abs() < 1e-3,
        "CP2.y sollte 10 sein (dir2 = West), war {:?}",
        cp2
    );

    // Prüfen ob B(0.5) ≈ apex
    let b_half = (p0 + 3.0 * cp1 + 3.0 * cp2 + p3) / 8.0;
    assert!(
        (b_half - apex).length() < 1e-3,
        "B(0.5)={:?}, erwartet {:?}",
        b_half,
        apex
    );
}

// ── auto_suggest_start_tangent / auto_suggest_end_tangent ──

#[test]
fn test_auto_suggest_start_tangent_picks_best_incoming() {
    let mut tool = CurveTool::new_cubic();

    tool.start = Some(ToolAnchor::NewPosition(Vec2::ZERO));
    tool.end = Some(ToolAnchor::NewPosition(Vec2::new(10.0, 0.0)));

    // Eingehende Verbindung von Westen (angle=PI zeigt nach links, Fortsetzung=Ost → passt)
    tool.tangents.start_neighbors = vec![ConnectedNeighbor {
        neighbor_id: 42,
        angle: std::f32::consts::PI,
        is_outgoing: false,
    }];

    tool.auto_suggest_start_tangent();

    assert!(
        matches!(tool.tangents.tangent_start, TangentSource::Connection { neighbor_id: 42, .. }),
        "Start-Tangente sollte auf Nachbar 42 gesetzt sein, war: {:?}",
        tool.tangents.tangent_start,
    );
}

#[test]
fn test_auto_suggest_end_tangent_picks_best_outgoing() {
    let mut tool = CurveTool::new_cubic();

    tool.start = Some(ToolAnchor::NewPosition(Vec2::ZERO));
    tool.end = Some(ToolAnchor::NewPosition(Vec2::new(10.0, 0.0)));

    // Ausgehende Verbindung nach Osten (angle=0 → Richtung away_dir stimmt überein)
    tool.tangents.end_neighbors = vec![ConnectedNeighbor {
        neighbor_id: 77,
        angle: 0.0,
        is_outgoing: true,
    }];

    tool.auto_suggest_end_tangent();

    assert!(
        matches!(tool.tangents.tangent_end, TangentSource::Connection { neighbor_id: 77, .. }),
        "End-Tangente sollte auf Nachbar 77 gesetzt sein, war: {:?}",
        tool.tangents.tangent_end,
    );
}

#[test]
fn test_auto_suggest_end_tangent_rejects_bad_direction() {
    let mut tool = CurveTool::new_cubic();

    tool.start = Some(ToolAnchor::NewPosition(Vec2::ZERO));
    tool.end = Some(ToolAnchor::NewPosition(Vec2::new(10.0, 0.0)));

    // Verbindung zeigt zurück zum Startpunkt (angle=PI → Richtung entgegen away_dir)
    tool.tangents.end_neighbors = vec![ConnectedNeighbor {
        neighbor_id: 88,
        angle: std::f32::consts::PI,
        is_outgoing: true,
    }];

    tool.auto_suggest_end_tangent();

    assert!(
        matches!(tool.tangents.tangent_end, TangentSource::None),
        "End-Tangente sollte None bleiben bei schlechter Richtung, war: {:?}",
        tool.tangents.tangent_end,
    );
}

#[test]
fn test_both_tangents_auto_suggested_on_cubic_end_click() {
    let mut tool = CurveTool::new_cubic();
    let road_map = RoadMap::new(3);

    // Manuell Setup: Start hat eingehende, Ende hat ausgehende Verbindung
    tool.on_click(Vec2::ZERO, &road_map, false); // Phase → End
    // Nachbarn manuell setzen (normalerweise via populate_neighbors)
    tool.tangents.start_neighbors = vec![ConnectedNeighbor {
        neighbor_id: 10,
        angle: std::f32::consts::PI, // von Westen → Fortsetzung Ost
        is_outgoing: false,
    }];

    tool.on_click(Vec2::new(10.0, 0.0), &road_map, false); // Phase → Control
    // end_neighbors direkt setzen und auto_suggest erneut aufrufen
    tool.tangents.end_neighbors = vec![ConnectedNeighbor {
        neighbor_id: 20,
        angle: 0.0, // nach Osten
        is_outgoing: true,
    }];
    tool.auto_suggest_start_tangent();
    tool.auto_suggest_end_tangent();

    assert!(
        matches!(tool.tangents.tangent_start, TangentSource::Connection { neighbor_id: 10, .. }),
        "Start-Tangente erwartet: {:?}",
        tool.tangents.tangent_start,
    );
    assert!(
        matches!(tool.tangents.tangent_end, TangentSource::Connection { neighbor_id: 20, .. }),
        "End-Tangente erwartet: {:?}",
        tool.tangents.tangent_end,
    );
}
