//! Handler für Node/Connection-Editing, Marker und Editor-Werkzeug.

use crate::app::use_cases;
use crate::app::AppState;
use crate::core::{ConnectionDirection, ConnectionPriority};

pub fn set_editor_tool(state: &mut AppState, tool: crate::app::state::EditorTool) {
    state.editor.active_tool = tool;
    state.editor.connect_source_node = None;
    log::info!("Editor-Werkzeug: {:?}", tool);
}

pub fn add_node(state: &mut AppState, world_pos: glam::Vec2) {
    use_cases::editing::add_node_at_position(state, world_pos);
}

pub fn delete_selected(state: &mut AppState) {
    use_cases::editing::delete_selected_nodes(state);
}

pub fn connect_tool_pick(state: &mut AppState, world_pos: glam::Vec2, max_distance: f32) {
    use_cases::editing::connect_tool_pick_node(state, world_pos, max_distance);
}

pub fn add_connection(
    state: &mut AppState,
    from_id: u64,
    to_id: u64,
    direction: ConnectionDirection,
    priority: ConnectionPriority,
) {
    use_cases::editing::add_connection(state, from_id, to_id, direction, priority);
}

pub fn remove_connection_between(state: &mut AppState, node_a: u64, node_b: u64) {
    use_cases::editing::remove_connection_between(state, node_a, node_b);
}

pub fn set_connection_direction(
    state: &mut AppState,
    start_id: u64,
    end_id: u64,
    direction: ConnectionDirection,
) {
    use_cases::editing::set_connection_direction(state, start_id, end_id, direction);
}

pub fn set_connection_priority(
    state: &mut AppState,
    start_id: u64,
    end_id: u64,
    priority: ConnectionPriority,
) {
    use_cases::editing::set_connection_priority(state, start_id, end_id, priority);
}

pub fn set_default_direction(state: &mut AppState, direction: ConnectionDirection) {
    state.editor.default_direction = direction;
    if let Some(tool) = state.editor.tool_manager.active_tool_mut() {
        tool.set_direction(direction);
    }
    log::info!("Standard-Verbindungsrichtung: {:?}", direction);
}

pub fn set_default_priority(state: &mut AppState, priority: ConnectionPriority) {
    state.editor.default_priority = priority;
    if let Some(tool) = state.editor.tool_manager.active_tool_mut() {
        tool.set_priority(priority);
    }
    log::info!("Standard-Straßenart: {:?}", priority);
}

pub fn set_all_directions_between_selected(
    state: &mut AppState,
    direction: ConnectionDirection,
) {
    use_cases::editing::set_all_connections_direction_between_selected(state, direction);
}

pub fn remove_all_between_selected(state: &mut AppState) {
    use_cases::editing::remove_all_connections_between_selected(state);
}

pub fn invert_all_between_selected(state: &mut AppState) {
    use_cases::editing::invert_all_connections_between_selected(state);
}

pub fn set_all_priorities_between_selected(
    state: &mut AppState,
    priority: ConnectionPriority,
) {
    use_cases::editing::set_all_connections_priority_between_selected(state, priority);
}

pub fn connect_selected(state: &mut AppState) {
    let ids: Vec<u64> = state.selection.selected_node_ids.iter().copied().collect();
    if ids.len() == 2 {
        let direction = state.editor.default_direction;
        let priority = state.editor.default_priority;
        use_cases::editing::add_connection(state, ids[0], ids[1], direction, priority);
    }
}

pub fn create_marker(state: &mut AppState, node_id: u64, name: &str, group: &str) {
    use_cases::editing::create_marker(state, node_id, name, group);
}

pub fn remove_marker(state: &mut AppState, node_id: u64) {
    use_cases::editing::remove_marker(state, node_id);
}

pub fn open_marker_dialog(state: &mut AppState, node_id: u64, is_new: bool) {
    use_cases::editing::open_marker_dialog(state, node_id, is_new);
}

pub fn update_marker(state: &mut AppState, node_id: u64, name: &str, group: &str) {
    use_cases::editing::update_marker(state, node_id, name, group);
}
