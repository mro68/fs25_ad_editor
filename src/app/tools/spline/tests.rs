use super::geometry::*;
use super::*;

// ── phantom_from_tangent ──

#[test]
fn test_phantom_from_tangent_east() {
    // Tangente zeigt nach Osten (0°), Phantom soll nach Westen (weg vom Nachbar)
    let anchor = Vec2::new(0.0, 0.0);
    let neighbor = Vec2::new(5.0, 0.0);
    let phantom = SplineTool::phantom_from_tangent(anchor, 0.0, neighbor);
    // dist = 5, dir = angle + PI = 180° → (-1, 0) → phantom = (-5, 0)
    assert!(phantom.x < 0.0, "Phantom sollte westlich liegen, war: {:?}", phantom);
    assert!((phantom.y).abs() < 0.01);
    assert!((phantom.x + 5.0).abs() < 0.01);
}

#[test]
fn test_phantom_from_tangent_min_distance() {
    // Gleicher Punkt → min-Abstand von 1.0 wird erzwungen
    let anchor = Vec2::new(3.0, 3.0);
    let neighbor = Vec2::new(3.0, 3.0); // Abstand 0
    let phantom = SplineTool::phantom_from_tangent(anchor, 0.0, neighbor);
    // dist = max(0, 1.0) = 1.0
    assert!((phantom.distance(anchor) - 1.0).abs() < 0.01);
}

// ── compute_phantoms ──

#[test]
fn test_compute_phantoms_none_when_no_tangent() {
    let pts = vec![Vec2::ZERO, Vec2::new(5.0, 0.0), Vec2::new(10.0, 0.0)];
    let (start, end) =
        SplineTool::compute_phantoms(&pts, TangentSource::None, TangentSource::None);
    assert!(start.is_none());
    assert!(end.is_none());
}

#[test]
fn test_compute_phantoms_start_only() {
    let pts = vec![Vec2::ZERO, Vec2::new(5.0, 0.0), Vec2::new(10.0, 0.0)];
    let tangent = TangentSource::Connection {
        neighbor_id: 1,
        angle: 0.0,
    };
    let (start, end) = SplineTool::compute_phantoms(&pts, tangent, TangentSource::None);
    assert!(start.is_some());
    assert!(end.is_none());
    // Phantom liegt westlich vom Startpunkt
    assert!(start.unwrap().x < 0.0);
}

#[test]
fn test_compute_phantoms_end_only() {
    let pts = vec![Vec2::ZERO, Vec2::new(5.0, 0.0), Vec2::new(10.0, 0.0)];
    let tangent = TangentSource::Connection {
        neighbor_id: 2,
        angle: std::f32::consts::PI, // zeigt nach Westen
    };
    let (start, end) = SplineTool::compute_phantoms(&pts, TangentSource::None, tangent);
    assert!(start.is_none());
    assert!(end.is_some());
    // angle=PI → dir = Vec2::from_angle(0) = (+1, 0) → Phantom östlich vom Ende
    assert!(end.unwrap().x > 10.0);
}

// ── Catmull-Rom-Geometrie ──

#[test]
fn test_catmull_rom_two_points_straight_line() {
    let points = vec![Vec2::ZERO, Vec2::new(10.0, 0.0)];
    let result = catmull_rom_chain(&points, 10);
    assert_eq!(result.len(), 11);
    assert!((result[0] - Vec2::ZERO).length() < 0.001);
    assert!((result[10] - Vec2::new(10.0, 0.0)).length() < 0.001);
}

#[test]
fn test_catmull_rom_passes_through_control_points() {
    let points = vec![
        Vec2::new(0.0, 0.0),
        Vec2::new(5.0, 10.0),
        Vec2::new(10.0, 0.0),
    ];
    let result = catmull_rom_chain(&points, 20);

    // Startpunkt
    assert!((result[0] - points[0]).length() < 0.01);
    // Mittelpunkt (bei t=0 des zweiten Segments = Index 20)
    assert!((result[20] - points[1]).length() < 0.01);
    // Endpunkt
    assert!(
        (result.last().unwrap().distance(points[2])) < 0.01,
        "Endpunkt: {:?} vs {:?}",
        result.last(),
        points[2]
    );
}

#[test]
fn test_catmull_rom_four_points() {
    let points = vec![
        Vec2::new(0.0, 0.0),
        Vec2::new(3.0, 5.0),
        Vec2::new(7.0, 5.0),
        Vec2::new(10.0, 0.0),
    ];
    let result = catmull_rom_chain(&points, 10);

    // Muss mindestens (4-1)*10 + 1 = 31 Punkte haben
    assert_eq!(result.len(), 31);
    // Start- und Endpunkte
    assert!((result[0] - points[0]).length() < 0.01);
    assert!(result.last().unwrap().distance(points[3]) < 0.01);
    // Durchlaufen durch Zwischenpunkte
    assert!((result[10] - points[1]).length() < 0.01);
    assert!((result[20] - points[2]).length() < 0.01);
}

#[test]
fn test_resample_preserves_endpoints() {
    let polyline = vec![
        Vec2::new(0.0, 0.0),
        Vec2::new(5.0, 0.0),
        Vec2::new(10.0, 0.0),
    ];
    let resampled = resample_by_distance(&polyline, 3.0);

    assert!((resampled[0] - Vec2::ZERO).length() < 0.01);
    assert!((resampled.last().unwrap().distance(Vec2::new(10.0, 0.0))) < 0.01);
}

#[test]
fn test_resample_spacing() {
    let polyline = vec![
        Vec2::new(0.0, 0.0),
        Vec2::new(5.0, 0.0),
        Vec2::new(10.0, 0.0),
    ];
    let resampled = resample_by_distance(&polyline, 2.0);

    // 10m / 2m = 5 Segmente → 6 Punkte
    assert_eq!(resampled.len(), 6);
    for i in 0..resampled.len() - 1 {
        let dist = resampled[i].distance(resampled[i + 1]);
        assert!(
            (dist - 2.0).abs() < 0.1,
            "Segment {} hat Abstand {:.3}m",
            i,
            dist
        );
    }
}

// ── Tool-Flow ──

#[test]
fn test_spline_tool_click_flow() {
    let mut tool = SplineTool::new();
    let road_map = RoadMap::new(3);

    assert!(!tool.is_ready());
    assert_eq!(tool.status_text(), "Startpunkt klicken");

    // Erster Klick
    let action = tool.on_click(Vec2::ZERO, &road_map, false);
    assert_eq!(action, ToolAction::Continue);
    assert!(!tool.is_ready());

    // Zweiter Klick → bereit
    let action = tool.on_click(Vec2::new(10.0, 0.0), &road_map, false);
    assert_eq!(action, ToolAction::UpdatePreview);
    assert!(tool.is_ready());

    // Dritter Klick → immer noch bereit, Spline wird aktualisiert
    let action = tool.on_click(Vec2::new(5.0, 8.0), &road_map, false);
    assert_eq!(action, ToolAction::UpdatePreview);
    assert!(tool.is_ready());
    assert_eq!(tool.anchors.len(), 3);
}

#[test]
fn test_spline_tool_execute() {
    let mut tool = SplineTool::new();
    tool.seg.max_segment_length = 2.0;
    let road_map = RoadMap::new(3);

    tool.on_click(Vec2::ZERO, &road_map, false);
    tool.on_click(Vec2::new(10.0, 0.0), &road_map, false);
    tool.on_click(Vec2::new(5.0, 8.0), &road_map, false);

    let result = tool.execute(&road_map).expect("Ergebnis erwartet");
    assert!(result.new_nodes.len() >= 3);
    assert_eq!(
        result.internal_connections.len(),
        result.new_nodes.len() - 1,
    );
}

#[test]
fn test_spline_tool_reset() {
    let mut tool = SplineTool::new();
    let road_map = RoadMap::new(3);

    tool.on_click(Vec2::ZERO, &road_map, false);
    tool.on_click(Vec2::new(10.0, 0.0), &road_map, false);
    assert!(tool.is_ready());

    tool.reset();
    assert!(!tool.is_ready());
    assert!(tool.anchors.is_empty());
}

#[test]
fn test_spline_tool_chaining() {
    let mut tool = SplineTool::new();
    let road_map = RoadMap::new(3);

    // Erste Strecke
    tool.on_click(Vec2::ZERO, &road_map, false);
    tool.on_click(Vec2::new(10.0, 0.0), &road_map, false);
    tool.on_click(Vec2::new(5.0, 8.0), &road_map, false);
    tool.set_last_created(vec![100, 101, 102, 103], &road_map);
    tool.reset();

    // Verkettung: nächster Klick übernimmt letzten Endpunkt
    let action = tool.on_click(Vec2::new(20.0, 0.0), &road_map, false);
    assert_eq!(action, ToolAction::UpdatePreview);
    assert_eq!(tool.anchors.len(), 2);
}

#[test]
fn test_spline_tool_preview_with_cursor() {
    let mut tool = SplineTool::new();
    let road_map = RoadMap::new(3);

    tool.on_click(Vec2::ZERO, &road_map, false);
    tool.on_click(Vec2::new(10.0, 0.0), &road_map, false);

    let preview = tool.preview(Vec2::new(5.0, 8.0), &road_map);
    // Vorschau sollte Nodes und Connections enthalten
    assert!(!preview.nodes.is_empty());
    assert!(!preview.connections.is_empty());
}

#[test]
fn test_spline_execute_from_anchors() {
    let mut tool = SplineTool::new();
    tool.seg.max_segment_length = 2.0;
    let road_map = RoadMap::new(3);

    tool.on_click(Vec2::ZERO, &road_map, false);
    tool.on_click(Vec2::new(10.0, 0.0), &road_map, false);
    tool.on_click(Vec2::new(5.0, 8.0), &road_map, false);
    let original = tool.execute(&road_map).unwrap();
    tool.set_last_created(vec![1, 2, 3, 4, 5], &road_map);
    tool.reset();

    // Nachbearbeitung mit anderer Segment-Länge
    tool.seg.max_segment_length = 5.0;
    let result = tool
        .execute_from_anchors(&road_map)
        .expect("Ergebnis erwartet");
    assert!(result.new_nodes.len() < original.new_nodes.len());
}
