use crate::app::use_cases;
use crate::app::AppState;
use crate::core::NodeFlag;

/// Aktiviert ein Editor-Werkzeug und setzt tool-spezifische Zwischenselektion zurueck.
pub fn set_editor_tool(state: &mut AppState, tool: crate::app::state::EditorTool) {
    state.editor.active_tool = tool;
    state.editor.connect_source_node = None;
    log::info!("Editor-Werkzeug: {:?}", tool);
}

/// Fuegt einen neuen Node an der uebergebenen Weltposition hinzu.
/// Trifft der Klick einen existierenden Node, wird dieser nur selektiert.
pub fn add_node(state: &mut AppState, world_pos: glam::Vec2) {
    match use_cases::editing::add_node_at_position(state, world_pos) {
        use_cases::editing::AddNodeResult::NoMap => {
            state.ui.status_message = Some("Kein Node hinzufuegbar: keine RoadMap geladen".into());
        }
        use_cases::editing::AddNodeResult::SelectedExisting(node_id) => {
            state.ui.status_message = None;
            log::info!("AddNode: Existierenden Node {} selektiert", node_id);
        }
        use_cases::editing::AddNodeResult::Created(node_id) => {
            state.ui.status_message = None;
            log::debug!("AddNode: Neuen Node {} erstellt", node_id);
        }
    }
}

/// Loescht alle aktuell selektierten Nodes.
pub fn delete_selected(state: &mut AppState) {
    use_cases::editing::delete_selected_nodes(state);
}

/// Verarbeitet einen Pick fuer das Connect-Tool.
pub fn connect_tool_pick(state: &mut AppState, world_pos: glam::Vec2, max_distance: f32) {
    use_cases::editing::connect_tool_pick_node(state, world_pos, max_distance);
}

/// Setzt das Flag eines bestehenden Nodes.
pub fn set_node_flag(state: &mut AppState, node_id: u64, flag: NodeFlag) {
    use_cases::editing::set_node_flag(state, node_id, flag);
}

/// Verteilt die selektierten Nodes gleichmaessig entlang eines Catmull-Rom-Splines.
pub fn resample_path(state: &mut AppState) {
    use_cases::editing::resample_selected_path(state);
}

/// Zeichnet alle erkannten Farmland-Polygone als Wegpunkt-Ring nach (Batch-Operation).
pub fn trace_all_fields(
    state: &mut AppState,
    spacing: f32,
    offset: f32,
    tolerance: f32,
    corner_angle: Option<f32>,
    corner_rounding_radius: Option<f32>,
    corner_rounding_max_angle_deg: Option<f32>,
) {
    use_cases::editing::trace_all_fields(
        state,
        spacing,
        offset,
        tolerance,
        corner_angle,
        corner_rounding_radius,
        corner_rounding_max_angle_deg,
    );
}

/// Aktiviert die Streckenteilung wenn mindestens 2 Nodes selektiert sind.
pub fn streckenteilung_aktivieren(state: &mut AppState) {
    if state.selection.selected_node_ids.len() >= 2 {
        state.ui.distanzen.active = true;
        if state.ui.distanzen.distance < 1.0 {
            state.ui.distanzen.distance = 1.0;
        }
        if state.ui.distanzen.count < 2 {
            state.ui.distanzen.sync_from_distance();
        }
    }
}
