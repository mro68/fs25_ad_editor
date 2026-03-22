//! Use-Case: Segment-Selektion zwischen Kreuzungen (Doppelklick).

use crate::core::{ConnectionDirection, ConnectionPriority, NodeFlag};
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
    /// Flag des Hit-Nodes (Strassenart-Kontext fuer Gewichtung).
    start_flag: NodeFlag,
}

/// Bewertet einen Nachbar-Kandidaten fuer den Segment-Walk.
///
/// Hoehere Werte = besserer Kandidat. Kriterien (absteigend gewichtet):
/// 1. Gleiche Strassenart wie Start-Node: +40
/// 2. Hauptstrasse (Regular-Priority): +20
/// 3. Gerichtet (Regular/Reverse) vor Bidirektional (Dual): +10
/// 4. Geringste Winkelabweichung als Tiebreaker: 0..+10
fn score_neighbor(
    neighbor: &AdjacencyNeighbor,
    start_flag: NodeFlag,
    incoming_angle: Option<f32>,
) -> i32 {
    let mut score: i32 = 0;

    // Kriterium 1: Gleiche Strassenart wie Start-Node (+40)
    if neighbor.target_flag == start_flag {
        score += 40;
    }

    // Kriterium 2: Hauptstrasse bevorzugt (+20)
    if neighbor.connection_priority == ConnectionPriority::Regular {
        score += 20;
    }

    // Kriterium 3: Gerichtet > Bidirektional (+10)
    if neighbor.connection_direction != ConnectionDirection::Dual {
        score += 10;
    }

    // Kriterium 4: Winkelabweichung als Tiebreaker (+0..+10)
    if let Some(in_angle) = incoming_angle {
        let deviation = angle_deviation(in_angle, neighbor.angle);
        let angle_score = ((1.0 - deviation / std::f32::consts::PI) * 10.0) as i32;
        score += angle_score.clamp(0, 10);
    }

    score
}

/// Laeuft entlang einer Kante von Nodes bis zur naechsten Segmentgrenze.
///
/// Abbruchbedingungen werden ueber `WalkConfig` konfiguriert.
/// Bei Kreuzungen (>2 Nachbarn) wird score-basiert der beste Kandidat gewaehlt.
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

        // Eingangswinkel fuer Constraint + Scoring ermitteln
        let incoming_angle = adjacency
            .get(&previous)
            .and_then(|ns| ns.iter().find(|n| n.node_id == current))
            .map(|n| n.angle);

        // Kandidaten: alle Nachbarn ausser previous
        let candidates: Vec<&AdjacencyNeighbor> =
            neighbors.iter().filter(|n| n.node_id != previous).collect();

        if candidates.is_empty() {
            break;
        }

        // Harter Winkel-Constraint: nur Kandidaten innerhalb max_angle_rad behalten
        let viable: Vec<&AdjacencyNeighbor> = if config.max_angle_rad > 0.0 {
            if let Some(in_angle) = incoming_angle {
                candidates
                    .into_iter()
                    .filter(|n| angle_deviation(in_angle, n.angle) <= config.max_angle_rad)
                    .collect()
            } else {
                candidates
            }
        } else {
            candidates
        };

        if viable.is_empty() {
            break;
        }

        // Besten Kandidaten nach Score waehlen
        // SAFETY: viable ist nicht leer (oben geprueft)
        let next_entry = viable
            .into_iter()
            .max_by_key(|n| score_neighbor(n, config.start_flag, incoming_angle))
            .expect("viable ist nicht leer");

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

    let start_flag = road_map
        .nodes
        .get(&hit_id)
        .map(|n| n.flag)
        .unwrap_or(NodeFlag::Regular);

    let config = WalkConfig {
        stop_at_junction,
        max_angle_rad: if max_angle_deg > 0.0 {
            max_angle_deg.to_radians()
        } else {
            0.0
        },
        start_flag,
    };

    let mut paths = neighbors
        .into_iter()
        .map(|neighbor| walk_to_segment_boundary(hit_id, neighbor, &adjacency, &config))
        .collect::<Vec<_>>();

    if paths.len() > 2 {
        // Bevorzuge Pfade die am besten zur Strassenart des Hit-Nodes passen
        paths.sort_by(|a, b| {
            let quality = |path: &[u64]| -> usize {
                path.iter()
                    .filter(|id| {
                        road_map
                            .nodes
                            .get(*id)
                            .map(|n| n.flag == start_flag)
                            .unwrap_or(false)
                    })
                    .count()
            };
            quality(b).cmp(&quality(a)) // absteigend: Pfade mit mehr matches zuerst
        });
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

    // --- Neue Tests fuer Gewichtung ---

    /// Baut eine T-Kreuzung: SubPrio-Kette 30-31-32-33-34, mit Regular-Abzweig 32→40→41
    ///
    /// Topologie:
    /// ```
    /// SubPrio: 30 ↔ 31 ↔ [32] ↔ 33 ↔ 34   (SubPrio-Kette)
    ///                       ↓
    /// Regular:              40 → 41           (Hauptstrassenabzweig)
    /// ```
    fn build_t_junction_subprio_map() -> RoadMap {
        let mut map = RoadMap::new(5);
        // SubPrio-Kette
        map.add_node(MapNode::new(
            30,
            glam::Vec2::new(-20.0, 0.0),
            NodeFlag::SubPrio,
        ));
        map.add_node(MapNode::new(
            31,
            glam::Vec2::new(-10.0, 0.0),
            NodeFlag::SubPrio,
        ));
        map.add_node(MapNode::new(
            32,
            glam::Vec2::new(0.0, 0.0),
            NodeFlag::SubPrio,
        ));
        map.add_node(MapNode::new(
            33,
            glam::Vec2::new(10.0, 0.0),
            NodeFlag::SubPrio,
        ));
        map.add_node(MapNode::new(
            34,
            glam::Vec2::new(20.0, 0.0),
            NodeFlag::SubPrio,
        ));
        // Regular-Abzweig (senkrecht)
        map.add_node(MapNode::new(
            40,
            glam::Vec2::new(0.0, 10.0),
            NodeFlag::Regular,
        ));
        map.add_node(MapNode::new(
            41,
            glam::Vec2::new(0.0, 20.0),
            NodeFlag::Regular,
        ));

        let conn = |s, e, sx: f32, sy: f32, ex: f32, ey: f32, prio| {
            Connection::new(
                s,
                e,
                ConnectionDirection::Regular,
                prio,
                glam::Vec2::new(sx, sy),
                glam::Vec2::new(ex, ey),
            )
        };
        // SubPrio-Verbindungen
        map.add_connection(conn(
            30,
            31,
            -20.0,
            0.0,
            -10.0,
            0.0,
            ConnectionPriority::SubPriority,
        ));
        map.add_connection(conn(
            31,
            30,
            -10.0,
            0.0,
            -20.0,
            0.0,
            ConnectionPriority::SubPriority,
        ));
        map.add_connection(conn(
            31,
            32,
            -10.0,
            0.0,
            0.0,
            0.0,
            ConnectionPriority::SubPriority,
        ));
        map.add_connection(conn(
            32,
            31,
            0.0,
            0.0,
            -10.0,
            0.0,
            ConnectionPriority::SubPriority,
        ));
        map.add_connection(conn(
            32,
            33,
            0.0,
            0.0,
            10.0,
            0.0,
            ConnectionPriority::SubPriority,
        ));
        map.add_connection(conn(
            33,
            32,
            10.0,
            0.0,
            0.0,
            0.0,
            ConnectionPriority::SubPriority,
        ));
        map.add_connection(conn(
            33,
            34,
            10.0,
            0.0,
            20.0,
            0.0,
            ConnectionPriority::SubPriority,
        ));
        map.add_connection(conn(
            34,
            33,
            20.0,
            0.0,
            10.0,
            0.0,
            ConnectionPriority::SubPriority,
        ));
        // Regular-Verbindung (Abzweig von 32 senkrecht)
        map.add_connection(conn(
            32,
            40,
            0.0,
            0.0,
            0.0,
            10.0,
            ConnectionPriority::Regular,
        ));
        map.add_connection(conn(
            40,
            41,
            0.0,
            10.0,
            0.0,
            20.0,
            ConnectionPriority::Regular,
        ));
        map.ensure_spatial_index();
        map
    }

    #[test]
    fn segment_walk_prefers_same_roadtype_subprio() {
        // Doppelklick auf Node 32 (SubPrio).
        // Walk soll trotz Regular-Abzweig (32→40→41) auf der SubPrio-Kette bleiben.
        // stop_at_junction=false → Kreuzungs-Stopp deaktiviert, Gewichtung entscheidet
        let mut state = AppState::new();
        state.road_map = Some(Arc::new(build_t_junction_subprio_map()));

        select_segment_between_nearest_intersections(
            &mut state,
            glam::Vec2::new(0.0, 0.0), // Node 32
            1.0,
            false,
            false, // stop_at_junction=false
            0.0,   // Winkelcheck aus (senkrechter Abzweig wuerde sowieso gefiltert)
        );

        // SubPrio-Kette soll vollstaendig selektiert sein
        for node_id in [30_u64, 31, 32, 33, 34] {
            assert!(
                state.selection.selected_node_ids.contains(&node_id),
                "Node {node_id} sollte selektiert sein (SubPrio-Kette)"
            );
        }
        // Regular-Abzweig soll NICHT selektiert sein
        assert!(
            !state.selection.selected_node_ids.contains(&40),
            "Node 40 sollte NICHT selektiert sein (Regular-Abzweig, andere Strassenart)"
        );
        assert!(
            !state.selection.selected_node_ids.contains(&41),
            "Node 41 sollte NICHT selektiert sein (Regular-Abzweig)"
        );
    }

    #[test]
    fn segment_walk_angle_constraint_blocks_sharp_turn() {
        // T-Kreuzung: Gerade Strecke 10→11→12 (horizontal), Abzweig 11→20 (senkrecht, 90°)
        // max_angle=89°: Walk von 10 ausgehend soll 11→12 bevorzugen und 11→20 verwerfen
        let mut map = RoadMap::new(4);
        map.add_node(MapNode::new(
            10,
            glam::Vec2::new(-10.0, 0.0),
            NodeFlag::Regular,
        ));
        map.add_node(MapNode::new(
            11,
            glam::Vec2::new(0.0, 0.0),
            NodeFlag::Regular,
        ));
        map.add_node(MapNode::new(
            12,
            glam::Vec2::new(10.0, 0.0),
            NodeFlag::Regular,
        ));
        map.add_node(MapNode::new(
            20,
            glam::Vec2::new(0.0, 10.0),
            NodeFlag::Regular,
        ));

        let conn = |s, e, sx: f32, sy: f32, ex: f32, ey: f32| {
            Connection::new(
                s,
                e,
                ConnectionDirection::Regular,
                ConnectionPriority::Regular,
                glam::Vec2::new(sx, sy),
                glam::Vec2::new(ex, ey),
            )
        };
        map.add_connection(conn(10, 11, -10.0, 0.0, 0.0, 0.0));
        map.add_connection(conn(11, 12, 0.0, 0.0, 10.0, 0.0));
        map.add_connection(conn(11, 20, 0.0, 0.0, 0.0, 10.0)); // 90°-Abzweig
        map.ensure_spatial_index();

        let mut state = AppState::new();
        state.road_map = Some(Arc::new(map));

        select_segment_between_nearest_intersections(
            &mut state,
            glam::Vec2::new(-10.0, 0.0), // Node 10
            1.0,
            false,
            false, // stop_at_junction=false
            89.0,  // max_angle=89°: scharfer Knick (90°) wird geblockt
        );

        // 10 und 11 sollen selektiert sein
        assert!(state.selection.selected_node_ids.contains(&10));
        assert!(state.selection.selected_node_ids.contains(&11));
        // 12 soll selektiert sein (geradeaus, 0° Abweichung)
        assert!(
            state.selection.selected_node_ids.contains(&12),
            "Node 12 sollte selektiert sein (geradeaus)"
        );
        // 20 soll NICHT selektiert sein (90° > 89°-Constraint)
        assert!(
            !state.selection.selected_node_ids.contains(&20),
            "Node 20 sollte NICHT selektiert sein (90°-Abzweig > 89°-Constraint)"
        );
    }
}
