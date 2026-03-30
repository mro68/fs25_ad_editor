//! Use-Case: Prioritaet einer bestehenden Verbindung aendern.

use crate::app::AppState;
use crate::core::ConnectionPriority;
use std::sync::Arc;

/// Aendert die Prioritaet einer bestehenden Verbindung.
pub fn set_connection_priority(
    state: &mut AppState,
    start_id: u64,
    end_id: u64,
    priority: ConnectionPriority,
) {
    let Some(road_map_arc) = state.road_map.as_ref() else {
        return;
    };

    // Pruefe ob Verbindung existiert
    if road_map_arc.find_connection(start_id, end_id).is_none() {
        log::warn!("Verbindung {}→{} nicht gefunden", start_id, end_id);
        return;
    }

    // Snapshot VOR Mutation
    state.record_undo_snapshot();

    let Some(road_map_arc) = state.road_map.as_mut() else {
        log::warn!("Prioritaet nicht aenderbar: keine RoadMap geladen");
        return;
    };
    let road_map = Arc::make_mut(road_map_arc);
    road_map.set_connection_priority(start_id, end_id, priority);
    // Flags der betroffenen Nodes neu berechnen
    road_map.recalculate_node_flags(&[start_id, end_id]);

    log::info!(
        "Verbindung {}→{} Prioritaet auf {:?} geaendert",
        start_id,
        end_id,
        priority
    );
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::core::{
        Connection, ConnectionDirection, ConnectionPriority, MapNode, NodeFlag, RoadMap,
    };
    use glam::Vec2;

    /// Hilfsfunktion: AppState mit einer Verbindung aufbauen
    fn make_state_with_connection(priority: ConnectionPriority) -> AppState {
        let mut state = AppState::new();
        let mut map = RoadMap::new(3);
        map.add_node(MapNode::new(1, Vec2::new(0.0, 0.0), NodeFlag::Regular));
        map.add_node(MapNode::new(2, Vec2::new(10.0, 0.0), NodeFlag::Regular));
        map.add_connection(Connection::new(
            1,
            2,
            ConnectionDirection::Regular,
            priority,
            Vec2::new(0.0, 0.0),
            Vec2::new(10.0, 0.0),
        ));
        state.road_map = Some(Arc::new(map));
        state
    }

    #[test]
    fn test_set_priority_updates_connections() {
        // Priorität auf SubPriority setzen → Connection wird geändert
        let mut state = make_state_with_connection(ConnectionPriority::Regular);
        set_connection_priority(&mut state, 1, 2, ConnectionPriority::SubPriority);
        let rm = state.road_map.as_deref().unwrap();
        let conn = rm.find_connection(1, 2).unwrap();
        assert_eq!(conn.priority, ConnectionPriority::SubPriority);
    }
}
