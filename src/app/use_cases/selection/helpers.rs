//! Gemeinsame Hilfsfunktionen fuer Selektionslogik.

use crate::core::{ConnectionDirection, ConnectionPriority, NodeFlag, RoadMap};
use crate::AppState;
use std::collections::HashMap;

/// Loescht die aktuelle Selektion explizit.
pub fn clear_selection(state: &mut AppState) {
    state.selection.ids_mut().clear();
    state.selection.selection_anchor_node_id = None;
}

/// Berechnet das achsen-alignierte Bounding-Rect aus zwei Eckpunkten.
pub(super) fn rect_min_max(a: glam::Vec2, b: glam::Vec2) -> (glam::Vec2, glam::Vec2) {
    (
        glam::Vec2::new(a.x.min(b.x), a.y.min(b.y)),
        glam::Vec2::new(a.x.max(b.x), a.y.max(b.y)),
    )
}

/// Baut eine ungerichtete Adjazenzliste aus den Connections der RoadMap.
///
/// Duplikate werden entfernt, damit bidirektionale Verbindungen (A→B + B→A)
/// den Grad eines Nodes nicht kuenstlich verdoppeln.
pub(super) fn build_undirected_adjacency(road_map: &RoadMap) -> HashMap<u64, Vec<u64>> {
    use std::collections::HashSet;
    let mut adjacency_set: HashMap<u64, HashSet<u64>> = HashMap::new();

    for connection in road_map.connections_iter() {
        if road_map.nodes.contains_key(&connection.start_id)
            && road_map.nodes.contains_key(&connection.end_id)
        {
            adjacency_set
                .entry(connection.start_id)
                .or_default()
                .insert(connection.end_id);
            adjacency_set
                .entry(connection.end_id)
                .or_default()
                .insert(connection.start_id);
        }
    }

    adjacency_set
        .into_iter()
        .map(|(k, v)| (k, v.into_iter().collect()))
        .collect()
}

/// Nachbar-Eintrag mit Winkelinformation fuer den Segment-Walk.
pub(super) struct AdjacencyNeighbor {
    /// ID des Nachbar-Nodes.
    pub node_id: u64,
    /// atan2-Winkel der Verbindung (Richtung: aktueller Node → Nachbar).
    pub angle: f32,
    /// Flag des Ziel-Nodes (Strassenart).
    pub target_flag: NodeFlag,
    /// Richtung der Verbindung (gerichtet vs. bidirektional).
    pub connection_direction: ConnectionDirection,
    /// Prioritaet der Verbindung (Haupt- vs. Nebenstrasse).
    pub connection_priority: ConnectionPriority,
}

/// Baut eine ungerichtete Adjazenzliste mit Winkelinformation.
///
/// Jeder Eintrag enthaelt den Nachbar-Node und den Winkel der Verbindung
/// (Richtung: aktueller Node → Nachbar). Duplikate werden dedupliziert.
pub(super) fn build_undirected_adjacency_with_angles(
    road_map: &RoadMap,
) -> HashMap<u64, Vec<AdjacencyNeighbor>> {
    let mut adjacency: HashMap<u64, Vec<AdjacencyNeighbor>> = HashMap::new();
    for connection in road_map.connections_iter() {
        let s = connection.start_id;
        let e = connection.end_id;
        if !road_map.nodes.contains_key(&s) || !road_map.nodes.contains_key(&e) {
            continue;
        }
        let angle_s_to_e = connection.angle;
        let angle_e_to_s = connection.angle + std::f32::consts::PI;
        let entry_s = adjacency.entry(s).or_default();
        if !entry_s.iter().any(|n| n.node_id == e) {
            entry_s.push(AdjacencyNeighbor {
                node_id: e,
                angle: angle_s_to_e,
                target_flag: road_map.nodes[&e].flag,
                connection_direction: connection.direction,
                connection_priority: connection.priority,
            });
        }
        let entry_e = adjacency.entry(e).or_default();
        if !entry_e.iter().any(|n| n.node_id == s) {
            entry_e.push(AdjacencyNeighbor {
                node_id: s,
                angle: angle_e_to_s,
                target_flag: road_map.nodes[&s].flag,
                connection_direction: connection.direction,
                connection_priority: connection.priority,
            });
        }
    }
    adjacency
}
