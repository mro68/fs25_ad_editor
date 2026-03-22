//! Use-Case: Segment-Selektion zwischen Kreuzungen (Doppelklick).

use crate::shared::angle_deviation;
use crate::AppState;
use std::collections::HashMap;

use super::helpers::{build_undirected_adjacency_with_angles, clear_selection, AdjacencyNeighbor};

/// Konfig fuer die Abbruchbedingungen des Segment-Walks.
struct WalkConfig {
    /// Stopp bei Knoten mit Grad != 2 (Kreuzung).
    stop_at_junction: bool,
    /// Max. Winkelabweichung in Radiant (0.0 = deaktiviert).
    max_angle_rad: f32,
}

/// Laeuft entlang einer Kette von Nodes bis zur naechsten Segmentgrenze.
///
/// Abbruchbedingungen werden ueber `WalkConfig` konfiguriert.
fn walk_to_segment_boundary(
    start: u64,
    first_neighbor: u64,
    adjacency: &HashMap<u64, Vec<AdjacencyNeighbor>>,
    config: &WalkConfig,
) -> Vec<u64> {
    let mut path = vec![start, first_neighbor];
    let mut previous = start;
    let mut current = first_neighbor;

    while let Some(neighbors) = adjacency.get(&current) {
        // Abbruch: Kreuzung (degree != 2)
        if config.stop_at_junction && neighbors.len() != 2 {
            break;
        }

        // Naechsten Node bestimmen (nicht previous)
        let Some(next_entry) = neighbors.iter().find(|n| n.node_id != previous) else {
            break;
        };

        // Abbruch: Winkelabweichung pruefen
        if config.max_angle_rad > 0.0 {
            let incoming_angle = adjacency
                .get(&previous)
                .and_then(|ns| ns.iter().find(|n| n.node_id == current))
                .map(|n| n.angle);
            if let Some(in_angle) = incoming_angle {
                let deviation = angle_deviation(in_angle, next_entry.angle);
                if deviation > config.max_angle_rad {
                    break;
                }
            }
        }

        if path.contains(&next_entry.node_id) {
            break;
        }

        path.push(next_entry.node_id);
        previous = current;
        current = next_entry.node_id;
    }

    path
}

/// Selektiert den Korridor um den getroffenen Node bis zu den naechsten Segmentgrenzen.
///
/// Segmentgrenzen koennen Kreuzungen (Grad != 2) und/oder Winkelabweichungen sein,
/// je nach Konfiguration. Bei `additive = true` wird das Segment zur bestehenden
/// Selektion hinzugefuegt.
pub fn select_segment_between_nearest_intersections(
    state: &mut AppState,
    world_pos: glam::Vec2,
    max_distance: f32,
    additive: bool,
    stop_at_junction: bool,
    max_angle_deg: f32,
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

    let adjacency = build_undirected_adjacency_with_angles(road_map);
    let neighbors = adjacency
        .get(&hit_id)
        .map(|ns| ns.iter().map(|n| n.node_id).collect::<Vec<_>>())
        .unwrap_or_default();

    if neighbors.is_empty() {
        if !additive {
            state.selection.ids_mut().clear();
        }
        state.selection.ids_mut().insert(hit_id);
        state.selection.selection_anchor_node_id = Some(hit_id);
        return;
    }

    let config = WalkConfig {
        stop_at_junction,
        max_angle_rad: if max_angle_deg > 0.0 {
            max_angle_deg.to_radians()
        } else {
            0.0
        },
    };

    let mut paths = neighbors
        .into_iter()
        .map(|neighbor| walk_to_segment_boundary(hit_id, neighbor, &adjacency, &config))
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
            true,
            15.0,
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
        // Bidirektionale Strasse: 10 ↔ 11 ↔ 12 ↔ 13 ↔ 14
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
        // Bidirektionale Hauptstrasse (Hin- UND Rueckrichtung)
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
            true,
            15.0,
        );

        // Alle 5 Nodes des Segments sollen selektiert sein
        for node_id in [10_u64, 11, 12, 13, 14] {
            assert!(
                state.selection.selected_node_ids.contains(&node_id),
                "Node {} sollte selektiert sein (bidirektionale Strasse)",
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

    // --- Neue Tests fuer Walk-Abbruch bei Winkelabweichung ---

    /// Hilfsfunktion: L-foermige Strecke 10→11→12 mit 90°-Knick bei Node 11.
    /// Node 10: (0,0), Node 11: (10,0), Node 12: (10,10)
    fn build_l_shaped_map() -> RoadMap {
        let mut map = RoadMap::new(3);
        map.add_node(MapNode::new(
            10,
            glam::Vec2::new(0.0, 0.0),
            NodeFlag::Regular,
        ));
        map.add_node(MapNode::new(
            11,
            glam::Vec2::new(10.0, 0.0),
            NodeFlag::Regular,
        ));
        map.add_node(MapNode::new(
            12,
            glam::Vec2::new(10.0, 10.0),
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
        map.add_connection(conn(10, 11, 0.0, 0.0, 10.0, 0.0));
        map.add_connection(conn(11, 12, 10.0, 0.0, 10.0, 10.0));
        map.ensure_spatial_index();
        map
    }

    #[test]
    fn segment_walk_stops_at_angle() {
        // L-foermige Strecke: 10→11→12 mit 90°-Knick bei Node 11
        // Klick auf Node 10 (vor dem Knick), max_angle=15° → Walk darf nicht um die Ecke gehen
        let mut state = AppState::new();
        state.road_map = Some(Arc::new(build_l_shaped_map()));

        select_segment_between_nearest_intersections(
            &mut state,
            glam::Vec2::new(0.0, 0.0), // Node 10 (vor dem Knick)
            1.0,
            false,
            false, // stop_at_junction=false: Grad-Pruefung deaktiviert
            15.0,  // max_angle=15°: Winkel-Abbruch aktiv
        );

        // Node 10 und 11 sollen selektiert sein (Walk erreicht Knick und stoppt)
        assert!(state.selection.selected_node_ids.contains(&10));
        assert!(state.selection.selected_node_ids.contains(&11));
        // Node 12 soll NICHT selektiert sein (90° > 15°)
        assert!(
            !state.selection.selected_node_ids.contains(&12),
            "Node 12 sollte NICHT selektiert sein (90°-Knick ueberschreitet 15°-Limit)"
        );
    }

    #[test]
    fn segment_walk_angle_disabled() {
        // Gleiche L-foermige Strecke, aber max_angle=0 → Winkelcheck deaktiviert
        // stop_at_junction=false → Alle 3 Nodes werden selektiert
        let mut state = AppState::new();
        state.road_map = Some(Arc::new(build_l_shaped_map()));

        select_segment_between_nearest_intersections(
            &mut state,
            glam::Vec2::new(0.0, 0.0), // Node 10 (Startpunkt)
            1.0,
            false,
            false, // Kreuzungs-Stopp aus
            0.0,   // Winkelcheck deaktiviert
        );

        // Alle 3 Nodes sollen selektiert sein
        for node_id in [10_u64, 11, 12] {
            assert!(
                state.selection.selected_node_ids.contains(&node_id),
                "Node {node_id} sollte selektiert sein (Winkelcheck deaktiviert)"
            );
        }
    }
}
