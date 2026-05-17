//! Geometrie und Validierung fuer die Arc- und Quadratic-Pfade des Verrundungs-Tools.

use super::state::{ArcOnePointState, QuadraticThreePointState};
use crate::app::tools::curve::geometry::{compute_curve_positions, quadratic_bezier};
use crate::app::tools::RouteToolConnectedNeighborSeed;
use crate::core::{ConnectionDirection, ConnectionPriority, RoadMap};
use glam::Vec2;
use std::collections::{HashMap, HashSet};

const MIN_CORNER_ANGLE_RAD: f32 = 5.0_f32.to_radians();
const MAX_CORNER_ANGLE_RAD: f32 = std::f32::consts::PI - 5.0_f32.to_radians();
const EPSILON: f32 = 1e-3;
const LINE_TOLERANCE_M: f32 = 0.05;

/// Eindeutiger Seitenkontext eines lokalen Replace-Pfads.
#[derive(Debug, Clone, Copy, PartialEq)]
pub(crate) struct ArcSide {
    pub neighbor_id: u64,
    pub neighbor_position: Vec2,
    pub angle: f32,
    pub has_incoming: bool,
    pub has_outgoing: bool,
}

/// Vorberechneter Arc-Plan fuer Preview und Execute.
#[derive(Debug, Clone, PartialEq)]
pub(crate) struct ArcPlan {
    pub first_side: ArcSide,
    pub second_side: ArcSide,
    pub tangent_first: Vec2,
    pub tangent_second: Vec2,
    pub center: Vec2,
    pub radius_m: f32,
    pub tangent_distance: f32,
    pub sweep_angle: f32,
    pub arc_positions: Vec<Vec2>,
}

/// Vorberechneter Quadratic-Plan fuer Preview und Execute.
#[derive(Debug, Clone, PartialEq)]
pub(crate) struct QuadraticPlan {
    pub start_outer_side: ArcSide,
    pub end_outer_side: ArcSide,
    pub start_node_id: u64,
    pub control_node_id: u64,
    pub end_node_id: u64,
    pub control_point: Vec2,
    pub curve_positions: Vec<Vec2>,
}

/// Klarer Invalid-/Ready-Zustand fuer den Arc-Modus.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum ArcValidation {
    NeedSingleSelection,
    MissingCornerPosition,
    NeedTwoRouteSides,
    AmbiguousJunction,
    NoThroughPath,
    DegenerateStretch,
    UnsupportedCornerAngle,
    RadiusTooLarge,
    Ready,
}

/// Klarer Invalid-/Ready-Zustand fuer den 3-Punkt-Quadratic-Modus.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum QuadraticValidation {
    NeedOrderedThreeNodeChain,
    MissingChainNodeContext,
    MissingOuterStartStretch,
    AmbiguousOuterStartStretch,
    MissingOuterEndStretch,
    AmbiguousOuterEndStretch,
    ControlHasExternalConnections,
    BrokenSelectedChain,
    DegenerateOuterStretch,
    TangentsMissFixedControl,
    NoThroughPath,
    Ready,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) struct ArcTransition {
    pub forward: bool,
    pub direction: ConnectionDirection,
    pub priority: ConnectionPriority,
}

pub(crate) fn recompute_arc_plan(arc: &ArcOnePointState) -> (ArcValidation, Option<ArcPlan>) {
    let Some(corner_position) = arc.corner_position else {
        return if arc.selected_node_ids.len() == 1 {
            (ArcValidation::MissingCornerPosition, None)
        } else {
            (ArcValidation::NeedSingleSelection, None)
        };
    };

    if arc.selected_node_ids.len() != 1 {
        return (ArcValidation::NeedSingleSelection, None);
    }

    let sides = collect_unique_sides(&arc.selected_neighbors);
    if sides.len() < 2 {
        return (ArcValidation::NeedTwoRouteSides, None);
    }
    if sides.len() > 2 {
        return (ArcValidation::AmbiguousJunction, None);
    }

    let first_side = sides[0];
    let second_side = sides[1];
    if !(first_side.has_incoming && second_side.has_outgoing
        || second_side.has_incoming && first_side.has_outgoing)
    {
        return (ArcValidation::NoThroughPath, None);
    }

    match build_arc_plan_from_sides(
        corner_position,
        first_side,
        second_side,
        arc.radius_m,
        arc.sample_spacing_m,
    ) {
        Ok(plan) => (ArcValidation::Ready, Some(plan)),
        Err(validation) => (validation, None),
    }
}

pub(crate) fn build_arc_plan_from_payload(
    corner_position: Vec2,
    first_neighbor_id: u64,
    first_neighbor_position: Vec2,
    second_neighbor_id: u64,
    second_neighbor_position: Vec2,
    radius_m: f32,
    sample_spacing_m: f32,
) -> Option<ArcPlan> {
    build_arc_plan_from_sides(
        corner_position,
        ArcSide {
            neighbor_id: first_neighbor_id,
            neighbor_position: first_neighbor_position,
            angle: (first_neighbor_position - corner_position).to_angle(),
            has_incoming: true,
            has_outgoing: true,
        },
        ArcSide {
            neighbor_id: second_neighbor_id,
            neighbor_position: second_neighbor_position,
            angle: (second_neighbor_position - corner_position).to_angle(),
            has_incoming: true,
            has_outgoing: true,
        },
        radius_m,
        sample_spacing_m,
    )
    .ok()
}

fn build_arc_plan_from_sides(
    corner_position: Vec2,
    first_side: ArcSide,
    second_side: ArcSide,
    radius_m: f32,
    sample_spacing_m: f32,
) -> Result<ArcPlan, ArcValidation> {
    let first_vec = first_side.neighbor_position - corner_position;
    let second_vec = second_side.neighbor_position - corner_position;
    let first_len = first_vec.length();
    let second_len = second_vec.length();
    if first_len <= EPSILON || second_len <= EPSILON {
        return Err(ArcValidation::DegenerateStretch);
    }

    let first_dir = first_vec / first_len;
    let second_dir = second_vec / second_len;
    let turn_angle = first_dir
        .perp_dot(second_dir)
        .atan2(first_dir.dot(second_dir));
    let corner_angle = turn_angle.abs();
    if !(MIN_CORNER_ANGLE_RAD..=MAX_CORNER_ANGLE_RAD).contains(&corner_angle) {
        return Err(ArcValidation::UnsupportedCornerAngle);
    }

    let half_angle = corner_angle * 0.5;
    let tangent_distance = radius_m / half_angle.tan();
    if !tangent_distance.is_finite() || tangent_distance <= EPSILON {
        return Err(ArcValidation::UnsupportedCornerAngle);
    }
    if tangent_distance >= first_len - EPSILON || tangent_distance >= second_len - EPSILON {
        return Err(ArcValidation::RadiusTooLarge);
    }

    let bisector = (first_dir + second_dir).normalize_or_zero();
    if bisector == Vec2::ZERO {
        return Err(ArcValidation::UnsupportedCornerAngle);
    }

    let center_distance = radius_m / half_angle.sin();
    if !center_distance.is_finite() || center_distance <= EPSILON {
        return Err(ArcValidation::UnsupportedCornerAngle);
    }

    let tangent_first = corner_position + first_dir * tangent_distance;
    let tangent_second = corner_position + second_dir * tangent_distance;
    let center = corner_position + bisector * center_distance;

    let start_angle = (tangent_first - center).to_angle();
    let sweep_angle = -turn_angle;
    let arc_length = radius_m * sweep_angle.abs();
    let segment_count = ((arc_length / sample_spacing_m.max(0.5)).ceil() as usize).max(2);
    let mut arc_positions = Vec::with_capacity(segment_count + 1);
    for step in 0..=segment_count {
        let t = step as f32 / segment_count as f32;
        let angle = start_angle + sweep_angle * t;
        arc_positions.push(center + Vec2::from_angle(angle) * radius_m);
    }
    if let Some(first) = arc_positions.first_mut() {
        *first = tangent_first;
    }
    if let Some(last) = arc_positions.last_mut() {
        *last = tangent_second;
    }

    Ok(ArcPlan {
        first_side,
        second_side,
        tangent_first,
        tangent_second,
        center,
        radius_m,
        tangent_distance,
        sweep_angle,
        arc_positions,
    })
}

pub(crate) fn recompute_quadratic_plan(
    quadratic: &QuadraticThreePointState,
) -> (QuadraticValidation, Option<QuadraticPlan>) {
    let [start_node_id, control_node_id, end_node_id] = quadratic.chain_node_ids.as_slice() else {
        return (QuadraticValidation::NeedOrderedThreeNodeChain, None);
    };
    let [start_position, control_position, end_position] = quadratic.chain_positions.as_slice()
    else {
        return (QuadraticValidation::NeedOrderedThreeNodeChain, None);
    };

    let selected_ids: HashSet<u64> = quadratic.selected_node_ids.iter().copied().collect();
    if selected_ids.len() != 3
        || !selected_ids.contains(start_node_id)
        || !selected_ids.contains(control_node_id)
        || !selected_ids.contains(end_node_id)
    {
        return (QuadraticValidation::NeedOrderedThreeNodeChain, None);
    }

    let Some(start_neighbors) = quadratic.selected_neighbors.get(start_node_id) else {
        return (QuadraticValidation::MissingChainNodeContext, None);
    };
    let Some(control_neighbors) = quadratic.selected_neighbors.get(control_node_id) else {
        return (QuadraticValidation::MissingChainNodeContext, None);
    };
    let Some(end_neighbors) = quadratic.selected_neighbors.get(end_node_id) else {
        return (QuadraticValidation::MissingChainNodeContext, None);
    };

    let start_outer_side = match resolve_outer_side(start_neighbors, &selected_ids) {
        Ok(side) => side,
        Err(OuterSideError::Missing) => {
            return (QuadraticValidation::MissingOuterStartStretch, None)
        }
        Err(OuterSideError::Ambiguous) => {
            return (QuadraticValidation::AmbiguousOuterStartStretch, None)
        }
    };
    let end_outer_side = match resolve_outer_side(end_neighbors, &selected_ids) {
        Ok(side) => side,
        Err(OuterSideError::Missing) => return (QuadraticValidation::MissingOuterEndStretch, None),
        Err(OuterSideError::Ambiguous) => {
            return (QuadraticValidation::AmbiguousOuterEndStretch, None)
        }
    };

    if collect_unique_sides(control_neighbors)
        .into_iter()
        .any(|side| !selected_ids.contains(&side.neighbor_id))
    {
        return (QuadraticValidation::ControlHasExternalConnections, None);
    }

    let Some(start_to_control) = find_side(start_neighbors, *control_node_id) else {
        return (QuadraticValidation::BrokenSelectedChain, None);
    };
    let Some(control_to_start) = find_side(control_neighbors, *start_node_id) else {
        return (QuadraticValidation::BrokenSelectedChain, None);
    };
    let Some(end_to_control) = find_side(end_neighbors, *control_node_id) else {
        return (QuadraticValidation::BrokenSelectedChain, None);
    };
    let Some(control_to_end) = find_side(control_neighbors, *end_node_id) else {
        return (QuadraticValidation::BrokenSelectedChain, None);
    };

    if start_position.distance(start_outer_side.neighbor_position) <= EPSILON
        || end_position.distance(end_outer_side.neighbor_position) <= EPSILON
    {
        return (QuadraticValidation::DegenerateOuterStretch, None);
    }

    let start_matches = start_supports_fixed_control(
        start_outer_side.neighbor_position,
        *start_position,
        *control_position,
    );
    let end_matches = end_supports_fixed_control(
        *end_position,
        end_outer_side.neighbor_position,
        *control_position,
    );
    if !start_matches || !end_matches {
        return (QuadraticValidation::TangentsMissFixedControl, None);
    }

    let has_forward = start_to_control.has_outgoing && control_to_end.has_outgoing;
    let has_reverse = end_to_control.has_outgoing && control_to_start.has_outgoing;
    if !(has_forward || has_reverse) {
        return (QuadraticValidation::NoThroughPath, None);
    }

    let curve_positions = build_quadratic_positions(
        *start_position,
        *control_position,
        *end_position,
        quadratic.sample_spacing_m,
    );

    (
        QuadraticValidation::Ready,
        Some(QuadraticPlan {
            start_outer_side,
            end_outer_side,
            start_node_id: *start_node_id,
            control_node_id: *control_node_id,
            end_node_id: *end_node_id,
            control_point: *control_position,
            curve_positions,
        }),
    )
}

pub(crate) fn build_quadratic_plan_from_payload(
    road_map: &RoadMap,
    start_node_id: u64,
    end_node_id: u64,
    start_outer_neighbor_id: u64,
    end_outer_neighbor_id: u64,
    control_point: Vec2,
    sample_spacing_m: f32,
) -> Option<QuadraticPlan> {
    let start_position = road_map.node_position(start_node_id)?;
    let end_position = road_map.node_position(end_node_id)?;
    let start_outer_side = payload_outer_side(road_map, start_node_id, start_outer_neighbor_id)?;
    let end_outer_side = payload_outer_side(road_map, end_node_id, end_outer_neighbor_id)?;

    if start_position.distance(control_point) <= EPSILON
        || end_position.distance(control_point) <= EPSILON
    {
        return None;
    }

    if start_position.distance(start_outer_side.neighbor_position) <= EPSILON
        || end_position.distance(end_outer_side.neighbor_position) <= EPSILON
    {
        return None;
    }

    let start_matches = start_supports_fixed_control(
        start_outer_side.neighbor_position,
        start_position,
        control_point,
    );
    let end_matches = end_supports_fixed_control(
        end_position,
        end_outer_side.neighbor_position,
        control_point,
    );
    if !start_matches || !end_matches {
        return None;
    }

    Some(QuadraticPlan {
        start_outer_side,
        end_outer_side,
        start_node_id,
        control_node_id: 0,
        end_node_id,
        control_point,
        curve_positions: build_quadratic_positions(
            start_position,
            control_point,
            end_position,
            sample_spacing_m,
        ),
    })
}

fn build_quadratic_positions(
    start_position: Vec2,
    control_point: Vec2,
    end_position: Vec2,
    sample_spacing_m: f32,
) -> Vec<Vec2> {
    let mut curve_positions = compute_curve_positions(
        |t| quadratic_bezier(start_position, control_point, end_position, t),
        sample_spacing_m.max(0.5),
    );
    if curve_positions.len() < 3 {
        curve_positions = vec![
            start_position,
            quadratic_bezier(start_position, control_point, end_position, 0.5),
            end_position,
        ];
    } else {
        if let Some(first) = curve_positions.first_mut() {
            *first = start_position;
        }
        if let Some(last) = curve_positions.last_mut() {
            *last = end_position;
        }
    }
    curve_positions
}

fn payload_outer_side(road_map: &RoadMap, node_id: u64, neighbor_id: u64) -> Option<ArcSide> {
    let neighbor_position = road_map.node_position(neighbor_id)?;
    let mut side = ArcSide {
        neighbor_id,
        neighbor_position,
        angle: (neighbor_position - road_map.node_position(node_id)?).to_angle(),
        has_incoming: false,
        has_outgoing: false,
    };
    let mut found = false;

    for neighbor in road_map.connected_neighbors(node_id) {
        if neighbor.neighbor_id != neighbor_id {
            continue;
        }

        found = true;
        side.angle = neighbor.angle;
        if neighbor.is_outgoing {
            side.has_outgoing = true;
        } else {
            side.has_incoming = true;
        }
    }

    found.then_some(side)
}

pub(crate) fn collect_transitions(
    road_map: &RoadMap,
    corner_id: u64,
    plan: &ArcPlan,
) -> Vec<ArcTransition> {
    collect_path_transitions(
        road_map,
        &[
            plan.first_side.neighbor_id,
            corner_id,
            plan.second_side.neighbor_id,
        ],
    )
}

pub(crate) fn collect_quadratic_transitions(
    road_map: &RoadMap,
    plan: &QuadraticPlan,
) -> Vec<ArcTransition> {
    collect_path_transitions(
        road_map,
        &[plan.start_node_id, plan.control_node_id, plan.end_node_id],
    )
}

fn collect_unique_sides(neighbors: &[RouteToolConnectedNeighborSeed]) -> Vec<ArcSide> {
    let mut by_neighbor: HashMap<u64, ArcSide> = HashMap::new();
    for neighbor in neighbors {
        let side = by_neighbor.entry(neighbor.neighbor_id).or_insert(ArcSide {
            neighbor_id: neighbor.neighbor_id,
            neighbor_position: neighbor.position,
            angle: neighbor.angle,
            has_incoming: false,
            has_outgoing: false,
        });
        side.neighbor_position = neighbor.position;
        side.angle = neighbor.angle;
        if neighbor.is_outgoing {
            side.has_outgoing = true;
        } else {
            side.has_incoming = true;
        }
    }

    let mut sides: Vec<ArcSide> = by_neighbor.into_values().collect();
    sides.sort_by(|left, right| left.angle.total_cmp(&right.angle));
    sides
}

fn find_side(neighbors: &[RouteToolConnectedNeighborSeed], neighbor_id: u64) -> Option<ArcSide> {
    collect_unique_sides(neighbors)
        .into_iter()
        .find(|side| side.neighbor_id == neighbor_id)
}

fn collect_path_transitions(road_map: &RoadMap, node_path: &[u64]) -> Vec<ArcTransition> {
    if node_path.len() < 2 {
        return Vec::new();
    }

    let mut transitions = Vec::new();
    if let Some((direction, priority)) = merge_path_connections(road_map, node_path) {
        transitions.push(ArcTransition {
            forward: true,
            direction,
            priority,
        });
    }

    let mut reverse_path = node_path.to_vec();
    reverse_path.reverse();
    if let Some((direction, priority)) = merge_path_connections(road_map, &reverse_path) {
        transitions.push(ArcTransition {
            forward: false,
            direction,
            priority,
        });
    }

    transitions
}

fn merge_path_connections(
    road_map: &RoadMap,
    node_path: &[u64],
) -> Option<(ConnectionDirection, ConnectionPriority)> {
    let mut direction = None;
    let mut priority = None;

    for pair in node_path.windows(2) {
        let segment = oriented_connections(road_map, pair[0], pair[1]);
        let (segment_direction, segment_priority) = merge_connection_set(&segment)?;
        direction = Some(match direction {
            Some(current) => merge_directions(current, segment_direction),
            None => segment_direction,
        });
        priority = Some(match priority {
            Some(current) => merge_priorities(current, segment_priority),
            None => segment_priority,
        });
    }

    Some((
        direction.expect("invariant: Pfad enthaelt mindestens ein Segment"),
        priority.expect("invariant: Pfad enthaelt mindestens ein Segment"),
    ))
}

fn oriented_connections(
    road_map: &RoadMap,
    start_id: u64,
    end_id: u64,
) -> Vec<(ConnectionDirection, ConnectionPriority)> {
    road_map
        .find_connections_between(start_id, end_id)
        .into_iter()
        .filter(|connection| connection.start_id == start_id && connection.end_id == end_id)
        .map(|connection| (connection.direction, connection.priority))
        .collect()
}

fn merge_connection_set(
    connections: &[(ConnectionDirection, ConnectionPriority)],
) -> Option<(ConnectionDirection, ConnectionPriority)> {
    if connections.is_empty() {
        return None;
    }

    let mut direction = connections[0].0;
    let mut priority = connections[0].1;
    for &(next_direction, next_priority) in connections.iter().skip(1) {
        direction = merge_directions(direction, next_direction);
        priority = merge_priorities(priority, next_priority);
    }

    Some((direction, priority))
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum OuterSideError {
    Missing,
    Ambiguous,
}

fn resolve_outer_side(
    neighbors: &[RouteToolConnectedNeighborSeed],
    selected_ids: &HashSet<u64>,
) -> Result<ArcSide, OuterSideError> {
    let outer_sides: Vec<ArcSide> = collect_unique_sides(neighbors)
        .into_iter()
        .filter(|side| !selected_ids.contains(&side.neighbor_id))
        .collect();
    match outer_sides.as_slice() {
        [] => Err(OuterSideError::Missing),
        [side] => Ok(*side),
        _ => Err(OuterSideError::Ambiguous),
    }
}

fn start_supports_fixed_control(outer_neighbor: Vec2, start: Vec2, control: Vec2) -> bool {
    line_distance(control, outer_neighbor, start) <= LINE_TOLERANCE_M
        && (control - start).dot(start - outer_neighbor) > EPSILON
}

fn end_supports_fixed_control(end: Vec2, outer_neighbor: Vec2, control: Vec2) -> bool {
    line_distance(control, end, outer_neighbor) <= LINE_TOLERANCE_M
        && (end - control).dot(outer_neighbor - end) > EPSILON
}

fn line_distance(point: Vec2, line_start: Vec2, line_end: Vec2) -> f32 {
    let line = line_end - line_start;
    let line_length = line.length();
    if line_length <= EPSILON {
        return f32::INFINITY;
    }
    (point - line_start).perp_dot(line / line_length).abs()
}

fn merge_directions(a: ConnectionDirection, b: ConnectionDirection) -> ConnectionDirection {
    match (a, b) {
        (ConnectionDirection::Dual, _) | (_, ConnectionDirection::Dual) => {
            ConnectionDirection::Dual
        }
        (left, right) if left == right => left,
        _ => ConnectionDirection::Dual,
    }
}

fn merge_priorities(a: ConnectionPriority, b: ConnectionPriority) -> ConnectionPriority {
    match (a, b) {
        (ConnectionPriority::SubPriority, ConnectionPriority::SubPriority) => {
            ConnectionPriority::SubPriority
        }
        _ => ConnectionPriority::Regular,
    }
}
