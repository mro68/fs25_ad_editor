//! Use-Case: Verbindungen zwischen zwei Nodes entfernen.

use crate::app::AppState;
use std::sync::Arc;

/// Entfernt alle Verbindungen zwischen zwei Nodes (in beiden Richtungen).
pub fn remove_connection_between(state: &mut AppState, node_a: u64, node_b: u64) {
    let Some(road_map_arc) = state.road_map.as_ref() else {
        return;
    };

    // Prüfe ob überhaupt Verbindungen existieren
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
