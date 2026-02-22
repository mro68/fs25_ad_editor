//! Use-Case: Lasso-Selektion (Alt + Drag).

use crate::AppState;

use super::helpers::clear_selection;

/// Prüft ob ein Punkt auf einem Liniensegment liegt.
fn point_on_segment(point: glam::Vec2, a: glam::Vec2, b: glam::Vec2) -> bool {
    let ab = b - a;
    let ap = point - a;
    let cross = ab.perp_dot(ap).abs();
    if cross > 1e-4 {
        return false;
    }

    let dot = ap.dot(ab);
    if dot < 0.0 {
        return false;
    }

    let ab_len_sq = ab.length_squared();
    if dot > ab_len_sq {
        return false;
    }

    true
}

/// Prüft ob ein Punkt innerhalb eines Polygons liegt (Ray-Casting).
fn point_in_polygon(point: glam::Vec2, polygon: &[glam::Vec2]) -> bool {
    if polygon.len() < 3 {
        return false;
    }

    let mut inside = false;
    let mut previous = *polygon.last().expect("polygon has at least 3 points");

    for &current in polygon {
        if point_on_segment(point, previous, current) {
            return true;
        }

        let intersect = ((current.y > point.y) != (previous.y > point.y))
            && (point.x
                < (previous.x - current.x) * (point.y - current.y)
                    / ((previous.y - current.y).max(f32::EPSILON))
                    + current.x);

        if intersect {
            inside = !inside;
        }

        previous = current;
    }

    inside
}

/// Selektiert alle Nodes innerhalb eines Lasso-Polygons (inkl. Rand).
pub fn select_nodes_in_lasso(state: &mut AppState, polygon: &[glam::Vec2], additive: bool) {
    if polygon.len() < 3 {
        return;
    }

    let Some(road_map) = state.road_map.as_deref() else {
        if !additive {
            clear_selection(state);
        }
        return;
    };

    let (mut min, mut max) = (polygon[0], polygon[0]);
    for &point in polygon.iter().skip(1) {
        min.x = min.x.min(point.x);
        min.y = min.y.min(point.y);
        max.x = max.x.max(point.x);
        max.y = max.y.max(point.y);
    }

    let candidates = road_map.nodes_within_rect(min, max);

    if !additive {
        state.selection.ids_mut().clear();
    }

    for node_id in candidates {
        if let Some(node) = road_map.nodes.get(&node_id) {
            if point_in_polygon(node.position, polygon) {
                state.selection.ids_mut().insert(node_id);
            }
        }
    }

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
        map.ensure_spatial_index();

        let mut state = AppState::new();
        state.road_map = Some(Arc::new(map));
        state
    }

    #[test]
    fn select_nodes_in_lasso_selects_nodes_inside_polygon() {
        let mut state = with_path_test_map();
        let polygon = vec![
            glam::Vec2::new(-1.0, -1.0),
            glam::Vec2::new(25.0, -1.0),
            glam::Vec2::new(25.0, 2.0),
            glam::Vec2::new(-1.0, 2.0),
        ];
        select_nodes_in_lasso(&mut state, &polygon, false);
        assert!(state.selection.selected_node_ids.contains(&1));
        assert!(state.selection.selected_node_ids.contains(&2));
        assert!(state.selection.selected_node_ids.contains(&3));
    }
}
