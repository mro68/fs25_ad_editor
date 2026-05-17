//! Geometrie und Validierung fuer den Arc-Pfad des Verrundungs-Tools.

use super::state::ArcOnePointState;
use crate::app::tools::RouteToolConnectedNeighborSeed;
use crate::core::{ConnectionDirection, ConnectionPriority, RoadMap};
use glam::Vec2;
use std::collections::HashMap;

const MIN_CORNER_ANGLE_RAD: f32 = 5.0_f32.to_radians();
const MAX_CORNER_ANGLE_RAD: f32 = std::f32::consts::PI - 5.0_f32.to_radians();
const EPSILON: f32 = 1e-3;

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

pub(crate) fn preview_direction(transitions: &[ArcTransition]) -> ConnectionDirection {
    let has_forward = transitions.iter().any(|transition| transition.forward);
    let has_reverse = transitions.iter().any(|transition| !transition.forward);
    match (has_forward, has_reverse) {
        (true, true) => ConnectionDirection::Dual,
        (true, false) => transitions
            .iter()
            .find(|transition| transition.forward)
            .map(|transition| transition.direction)
            .unwrap_or(ConnectionDirection::Regular),
        (false, true) => transitions
            .iter()
            .find(|transition| !transition.forward)
            .map(|transition| transition.direction)
            .unwrap_or(ConnectionDirection::Reverse),
        (false, false) => ConnectionDirection::Regular,
    }
}

pub(crate) fn preview_priority(transitions: &[ArcTransition]) -> ConnectionPriority {
    if transitions
        .iter()
        .all(|transition| transition.priority == ConnectionPriority::SubPriority)
    {
        ConnectionPriority::SubPriority
    } else {
        ConnectionPriority::Regular
    }
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
