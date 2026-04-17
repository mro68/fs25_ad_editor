//! Schwebendes Bearbeitungspanel fuer aktive Edit-Modi.
//!
//! Wird ueber dem Viewport angezeigt, wenn ein Edit-Modus aktiv ist
//! (Streckenteilung, Route-Tool). Zeigt nur die modi-spezifischen

mod group_panel;
mod route_tool_panel;
mod streckenteilung_panel;

use crate::app::state::DistanzenState;
use crate::app::state::GroupEditState;
use crate::app::ui_contract::RouteToolPanelState;
use crate::app::ToolEditStore;
use crate::app::{
    AppIntent, ConnectionDirection, ConnectionPriority, EditorTool, GroupRecord, RoadMap,
};
use crate::shared::{EditorOptions, Language};
use indexmap::IndexSet;

pub use streckenteilung_panel::render_streckenteilung_controls;

/// Rendert das Floating-Edit-Panel und gibt erzeugte Events zurueck.
///
/// Das Panel erscheint an `panel_pos` (Bildschirmkoordinaten) und zeigt
/// nur die Steuerung fuer den gerade aktiven Edit-Modus.
#[allow(clippy::too_many_arguments)]
pub fn render_edit_panel(
    ctx: &egui::Context,
    road_map: Option<&RoadMap>,
    selected_node_ids: &IndexSet<u64>,
    distanzen_state: &mut DistanzenState,
    default_direction: ConnectionDirection,
    default_priority: ConnectionPriority,
    distance_wheel_step_m: f32,
    active_tool: EditorTool,
    route_tool: Option<RouteToolPanelState>,
    panel_pos: Option<egui::Pos2>,
    group_editing: Option<&GroupEditState>,
    group_record: Option<&GroupRecord>,
    tool_edit_store: Option<&ToolEditStore>,
    options: &mut EditorOptions,
    lang: Language,
) -> Vec<AppIntent> {
    let mut events = Vec::new();

    // Gruppen-Edit-Panel (hat Vorrang vor Streckenteilung)
    if let Some(edit_state) = group_editing {
        group_panel::render_group_edit_panel(
            ctx,
            edit_state,
            group_panel::GroupEditPanelContext::new(
                group_record,
                tool_edit_store,
                road_map,
                panel_pos,
                options,
                &mut events,
            ),
        );
        return events;
    }

    // Streckenteilung Edit-Modus
    if distanzen_state.active {
        streckenteilung_panel::render_streckenteilung_panel(
            ctx,
            road_map,
            selected_node_ids,
            distanzen_state,
            distance_wheel_step_m,
            panel_pos,
            &mut events,
        );
        return events;
    }

    // Route-Tool Edit-Modus (immer wenn Tool aktiv)
    if active_tool == EditorTool::Route
        && let Some(route_tool) = route_tool
    {
        route_tool_panel::render_route_tool_panel(
            ctx,
            route_tool,
            route_tool_panel::RouteToolPanelContext::new(
                default_direction,
                default_priority,
                distance_wheel_step_m,
                panel_pos,
                lang,
                &mut events,
            ),
        );
    }

    events
}
