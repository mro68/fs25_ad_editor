//! Geometrie und Validierung fuer die Arc- und Quadratic-Pfade des Verrundungs-Tools.

use super::state::{ArcOnePointState, QuadraticThreePointState};
use crate::app::tools::curve::geometry::{compute_curve_positions, quadratic_bezier};
use crate::app::tools::{
    RouteToolAnchorPathSeed, RouteToolConnectedNeighborSeed, RouteToolLinearStretchSeed,
};
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

/// Vorberechneter Quadratic-Plan fuer Preview und Execute.
#[derive(Debug, Clone, PartialEq)]
pub(crate) struct QuadraticPlan {
    pub start_outer_side: ArcSide,
    pub end_outer_side: ArcSide,
    pub start_anchor_path: QuadraticAnchorPath,
    pub end_anchor_path: QuadraticAnchorPath,
    pub start_node_id: u64,
    pub control_node_id: u64,
    pub end_node_id: u64,
    pub replace_path: Vec<u64>,
    pub control_point: Vec2,
    pub curve_positions: Vec<Vec2>,
}

/// Vorberechneter Anchor-Pfad einer Quadratic-Verrundung.
#[derive(Debug, Clone, PartialEq)]
pub(crate) struct QuadraticAnchorPath {
    pub node_ids: Vec<u64>,
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

#[derive(Debug, Clone, PartialEq)]
struct ArcStretchCut {
    anchor_id: u64,
    anchor_position: Vec2,
    consumed_node_ids: Vec<u64>,
    tangent_point: Vec2,
}

#[derive(Debug, Clone)]
struct QuadraticCandidatePlan {
    start_node_id: u64,
    control_node_id: u64,
    end_node_id: u64,
    start_to_control: RouteToolAnchorPathSeed,
    control_to_end: RouteToolAnchorPathSeed,
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
        arc.sample_spacing_m,
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
    sample_spacing_m: f32,
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
        sample_spacing_m,
    )
    .ok()
}

fn build_arc_plan_from_stretches(
    corner_position: Vec2,
    first_stretch: &RouteToolLinearStretchSeed,
    second_stretch: &RouteToolLinearStretchSeed,
    radius_m: f32,
    sample_spacing_m: f32,
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

pub(crate) fn recompute_quadratic_plan(
    quadratic: &QuadraticThreePointState,
) -> (QuadraticValidation, Option<QuadraticPlan>) {
    let selected_ids: HashSet<u64> = quadratic.selected_node_ids.iter().copied().collect();
    if selected_ids.len() != 3 {
        return (QuadraticValidation::NeedOrderedThreeNodeChain, None);
    }

    let Some(mut candidates) = build_quadratic_candidates(&quadratic.selected_anchor_paths) else {
        return (QuadraticValidation::BrokenSelectedChain, None);
    };
    candidates.sort_by_key(|candidate| {
        (
            !candidate_has_forward_path(candidate),
            candidate.start_node_id,
            candidate.end_node_id,
        )
    });

    let mut first_error = None;
    for candidate in candidates {
        match build_quadratic_plan_from_candidate(quadratic, &selected_ids, &candidate) {
            Ok(plan) => return (QuadraticValidation::Ready, Some(plan)),
            Err(validation) => {
                if first_error.is_none() {
                    first_error = Some(validation);
                }
            }
        }
    }

    (
        first_error.unwrap_or(QuadraticValidation::BrokenSelectedChain),
        None,
    )
}

fn build_quadratic_plan_from_candidate(
    quadratic: &QuadraticThreePointState,
    selected_ids: &HashSet<u64>,
    candidate: &QuadraticCandidatePlan,
) -> Result<QuadraticPlan, QuadraticValidation> {
    let Some(&start_position) = quadratic.selected_positions.get(&candidate.start_node_id) else {
        return Err(QuadraticValidation::MissingChainNodeContext);
    };
    let Some(&control_position) = quadratic.selected_positions.get(&candidate.control_node_id)
    else {
        return Err(QuadraticValidation::MissingChainNodeContext);
    };
    let Some(&end_position) = quadratic.selected_positions.get(&candidate.end_node_id) else {
        return Err(QuadraticValidation::MissingChainNodeContext);
    };

    if !selected_ids.contains(&candidate.start_node_id)
        || !selected_ids.contains(&candidate.control_node_id)
        || !selected_ids.contains(&candidate.end_node_id)
    {
        return Err(QuadraticValidation::BrokenSelectedChain);
    }

    let Some(start_neighbors) = quadratic.selected_neighbors.get(&candidate.start_node_id) else {
        return Err(QuadraticValidation::MissingChainNodeContext);
    };
    let Some(control_neighbors) = quadratic.selected_neighbors.get(&candidate.control_node_id)
    else {
        return Err(QuadraticValidation::MissingChainNodeContext);
    };
    let Some(end_neighbors) = quadratic.selected_neighbors.get(&candidate.end_node_id) else {
        return Err(QuadraticValidation::MissingChainNodeContext);
    };

    let Some(start_path_neighbor_id) = candidate.start_to_control.node_ids.get(1).copied() else {
        return Err(QuadraticValidation::BrokenSelectedChain);
    };
    let Some(control_from_start_neighbor_id) = candidate
        .start_to_control
        .node_ids
        .iter()
        .nth_back(1)
        .copied()
    else {
        return Err(QuadraticValidation::BrokenSelectedChain);
    };
    let Some(control_to_end_neighbor_id) = candidate.control_to_end.node_ids.get(1).copied() else {
        return Err(QuadraticValidation::BrokenSelectedChain);
    };
    let Some(end_path_neighbor_id) = candidate
        .control_to_end
        .node_ids
        .iter()
        .nth_back(1)
        .copied()
    else {
        return Err(QuadraticValidation::BrokenSelectedChain);
    };

    if control_from_start_neighbor_id == control_to_end_neighbor_id {
        return Err(QuadraticValidation::BrokenSelectedChain);
    }

    if find_side(start_neighbors, start_path_neighbor_id).is_none()
        || find_side(control_neighbors, control_from_start_neighbor_id).is_none()
        || find_side(control_neighbors, control_to_end_neighbor_id).is_none()
        || find_side(end_neighbors, end_path_neighbor_id).is_none()
    {
        return Err(QuadraticValidation::BrokenSelectedChain);
    }

    let start_outer_side = match resolve_outer_side(start_neighbors, &[start_path_neighbor_id]) {
        Ok(side) => side,
        Err(OuterSideError::Missing) => return Err(QuadraticValidation::MissingOuterStartStretch),
        Err(OuterSideError::Ambiguous) => {
            return Err(QuadraticValidation::AmbiguousOuterStartStretch)
        }
    };
    let end_outer_side = match resolve_outer_side(end_neighbors, &[end_path_neighbor_id]) {
        Ok(side) => side,
        Err(OuterSideError::Missing) => return Err(QuadraticValidation::MissingOuterEndStretch),
        Err(OuterSideError::Ambiguous) => {
            return Err(QuadraticValidation::AmbiguousOuterEndStretch)
        }
    };

    let control_path_neighbor_ids = [control_from_start_neighbor_id, control_to_end_neighbor_id];
    if collect_unique_sides(control_neighbors)
        .into_iter()
        .any(|side| !control_path_neighbor_ids.contains(&side.neighbor_id))
    {
        return Err(QuadraticValidation::ControlHasExternalConnections);
    }

    if start_position.distance(start_outer_side.neighbor_position) <= EPSILON
        || end_position.distance(end_outer_side.neighbor_position) <= EPSILON
    {
        return Err(QuadraticValidation::DegenerateOuterStretch);
    }

    let start_matches = start_supports_fixed_control(
        start_outer_side.neighbor_position,
        start_position,
        control_position,
    );
    let end_matches = end_supports_fixed_control(
        end_position,
        end_outer_side.neighbor_position,
        control_position,
    );
    if !start_matches || !end_matches {
        return Err(QuadraticValidation::TangentsMissFixedControl);
    }

    let has_forward = candidate_has_forward_path(candidate);
    let has_reverse =
        candidate.start_to_control.has_reverse_path && candidate.control_to_end.has_reverse_path;
    if !(has_forward || has_reverse) {
        return Err(QuadraticValidation::NoThroughPath);
    }

    let Some(replace_path) = combine_anchor_paths(
        &candidate.start_to_control.node_ids,
        &candidate.control_to_end.node_ids,
    ) else {
        return Err(QuadraticValidation::BrokenSelectedChain);
    };

    let curve_positions = build_quadratic_positions(
        start_position,
        control_position,
        end_position,
        quadratic.sample_spacing_m,
    );

    Ok(QuadraticPlan {
        start_outer_side,
        end_outer_side,
        start_anchor_path: QuadraticAnchorPath {
            node_ids: candidate.start_to_control.node_ids.clone(),
        },
        end_anchor_path: QuadraticAnchorPath {
            node_ids: candidate.control_to_end.node_ids.clone(),
        },
        start_node_id: candidate.start_node_id,
        control_node_id: candidate.control_node_id,
        end_node_id: candidate.end_node_id,
        replace_path,
        control_point: control_position,
        curve_positions,
    })
}

pub(crate) fn build_quadratic_plan_from_payload(
    road_map: &RoadMap,
    node_ids: [u64; 2],
    outer_neighbor_ids: [u64; 2],
    anchor_path_node_ids: [&[u64]; 2],
    control_point: Vec2,
    sample_spacing_m: f32,
) -> Option<QuadraticPlan> {
    let [start_node_id, end_node_id] = node_ids;
    let [start_outer_neighbor_id, end_outer_neighbor_id] = outer_neighbor_ids;
    let [start_control_path_node_ids, control_end_path_node_ids] = anchor_path_node_ids;

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

    let replace_path =
        combine_anchor_paths(start_control_path_node_ids, control_end_path_node_ids)?;
    let control_node_id = *start_control_path_node_ids.last()?;
    if control_end_path_node_ids.first().copied()? != control_node_id {
        return None;
    }

    Some(QuadraticPlan {
        start_outer_side,
        end_outer_side,
        start_anchor_path: QuadraticAnchorPath {
            node_ids: start_control_path_node_ids.to_vec(),
        },
        end_anchor_path: QuadraticAnchorPath {
            node_ids: control_end_path_node_ids.to_vec(),
        },
        start_node_id,
        control_node_id,
        end_node_id,
        replace_path,
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

pub(crate) fn collect_quadratic_transitions(
    road_map: &RoadMap,
    plan: &QuadraticPlan,
) -> Vec<ArcTransition> {
    collect_path_transitions(road_map, &plan.replace_path)
}

fn build_quadratic_candidates(
    anchor_paths: &[RouteToolAnchorPathSeed],
) -> Option<Vec<QuadraticCandidatePlan>> {
    let [first_path, second_path] = anchor_paths else {
        return None;
    };

    let (first_start, first_end) = path_endpoints(first_path)?;
    let (second_start, second_end) = path_endpoints(second_path)?;

    let mut endpoint_counts = HashMap::<u64, usize>::new();
    for node_id in [first_start, first_end, second_start, second_end] {
        *endpoint_counts.entry(node_id).or_default() += 1;
    }

    let control_node_id = endpoint_counts
        .iter()
        .find_map(|(&node_id, &count)| (count == 2).then_some(node_id))?;
    let first_end_node = path_outer_endpoint(first_path, control_node_id)?;
    let second_end_node = path_outer_endpoint(second_path, control_node_id)?;

    Some(vec![
        QuadraticCandidatePlan {
            start_node_id: first_end_node,
            control_node_id,
            end_node_id: second_end_node,
            start_to_control: orient_anchor_path(first_path, first_end_node, control_node_id)?,
            control_to_end: orient_anchor_path(second_path, control_node_id, second_end_node)?,
        },
        QuadraticCandidatePlan {
            start_node_id: second_end_node,
            control_node_id,
            end_node_id: first_end_node,
            start_to_control: orient_anchor_path(second_path, second_end_node, control_node_id)?,
            control_to_end: orient_anchor_path(first_path, control_node_id, first_end_node)?,
        },
    ])
}

fn path_endpoints(path: &RouteToolAnchorPathSeed) -> Option<(u64, u64)> {
    Some((
        path.node_ids.first().copied()?,
        path.node_ids.last().copied()?,
    ))
}

fn path_outer_endpoint(path: &RouteToolAnchorPathSeed, shared_node_id: u64) -> Option<u64> {
    let (path_start, path_end) = path_endpoints(path)?;
    match (path_start == shared_node_id, path_end == shared_node_id) {
        (true, false) => Some(path_end),
        (false, true) => Some(path_start),
        _ => None,
    }
}

fn orient_anchor_path(
    path: &RouteToolAnchorPathSeed,
    start_node_id: u64,
    end_node_id: u64,
) -> Option<RouteToolAnchorPathSeed> {
    let (path_start, path_end) = path_endpoints(path)?;
    if path_start == start_node_id && path_end == end_node_id {
        return Some(path.clone());
    }
    if path_start != end_node_id || path_end != start_node_id {
        return None;
    }

    let mut reversed = path.clone();
    reversed.node_ids.reverse();
    std::mem::swap(
        &mut reversed.has_forward_path,
        &mut reversed.has_reverse_path,
    );
    Some(reversed)
}

fn candidate_has_forward_path(candidate: &QuadraticCandidatePlan) -> bool {
    candidate.start_to_control.has_forward_path && candidate.control_to_end.has_forward_path
}

fn combine_anchor_paths(start_path: &[u64], end_path: &[u64]) -> Option<Vec<u64>> {
    if start_path.len() < 2 || end_path.len() < 2 {
        return None;
    }
    if start_path.last().copied()? != end_path.first().copied()? {
        return None;
    }

    let mut replace_path = start_path.to_vec();
    replace_path.extend(end_path.iter().skip(1).copied());
    Some(replace_path)
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
    excluded_neighbor_ids: &[u64],
) -> Result<ArcSide, OuterSideError> {
    let outer_sides: Vec<ArcSide> = collect_unique_sides(neighbors)
        .into_iter()
        .filter(|side| !excluded_neighbor_ids.contains(&side.neighbor_id))
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
