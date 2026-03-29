//! Use-Case: Verbindungen erstellen (direkt und ueber Connect-Tool).

use crate::app::AppState;
use crate::core::{Connection, ConnectionDirection, ConnectionPriority};
use std::sync::Arc;

/// Erstellt eine Verbindung zwischen zwei Nodes.
///
/// Validiert gegen Self-Loops und Duplikate.
pub fn add_connection(
    state: &mut AppState,
    from_id: u64,
    to_id: u64,
    direction: ConnectionDirection,
    priority: ConnectionPriority,
) {
    if from_id == to_id {
        log::warn!("Self-Loop nicht erlaubt (Node {})", from_id);
        return;
    }

    let Some(road_map_arc) = state.road_map.as_ref() else {
        return;
    };

    // Pruefe ob beide Nodes existieren
    if !road_map_arc.nodes.contains_key(&from_id) || !road_map_arc.nodes.contains_key(&to_id) {
        log::warn!(
            "Verbindung nicht moeglich: Node {} oder {} existiert nicht",
            from_id,
            to_id
        );
        return;
    }

    // Duplikat-Check: exaktes Match auf start_id + end_id
    if road_map_arc.has_connection(from_id, to_id) {
        log::warn!("Verbindung {}→{} existiert bereits", from_id, to_id);
        return;
    }

    // Snapshot VOR Mutation
    state.record_undo_snapshot();

    let Some(road_map_arc) = state.road_map.as_mut() else {
        log::warn!("Verbindung nicht moeglich: keine RoadMap geladen");
        return;
    };
    let road_map = Arc::make_mut(road_map_arc);

    let start_pos = road_map.nodes[&from_id].position;
    let end_pos = road_map.nodes[&to_id].position;

    let conn = Connection::new(from_id, to_id, direction, priority, start_pos, end_pos);
    road_map.add_connection(conn);
    // Flags der betroffenen Nodes neu berechnen
    road_map.recalculate_node_flags(&[from_id, to_id]);

    log::info!(
        "Verbindung {}→{} ({:?}) erstellt",
        from_id,
        to_id,
        direction
    );
}

/// Connect-Tool: Naechsten Node an Weltposition picken.
///
/// Beim ersten Klick wird der Source-Node gesetzt.
/// Beim zweiten Klick wird die Verbindung erstellt und der Source zurueckgesetzt.
pub fn connect_tool_pick_node(state: &mut AppState, world_pos: glam::Vec2, max_distance: f32) {
    let Some(road_map) = state.road_map.as_deref() else {
        return;
    };

    let hit = road_map
        .nearest_node(world_pos)
        .filter(|hit| hit.distance <= max_distance)
        .map(|hit| hit.node_id);

    let Some(node_id) = hit else {
        // Kein Node getroffen — Source zuruecksetzen
        state.editor.connect_source_node = None;
        log::debug!("Connect-Tool: kein Node gefunden, Source zurueckgesetzt");
        return;
    };

    if let Some(source_id) = state.editor.connect_source_node.take() {
        // Zweiter Klick: Verbindung erstellen
        let direction = state.editor.default_direction;
        let priority = state.editor.default_priority;
        add_connection(state, source_id, node_id, direction, priority);
    } else {
        // Erster Klick: Source setzen
        state.editor.connect_source_node = Some(node_id);
        // Source-Node selektieren als visuelles Feedback
        state.selection.ids_mut().clear();
        state.selection.ids_mut().insert(node_id);
        state.selection.selection_anchor_node_id = Some(node_id);
        log::info!("Connect-Tool: Startknoten {} gewaehlt", node_id);
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::core::{MapNode, NodeFlag, RoadMap};
    use glam::Vec2;

    /// Hilfsfunktion: AppState mit zwei verbundenen Nodes aufbauen
    fn make_state_with_two_nodes() -> AppState {
        let mut state = AppState::new();
        let mut map = RoadMap::new(3);
        map.add_node(MapNode::new(1, Vec2::new(0.0, 0.0), NodeFlag::Regular));
        map.add_node(MapNode::new(2, Vec2::new(10.0, 0.0), NodeFlag::Regular));
        state.road_map = Some(Arc::new(map));
        state
    }

    #[test]
    fn test_connect_creates_bidirectional() {
        // Node 1 und 2 verbinden → beide sehen sich als Nachbarn
        let mut state = make_state_with_two_nodes();
        add_connection(
            &mut state,
            1,
            2,
            ConnectionDirection::Regular,
            ConnectionPriority::Regular,
        );
        let rm = state.road_map.as_deref().unwrap();
        let n1 = rm.connected_neighbors(1);
        let n2 = rm.connected_neighbors(2);
        assert!(n1.iter().any(|n| n.neighbor_id == 2));
        assert!(n2.iter().any(|n| n.neighbor_id == 1));
    }

    #[test]
    fn test_connect_self_loop_rejected() {
        // Self-Loop darf nicht entstehen
        let mut state = make_state_with_two_nodes();
        add_connection(
            &mut state,
            1,
            1,
            ConnectionDirection::Regular,
            ConnectionPriority::Regular,
        );
        let rm = state.road_map.as_deref().unwrap();
        assert_eq!(rm.connection_count(), 0);
    }

    #[test]
    fn test_connect_missing_node_rejected() {
        // Verbindung zu nicht-existierender ID → kein Effekt
        let mut state = make_state_with_two_nodes();
        add_connection(
            &mut state,
            1,
            99,
            ConnectionDirection::Regular,
            ConnectionPriority::Regular,
        );
        let rm = state.road_map.as_deref().unwrap();
        assert_eq!(rm.connection_count(), 0);
    }

    #[test]
    fn test_connect_duplicate_rejected() {
        // Doppelter Verbindungsaufruf → nur eine Connection
        let mut state = make_state_with_two_nodes();
        add_connection(
            &mut state,
            1,
            2,
            ConnectionDirection::Regular,
            ConnectionPriority::Regular,
        );
        add_connection(
            &mut state,
            1,
            2,
            ConnectionDirection::Regular,
            ConnectionPriority::Regular,
        );
        let rm = state.road_map.as_deref().unwrap();
        assert_eq!(rm.connection_count(), 1);
    }
}
