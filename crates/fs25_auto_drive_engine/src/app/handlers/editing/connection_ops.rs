use crate::app::use_cases;
use crate::app::AppState;
use crate::core::{ConnectionDirection, ConnectionPriority};

/// Erstellt eine Verbindung zwischen zwei Nodes.
pub fn add_connection(
    state: &mut AppState,
    from_id: u64,
    to_id: u64,
    direction: ConnectionDirection,
    priority: ConnectionPriority,
) {
    use_cases::editing::add_connection(state, from_id, to_id, direction, priority);
}

/// Entfernt Verbindungen zwischen zwei Nodes in beide Richtungen.
pub fn remove_connection_between(state: &mut AppState, node_a: u64, node_b: u64) {
    use_cases::editing::remove_connection_between(state, node_a, node_b);
}

/// Setzt die Richtung einer bestehenden Verbindung.
pub fn set_connection_direction(
    state: &mut AppState,
    start_id: u64,
    end_id: u64,
    direction: ConnectionDirection,
) {
    use_cases::editing::set_connection_direction(state, start_id, end_id, direction);
}

/// Setzt die Prioritaet einer bestehenden Verbindung.
pub fn set_connection_priority(
    state: &mut AppState,
    start_id: u64,
    end_id: u64,
    priority: ConnectionPriority,
) {
    use_cases::editing::set_connection_priority(state, start_id, end_id, priority);
}

/// Aktualisiert die Standard-Richtung fuer neue Verbindungen.
pub fn set_default_direction(state: &mut AppState, direction: ConnectionDirection) {
    state.editor.default_direction = direction;
    let host_context = crate::app::handlers::route_tool::build_host_context(state);
    state.editor.tool_manager.sync_active_host(&host_context);
    log::info!("Standard-Verbindungsrichtung: {:?}", direction);
}

/// Aktualisiert die Standard-Prioritaet fuer neue Verbindungen.
pub fn set_default_priority(state: &mut AppState, priority: ConnectionPriority) {
    state.editor.default_priority = priority;
    let host_context = crate::app::handlers::route_tool::build_host_context(state);
    state.editor.tool_manager.sync_active_host(&host_context);
    log::info!("Standard-Strassenart: {:?}", priority);
}

/// Setzt die Richtung aller Verbindungen zwischen selektierten Nodes.
pub fn set_all_directions_between_selected(state: &mut AppState, direction: ConnectionDirection) {
    use_cases::editing::set_all_connections_direction_between_selected(state, direction);
}

/// Entfernt alle Verbindungen zwischen selektierten Nodes.
pub fn remove_all_between_selected(state: &mut AppState) {
    use_cases::editing::remove_all_connections_between_selected(state);
}

/// Invertiert die Richtung aller Verbindungen zwischen selektierten Nodes.
pub fn invert_all_between_selected(state: &mut AppState) {
    use_cases::editing::invert_all_connections_between_selected(state);
}

/// Setzt die Prioritaet aller Verbindungen zwischen selektierten Nodes.
pub fn set_all_priorities_between_selected(state: &mut AppState, priority: ConnectionPriority) {
    use_cases::editing::set_all_connections_priority_between_selected(state, priority);
}

/// Verbindet genau zwei selektierte Nodes mit den aktuellen Standardwerten.
/// Die Reihenfolge (from → to) entspricht der Klick-Reihenfolge in der IndexSet.
pub fn connect_selected(state: &mut AppState) {
    let ids: Vec<u64> = state.selection.selected_node_ids.iter().copied().collect();
    if ids.len() == 2 {
        let direction = state.editor.default_direction;
        let priority = state.editor.default_priority;
        use_cases::editing::add_connection(state, ids[0], ids[1], direction, priority);
    }
}
