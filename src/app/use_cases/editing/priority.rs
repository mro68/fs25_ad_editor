//! Use-Case: Priorität einer bestehenden Verbindung ändern.

use crate::app::AppState;
use crate::core::ConnectionPriority;
use std::sync::Arc;

/// Ändert die Priorität einer bestehenden Verbindung.
pub fn set_connection_priority(
    state: &mut AppState,
    start_id: u64,
    end_id: u64,
    priority: ConnectionPriority,
) {
    let Some(road_map_arc) = state.road_map.as_ref() else {
        return;
    };

    // Prüfe ob Verbindung existiert
    if road_map_arc.find_connection(start_id, end_id).is_none() {
        log::warn!("Verbindung {}→{} nicht gefunden", start_id, end_id);
        return;
    }

    // Snapshot VOR Mutation
    state.record_undo_snapshot();

    let Some(road_map_arc) = state.road_map.as_mut() else {
        log::warn!("Priorität nicht änderbar: keine RoadMap geladen");
        return;
    };
    let road_map = Arc::make_mut(road_map_arc);
    road_map.set_connection_priority(start_id, end_id, priority);
    // Flags der betroffenen Nodes neu berechnen
    road_map.recalculate_node_flags(&[start_id, end_id]);

    log::info!(
        "Verbindung {}→{} Priorität auf {:?} geändert",
        start_id,
        end_id,
        priority
    );
}
