//! Use-Case: Verschieben selektierter Nodes.

use crate::AppState;
use std::sync::Arc;

/// Verschiebt alle selektierten Nodes um den gegebenen Welt-Offset.
pub fn move_selected_nodes(state: &mut AppState, delta_world: glam::Vec2) {
    if delta_world == glam::Vec2::ZERO {
        return;
    }

    let Some(road_map) = state.road_map.as_mut() else {
        return;
    };

    if state.selection.selected_node_ids.is_empty() {
        return;
    }

    let road_map_mut = Arc::make_mut(road_map);
    let mut moved_any = false;

    for node_id in &state.selection.selected_node_ids {
        if let Some(node) = road_map_mut.nodes.get_mut(node_id) {
            node.position += delta_world;
            moved_any = true;
        }
    }

    if moved_any {
        road_map_mut.rebuild_connection_geometry();
        road_map_mut.ensure_spatial_index();
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{MapNode, NodeFlag, RoadMap};
    use std::sync::Arc;

    #[test]
    fn move_selected_nodes_moves_all_selected() {
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
        state.selection.selected_node_ids.insert(1);
        state.selection.selected_node_ids.insert(2);

        move_selected_nodes(&mut state, glam::Vec2::new(2.0, 3.0));

        let road_map = state.road_map.as_ref().expect("map vorhanden");
        let node1 = road_map.nodes.get(&1).expect("node 1 vorhanden");
        let node2 = road_map.nodes.get(&2).expect("node 2 vorhanden");
        assert_eq!(node1.position, glam::Vec2::new(2.0, 3.0));
        assert_eq!(node2.position, glam::Vec2::new(12.0, 3.0));
    }
}
