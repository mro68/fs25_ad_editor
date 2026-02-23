use super::super::{RouteTool, ToolAction, ToolAnchor};
use super::geometry::compute_line_positions;
use super::state::StraightLineTool;
use crate::core::{MapNode, NodeFlag, RoadMap};
use glam::Vec2;

#[test]
fn test_compute_line_positions_basic() {
    let positions = compute_line_positions(Vec2::ZERO, Vec2::new(12.0, 0.0), 6.0);
    assert_eq!(positions.len(), 3);
    assert!((positions[0] - Vec2::ZERO).length() < 0.01);
    assert!((positions[1] - Vec2::new(6.0, 0.0)).length() < 0.01);
    assert!((positions[2] - Vec2::new(12.0, 0.0)).length() < 0.01);
}

#[test]
fn test_compute_line_positions_short_segment() {
    let positions = compute_line_positions(Vec2::ZERO, Vec2::new(3.0, 0.0), 6.0);
    assert_eq!(positions.len(), 2);
}

#[test]
fn test_tool_click_flow() {
    let mut tool = StraightLineTool::new();
    let road_map = RoadMap::new(3);

    assert!(!tool.is_ready());
    let action = tool.on_click(Vec2::ZERO, &road_map, false);
    assert_eq!(action, ToolAction::Continue);
    assert!(!tool.is_ready());

    let action = tool.on_click(Vec2::new(12.0, 0.0), &road_map, false);
    assert_eq!(action, ToolAction::ReadyToExecute);
    assert!(tool.is_ready());
}

#[test]
fn test_tool_execute() {
    let mut tool = StraightLineTool::new();
    tool.seg.max_segment_length = 6.0;
    let road_map = RoadMap::new(3);

    tool.on_click(Vec2::ZERO, &road_map, false);
    tool.on_click(Vec2::new(12.0, 0.0), &road_map, false);

    let result = tool.execute(&road_map).expect("Ergebnis erwartet");
    assert_eq!(result.new_nodes.len(), 3);
    assert_eq!(result.internal_connections.len(), 2);
}

#[test]
fn test_tool_reset() {
    let mut tool = StraightLineTool::new();
    let road_map = RoadMap::new(3);

    tool.on_click(Vec2::ZERO, &road_map, false);
    tool.on_click(Vec2::new(10.0, 0.0), &road_map, false);
    assert!(tool.is_ready());

    tool.reset();
    assert!(!tool.is_ready());
}

#[test]
fn test_chaining_uses_last_end_as_start() {
    let mut tool = StraightLineTool::new();
    tool.seg.max_segment_length = 6.0;
    let mut road_map = RoadMap::new(3);

    tool.on_click(Vec2::ZERO, &road_map, false);
    tool.on_click(Vec2::new(12.0, 0.0), &road_map, false);

    // Simuliere: Nodes wurden in RoadMap erstellt
    road_map.add_node(MapNode::new(100, Vec2::ZERO, NodeFlag::Regular));
    road_map.add_node(MapNode::new(101, Vec2::new(6.0, 0.0), NodeFlag::Regular));
    road_map.add_node(MapNode::new(102, Vec2::new(12.0, 0.0), NodeFlag::Regular));
    road_map.ensure_spatial_index();

    tool.set_last_created(vec![100, 101, 102], &road_map);
    tool.reset();

    let action = tool.on_click(Vec2::new(24.0, 0.0), &road_map, false);
    assert_eq!(action, ToolAction::ReadyToExecute);
    assert!(tool.is_ready());

    let result = tool.execute(&road_map).expect("Ergebnis erwartet");
    // Start ist ExistingNode(102) → kein neuer Node dafür
    assert_eq!(result.new_nodes.len(), 2);
    assert!((result.new_nodes[1].0 - Vec2::new(24.0, 0.0)).length() < 0.01);
    // Externe Verbindung zum existierenden Startknoten
    assert_eq!(result.external_connections.len(), 1);
}

#[test]
fn test_chaining_no_duplicate_node_at_junction() {
    let mut tool = StraightLineTool::new();
    tool.seg.max_segment_length = 10.0;
    let mut road_map = RoadMap::new(3);

    // Linie 1: (0,0) → (10,0)
    tool.on_click(Vec2::ZERO, &road_map, false);
    tool.on_click(Vec2::new(10.0, 0.0), &road_map, false);

    road_map.add_node(MapNode::new(1, Vec2::ZERO, NodeFlag::Regular));
    road_map.add_node(MapNode::new(2, Vec2::new(10.0, 0.0), NodeFlag::Regular));
    road_map.ensure_spatial_index();
    tool.set_last_created(vec![1, 2], &road_map);
    tool.reset();

    // Linie 2 verkettet: Start = Node 2, Ende = (20, 0)
    let action = tool.on_click(Vec2::new(20.0, 0.0), &road_map, false);
    assert_eq!(action, ToolAction::ReadyToExecute);

    // Start muss ExistingNode(2) sein, nicht NewPosition
    let start = tool.start;
    assert!(
        matches!(start, Some(ToolAnchor::ExistingNode(2, _))),
        "Start-Anker muss ExistingNode(2) sein, war: {:?}",
        start,
    );

    let result = tool.execute(&road_map).expect("Ergebnis erwartet");
    // Nur 1 neuer Node (Endpunkt), Start ist existierend
    assert_eq!(result.new_nodes.len(), 1);
    assert!((result.new_nodes[0].0 - Vec2::new(20.0, 0.0)).length() < 0.01);
}

#[test]
fn test_last_created_ids_preserved_after_reset() {
    let mut tool = StraightLineTool::new();
    let road_map = RoadMap::new(3);

    tool.on_click(Vec2::ZERO, &road_map, false);
    tool.on_click(Vec2::new(10.0, 0.0), &road_map, false);

    tool.set_last_created(vec![1, 2, 3], &road_map);
    tool.reset();

    assert_eq!(tool.last_created_ids(), &[1, 2, 3]);
    assert!(tool.last_end_anchor().is_some());
}

#[test]
fn test_execute_from_anchors() {
    let mut tool = StraightLineTool::new();
    tool.seg.max_segment_length = 5.0;
    let road_map = RoadMap::new(3);

    tool.on_click(Vec2::ZERO, &road_map, false);
    tool.on_click(Vec2::new(10.0, 0.0), &road_map, false);
    tool.set_last_created(vec![1, 2, 3], &road_map);
    tool.reset();

    tool.seg.max_segment_length = 10.0;
    let result = tool
        .execute_from_anchors(&road_map)
        .expect("Ergebnis erwartet");
    assert_eq!(result.new_nodes.len(), 2);
}
