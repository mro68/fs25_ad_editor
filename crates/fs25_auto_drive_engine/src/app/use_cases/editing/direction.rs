//! Use-Case: Richtung einer bestehenden Verbindung aendern.

use crate::app::AppState;
use crate::core::ConnectionDirection;
use std::sync::Arc;

/// Aendert die Richtung einer bestehenden Verbindung.
///
/// Bei Wechsel von Dual auf Regular/Reverse wird eine eventuell vorhandene
/// Gegen-Connection (end→start mit Dual) entfernt, damit keine Geister-Pfeile bleiben.
pub fn set_connection_direction(
    state: &mut AppState,
    start_id: u64,
    end_id: u64,
    direction: ConnectionDirection,
) {
    let Some(road_map_arc) = state.road_map.as_ref() else {
        return;
    };

    // Pruefe ob Verbindung existiert
    let old_direction = match road_map_arc.find_connection(start_id, end_id) {
        Some(conn) => conn.direction,
        None => {
            log::warn!("Verbindung {}→{} nicht gefunden", start_id, end_id);
            return;
        }
    };

    // Snapshot VOR Mutation
    state.record_undo_snapshot();

    let Some(road_map_arc) = state.road_map.as_mut() else {
        log::warn!(
            "Verbindung {}→{} nicht aenderbar: keine RoadMap geladen",
            start_id,
            end_id
        );
        return;
    };
    let road_map = Arc::make_mut(road_map_arc);
    road_map.set_connection_direction(start_id, end_id, direction);

    // Bei Wechsel weg von Dual: Gegen-Connection entfernen (falls vorhanden)
    if old_direction == ConnectionDirection::Dual
        && direction != ConnectionDirection::Dual
        && road_map.remove_connection(end_id, start_id)
    {
        log::debug!(
            "Gegen-Connection {}→{} entfernt (Dual → {:?})",
            end_id,
            start_id,
            direction
        );
    }

    road_map.recalculate_node_flags(&[start_id, end_id]);

    log::info!(
        "Verbindung {}→{} auf {:?} geaendert",
        start_id,
        end_id,
        direction
    );
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::core::{
        Connection, ConnectionDirection, ConnectionPriority, MapNode, NodeFlag, RoadMap,
    };
    use glam::Vec2;

    /// Hilfsfunktion: AppState mit zwei Nodes und einer Verbindung A→B aufbauen
    fn make_state_ab(dir: ConnectionDirection) -> AppState {
        let mut state = AppState::new();
        let mut map = RoadMap::new(3);
        map.add_node(MapNode::new(1, Vec2::new(0.0, 0.0), NodeFlag::Regular));
        map.add_node(MapNode::new(2, Vec2::new(10.0, 0.0), NodeFlag::Regular));
        map.add_connection(Connection::new(
            1,
            2,
            dir,
            ConnectionPriority::Regular,
            Vec2::new(0.0, 0.0),
            Vec2::new(10.0, 0.0),
        ));
        state.road_map = Some(Arc::new(map));
        state
    }

    #[test]
    fn test_toggle_dual_creates_reverse() {
        // A→B (Regular) + B→A (Regular) bereits vorhanden → toggle A→B auf Dual → beide bleiben
        let mut state = make_state_ab(ConnectionDirection::Regular);
        // Rückwärts-Connection manuell hinzufügen
        Arc::make_mut(state.road_map.as_mut().unwrap()).add_connection(Connection::new(
            2,
            1,
            ConnectionDirection::Regular,
            ConnectionPriority::Regular,
            Vec2::new(10.0, 0.0),
            Vec2::new(0.0, 0.0),
        ));
        set_connection_direction(&mut state, 1, 2, ConnectionDirection::Dual);
        let rm = state.road_map.as_deref().unwrap();
        // Beide Verbindungen müssen noch existieren (kein Entfernen beim Wechsel zu Dual)
        assert!(rm.has_connection(1, 2));
        assert!(rm.has_connection(2, 1));
    }

    #[test]
    fn test_toggle_regular_removes_reverse() {
        // A→B (Dual) + B→A (Dual) → toggle A→B auf Regular → B→A wird entfernt
        let mut state = make_state_ab(ConnectionDirection::Dual);
        Arc::make_mut(state.road_map.as_mut().unwrap()).add_connection(Connection::new(
            2,
            1,
            ConnectionDirection::Dual,
            ConnectionPriority::Regular,
            Vec2::new(10.0, 0.0),
            Vec2::new(0.0, 0.0),
        ));
        set_connection_direction(&mut state, 1, 2, ConnectionDirection::Regular);
        let rm = state.road_map.as_deref().unwrap();
        assert!(rm.has_connection(1, 2));
        assert!(!rm.has_connection(2, 1));
    }
}
