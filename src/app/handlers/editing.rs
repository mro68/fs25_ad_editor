//! Handler für Node/Connection-Editing, Marker und Editor-Werkzeug.

use crate::app::use_cases;
use crate::app::AppState;
use crate::core::{ConnectionDirection, ConnectionPriority};

/// Aktiviert ein Editor-Werkzeug und setzt tool-spezifische Zwischenselektion zurück.
pub fn set_editor_tool(state: &mut AppState, tool: crate::app::state::EditorTool) {
    state.editor.active_tool = tool;
    state.editor.connect_source_node = None;
    log::info!("Editor-Werkzeug: {:?}", tool);
}

/// Fügt einen neuen Node an der übergebenen Weltposition hinzu.
pub fn add_node(state: &mut AppState, world_pos: glam::Vec2) {
    use_cases::editing::add_node_at_position(state, world_pos);
}

/// Löscht alle aktuell selektierten Nodes.
pub fn delete_selected(state: &mut AppState) {
    use_cases::editing::delete_selected_nodes(state);
}

/// Verarbeitet einen Pick für das Connect-Tool.
pub fn connect_tool_pick(state: &mut AppState, world_pos: glam::Vec2, max_distance: f32) {
    use_cases::editing::connect_tool_pick_node(state, world_pos, max_distance);
}
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

/// Setzt die Priorität einer bestehenden Verbindung.
pub fn set_connection_priority(
    state: &mut AppState,
    start_id: u64,
    end_id: u64,
    priority: ConnectionPriority,
) {
    use_cases::editing::set_connection_priority(state, start_id, end_id, priority);
}

/// Aktualisiert die Standard-Richtung für neue Verbindungen.
pub fn set_default_direction(state: &mut AppState, direction: ConnectionDirection) {
    state.editor.default_direction = direction;
    if let Some(tool) = state.editor.tool_manager.active_tool_mut() {
        tool.set_direction(direction);
    }
    log::info!("Standard-Verbindungsrichtung: {:?}", direction);
}

/// Aktualisiert die Standard-Priorität für neue Verbindungen.
pub fn set_default_priority(state: &mut AppState, priority: ConnectionPriority) {
    state.editor.default_priority = priority;
    if let Some(tool) = state.editor.tool_manager.active_tool_mut() {
        tool.set_priority(priority);
    }
    log::info!("Standard-Straßenart: {:?}", priority);
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

/// Setzt die Priorität aller Verbindungen zwischen selektierten Nodes.
pub fn set_all_priorities_between_selected(state: &mut AppState, priority: ConnectionPriority) {
    use_cases::editing::set_all_connections_priority_between_selected(state, priority);
}

/// Verbindet genau zwei selektierte Nodes mit den aktuellen Standardwerten.
pub fn connect_selected(state: &mut AppState) {
    let ids: Vec<u64> = state.selection.selected_node_ids.iter().copied().collect();
    if ids.len() == 2 {
        let direction = state.editor.default_direction;
        let priority = state.editor.default_priority;
        use_cases::editing::add_connection(state, ids[0], ids[1], direction, priority);
    }
}

/// Erstellt einen Map-Marker für einen Node.
pub fn create_marker(state: &mut AppState, node_id: u64, name: &str, group: &str) {
    use_cases::editing::create_marker(state, node_id, name, group);
}

/// Entfernt den Map-Marker eines Nodes.
pub fn remove_marker(state: &mut AppState, node_id: u64) {
    use_cases::editing::remove_marker(state, node_id);
}

/// Öffnet den Marker-Dialog zum Erstellen oder Bearbeiten.
pub fn open_marker_dialog(state: &mut AppState, node_id: u64, is_new: bool) {
    use_cases::editing::open_marker_dialog(state, node_id, is_new);
}

/// Aktualisiert Name/Gruppe eines bestehenden Markers.
pub fn update_marker(state: &mut AppState, node_id: u64, name: &str, group: &str) {
    use_cases::editing::update_marker(state, node_id, name, group);
}

/// Lädt ein gespeichertes Segment zur nachträglichen Bearbeitung.
///
/// Löscht die zugehörigen Nodes aus der RoadMap, aktiviert das passende
/// Route-Tool und befüllt es mit den gespeicherten Parametern.
pub fn edit_segment(state: &mut AppState, record_id: u64) {
    use crate::app::state::EditorTool;

    // Record aus Registry holen (Klon, da wir state danach mutieren)
    let record = match state.segment_registry.get(record_id) {
        Some(r) => r.clone(),
        None => {
            log::warn!("Segment-Record {} nicht gefunden", record_id);
            return;
        }
    };

    let tool_index = record.kind.tool_index();

    // Undo-Snapshot vor Löschung
    state.record_undo_snapshot();

    // Segment-Nodes löschen
    use_cases::editing::delete_nodes_by_ids(state, &record.node_ids.clone());

    // Record aus Registry entfernen (wird beim erneuten execute() neu angelegt)
    state.segment_registry.remove(record_id);

    // Passendes Route-Tool aktivieren
    super::route_tool::select(state, tool_index);
    state.editor.active_tool = EditorTool::Route;

    // Tool mit gespeicherten Parametern laden
    if let Some(tool) = state.editor.tool_manager.active_tool_mut() {
        let kind = record.kind.clone();
        tool.load_for_edit(&record, &kind);
    }

    log::info!(
        "Segment {} geladen für Bearbeitung (Tool-Index {})",
        record_id,
        tool_index
    );
}

/// Verteilt die selektierten Nodes gleichmäßig entlang eines Catmull-Rom-Splines.
pub fn resample_path(state: &mut AppState) {
    use_cases::editing::resample_selected_path(state);
}
