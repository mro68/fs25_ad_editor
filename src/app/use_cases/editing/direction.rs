//! Use-Case: Richtung einer bestehenden Verbindung ändern.

use crate::app::AppState;
use crate::core::ConnectionDirection;
use std::sync::Arc;

/// Ändert die Richtung einer bestehenden Verbindung.
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

    // Prüfe ob Verbindung existiert
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
            "Verbindung {}→{} nicht änderbar: keine RoadMap geladen",
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
        "Verbindung {}→{} auf {:?} geändert",
        start_id,
        end_id,
        direction
    );
}
