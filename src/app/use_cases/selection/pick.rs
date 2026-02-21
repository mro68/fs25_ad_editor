//! Use-Case: Node-Selektion per Klick (Nearest-Node-Pick).

use crate::core::RoadMap;
use crate::AppState;
use std::collections::{HashMap, HashSet, VecDeque};

use super::helpers::build_undirected_adjacency;

/// Berechnet den kürzesten Pfad (BFS) zwischen zwei Nodes.
fn shortest_path_nodes(road_map: &RoadMap, start: u64, goal: u64) -> Option<Vec<u64>> {
    if start == goal {
        return Some(vec![start]);
    }

    if !road_map.nodes.contains_key(&start) || !road_map.nodes.contains_key(&goal) {
        return None;
    }

    let adjacency = build_undirected_adjacency(road_map);

    let mut queue = VecDeque::new();
    let mut visited = HashSet::new();
    let mut predecessors: HashMap<u64, u64> = HashMap::new();

    queue.push_back(start);
    visited.insert(start);

    while let Some(current) = queue.pop_front() {
        if current == goal {
            break;
        }

        if let Some(neighbors) = adjacency.get(&current) {
            for &neighbor in neighbors {
                if visited.insert(neighbor) {
                    predecessors.insert(neighbor, current);
                    queue.push_back(neighbor);
                }
            }
        }
    }

    if !visited.contains(&goal) {
        return None;
    }

    let mut path = vec![goal];
    let mut current = goal;

    while current != start {
        let &previous = predecessors.get(&current)?;
        path.push(previous);
        current = previous;
    }

    path.reverse();
    Some(path)
}

/// Selektiert den nächsten Node zur gegebenen Weltposition.
///
/// Falls kein Node innerhalb von `max_distance` gefunden wird, wird die Selektion gelöscht.
pub fn select_nearest_node(
    state: &mut AppState,
    world_pos: glam::Vec2,
    max_distance: f32,
    additive: bool,
    extend_path: bool,
) {
    if max_distance < 0.0 {
        state.selection.selected_node_ids.clear();
        state.selection.selection_anchor_node_id = None;
        return;
    }

    let Some(road_map) = state.road_map.as_deref() else {
        state.selection.selected_node_ids.clear();
        state.selection.selection_anchor_node_id = None;
        return;
    };

    let hit = road_map
        .nearest_node(world_pos)
        .filter(|hit| hit.distance <= max_distance)
        .map(|hit| hit.node_id);

    if additive {
        if let Some(node_id) = hit {
            if state.selection.selected_node_ids.contains(&node_id) {
                state.selection.selected_node_ids.remove(&node_id);
                state.selection.selection_anchor_node_id =
                    state.selection.selected_node_ids.iter().copied().next();
            } else {
                if extend_path {
                    let anchor = state
                        .selection
                        .selection_anchor_node_id
                        .filter(|anchor_id| state.selection.selected_node_ids.contains(anchor_id))
                        .or_else(|| state.selection.selected_node_ids.iter().copied().next());

                    if let Some(anchor_id) = anchor {
                        if let Some(path_nodes) = shortest_path_nodes(road_map, anchor_id, node_id)
                        {
                            state.selection.selected_node_ids.extend(path_nodes);
                        } else {
                            state.selection.selected_node_ids.insert(node_id);
                        }
                    } else {
                        state.selection.selected_node_ids.insert(node_id);
                    }
                } else {
                    state.selection.selected_node_ids.insert(node_id);
                }

                state.selection.selection_anchor_node_id = Some(node_id);
            }
        }
    } else {
        state.selection.selected_node_ids.clear();
        state.selection.selection_anchor_node_id = None;
        if let Some(node_id) = hit {
            state.selection.selected_node_ids.insert(node_id);
            state.selection.selection_anchor_node_id = Some(node_id);
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{Connection, ConnectionDirection, ConnectionPriority, MapNode, NodeFlag, RoadMap};
    use std::sync::Arc;

    fn with_test_map() -> AppState {
        let mut map = RoadMap::new(3);
        map.add_node(MapNode::new(
            1,
            glam::Vec2::new(0.0, 0.0),
            NodeFlag::Regular,
        ));
        map.add_node(MapNode::new(
            2,
            glam::Vec2::new(10.0, 0.0),
            NodeFlag::Regular,
        ));

        let mut state = AppState::new();
        state.road_map = Some(Arc::new(map));
        state
    }

    fn with_path_test_map() -> AppState {
        let mut map = RoadMap::new(3);
        map.add_node(MapNode::new(
            1,
            glam::Vec2::new(0.0, 0.0),
            NodeFlag::Regular,
        ));
        map.add_node(MapNode::new(
            2,
            glam::Vec2::new(10.0, 0.0),
            NodeFlag::Regular,
        ));
        map.add_node(MapNode::new(
            3,
            glam::Vec2::new(20.0, 0.0),
            NodeFlag::Regular,
        ));
        map.add_connection(Connection::new(
            1,
            2,
            ConnectionDirection::Regular,
            ConnectionPriority::Regular,
            glam::Vec2::new(0.0, 0.0),
            glam::Vec2::new(10.0, 0.0),
        ));
        map.add_connection(Connection::new(
            2,
            3,
            ConnectionDirection::Regular,
            ConnectionPriority::Regular,
            glam::Vec2::new(10.0, 0.0),
            glam::Vec2::new(20.0, 0.0),
        ));

        let mut state = AppState::new();
        state.road_map = Some(Arc::new(map));
        state
    }

    #[test]
    fn selects_nearest_node_within_max_distance() {
        let mut state = with_test_map();
        select_nearest_node(&mut state, glam::Vec2::new(0.4, 0.1), 2.0, false, false);
        assert!(state.selection.selected_node_ids.contains(&1));
        assert_eq!(state.selection.selected_node_ids.len(), 1);
    }

    #[test]
    fn clears_selection_if_no_nearby_node_exists() {
        let mut state = with_test_map();
        state.selection.selected_node_ids.insert(2);
        select_nearest_node(&mut state, glam::Vec2::new(100.0, 100.0), 3.0, false, false);
        assert!(state.selection.selected_node_ids.is_empty());
    }

    #[test]
    fn additive_selection_toggles_nodes() {
        let mut state = with_test_map();
        select_nearest_node(&mut state, glam::Vec2::new(0.1, 0.1), 2.0, false, false);
        select_nearest_node(&mut state, glam::Vec2::new(10.1, 0.1), 2.0, true, false);
        assert!(state.selection.selected_node_ids.contains(&1));
        assert!(state.selection.selected_node_ids.contains(&2));

        select_nearest_node(&mut state, glam::Vec2::new(10.1, 0.1), 2.0, true, false);
        assert!(state.selection.selected_node_ids.contains(&1));
        assert!(!state.selection.selected_node_ids.contains(&2));
    }

    #[test]
    fn additive_selection_selects_path_between_anchor_and_second_pick() {
        let mut state = with_path_test_map();
        select_nearest_node(&mut state, glam::Vec2::new(0.1, 0.0), 2.0, false, false);
        select_nearest_node(&mut state, glam::Vec2::new(20.1, 0.0), 2.0, true, true);
        assert!(state.selection.selected_node_ids.contains(&1));
        assert!(state.selection.selected_node_ids.contains(&2));
        assert!(state.selection.selected_node_ids.contains(&3));
        assert_eq!(state.selection.selected_node_ids.len(), 3);
        assert_eq!(state.selection.selection_anchor_node_id, Some(3));
    }

    #[test]
    fn additive_without_extend_path_does_not_select_intermediate_nodes() {
        let mut state = with_path_test_map();
        select_nearest_node(&mut state, glam::Vec2::new(0.1, 0.0), 2.0, false, false);
        select_nearest_node(&mut state, glam::Vec2::new(20.1, 0.0), 2.0, true, false);
        assert!(state.selection.selected_node_ids.contains(&1));
        assert!(state.selection.selected_node_ids.contains(&3));
        assert!(!state.selection.selected_node_ids.contains(&2));
    }
}
