//! Geometrie und Validierung fuer den Arc-only-Pfad des Verrundungs-Tools.

use super::state::{clamp_arc_max_angle_deg, ArcOnePointState};
use crate::app::tools::RouteToolLinearStretchSeed;
use crate::core::{ConnectionDirection, ConnectionPriority, RoadMap};
use glam::Vec2;

const MIN_CORNER_ANGLE_RAD: f32 = 5.0_f32.to_radians();
const MAX_CORNER_ANGLE_RAD: f32 = std::f32::consts::PI - 5.0_f32.to_radians();
const EPSILON: f32 = 1e-3;

/// Vorberechneter Arc-Anker eines lokalen Replace-Pfads.
#[derive(Debug, Clone, PartialEq)]
pub(crate) struct ArcSidePlan {
    pub anchor_id: u64,
    pub anchor_position: Vec2,
    pub consumed_node_ids: Vec<u64>,
}

/// Vorberechneter Arc-Plan fuer Preview und Execute.
#[derive(Debug, Clone, PartialEq)]
pub(crate) struct ArcPlan {
    pub first_side: ArcSidePlan,
    pub second_side: ArcSidePlan,
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

#[derive(Debug, Clone, PartialEq)]
struct ArcStretchCut {
    anchor_id: u64,
    anchor_position: Vec2,
    consumed_node_ids: Vec<u64>,
    tangent_point: Vec2,
}

#[derive(Debug, Clone, Copy, PartialEq)]
struct ArcSegmentation {
    segment_count: usize,
    segment_angle: f32,
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

    if arc.selected_stretches.len() < 2 {
        return (ArcValidation::NeedTwoRouteSides, None);
    }
    if arc.selected_stretches.len() > 2 {
        return (ArcValidation::AmbiguousJunction, None);
    }

    let first_stretch = &arc.selected_stretches[0];
    let second_stretch = &arc.selected_stretches[1];
    if !(first_stretch.has_incoming && second_stretch.has_outgoing
        || second_stretch.has_incoming && first_stretch.has_outgoing)
    {
        return (ArcValidation::NoThroughPath, None);
    }

    match build_arc_plan_from_stretches(
        corner_position,
        first_stretch,
        second_stretch,
        arc.radius_m,
        arc.max_angle_deg,
    ) {
        Ok(plan) => (ArcValidation::Ready, Some(plan)),
        Err(validation) => (validation, None),
    }
}

pub(crate) fn build_arc_plan_from_payload(
    corner_position: Vec2,
    first_anchor_id: u64,
    first_anchor_position: Vec2,
    second_anchor_id: u64,
    second_anchor_position: Vec2,
    radius_m: f32,
    max_angle_deg: f32,
) -> Option<ArcPlan> {
    let first_stretch = RouteToolLinearStretchSeed {
        node_ids: vec![first_anchor_id],
        positions: vec![first_anchor_position],
        angle: (first_anchor_position - corner_position).to_angle(),
        has_incoming: true,
        has_outgoing: true,
    };
    let second_stretch = RouteToolLinearStretchSeed {
        node_ids: vec![second_anchor_id],
        positions: vec![second_anchor_position],
        angle: (second_anchor_position - corner_position).to_angle(),
        has_incoming: true,
        has_outgoing: true,
    };

    build_arc_plan_from_stretches(
        corner_position,
        &first_stretch,
        &second_stretch,
        radius_m,
        max_angle_deg,
    )
    .ok()
}

fn build_arc_plan_from_stretches(
    corner_position: Vec2,
    first_stretch: &RouteToolLinearStretchSeed,
    second_stretch: &RouteToolLinearStretchSeed,
    radius_m: f32,
    max_angle_deg: f32,
) -> Result<ArcPlan, ArcValidation> {
    let Some(first_position) = first_stretch.positions.first().copied() else {
        return Err(ArcValidation::DegenerateStretch);
    };
    let Some(second_position) = second_stretch.positions.first().copied() else {
        return Err(ArcValidation::DegenerateStretch);
    };

    let first_vec = first_position - corner_position;
    let second_vec = second_position - corner_position;
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

    let bisector = (first_dir + second_dir).normalize_or_zero();
    if bisector == Vec2::ZERO {
        return Err(ArcValidation::UnsupportedCornerAngle);
    }

    let center_distance = radius_m / half_angle.sin();
    if !center_distance.is_finite() || center_distance <= EPSILON {
        return Err(ArcValidation::UnsupportedCornerAngle);
    }

    let first_cut = cut_stretch_at_distance(corner_position, first_stretch, tangent_distance)?;
    let second_cut = cut_stretch_at_distance(corner_position, second_stretch, tangent_distance)?;

    let tangent_first = first_cut.tangent_point;
    let tangent_second = second_cut.tangent_point;
    let center = corner_position + bisector * center_distance;

    let start_angle = (tangent_first - center).to_angle();
    let end_angle = (tangent_second - center).to_angle();
    let sweep_angle = normalized_sweep_angle(start_angle, end_angle);
    let segmentation = arc_segmentation(sweep_angle, max_angle_deg);
    let mut arc_positions = Vec::with_capacity(segmentation.segment_count + 1);
    for step in 0..=segmentation.segment_count {
        let angle = start_angle + segmentation.segment_angle * step as f32;
        arc_positions.push(center + Vec2::from_angle(angle) * radius_m);
    }
    if let Some(first) = arc_positions.first_mut() {
        *first = tangent_first;
    }
    if let Some(last) = arc_positions.last_mut() {
        *last = tangent_second;
    }

    Ok(ArcPlan {
        first_side: ArcSidePlan {
            anchor_id: first_cut.anchor_id,
            anchor_position: first_cut.anchor_position,
            consumed_node_ids: first_cut.consumed_node_ids,
        },
        second_side: ArcSidePlan {
            anchor_id: second_cut.anchor_id,
            anchor_position: second_cut.anchor_position,
            consumed_node_ids: second_cut.consumed_node_ids,
        },
        tangent_first,
        tangent_second,
        center,
        radius_m,
        tangent_distance,
        sweep_angle,
        arc_positions,
    })
}

fn arc_segmentation(sweep_angle: f32, max_angle_deg: f32) -> ArcSegmentation {
    let max_angle_rad = clamp_arc_max_angle_deg(max_angle_deg).to_radians();
    let segment_count = ((sweep_angle.abs() / max_angle_rad).ceil() as usize).max(2);
    ArcSegmentation {
        segment_count,
        segment_angle: sweep_angle / segment_count as f32,
    }
}

fn normalized_sweep_angle(start_angle: f32, end_angle: f32) -> f32 {
    let mut delta = end_angle - start_angle;
    if delta > std::f32::consts::PI {
        delta -= std::f32::consts::PI * 2.0;
    }
    if delta < -std::f32::consts::PI {
        delta += std::f32::consts::PI * 2.0;
    }
    delta
}

fn cut_stretch_at_distance(
    corner_position: Vec2,
    stretch: &RouteToolLinearStretchSeed,
    tangent_distance: f32,
) -> Result<ArcStretchCut, ArcValidation> {
    if stretch.node_ids.is_empty() || stretch.node_ids.len() != stretch.positions.len() {
        return Err(ArcValidation::DegenerateStretch);
    }

    let mut remaining = tangent_distance;
    let mut previous_position = corner_position;
    let mut consumed_node_ids = Vec::new();

    for (node_id, node_position) in stretch.node_ids.iter().zip(&stretch.positions) {
        let segment = *node_position - previous_position;
        let segment_length = segment.length();
        if segment_length <= EPSILON {
            return Err(ArcValidation::DegenerateStretch);
        }

        if remaining < segment_length - EPSILON {
            let tangent_point = previous_position + segment / segment_length * remaining;
            return Ok(ArcStretchCut {
                anchor_id: *node_id,
                anchor_position: *node_position,
                consumed_node_ids,
                tangent_point,
            });
        }

        remaining -= segment_length;
        consumed_node_ids.push(*node_id);
        previous_position = *node_position;
    }

    Err(ArcValidation::RadiusTooLarge)
}

pub(crate) fn collect_transitions(
    road_map: &RoadMap,
    corner_id: u64,
    plan: &ArcPlan,
) -> Vec<ArcTransition> {
    collect_path_transitions(road_map, &arc_replace_path(corner_id, plan))
}

pub(crate) fn arc_replace_path(corner_id: u64, plan: &ArcPlan) -> Vec<u64> {
    let mut node_path = Vec::with_capacity(
        plan.first_side.consumed_node_ids.len() + plan.second_side.consumed_node_ids.len() + 3,
    );
    node_path.push(plan.first_side.anchor_id);
    node_path.extend(plan.first_side.consumed_node_ids.iter().rev().copied());
    node_path.push(corner_id);
    node_path.extend(plan.second_side.consumed_node_ids.iter().copied());
    node_path.push(plan.second_side.anchor_id);
    node_path
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
