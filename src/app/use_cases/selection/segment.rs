//! Use-Case: Segment-Selektion zwischen Kreuzungen (Doppelklick).

use crate::AppState;
use std::collections::HashMap;

use super::helpers::{build_undirected_adjacency, clear_selection};

/// Läuft entlang einer Kette von Grad-2-Nodes bis zur nächsten Segmentgrenze.
fn walk_to_segment_boundary(
    start: u64,
    first_neighbor: u64,
    adjacency: &HashMap<u64, Vec<u64>>,
) -> Vec<u64> {
    let mut path = vec![start, first_neighbor];
    let mut previous = start;
    let mut current = first_neighbor;

    loop {
        let degree = adjacency.get(&current).map(|n| n.len()).unwrap_or(0);
        if degree != 2 {
            break;
        }

        let Some(neighbors) = adjacency.get(&current) else {
            break;
        };

        let Some(&next) = neighbors.iter().find(|&&neighbor| neighbor != previous) else {
            break;
        };

        if path.contains(&next) {
            break;
        }

        path.push(next);
        previous = current;
        current = next;
    }

    path
}

/// Selektiert den Korridor um den getroffenen Node bis zu den nächsten Segmentgrenzen.
///
/// Segmentgrenzen sind Nodes mit Grad != 2, also Verzweigungen oder Sackgassen.
/// Bei `additive = true` wird das Segment zur bestehenden Selektion hinzugefügt.
pub fn select_segment_between_nearest_intersections(
    state: &mut AppState,
    world_pos: glam::Vec2,
    max_distance: f32,
    additive: bool,
) {
    if max_distance < 0.0 {
        if !additive {
            clear_selection(state);
        }
        return;
    }

    let Some(road_map) = state.road_map.as_deref() else {
        if !additive {
            clear_selection(state);
        }
        return;
    };

    let Some(hit_id) = road_map
        .nearest_node(world_pos)
        .filter(|hit| hit.distance <= max_distance)
        .map(|hit| hit.node_id)
    else {
        if !additive {
            clear_selection(state);
        }
        return;
    };

    let adjacency = build_undirected_adjacency(road_map);
    let neighbors = adjacency.get(&hit_id).cloned().unwrap_or_default();

    if neighbors.is_empty() {
        if !additive {
            state.selection.ids_mut().clear();
        }
        state.selection.ids_mut().insert(hit_id);
        state.selection.selection_anchor_node_id = Some(hit_id);
        return;
    }

    let mut paths = neighbors
        .into_iter()
        .map(|neighbor| walk_to_segment_boundary(hit_id, neighbor, &adjacency))
        .collect::<Vec<_>>();

    if paths.len() > 2 {
        paths.sort_by_key(|path| path.len());
        paths.truncate(2);
    }

    if !additive {
        state.selection.ids_mut().clear();
    }
    for path in paths {
        state.selection.ids_mut().extend(path);
    }
    state.selection.ids_mut().insert(hit_id);
    state.selection.selection_anchor_node_id = Some(hit_id);
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{Connection, ConnectionDirection, ConnectionPriority, MapNode, NodeFlag, RoadMap};
    use std::sync::Arc;

    #[test]
    fn select_segment_between_nearest_intersections_selects_corridor_nodes() {
        let mut map = RoadMap::new(3);
        map.add_node(MapNode::new(
            10,
            glam::Vec2::new(-20.0, 0.0),
            NodeFlag::Regular,
        ));
        map.add_node(MapNode::new(
            11,
            glam::Vec2::new(-10.0, 0.0),
            NodeFlag::Regular,
        ));
        map.add_node(MapNode::new(
            12,
            glam::Vec2::new(0.0, 0.0),
            NodeFlag::Regular,
        ));
        map.add_node(MapNode::new(
            13,
            glam::Vec2::new(10.0, 0.0),
            NodeFlag::Regular,
        ));
        map.add_node(MapNode::new(
            14,
            glam::Vec2::new(20.0, 0.0),
            NodeFlag::Regular,
        ));
        map.add_node(MapNode::new(
            20,
            glam::Vec2::new(-20.0, 10.0),
            NodeFlag::Regular,
        ));
        map.add_node(MapNode::new(
            21,
            glam::Vec2::new(20.0, 10.0),
            NodeFlag::Regular,
        ));
        map.add_node(MapNode::new(
            22,
            glam::Vec2::new(-20.0, -10.0),
            NodeFlag::Regular,
        ));
        map.add_node(MapNode::new(
            23,
            glam::Vec2::new(20.0, -10.0),
            NodeFlag::Regular,
        ));

        let conn = |s, e, sx, sy, ex, ey| {
            Connection::new(
                s,
                e,
                ConnectionDirection::Regular,
                ConnectionPriority::Regular,
                glam::Vec2::new(sx, sy),
                glam::Vec2::new(ex, ey),
            )
        };
        map.add_connection(conn(10, 11, -20.0, 0.0, -10.0, 0.0));
        map.add_connection(conn(11, 12, -10.0, 0.0, 0.0, 0.0));
        map.add_connection(conn(12, 13, 0.0, 0.0, 10.0, 0.0));
        map.add_connection(conn(13, 14, 10.0, 0.0, 20.0, 0.0));
        map.add_connection(conn(10, 20, -20.0, 0.0, -20.0, 10.0));
        map.add_connection(conn(14, 21, 20.0, 0.0, 20.0, 10.0));
        map.add_connection(conn(10, 22, -20.0, 0.0, -20.0, -10.0));
        map.add_connection(conn(14, 23, 20.0, 0.0, 20.0, -10.0));
        map.ensure_spatial_index();

        let mut state = AppState::new();
        state.road_map = Some(Arc::new(map));

        select_segment_between_nearest_intersections(
            &mut state,
            glam::Vec2::new(0.2, 0.0),
            2.0,
            false,
        );

        for node_id in [10_u64, 11, 12, 13, 14] {
            assert!(state.selection.selected_node_ids.contains(&node_id));
        }
        assert!(!state.selection.selected_node_ids.contains(&20));
        assert!(!state.selection.selected_node_ids.contains(&21));
        assert!(!state.selection.selected_node_ids.contains(&22));
        assert!(!state.selection.selected_node_ids.contains(&23));
        assert_eq!(state.selection.selection_anchor_node_id, Some(12));
    }

    #[test]
    fn select_segment_works_for_bidirectional_connections() {
        // Bidirektionale Straße: 10 ↔ 11 ↔ 12 ↔ 13 ↔ 14
        // Kreuzung an 10 (Grad 3: 11, 20, 22) und 14 (Grad 3: 13, 21, 23)
        let mut map = RoadMap::new(3);
        map.add_node(MapNode::new(
            10,
            glam::Vec2::new(-20.0, 0.0),
            NodeFlag::Regular,
        ));
        map.add_node(MapNode::new(
            11,
            glam::Vec2::new(-10.0, 0.0),
            NodeFlag::Regular,
        ));
        map.add_node(MapNode::new(
            12,
            glam::Vec2::new(0.0, 0.0),
            NodeFlag::Regular,
        ));
        map.add_node(MapNode::new(
            13,
            glam::Vec2::new(10.0, 0.0),
            NodeFlag::Regular,
        ));
        map.add_node(MapNode::new(
            14,
            glam::Vec2::new(20.0, 0.0),
            NodeFlag::Regular,
        ));
        map.add_node(MapNode::new(
            20,
            glam::Vec2::new(-20.0, 10.0),
            NodeFlag::Regular,
        ));
        map.add_node(MapNode::new(
            21,
            glam::Vec2::new(20.0, 10.0),
            NodeFlag::Regular,
        ));
        map.add_node(MapNode::new(
            22,
            glam::Vec2::new(-20.0, -10.0),
            NodeFlag::Regular,
        ));
        map.add_node(MapNode::new(
            23,
            glam::Vec2::new(20.0, -10.0),
            NodeFlag::Regular,
        ));

        let conn = |s, e, sx, sy, ex, ey| {
            Connection::new(
                s,
                e,
                ConnectionDirection::Dual,
                ConnectionPriority::Regular,
                glam::Vec2::new(sx, sy),
                glam::Vec2::new(ex, ey),
            )
        };
        // Bidirektionale Hauptstraße (Hin- UND Rückrichtung)
        map.add_connection(conn(10, 11, -20.0, 0.0, -10.0, 0.0));
        map.add_connection(conn(11, 10, -10.0, 0.0, -20.0, 0.0));
        map.add_connection(conn(11, 12, -10.0, 0.0, 0.0, 0.0));
        map.add_connection(conn(12, 11, 0.0, 0.0, -10.0, 0.0));
        map.add_connection(conn(12, 13, 0.0, 0.0, 10.0, 0.0));
        map.add_connection(conn(13, 12, 10.0, 0.0, 0.0, 0.0));
        map.add_connection(conn(13, 14, 10.0, 0.0, 20.0, 0.0));
        map.add_connection(conn(14, 13, 20.0, 0.0, 10.0, 0.0));
        // Abzweigungen: je 2 pro Kreuzung → Grad 3 auch nach Deduplizierung
        map.add_connection(conn(10, 20, -20.0, 0.0, -20.0, 10.0));
        map.add_connection(conn(10, 22, -20.0, 0.0, -20.0, -10.0));
        map.add_connection(conn(14, 21, 20.0, 0.0, 20.0, 10.0));
        map.add_connection(conn(14, 23, 20.0, 0.0, 20.0, -10.0));
        map.ensure_spatial_index();

        let mut state = AppState::new();
        state.road_map = Some(Arc::new(map));

        // Doppelklick auf Node 12 (Mitte der Kette)
        select_segment_between_nearest_intersections(
            &mut state,
            glam::Vec2::new(0.2, 0.0),
            2.0,
            false,
        );

        // Alle 5 Nodes des Segments sollen selektiert sein
        for node_id in [10_u64, 11, 12, 13, 14] {
            assert!(
                state.selection.selected_node_ids.contains(&node_id),
                "Node {} sollte selektiert sein (bidirektionale Straße)",
                node_id
            );
        }
        // Abzweigungen nicht selektiert
        assert!(!state.selection.selected_node_ids.contains(&20));
        assert!(!state.selection.selected_node_ids.contains(&21));
        assert!(!state.selection.selected_node_ids.contains(&22));
        assert!(!state.selection.selected_node_ids.contains(&23));
        assert_eq!(state.selection.selection_anchor_node_id, Some(12));
    }
}
