use super::geometry::{build_arc_plan_from_payload, collect_transitions, ArcPlan, ArcValidation};
use super::state::{DEFAULT_ARC_MAX_ANGLE_DEG, MAX_ARC_MAX_ANGLE_DEG, MIN_ARC_MAX_ANGLE_DEG};
use super::RoundingTool;
use crate::app::tool_editing::RouteToolEditPayload;
use crate::app::tools::{
    RouteToolCore, RouteToolGroupEdit, RouteToolLinearStretchSeed, RouteToolPanelBridge,
    RouteToolRecreate, RouteToolSelectionSeed,
};
use crate::app::ui_contract::{RoundingPanelAction, RouteToolPanelAction};
use crate::core::{
    Connection, ConnectionDirection, ConnectionPriority, MapNode, NodeFlag, RoadMap,
};
use glam::Vec2;

fn single_step_stretch_seeds(road_map: &RoadMap, node_id: u64) -> Vec<RouteToolLinearStretchSeed> {
    let corner_position = road_map
        .node_position(node_id)
        .expect("Corner-Node erwartet");
    let mut by_neighbor = std::collections::HashMap::<u64, RouteToolLinearStretchSeed>::new();

    for neighbor in road_map.connected_neighbors(node_id) {
        let Some(position) = road_map.node_position(neighbor.neighbor_id) else {
            continue;
        };
        let stretch =
            by_neighbor
                .entry(neighbor.neighbor_id)
                .or_insert(RouteToolLinearStretchSeed {
                    node_ids: vec![neighbor.neighbor_id],
                    positions: vec![position],
                    angle: neighbor.angle,
                    has_incoming: false,
                    has_outgoing: false,
                });
        stretch.angle = neighbor.angle;
        if neighbor.is_outgoing {
            stretch.has_outgoing = true;
        } else {
            stretch.has_incoming = true;
        }
    }

    let mut stretches: Vec<RouteToolLinearStretchSeed> = by_neighbor.into_values().collect();
    stretches.sort_by(|left, right| left.angle.total_cmp(&right.angle));

    for stretch in &mut stretches {
        if stretch.positions.is_empty() {
            continue;
        }
        stretch.angle = (stretch.positions[0] - corner_position).to_angle();
    }
    stretches.sort_by(|left, right| left.angle.total_cmp(&right.angle));
    stretches
}

fn stretch_seed(
    road_map: &RoadMap,
    corner_id: u64,
    node_ids: &[u64],
    has_incoming: bool,
    has_outgoing: bool,
) -> RouteToolLinearStretchSeed {
    let corner_position = road_map
        .node_position(corner_id)
        .expect("Corner-Node erwartet");
    let positions: Vec<Vec2> = node_ids
        .iter()
        .map(|id| road_map.node_position(*id).expect("Stretch-Node erwartet"))
        .collect();

    RouteToolLinearStretchSeed {
        node_ids: node_ids.to_vec(),
        angle: (positions[0] - corner_position).to_angle(),
        positions,
        has_incoming,
        has_outgoing,
    }
}

fn load_single_corner_with_stretches(
    tool: &mut RoundingTool,
    road_map: &RoadMap,
    node_id: u64,
    mut stretches: Vec<RouteToolLinearStretchSeed>,
) {
    stretches.sort_by(|left, right| left.angle.total_cmp(&right.angle));
    tool.load_selection_seed(RouteToolSelectionSeed {
        node_ids: vec![node_id],
        positions: vec![road_map
            .node_position(node_id)
            .expect("Corner-Node erwartet")],
        connected_neighbors: vec![Vec::new()],
        linear_stretches: vec![stretches],
        anchor_paths: Vec::new(),
    });
}

fn load_single_corner(tool: &mut RoundingTool, road_map: &RoadMap, node_id: u64) {
    load_single_corner_with_stretches(
        tool,
        road_map,
        node_id,
        single_step_stretch_seeds(road_map, node_id),
    );
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

fn arc_stretch_map() -> RoadMap {
    let mut road_map = RoadMap::new(3);
    road_map.add_node(MapNode::new(10, Vec2::new(-20.0, 0.0), NodeFlag::Regular));
    road_map.add_node(MapNode::new(11, Vec2::new(-10.0, 0.0), NodeFlag::Regular));
    road_map.add_node(MapNode::new(1, Vec2::ZERO, NodeFlag::Regular));
    road_map.add_node(MapNode::new(21, Vec2::new(0.0, 10.0), NodeFlag::Regular));
    road_map.add_node(MapNode::new(20, Vec2::new(0.0, 20.0), NodeFlag::Regular));
    road_map.add_connection(Connection::new(
        10,
        11,
        ConnectionDirection::Regular,
        ConnectionPriority::Regular,
        Vec2::new(-20.0, 0.0),
        Vec2::new(-10.0, 0.0),
    ));
    road_map.add_connection(Connection::new(
        11,
        1,
        ConnectionDirection::Regular,
        ConnectionPriority::Regular,
        Vec2::new(-10.0, 0.0),
        Vec2::ZERO,
    ));
    road_map.add_connection(Connection::new(
        1,
        21,
        ConnectionDirection::Regular,
        ConnectionPriority::Regular,
        Vec2::ZERO,
        Vec2::new(0.0, 10.0),
    ));
    road_map.add_connection(Connection::new(
        21,
        20,
        ConnectionDirection::Regular,
        ConnectionPriority::Regular,
        Vec2::new(0.0, 10.0),
        Vec2::new(0.0, 20.0),
    ));
    road_map
}

fn normalized_angle_delta(delta: f32) -> f32 {
    let mut wrapped = delta;
    if wrapped > std::f32::consts::PI {
        wrapped -= std::f32::consts::PI * 2.0;
    }
    if wrapped < -std::f32::consts::PI {
        wrapped += std::f32::consts::PI * 2.0;
    }
    wrapped
}

fn arc_segment_angles_deg(plan: &ArcPlan) -> Vec<f32> {
    plan.arc_positions
        .windows(2)
        .map(|pair| {
            let start_angle = (pair[0] - plan.center).to_angle();
            let end_angle = (pair[1] - plan.center).to_angle();
            normalized_angle_delta(end_angle - start_angle)
                .abs()
                .to_degrees()
        })
        .collect()
}

fn assert_even_arc_segments(plan: &ArcPlan, max_angle_deg: f32) {
    let segment_angles = arc_segment_angles_deg(plan);
    let expected_angle = plan.sweep_angle.abs().to_degrees() / segment_angles.len() as f32;

    for (index, angle) in segment_angles.iter().enumerate() {
        assert!(
            (*angle - expected_angle).abs() < 0.01,
            "Segment {index} sollte gleichmaessig {expected_angle}° gross sein, ist aber {angle}°"
        );
        assert!(
            *angle <= max_angle_deg + 0.01,
            "Segment {index} ueberschreitet max_angle_deg={max_angle_deg} mit {angle}°"
        );
    }
}

#[test]
fn arc_defaults_to_max_angle_deg_22_5() {
    let tool = RoundingTool::new();
    assert_eq!(tool.arc.max_angle_deg, DEFAULT_ARC_MAX_ANGLE_DEG);
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
fn arc_plan_segments_follow_max_angle_limit() {
    let road_map = simple_corner_map();
    let mut tool = RoundingTool::new();
    tool.arc.radius_m = 5.0;
    load_single_corner(&mut tool, &road_map, 1);

    let default_plan = tool.arc.plan.as_ref().expect("Arc-Plan erwartet");
    let default_positions = default_plan.arc_positions.len();
    assert_eq!(default_positions, 5, "90° / 22.5° ergibt 4 Segmente");
    assert_even_arc_segments(default_plan, DEFAULT_ARC_MAX_ANGLE_DEG);

    tool.arc.max_angle_deg = 45.0;
    tool.refresh_arc_state();
    let coarse_plan = tool.arc.plan.as_ref().expect("Grober Arc-Plan erwartet");
    let coarse_positions = coarse_plan.arc_positions.len();
    assert_eq!(coarse_positions, 3, "90° / 45° ergibt 2 Segmente");
    assert_even_arc_segments(coarse_plan, 45.0);

    tool.arc.max_angle_deg = 15.0;
    tool.refresh_arc_state();
    let fine_plan = tool.arc.plan.as_ref().expect("Feiner Arc-Plan erwartet");
    let fine_positions = fine_plan.arc_positions.len();
    assert_eq!(fine_positions, 7, "90° / 15° ergibt 6 Segmente");
    assert_even_arc_segments(fine_plan, 15.0);
}

#[test]
fn arc_plan_splits_non_divisible_corner_angle_evenly() {
    let plan = build_arc_plan_from_payload(
        Vec2::ZERO,
        10,
        Vec2::new(20.0, 0.0),
        20,
        Vec2::from_angle(100.0_f32.to_radians()) * 20.0,
        5.0,
        30.0,
    )
    .expect("100°-Corner muss fuer den Restfall testbar bleiben");

    let segment_angles = arc_segment_angles_deg(&plan);
    assert_eq!(
        segment_angles.len(),
        3,
        "80° Arc-Sweep / 30° muss auf 3 Segmente aufrunden"
    );
    assert_even_arc_segments(&plan, 30.0);
    let expected_angle = 80.0 / 3.0;
    for (index, angle) in segment_angles.iter().enumerate() {
        assert!(
            (*angle - expected_angle).abs() < 0.01,
            "Restfall-Segment {index} sollte {expected_angle}° statt ungleichmaessigem Rest haben, ist aber {angle}°"
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
fn arc_stretch_reaches_tangent_points_beyond_first_neighbor_edge() {
    let road_map = arc_stretch_map();
    let mut tool = RoundingTool::new();
    tool.arc.radius_m = 15.0;
    load_single_corner_with_stretches(
        &mut tool,
        &road_map,
        1,
        vec![
            stretch_seed(&road_map, 1, &[11, 10], true, false),
            stretch_seed(&road_map, 1, &[21, 20], false, true),
        ],
    );

    assert_eq!(tool.arc.validation, ArcValidation::Ready);
    let plan = tool.arc.plan.as_ref().expect("Stretch-Arc-Plan erwartet");
    assert_eq!(plan.first_side.anchor_id, 20);
    assert_eq!(plan.second_side.anchor_id, 10);
    assert_eq!(plan.first_side.consumed_node_ids, vec![21]);
    assert_eq!(plan.second_side.consumed_node_ids, vec![11]);

    let result = tool
        .execute(&road_map)
        .expect("Stretch-Arc-ToolResult erwartet");
    assert_eq!(result.nodes_to_remove, vec![21, 1, 11]);

    let mut connected_anchors: Vec<u64> = result
        .external_connections
        .iter()
        .map(|(_, anchor_id, _, _, _)| *anchor_id)
        .collect();
    connected_anchors.sort_unstable();
    assert_eq!(connected_anchors, vec![10, 20]);
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
        "Radius passt nicht in mindestens eine Anschlussstrecke des Corner-Pfads."
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
fn arc_payload_persists_max_angle_for_recreate() {
    let road_map = simple_corner_map();
    let mut tool = RoundingTool::new();
    tool.arc.radius_m = 5.0;
    tool.arc.max_angle_deg = 18.0;
    load_single_corner(&mut tool, &road_map, 1);

    assert!(tool.execute(&road_map).is_some());
    tool.on_applied(&[101, 102, 103], &road_map);

    let payload = tool.build_edit_payload().expect("Arc-Payload erwartet");
    match payload {
        RouteToolEditPayload::RoundingArc {
            radius_m,
            max_angle_deg,
            ..
        } => {
            assert_eq!(radius_m, 5.0);
            assert_eq!(max_angle_deg, 18.0);
        }
        other => panic!("unerwarteter Payload-Typ: {other:?}"),
    }
}

#[test]
fn arc_panel_change_marks_recreate_and_updates_payload() {
    let road_map = simple_corner_map();
    let mut tool = RoundingTool::new();
    tool.arc.radius_m = 5.0;
    tool.arc.max_angle_deg = 18.0;
    load_single_corner(&mut tool, &road_map, 1);

    assert!(tool.execute(&road_map).is_some());
    tool.on_applied(&[101, 102, 103], &road_map);

    let initial_payload = tool
        .build_edit_payload()
        .expect("Arc-Payload nach Apply erwartet");
    match initial_payload {
        RouteToolEditPayload::RoundingArc {
            radius_m,
            max_angle_deg,
            ..
        } => {
            assert_eq!(radius_m, 5.0);
            assert_eq!(max_angle_deg, tool.arc.max_angle_deg);
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
        RouteToolEditPayload::RoundingArc {
            radius_m,
            max_angle_deg,
            ..
        } => {
            assert_eq!(radius_m, 9.0);
            assert_eq!(max_angle_deg, 18.0);
        }
        other => panic!("unerwarteter Payload-Typ: {other:?}"),
    }
}

#[test]
fn arc_max_angle_panel_change_marks_recreate_updates_preview_and_payload() {
    let road_map = simple_corner_map();
    let mut tool = RoundingTool::new();
    tool.arc.radius_m = 5.0;
    tool.arc.max_angle_deg = 18.0;
    load_single_corner(&mut tool, &road_map, 1);

    assert_eq!(
        tool.arc
            .plan
            .as_ref()
            .expect("Arc-Plan erwartet")
            .arc_positions
            .len(),
        6,
        "90° / 18° ergibt 5 Segmente"
    );

    assert!(tool.execute(&road_map).is_some());
    tool.on_applied(&[101, 102, 103], &road_map);

    let effect = tool.apply_panel_action(RouteToolPanelAction::Rounding(
        RoundingPanelAction::SetMaxAngleDeg(30.0),
    ));
    assert!(effect.changed);
    assert!(effect.needs_recreate);
    assert!(tool.needs_recreate());
    assert_eq!(tool.arc.max_angle_deg, 30.0);
    assert_eq!(
        tool.arc
            .plan
            .as_ref()
            .expect("Aktualisierter Arc-Plan erwartet")
            .arc_positions
            .len(),
        4,
        "90° / 30° ergibt 3 Segmente"
    );

    let updated_payload = tool
        .build_edit_payload()
        .expect("Aktualisierter Arc-Payload erwartet");
    match updated_payload {
        RouteToolEditPayload::RoundingArc {
            radius_m,
            max_angle_deg,
            ..
        } => {
            assert_eq!(radius_m, 5.0);
            assert_eq!(max_angle_deg, 30.0);
        }
        other => panic!("unerwarteter Payload-Typ: {other:?}"),
    }
}

#[test]
fn arc_panel_change_clamps_max_angle_before_segmenting() {
    let road_map = simple_corner_map();
    let mut tool = RoundingTool::new();
    tool.arc.radius_m = 5.0;
    load_single_corner(&mut tool, &road_map, 1);

    let min_effect = tool.apply_panel_action(RouteToolPanelAction::Rounding(
        RoundingPanelAction::SetMaxAngleDeg(0.0),
    ));
    assert!(min_effect.changed);
    assert_eq!(tool.arc.max_angle_deg, MIN_ARC_MAX_ANGLE_DEG);
    assert_eq!(
        tool.arc
            .plan
            .as_ref()
            .expect("Feiner Arc-Plan erwartet")
            .arc_positions
            .len(),
        91,
        "90° / clamp(0°→1°) ergibt 90 Segmente"
    );

    let max_effect = tool.apply_panel_action(RouteToolPanelAction::Rounding(
        RoundingPanelAction::SetMaxAngleDeg(999.0),
    ));
    assert!(max_effect.changed);
    assert_eq!(tool.arc.max_angle_deg, MAX_ARC_MAX_ANGLE_DEG);
    assert_eq!(
        tool.arc
            .plan
            .as_ref()
            .expect("Grober Arc-Plan erwartet")
            .arc_positions
            .len(),
        3,
        "90° / clamp(999°→45°) ergibt 2 Segmente"
    );
}

#[test]
fn arc_plan_keeps_minimum_two_segments_for_small_supported_sweeps() {
    let plan = build_arc_plan_from_payload(
        Vec2::ZERO,
        10,
        Vec2::new(20.0, 0.0),
        20,
        Vec2::from_angle(174.5_f32.to_radians()) * 20.0,
        0.5,
        45.0,
    )
    .expect("174.5°-Corner muss knapp innerhalb des gueltigen Bereichs liegen");

    assert_eq!(
        plan.arc_positions.len(),
        3,
        "Auch kleine Arc-Sweeps behalten mindestens 2 Segmente"
    );
}

#[test]
fn arc_plan_rejects_corner_angles_just_outside_supported_limits() {
    let valid_min = build_arc_plan_from_payload(
        Vec2::ZERO,
        10,
        Vec2::new(20.0, 0.0),
        20,
        Vec2::from_angle(5.1_f32.to_radians()) * 20.0,
        0.5,
        45.0,
    );
    let invalid_min = build_arc_plan_from_payload(
        Vec2::ZERO,
        10,
        Vec2::new(20.0, 0.0),
        20,
        Vec2::from_angle(4.9_f32.to_radians()) * 20.0,
        0.5,
        45.0,
    );
    let valid_max = build_arc_plan_from_payload(
        Vec2::ZERO,
        10,
        Vec2::new(20.0, 0.0),
        20,
        Vec2::from_angle(178.9_f32.to_radians()) * 20.0,
        0.5,
        45.0,
    );
    let invalid_max = build_arc_plan_from_payload(
        Vec2::ZERO,
        10,
        Vec2::new(20.0, 0.0),
        20,
        Vec2::from_angle(179.1_f32.to_radians()) * 20.0,
        0.5,
        45.0,
    );

    assert!(valid_min.is_some(), "5.1° muss noch unterstuetzt werden");
    assert!(invalid_min.is_none(), "4.9° muss verworfen werden");
    assert!(valid_max.is_some(), "178.9° muss noch unterstuetzt werden");
    assert!(invalid_max.is_none(), "179.1° muss verworfen werden");
}

#[test]
fn restored_arc_payload_executes_without_original_corner_node() {
    let road_map = arc_stretch_map();
    let mut source_tool = RoundingTool::new();
    source_tool.arc.radius_m = 15.0;
    load_single_corner_with_stretches(
        &mut source_tool,
        &road_map,
        1,
        vec![
            stretch_seed(&road_map, 1, &[11, 10], true, false),
            stretch_seed(&road_map, 1, &[21, 20], false, true),
        ],
    );

    let original_result = source_tool
        .execute(&road_map)
        .expect("Originales Arc-Result erwartet");
    assert_eq!(original_result.nodes_to_remove, vec![21, 1, 11]);
    source_tool.on_applied(&[201, 202, 203], &road_map);

    let payload = source_tool
        .build_edit_payload()
        .expect("Persistierter Arc-Payload erwartet");
    match &payload {
        RouteToolEditPayload::RoundingArc {
            first_anchor_id,
            second_anchor_id,
            ..
        } => {
            let mut anchors = vec![*first_anchor_id, *second_anchor_id];
            anchors.sort_unstable();
            assert_eq!(anchors, vec![10, 20]);
        }
        other => panic!("unerwarteter Payload-Typ: {other:?}"),
    }

    let mut recreated_map = arc_stretch_map();
    recreated_map.remove_node(11);
    recreated_map.remove_node(21);
    recreated_map.remove_node(1);

    let mut restored_tool = RoundingTool::new();
    restored_tool.restore_edit_payload(&payload);

    assert_eq!(
        restored_tool.status_text(),
        "Nachbearbeitung — Radius oder Max-Winkel anpassen und Enter zum Neuaufbau druecken."
    );
    assert!(
        restored_tool
            .preview(Vec2::ZERO, &recreated_map)
            .nodes
            .len()
            >= 4
    );

    assert_eq!(
        restored_tool.arc.max_angle_deg,
        source_tool.arc.max_angle_deg
    );

    let recreated_result = restored_tool
        .execute(&recreated_map)
        .expect("Arc-Recreate-Result erwartet");
    assert!(recreated_result.nodes_to_remove.is_empty());
    assert_eq!(recreated_result.external_connections.len(), 2);
    assert!(!recreated_result.new_nodes.is_empty());
}

/// Erstellt einen minimalen Arc-Plan fuer einen synthetischen Corner mit dem angegebenen Winkel.
///
/// Legt den Corner in den Ursprung, den ersten Nachbar-Node auf der positiven X-Achse (Abstand 1.0)
/// und den zweiten Nachbar-Node so, dass der Corner-Winkel exakt `angle_deg` Grad betraegt.
/// Gibt `Ok(ArcPlan)` zurueck, wenn der Winkel unterstuetzt wird, sonst `Err(UnsupportedCornerAngle)`.
fn arc_plan_for_corner_angle_deg(angle_deg: f32) -> Result<ArcPlan, ArcValidation> {
    let second_pos = Vec2::from_angle(angle_deg.to_radians());
    build_arc_plan_from_payload(Vec2::ZERO, 1, Vec2::new(1.0, 0.0), 2, second_pos, 0.2, 22.5)
        .ok_or(ArcValidation::UnsupportedCornerAngle)
}

#[test]
fn arc_corner_170_degrees_is_valid() {
    // Non-Regression: 170° liegt deutlich innerhalb der alten Schwelle von 175°
    assert!(
        arc_plan_for_corner_angle_deg(170.0).is_ok(),
        "170°-Corner muss als Arc verrundbar sein"
    );
}

#[test]
fn arc_corner_175_degrees_is_valid() {
    // An der alten oberen Schwelle (MAX_CORNER_ANGLE_RAD = PI - 5°)
    assert!(
        arc_plan_for_corner_angle_deg(175.0).is_ok(),
        "175°-Corner liegt genau an der alten Schwelle und muss noch gueltig sein"
    );
}

#[test]
fn arc_corner_179_degrees_is_valid() {
    // Rot bis CP-02: Schwelle wird erst in CP-02 auf nahezu 180° angehoben
    assert!(
        arc_plan_for_corner_angle_deg(179.0).is_ok(),
        "179°-Corner muss nach CP-02 gueltig sein"
    );
}

#[test]
fn arc_corner_180_degrees_is_rejected() {
    // Perfekt gerade Strecke (kein Corner) — dauerhaft ungueltig
    assert_eq!(
        arc_plan_for_corner_angle_deg(180.0),
        Err(ArcValidation::UnsupportedCornerAngle),
        "180°-Corner (gerade Linie) muss dauerhaft abgelehnt werden"
    );
}
