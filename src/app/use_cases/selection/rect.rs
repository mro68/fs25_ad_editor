//! Use-Case: Rechteck-Selektion (Shift + Drag).

use crate::AppState;

use super::helpers::{clear_selection, rect_min_max};

/// Selektiert alle Nodes im Rechteck (inkl. Rand).
pub fn select_nodes_in_rect(
    state: &mut AppState,
    corner_a: glam::Vec2,
    corner_b: glam::Vec2,
    additive: bool,
) {
    let Some(road_map) = state.road_map.as_deref() else {
        if !additive {
            clear_selection(state);
        }
        return;
    };

    let (min, max) = rect_min_max(corner_a, corner_b);
    let hit_ids = road_map.nodes_within_rect(min, max);

    if !additive {
        state.selection.selected_node_ids.clear();
    }

    state.selection.selected_node_ids.extend(hit_ids);
    state.selection.selection_anchor_node_id =
        state.selection.selected_node_ids.iter().copied().next();
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{Connection, ConnectionDirection, ConnectionPriority, MapNode, NodeFlag, RoadMap};
    use std::sync::Arc;

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
    fn select_nodes_in_rect_selects_nodes_inside_bounds() {
        let mut state = with_path_test_map();
        select_nodes_in_rect(
            &mut state,
            glam::Vec2::new(-1.0, -1.0),
            glam::Vec2::new(15.0, 1.0),
            false,
        );
        assert!(state.selection.selected_node_ids.contains(&1));
        assert!(state.selection.selected_node_ids.contains(&2));
        assert!(!state.selection.selected_node_ids.contains(&3));
    }
}
