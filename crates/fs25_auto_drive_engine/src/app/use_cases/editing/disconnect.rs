//! Use-Case: Verbindungen zwischen zwei Nodes entfernen.

use crate::app::AppState;
use std::sync::Arc;

/// Entfernt alle Verbindungen zwischen zwei Nodes (in beiden Richtungen).
pub fn remove_connection_between(state: &mut AppState, node_a: u64, node_b: u64) {
    let Some(road_map_arc) = state.road_map.as_ref() else {
        return;
    };

    // Pruefe ob ueberhaupt Verbindungen existieren
    if road_map_arc
        .find_connections_between(node_a, node_b)
        .is_empty()
    {
        log::debug!(
            "Keine Verbindung zwischen {} und {} gefunden",
            node_a,
            node_b
        );
        return;
    }

    // Snapshot VOR Mutation
    state.record_undo_snapshot();

    let Some(road_map_arc) = state.road_map.as_mut() else {
        log::warn!("Verbindungen nicht entfernbar: keine RoadMap geladen");
        return;
    };
    let road_map = Arc::make_mut(road_map_arc);
    let removed = road_map.remove_connections_between(node_a, node_b);
    // Flags der betroffenen Nodes neu berechnen
    road_map.recalculate_node_flags(&[node_a, node_b]);

    log::info!(
        "{} Verbindung(en) zwischen {} und {} entfernt",
        removed,
        node_a,
        node_b
    );
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::core::{
        Connection, ConnectionDirection, ConnectionPriority, MapNode, NodeFlag, RoadMap,
    };
    use glam::Vec2;

    #[test]
    fn test_disconnect_removes_both_directions() {
        // A↔B verbunden → disconnect → keine Connections mehr
        let mut state = AppState::new();
        let mut map = RoadMap::new(3);
        map.add_node(MapNode::new(1, Vec2::new(0.0, 0.0), NodeFlag::Regular));
        map.add_node(MapNode::new(2, Vec2::new(10.0, 0.0), NodeFlag::Regular));
        map.add_connection(Connection::new(
            1,
            2,
            ConnectionDirection::Regular,
            ConnectionPriority::Regular,
            Vec2::new(0.0, 0.0),
            Vec2::new(10.0, 0.0),
        ));
        map.add_connection(Connection::new(
            2,
            1,
            ConnectionDirection::Regular,
            ConnectionPriority::Regular,
            Vec2::new(10.0, 0.0),
            Vec2::new(0.0, 0.0),
        ));
        state.road_map = Some(Arc::new(map));

        remove_connection_between(&mut state, 1, 2);
        let rm = state.road_map.as_deref().unwrap();
        assert!(!rm.has_connection(1, 2));
        assert!(!rm.has_connection(2, 1));
    }

    #[test]
    fn test_disconnect_nonexistent_is_noop() {
        // Nicht verbundene Nodes → kein Fehler, keine Änderung
        let mut state = AppState::new();
        let mut map = RoadMap::new(3);
        map.add_node(MapNode::new(1, Vec2::new(0.0, 0.0), NodeFlag::Regular));
        map.add_node(MapNode::new(2, Vec2::new(10.0, 0.0), NodeFlag::Regular));
        state.road_map = Some(Arc::new(map));

        remove_connection_between(&mut state, 1, 2);
        let rm = state.road_map.as_deref().unwrap();
        assert_eq!(rm.connection_count(), 0);
    }
}
