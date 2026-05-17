use super::geometry::{collect_transitions, ArcValidation};
use super::RoundingTool;
use crate::app::tools::RouteToolCore;
use crate::app::tools::RouteToolPanelBridge;
use crate::app::tools::RouteToolSelectionSeed;
use crate::core::{
    Connection, ConnectionDirection, ConnectionPriority, MapNode, NodeFlag, RoadMap,
};
use glam::Vec2;

fn make_neighbor_seed(
    road_map: &RoadMap,
    node_id: u64,
) -> Vec<crate::app::tools::RouteToolConnectedNeighborSeed> {
    road_map
        .connected_neighbors(node_id)
        .into_iter()
        .filter_map(|neighbor| {
            let position = road_map.node_position(neighbor.neighbor_id)?;
            Some(crate::app::tools::RouteToolConnectedNeighborSeed::new(
                neighbor, position,
            ))
        })
        .collect()
}

fn load_single_corner(tool: &mut RoundingTool, road_map: &RoadMap, node_id: u64) {
    tool.load_selection_seed(RouteToolSelectionSeed {
        node_ids: vec![node_id],
        positions: vec![road_map
            .node_position(node_id)
            .expect("Corner-Node erwartet")],
        connected_neighbors: vec![make_neighbor_seed(road_map, node_id)],
    });
}

fn simple_corner_map() -> RoadMap {
    let mut road_map = RoadMap::new(3);
    road_map.add_node(MapNode::new(10, Vec2::new(-20.0, 0.0), NodeFlag::Regular));
    road_map.add_node(MapNode::new(1, Vec2::ZERO, NodeFlag::Regular));
    road_map.add_node(MapNode::new(20, Vec2::new(0.0, 20.0), NodeFlag::Regular));
    road_map.add_connection(Connection::new(
        10,
        1,
        ConnectionDirection::Regular,
        ConnectionPriority::Regular,
        Vec2::new(-20.0, 0.0),
        Vec2::ZERO,
    ));
    road_map.add_connection(Connection::new(
        1,
        20,
        ConnectionDirection::Regular,
        ConnectionPriority::Regular,
        Vec2::ZERO,
        Vec2::new(0.0, 20.0),
    ));
    road_map
}

#[test]
fn arc_plan_builds_true_circle_with_fixed_radius() {
    let road_map = simple_corner_map();
    let mut tool = RoundingTool::new();
    tool.arc.radius_m = 5.0;
    load_single_corner(&mut tool, &road_map, 1);

    let plan = tool.arc.plan.clone().expect("Arc-Plan erwartet");
    assert_eq!(tool.arc.validation, ArcValidation::Ready);
    let tangent_candidates = [plan.tangent_first, plan.tangent_second];
    assert!(tangent_candidates
        .iter()
        .any(|point| (*point - Vec2::new(-5.0, 0.0)).length() < 0.05));
    assert!(tangent_candidates
        .iter()
        .any(|point| (*point - Vec2::new(0.0, 5.0)).length() < 0.05));
    for point in &plan.arc_positions {
        let radius = (*point - plan.center).length();
        assert!(
            (radius - 5.0).abs() < 0.05,
            "Punkt liegt nicht auf Radius 5 m"
        );
    }
}

#[test]
fn execute_builds_local_replacement_and_removes_corner_node() {
    let road_map = simple_corner_map();
    let mut tool = RoundingTool::new();
    tool.arc.radius_m = 5.0;
    load_single_corner(&mut tool, &road_map, 1);

    assert!(
        tool.is_ready(),
        "Corner sollte fuer ArcOnePoint bereit sein"
    );
    assert_eq!(
        tool.status_text(),
        "Bereit — Enter verrundet den Corner mit festem Kreisbogen."
    );

    let preview = tool.preview(Vec2::ZERO, &road_map);
    assert!(
        preview.nodes.len() >= 4,
        "Preview muss beide Seiten plus Arc anzeigen"
    );

    let result = tool.execute(&road_map).expect("ToolResult erwartet");
    assert_eq!(result.nodes_to_remove, vec![1]);
    assert!(
        !result.new_nodes.is_empty(),
        "Neue RoundedCorner-Nodes erwartet"
    );
    assert!(result
        .new_nodes
        .iter()
        .all(|(_, flag)| *flag == NodeFlag::RoundedCorner));
    assert_eq!(result.external_connections.len(), 2);
    assert!(result.internal_connections.len() >= 2);
}

#[test]
fn ambiguous_junction_is_invalid_for_cp02() {
    let mut road_map = simple_corner_map();
    road_map.add_node(MapNode::new(30, Vec2::new(20.0, 0.0), NodeFlag::Regular));
    road_map.add_connection(Connection::new(
        1,
        30,
        ConnectionDirection::Regular,
        ConnectionPriority::Regular,
        Vec2::ZERO,
        Vec2::new(20.0, 0.0),
    ));

    let mut tool = RoundingTool::new();
    tool.arc.radius_m = 5.0;
    load_single_corner(&mut tool, &road_map, 1);

    assert!(!tool.is_ready());
    assert_eq!(tool.arc.validation, ArcValidation::AmbiguousJunction);
    assert_eq!(
        tool.status_text(),
        "Junction mit mehr als 2 Anschlussseiten ist in CP-02 ungueltig."
    );
    assert!(tool.preview(Vec2::ZERO, &road_map).nodes.is_empty());
    assert!(tool.execute(&road_map).is_none());
}

#[test]
fn radius_too_large_is_reported_clearly() {
    let mut road_map = RoadMap::new(3);
    road_map.add_node(MapNode::new(10, Vec2::new(-4.0, 0.0), NodeFlag::Regular));
    road_map.add_node(MapNode::new(1, Vec2::ZERO, NodeFlag::Regular));
    road_map.add_node(MapNode::new(20, Vec2::new(0.0, 4.0), NodeFlag::Regular));
    road_map.add_connection(Connection::new(
        10,
        1,
        ConnectionDirection::Regular,
        ConnectionPriority::Regular,
        Vec2::new(-4.0, 0.0),
        Vec2::ZERO,
    ));
    road_map.add_connection(Connection::new(
        1,
        20,
        ConnectionDirection::Regular,
        ConnectionPriority::Regular,
        Vec2::ZERO,
        Vec2::new(0.0, 4.0),
    ));

    let mut tool = RoundingTool::new();
    tool.arc.radius_m = 5.0;
    load_single_corner(&mut tool, &road_map, 1);

    assert_eq!(tool.arc.validation, ArcValidation::RadiusTooLarge);
    assert!(!tool.is_ready());
    assert_eq!(
        tool.status_text(),
        "Radius passt nicht in die erste Strecke auf mindestens einer Seite."
    );
}

#[test]
fn transitions_reflect_available_direction_through_corner() {
    let road_map = simple_corner_map();
    let mut tool = RoundingTool::new();
    tool.arc.radius_m = 5.0;
    load_single_corner(&mut tool, &road_map, 1);

    let plan = tool.arc.plan.as_ref().expect("Arc-Plan erwartet");
    let transitions = collect_transitions(&road_map, 1, plan);
    assert_eq!(transitions.len(), 1);
    assert_eq!(transitions[0].direction, ConnectionDirection::Regular);
    assert_eq!(transitions[0].priority, ConnectionPriority::Regular);
}
