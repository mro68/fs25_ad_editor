use crate::app::state::EditorTool;
use crate::app::tool_contract::RouteToolId;
use crate::app::tools::{
    route_tool_descriptor, OrderedNodeChain, RouteToolAnchorPathSeed,
    RouteToolConnectedNeighborSeed, RouteToolLinearStretchSeed, RouteToolSelectionSeed, ToolAction,
};
use crate::app::AppState;

const STRAIGHT_STRETCH_MIN_DOT: f32 = 0.999;

/// Aktiviert ein Route-Tool per stabiler Tool-ID.
pub(super) fn select(state: &mut AppState, tool_id: RouteToolId) {
    let descriptor = route_tool_descriptor(tool_id);
    state.editor.tool_manager.set_active_by_id(tool_id);
    state.editor.remember_route_tool(descriptor.group, tool_id);
    state.editor.active_tool = EditorTool::Route;
    state.editor.connect_source_node = None;
    let host_context = super::build_host_context(state);
    state.editor.tool_manager.sync_active_host(&host_context);

    init_chain_if_needed(state);
    init_selection_if_needed(state);

    log::info!("Route-Tool aktiviert: {:?}", tool_id);
}

/// Laedt die aktuelle Selektion als geordnete Kette in das aktive Tool,
/// falls dieses `needs_chain_input()` zurueckgibt.
pub(super) fn init_chain_if_needed(state: &mut AppState) {
    if state.editor.tool_manager.active_chain_input().is_none() {
        return;
    }

    let Some(road_map) = state.road_map.as_deref() else {
        return;
    };

    let Some(ordered_ids) = road_map.ordered_chain_nodes(&state.selection.selected_node_ids) else {
        log::debug!("Bypass-Tool: Selektion bildet keine lineare Kette");
        return;
    };

    let positions: Vec<glam::Vec2> = ordered_ids
        .iter()
        .filter_map(|id| road_map.node(*id).map(|n| n.position))
        .collect();

    if positions.len() < 2 {
        return;
    }

    let start_id = *ordered_ids
        .first()
        .expect("invariant: ordered_ids ist nicht-leer nach positions.len()<2-Guard");
    let end_id = *ordered_ids
        .last()
        .expect("invariant: ordered_ids ist nicht-leer nach positions.len()<2-Guard");

    let inner_ids = if ordered_ids.len() > 2 {
        ordered_ids[1..ordered_ids.len() - 1].to_vec()
    } else {
        Vec::new()
    };

    state
        .editor
        .tool_manager
        .load_active_chain(OrderedNodeChain {
            positions,
            start_id,
            end_id,
            inner_ids,
        });
    log::info!(
        "Route-Tool Kette geladen: {} Nodes ({} → {})",
        ordered_ids.len(),
        start_id,
        end_id
    );
}

/// Laedt die aktuelle Node-Selektion in das aktive Tool,
/// falls dieses `RouteToolSelectionInput` bereitstellt.
pub(super) fn init_selection_if_needed(state: &mut AppState) {
    if state.editor.tool_manager.active_selection_input().is_none() {
        return;
    }

    let Some(road_map) = state.road_map.as_deref() else {
        return;
    };

    let node_ids: Vec<u64> = state.selection.selected_node_ids.iter().copied().collect();
    let mut positions = Vec::with_capacity(node_ids.len());
    let mut connected_neighbors = Vec::with_capacity(node_ids.len());
    let mut linear_stretches = Vec::with_capacity(node_ids.len());

    for node_id in &node_ids {
        let node_position = road_map.node_position(*node_id);
        if let Some(position) = node_position {
            positions.push(position);
        }

        let neighbors = build_connected_neighbor_seeds(road_map, *node_id);
        let stretches = node_position
            .map(|position| build_linear_stretches(road_map, *node_id, position, &neighbors))
            .unwrap_or_default();

        connected_neighbors.push(neighbors);
        linear_stretches.push(stretches);
    }

    let anchor_paths = build_anchor_paths(road_map, &node_ids);

    if positions.len() != node_ids.len() {
        log::warn!(
            "Route-Tool Selektion enthaelt unbekannte Nodes: {} IDs, {} Positionen",
            node_ids.len(),
            positions.len()
        );
    }

    state
        .editor
        .tool_manager
        .load_active_selection(RouteToolSelectionSeed {
            node_ids,
            positions,
            connected_neighbors,
            linear_stretches,
            anchor_paths,
        });
}

fn build_connected_neighbor_seeds(
    road_map: &crate::core::RoadMap,
    node_id: u64,
) -> Vec<RouteToolConnectedNeighborSeed> {
    road_map
        .connected_neighbors(node_id)
        .into_iter()
        .filter_map(|neighbor| {
            let position = road_map.node_position(neighbor.neighbor_id)?;
            Some(RouteToolConnectedNeighborSeed::new(neighbor, position))
        })
        .collect()
}

fn build_linear_stretches(
    road_map: &crate::core::RoadMap,
    corner_id: u64,
    corner_position: glam::Vec2,
    neighbors: &[RouteToolConnectedNeighborSeed],
) -> Vec<RouteToolLinearStretchSeed> {
    #[derive(Clone, Copy)]
    struct StretchHeader {
        first_neighbor_id: u64,
        first_position: glam::Vec2,
        angle: f32,
        has_incoming: bool,
        has_outgoing: bool,
    }

    let mut by_neighbor = std::collections::HashMap::<u64, StretchHeader>::new();
    for neighbor in neighbors {
        let stretch = by_neighbor
            .entry(neighbor.neighbor_id)
            .or_insert(StretchHeader {
                first_neighbor_id: neighbor.neighbor_id,
                first_position: neighbor.position,
                angle: neighbor.angle,
                has_incoming: false,
                has_outgoing: false,
            });
        stretch.first_position = neighbor.position;
        stretch.angle = neighbor.angle;
        if neighbor.is_outgoing {
            stretch.has_outgoing = true;
        } else {
            stretch.has_incoming = true;
        }
    }

    let mut headers: Vec<StretchHeader> = by_neighbor.into_values().collect();
    headers.sort_by(|left, right| left.angle.total_cmp(&right.angle));

    headers
        .into_iter()
        .map(|header| {
            let mut node_ids = vec![header.first_neighbor_id];
            let mut positions = vec![header.first_position];
            let mut previous_id = corner_id;
            let mut previous_position = corner_position;
            let mut current_id = header.first_neighbor_id;
            let mut current_position = header.first_position;

            loop {
                let next_neighbor_ids = unique_next_neighbor_ids(road_map, current_id, previous_id);
                let [next_id] = next_neighbor_ids.as_slice() else {
                    break;
                };
                let Some(next_position) = road_map.node_position(*next_id) else {
                    break;
                };
                if !is_colinear_continuation(previous_position, current_position, next_position) {
                    break;
                }

                node_ids.push(*next_id);
                positions.push(next_position);
                previous_id = current_id;
                previous_position = current_position;
                current_id = *next_id;
                current_position = next_position;
            }

            RouteToolLinearStretchSeed {
                node_ids,
                positions,
                angle: header.angle,
                has_incoming: header.has_incoming,
                has_outgoing: header.has_outgoing,
            }
        })
        .collect()
}

fn unique_next_neighbor_ids(
    road_map: &crate::core::RoadMap,
    node_id: u64,
    previous_id: u64,
) -> Vec<u64> {
    let mut next_ids = Vec::new();
    for &(neighbor_id, _) in road_map.neighbors(node_id) {
        if neighbor_id == previous_id || next_ids.contains(&neighbor_id) {
            continue;
        }
        next_ids.push(neighbor_id);
    }
    next_ids
}

fn build_anchor_paths(
    road_map: &crate::core::RoadMap,
    selected_node_ids: &[u64],
) -> Vec<RouteToolAnchorPathSeed> {
    if selected_node_ids.len() != 3 {
        return Vec::new();
    }

    let selected_ids: std::collections::HashSet<u64> = selected_node_ids.iter().copied().collect();
    if selected_ids.len() != 3 {
        return Vec::new();
    }

    let mut pair_paths = Vec::<((u64, u64), RouteToolAnchorPathSeed)>::new();
    for start_index in 0..selected_node_ids.len() {
        for end_index in start_index + 1..selected_node_ids.len() {
            let start_id = selected_node_ids[start_index];
            let end_id = selected_node_ids[end_index];
            match resolve_anchor_path_between(road_map, start_id, end_id, &selected_ids) {
                Ok(Some(path)) => pair_paths.push(((start_id, end_id), path)),
                Ok(None) => {}
                Err(()) => return Vec::new(),
            }
        }
    }

    if pair_paths.len() != 2 {
        return Vec::new();
    }

    let mut degrees = std::collections::HashMap::<u64, usize>::new();
    for ((start_id, end_id), _) in &pair_paths {
        *degrees.entry(*start_id).or_default() += 1;
        *degrees.entry(*end_id).or_default() += 1;
    }

    let Some(control_id) = degrees
        .iter()
        .find_map(|(&node_id, &degree)| (degree == 2).then_some(node_id))
    else {
        return Vec::new();
    };

    if degrees.len() != 3 || degrees.values().filter(|&&degree| degree == 1).count() != 2 {
        return Vec::new();
    }

    if !pair_paths
        .iter()
        .any(|((left_id, right_id), _)| *left_id == control_id || *right_id == control_id)
    {
        return Vec::new();
    }

    pair_paths.into_iter().map(|(_, path)| path).collect()
}

fn resolve_anchor_path_between(
    road_map: &crate::core::RoadMap,
    start_id: u64,
    end_id: u64,
    selected_ids: &std::collections::HashSet<u64>,
) -> Result<Option<RouteToolAnchorPathSeed>, ()> {
    let mut direct_neighbors = Vec::new();
    for &(neighbor_id, _) in road_map.neighbors(start_id) {
        if !direct_neighbors.contains(&neighbor_id) {
            direct_neighbors.push(neighbor_id);
        }
    }

    let mut found_paths = Vec::<Vec<u64>>::new();
    for neighbor_id in direct_neighbors {
        let Some(path) = follow_anchor_path_branch(road_map, start_id, neighbor_id, selected_ids)
        else {
            continue;
        };
        if path.last().copied() != Some(end_id) {
            continue;
        }

        if !found_paths.is_empty() {
            return Err(());
        }
        found_paths.push(path);
    }

    let Some(node_ids) = found_paths.into_iter().next() else {
        return Ok(None);
    };

    let mut reverse_node_ids = node_ids.clone();
    reverse_node_ids.reverse();

    Ok(Some(RouteToolAnchorPathSeed {
        has_forward_path: path_is_traversable(road_map, &node_ids),
        has_reverse_path: path_is_traversable(road_map, &reverse_node_ids),
        node_ids,
    }))
}

fn follow_anchor_path_branch(
    road_map: &crate::core::RoadMap,
    start_id: u64,
    first_neighbor_id: u64,
    selected_ids: &std::collections::HashSet<u64>,
) -> Option<Vec<u64>> {
    let mut path = vec![start_id, first_neighbor_id];
    let mut previous_id = start_id;
    let mut current_id = first_neighbor_id;

    loop {
        if selected_ids.contains(&current_id) && current_id != start_id {
            return Some(path);
        }

        let next_neighbor_ids = unique_next_neighbor_ids(road_map, current_id, previous_id);
        let [next_id] = next_neighbor_ids.as_slice() else {
            return None;
        };
        if path.contains(next_id) {
            return None;
        }

        path.push(*next_id);
        previous_id = current_id;
        current_id = *next_id;
    }
}

fn path_is_traversable(road_map: &crate::core::RoadMap, node_ids: &[u64]) -> bool {
    node_ids
        .windows(2)
        .all(|segment| road_map.has_connection(segment[0], segment[1]))
}

fn is_colinear_continuation(
    previous_position: glam::Vec2,
    current_position: glam::Vec2,
    next_position: glam::Vec2,
) -> bool {
    let incoming = current_position - previous_position;
    let outgoing = next_position - current_position;
    let incoming_length = incoming.length();
    let outgoing_length = outgoing.length();
    if incoming_length <= f32::EPSILON || outgoing_length <= f32::EPSILON {
        return false;
    }

    let incoming_dir = incoming / incoming_length;
    let outgoing_dir = outgoing / outgoing_length;
    incoming_dir.dot(outgoing_dir) >= STRAIGHT_STRETCH_MIN_DOT
}

/// Aktiviert ein Route-Tool und setzt Start/End-Anker aus zwei selektierten Nodes.
pub(super) fn select_with_anchors(
    state: &mut AppState,
    tool_id: RouteToolId,
    start_node_id: u64,
    end_node_id: u64,
) {
    select(state, tool_id);

    if let Some(tool) = state.editor.tool_manager.active_tool_mut() {
        tool.reset();
    }

    let (start_pos, end_pos) = {
        let Some(road_map) = state.road_map.as_deref() else {
            return;
        };
        let start = road_map.node(start_node_id);
        let end = road_map.node(end_node_id);
        match (start, end) {
            (Some(s), Some(e)) => (s.position, e.position),
            _ => {
                log::warn!(
                    "Route-Tool mit Ankern: Nodes {}/{} nicht gefunden",
                    start_node_id,
                    end_node_id
                );
                return;
            }
        }
    };

    let (old_selected, old_anchor) =
        crate::app::handlers::helpers::capture_selection_snapshot(state);
    state.selection.ids_mut().clear();
    crate::app::handlers::helpers::record_selection_if_changed(state, old_selected, old_anchor);

    let action1 = {
        let Some(road_map) = state.road_map.as_deref() else {
            return;
        };
        let Some(tool) = state.editor.tool_manager.active_tool_mut() else {
            return;
        };
        tool.on_click(start_pos, road_map, false)
    };

    if action1 == ToolAction::ReadyToExecute {
        super::apply::execute_and_apply(state);
        return;
    }

    let action2 = {
        let Some(road_map) = state.road_map.as_deref() else {
            return;
        };
        let Some(tool) = state.editor.tool_manager.active_tool_mut() else {
            return;
        };
        tool.on_click(end_pos, road_map, false)
    };

    if action2 == ToolAction::ReadyToExecute {
        super::apply::execute_and_apply(state);
    }
}

#[cfg(test)]
mod tests {
    use super::build_anchor_paths;
    use crate::core::{
        Connection, ConnectionDirection, ConnectionPriority, MapNode, NodeFlag, RoadMap,
    };
    use glam::Vec2;

    fn unique_anchor_span_map() -> RoadMap {
        let mut road_map = RoadMap::new(3);
        road_map.add_node(MapNode::new(10, Vec2::new(-20.0, 0.0), NodeFlag::Regular));
        road_map.add_node(MapNode::new(1, Vec2::new(-10.0, 0.0), NodeFlag::Regular));
        road_map.add_node(MapNode::new(2, Vec2::new(-5.0, 0.0), NodeFlag::Regular));
        road_map.add_node(MapNode::new(3, Vec2::ZERO, NodeFlag::Regular));
        road_map.add_node(MapNode::new(4, Vec2::new(5.0, 5.0), NodeFlag::Regular));
        road_map.add_node(MapNode::new(5, Vec2::new(10.0, 10.0), NodeFlag::Regular));
        road_map.add_node(MapNode::new(30, Vec2::new(20.0, 20.0), NodeFlag::Regular));
        for (start_id, end_id, start_pos, end_pos) in [
            (10, 1, Vec2::new(-20.0, 0.0), Vec2::new(-10.0, 0.0)),
            (1, 2, Vec2::new(-10.0, 0.0), Vec2::new(-5.0, 0.0)),
            (2, 3, Vec2::new(-5.0, 0.0), Vec2::ZERO),
            (3, 4, Vec2::ZERO, Vec2::new(5.0, 5.0)),
            (4, 5, Vec2::new(5.0, 5.0), Vec2::new(10.0, 10.0)),
            (5, 30, Vec2::new(10.0, 10.0), Vec2::new(20.0, 20.0)),
        ] {
            road_map.add_connection(Connection::new(
                start_id,
                end_id,
                ConnectionDirection::Regular,
                ConnectionPriority::Regular,
                start_pos,
                end_pos,
            ));
        }
        road_map
    }

    fn ambiguous_anchor_span_map() -> RoadMap {
        let mut road_map = unique_anchor_span_map();
        road_map.add_node(MapNode::new(6, Vec2::new(-5.0, -5.0), NodeFlag::Regular));
        road_map.add_connection(Connection::new(
            1,
            6,
            ConnectionDirection::Regular,
            ConnectionPriority::Regular,
            Vec2::new(-10.0, 0.0),
            Vec2::new(-5.0, -5.0),
        ));
        road_map.add_connection(Connection::new(
            6,
            3,
            ConnectionDirection::Regular,
            ConnectionPriority::Regular,
            Vec2::new(-5.0, -5.0),
            Vec2::ZERO,
        ));
        road_map
    }

    #[test]
    fn build_anchor_paths_resolves_intermediate_nodes_between_selected_anchors() {
        let road_map = unique_anchor_span_map();
        let mut node_paths: Vec<Vec<u64>> = build_anchor_paths(&road_map, &[1, 3, 5])
            .into_iter()
            .map(|path| path.node_ids)
            .collect();
        node_paths.sort();

        assert_eq!(node_paths, vec![vec![1, 2, 3], vec![3, 4, 5]]);
    }

    #[test]
    fn build_anchor_paths_rejects_ambiguous_spans() {
        let road_map = ambiguous_anchor_span_map();
        assert!(build_anchor_paths(&road_map, &[1, 3, 5]).is_empty());
    }
}
