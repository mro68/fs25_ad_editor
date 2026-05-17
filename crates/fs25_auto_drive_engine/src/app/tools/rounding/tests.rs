use super::geometry::{
    collect_quadratic_transitions, collect_transitions, ArcValidation, QuadraticValidation,
};
use super::{RoundingMode, RoundingTool};
use crate::app::tool_editing::RouteToolEditPayload;
use crate::app::tools::{
    OrderedNodeChain, RouteToolChainInput, RouteToolCore, RouteToolGroupEdit, RouteToolPanelBridge,
    RouteToolRecreate, RouteToolSelectionSeed,
};
use crate::app::ui_contract::{RoundingPanelAction, RouteToolPanelAction};
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

fn load_three_point_chain(tool: &mut RoundingTool, road_map: &RoadMap, ordered_ids: [u64; 3]) {
    let positions: Vec<Vec2> = ordered_ids
        .iter()
        .map(|id| road_map.node_position(*id).expect("Chain-Node erwartet"))
        .collect();
    tool.mode = RoundingMode::QuadraticThreePoint;
    tool.load_chain(OrderedNodeChain {
        positions: positions.clone(),
        start_id: ordered_ids[0],
        end_id: ordered_ids[2],
        inner_ids: vec![ordered_ids[1]],
    });
    tool.load_selection_seed(RouteToolSelectionSeed {
        node_ids: ordered_ids.to_vec(),
        positions,
        connected_neighbors: ordered_ids
            .iter()
            .map(|id| make_neighbor_seed(road_map, *id))
            .collect(),
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

fn quadratic_ready_map() -> RoadMap {
    let mut road_map = RoadMap::new(3);
    road_map.add_node(MapNode::new(10, Vec2::new(-20.0, 0.0), NodeFlag::Regular));
    road_map.add_node(MapNode::new(1, Vec2::new(-10.0, 0.0), NodeFlag::Regular));
    road_map.add_node(MapNode::new(2, Vec2::ZERO, NodeFlag::Regular));
    road_map.add_node(MapNode::new(3, Vec2::new(10.0, 10.0), NodeFlag::Regular));
    road_map.add_node(MapNode::new(30, Vec2::new(20.0, 20.0), NodeFlag::Regular));
    road_map.add_connection(Connection::new(
        10,
        1,
        ConnectionDirection::Regular,
        ConnectionPriority::Regular,
        Vec2::new(-20.0, 0.0),
        Vec2::new(-10.0, 0.0),
    ));
    road_map.add_connection(Connection::new(
        1,
        2,
        ConnectionDirection::Regular,
        ConnectionPriority::Regular,
        Vec2::new(-10.0, 0.0),
        Vec2::ZERO,
    ));
    road_map.add_connection(Connection::new(
        2,
        3,
        ConnectionDirection::Regular,
        ConnectionPriority::Regular,
        Vec2::ZERO,
        Vec2::new(10.0, 10.0),
    ));
    road_map.add_connection(Connection::new(
        3,
        30,
        ConnectionDirection::Regular,
        ConnectionPriority::Regular,
        Vec2::new(10.0, 10.0),
        Vec2::new(20.0, 20.0),
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

#[test]
fn quadratic_three_point_builds_preview_and_local_replace() {
    let road_map = quadratic_ready_map();
    let mut tool = RoundingTool::new();
    load_three_point_chain(&mut tool, &road_map, [1, 2, 3]);

    assert_eq!(tool.quadratic.validation, QuadraticValidation::Ready);
    assert!(
        tool.is_ready(),
        "3-Punkt-Kette sollte fuer Quadratic bereit sein"
    );
    assert_eq!(
        tool.status_text(),
        "Bereit — Enter ersetzt die mittlere Node durch eine quadratische Verrundung."
    );

    let preview = tool.preview(Vec2::ZERO, &road_map);
    assert!(
        preview.nodes.len() >= 3,
        "Quadratic-Preview braucht Start, Mitte und Ende"
    );

    let plan = tool
        .quadratic
        .plan
        .as_ref()
        .expect("Quadratic-Plan erwartet");
    assert_eq!(
        plan.curve_positions.first().copied(),
        Some(Vec2::new(-10.0, 0.0))
    );
    assert_eq!(
        plan.curve_positions.last().copied(),
        Some(Vec2::new(10.0, 10.0))
    );

    let transitions = collect_quadratic_transitions(&road_map, plan);
    assert_eq!(transitions.len(), 1);
    assert!(transitions[0].forward);

    let result = tool
        .execute(&road_map)
        .expect("Quadratic-ToolResult erwartet");
    assert_eq!(result.nodes_to_remove, vec![2]);
    assert!(
        !result.new_nodes.is_empty(),
        "Quadratic braucht innere RoundedCorner-Nodes"
    );
    assert!(result
        .new_nodes
        .iter()
        .all(|(_, flag)| *flag == NodeFlag::RoundedCorner));
    assert_eq!(result.external_connections.len(), 2);
}

#[test]
fn quadratic_three_point_reports_outer_tangent_mismatch() {
    let mut road_map = quadratic_ready_map();
    road_map.remove_node(30);
    road_map.add_node(MapNode::new(30, Vec2::new(20.0, 10.0), NodeFlag::Regular));
    road_map.add_connection(Connection::new(
        3,
        30,
        ConnectionDirection::Regular,
        ConnectionPriority::Regular,
        Vec2::new(10.0, 10.0),
        Vec2::new(20.0, 10.0),
    ));

    let mut tool = RoundingTool::new();
    load_three_point_chain(&mut tool, &road_map, [1, 2, 3]);

    assert_eq!(
        tool.quadratic.validation,
        QuadraticValidation::TangentsMissFixedControl
    );
    assert!(!tool.is_ready());
    assert_eq!(
        tool.status_text(),
        "Die Aussenstrecken muessen sich mit passender Richtung im festen Steuerpunkt P2 schneiden."
    );
    assert!(tool.preview(Vec2::ZERO, &road_map).nodes.is_empty());
    assert!(tool.execute(&road_map).is_none());
}

#[test]
fn quadratic_three_point_rejects_control_branches() {
    let mut road_map = quadratic_ready_map();
    road_map.add_node(MapNode::new(40, Vec2::new(0.0, -10.0), NodeFlag::Regular));
    road_map.add_connection(Connection::new(
        2,
        40,
        ConnectionDirection::Regular,
        ConnectionPriority::Regular,
        Vec2::ZERO,
        Vec2::new(0.0, -10.0),
    ));

    let mut tool = RoundingTool::new();
    load_three_point_chain(&mut tool, &road_map, [1, 2, 3]);

    assert_eq!(
        tool.quadratic.validation,
        QuadraticValidation::ControlHasExternalConnections
    );
    assert_eq!(
        tool.status_text(),
        "P2 darf in CP-03 keine zusaetzlichen Aussenverbindungen haben."
    );
    assert!(tool.execute(&road_map).is_none());
}

#[test]
fn arc_panel_change_marks_recreate_and_updates_payload() {
    let road_map = simple_corner_map();
    let mut tool = RoundingTool::new();
    tool.arc.radius_m = 5.0;
    load_single_corner(&mut tool, &road_map, 1);

    assert!(tool.execute(&road_map).is_some());
    tool.on_applied(&[101, 102, 103], &road_map);

    let initial_payload = tool
        .build_edit_payload()
        .expect("Arc-Payload nach Apply erwartet");
    match initial_payload {
        RouteToolEditPayload::RoundingArc {
            radius_m,
            sample_spacing_m,
            ..
        } => {
            assert_eq!(radius_m, 5.0);
            assert_eq!(sample_spacing_m, tool.arc.sample_spacing_m);
        }
        other => panic!("unerwarteter Payload-Typ: {other:?}"),
    }

    let effect = tool.apply_panel_action(RouteToolPanelAction::Rounding(
        RoundingPanelAction::SetArcRadius(9.0),
    ));
    assert!(effect.changed);
    assert!(effect.needs_recreate);
    assert!(tool.needs_recreate());

    let updated_payload = tool
        .build_edit_payload()
        .expect("Aktualisierter Arc-Payload erwartet");
    match updated_payload {
        RouteToolEditPayload::RoundingArc { radius_m, .. } => assert_eq!(radius_m, 9.0),
        other => panic!("unerwarteter Payload-Typ: {other:?}"),
    }
}

#[test]
fn restored_arc_payload_executes_without_original_corner_node() {
    let road_map = simple_corner_map();
    let mut source_tool = RoundingTool::new();
    source_tool.arc.radius_m = 5.0;
    load_single_corner(&mut source_tool, &road_map, 1);

    let original_result = source_tool
        .execute(&road_map)
        .expect("Originales Arc-Result erwartet");
    assert_eq!(original_result.nodes_to_remove, vec![1]);
    source_tool.on_applied(&[201, 202, 203], &road_map);

    let payload = source_tool
        .build_edit_payload()
        .expect("Persistierter Arc-Payload erwartet");

    let mut recreated_map = simple_corner_map();
    recreated_map.remove_node(1);

    let mut restored_tool = RoundingTool::new();
    restored_tool.restore_edit_payload(&payload);

    assert_eq!(
        restored_tool.status_text(),
        "Nachbearbeitung — Radius oder Abtastung anpassen und Enter zum Neuaufbau druecken."
    );
    assert!(
        restored_tool
            .preview(Vec2::ZERO, &recreated_map)
            .nodes
            .len()
            >= 4
    );

    let recreated_result = restored_tool
        .execute(&recreated_map)
        .expect("Arc-Recreate-Result erwartet");
    assert!(recreated_result.nodes_to_remove.is_empty());
    assert_eq!(recreated_result.external_connections.len(), 2);
    assert!(!recreated_result.new_nodes.is_empty());
}

#[test]
fn restored_quadratic_payload_executes_without_original_control_node() {
    let road_map = quadratic_ready_map();
    let mut source_tool = RoundingTool::new();
    load_three_point_chain(&mut source_tool, &road_map, [1, 2, 3]);

    let original_result = source_tool
        .execute(&road_map)
        .expect("Originales Quadratic-Result erwartet");
    assert_eq!(original_result.nodes_to_remove, vec![2]);
    source_tool.on_applied(&[301, 302, 303], &road_map);

    let payload = source_tool
        .build_edit_payload()
        .expect("Persistierter Quadratic-Payload erwartet");
    match &payload {
        RouteToolEditPayload::RoundingQuadratic {
            start_outer_neighbor_id,
            end_outer_neighbor_id,
            ..
        } => {
            assert_eq!(*start_outer_neighbor_id, 10);
            assert_eq!(*end_outer_neighbor_id, 30);
        }
        other => panic!("unerwarteter Payload-Typ: {other:?}"),
    }

    let mut recreated_map = quadratic_ready_map();
    recreated_map.remove_node(2);

    let mut restored_tool = RoundingTool::new();
    restored_tool.restore_edit_payload(&payload);

    assert_eq!(
        restored_tool.status_text(),
        "Nachbearbeitung — Abtastung anpassen und Enter zum Neuaufbau druecken."
    );
    assert!(
        restored_tool
            .preview(Vec2::ZERO, &recreated_map)
            .nodes
            .len()
            >= 3
    );

    let recreated_result = restored_tool
        .execute(&recreated_map)
        .expect("Quadratic-Recreate-Result erwartet");
    assert!(recreated_result.nodes_to_remove.is_empty());
    assert_eq!(recreated_result.external_connections.len(), 2);
    assert!(!recreated_result.new_nodes.is_empty());
}

#[test]
fn restored_quadratic_payload_rejects_changed_outer_context() {
    let road_map = quadratic_ready_map();
    let mut source_tool = RoundingTool::new();
    load_three_point_chain(&mut source_tool, &road_map, [1, 2, 3]);

    assert!(source_tool.execute(&road_map).is_some());
    source_tool.on_applied(&[301, 302, 303], &road_map);

    let payload = source_tool
        .build_edit_payload()
        .expect("Persistierter Quadratic-Payload erwartet");

    let mut recreated_map = quadratic_ready_map();
    recreated_map.remove_node(2);
    assert!(recreated_map.update_node_position(10, Vec2::new(-20.0, 5.0)));

    let mut restored_tool = RoundingTool::new();
    restored_tool.restore_edit_payload(&payload);

    assert!(restored_tool
        .preview(Vec2::ZERO, &recreated_map)
        .nodes
        .is_empty());
    assert!(restored_tool.execute(&recreated_map).is_none());
    assert!(restored_tool.execute_from_anchors(&recreated_map).is_none());
}
