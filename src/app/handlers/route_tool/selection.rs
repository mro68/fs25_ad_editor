use crate::app::state::EditorTool;
use crate::app::tool_contract::RouteToolId;
use crate::app::tools::{route_tool_descriptor, OrderedNodeChain, ToolAction};
use crate::app::AppState;

/// Aktiviert ein Route-Tool per stabiler Tool-ID.
pub(super) fn select(state: &mut AppState, tool_id: RouteToolId) {
    let descriptor = route_tool_descriptor(tool_id);
    state.editor.tool_manager.set_active_by_id(tool_id);
    state.editor.remember_route_tool(descriptor.group, tool_id);
    state.editor.active_tool = EditorTool::Route;
    state.editor.connect_source_node = None;
    let host_context = super::build_host_context(state);
    state.editor.tool_manager.sync_active_host(&host_context);

    init_chain_if_needed(state);

    log::info!("Route-Tool aktiviert: {:?}", tool_id);
}

/// Laedt die aktuelle Selektion als geordnete Kette in das aktive Tool,
/// falls dieses `needs_chain_input()` zurueckgibt.
pub(super) fn init_chain_if_needed(state: &mut AppState) {
    if state.editor.tool_manager.active_chain_input().is_none() {
        return;
    }

    let Some(road_map) = state.road_map.as_deref() else {
        return;
    };

    let Some(ordered_ids) = road_map.ordered_chain_nodes(&state.selection.selected_node_ids) else {
        log::debug!("Bypass-Tool: Selektion bildet keine lineare Kette");
        return;
    };

    let positions: Vec<glam::Vec2> = ordered_ids
        .iter()
        .filter_map(|id| road_map.node(*id).map(|n| n.position))
        .collect();

    if positions.len() < 2 {
        return;
    }

    let start_id = *ordered_ids
        .first()
        .expect("invariant: ordered_ids ist nicht-leer nach positions.len()<2-Guard");
    let end_id = *ordered_ids
        .last()
        .expect("invariant: ordered_ids ist nicht-leer nach positions.len()<2-Guard");

    let inner_ids = if ordered_ids.len() > 2 {
        ordered_ids[1..ordered_ids.len() - 1].to_vec()
    } else {
        Vec::new()
    };

    state
        .editor
        .tool_manager
        .load_active_chain(OrderedNodeChain {
            positions,
            start_id,
            end_id,
            inner_ids,
        });
    log::info!(
        "Route-Tool Kette geladen: {} Nodes ({} → {})",
        ordered_ids.len(),
        start_id,
        end_id
    );
}

/// Aktiviert ein Route-Tool und setzt Start/End-Anker aus zwei selektierten Nodes.
pub(super) fn select_with_anchors(
    state: &mut AppState,
    tool_id: RouteToolId,
    start_node_id: u64,
    end_node_id: u64,
) {
    select(state, tool_id);

    if let Some(tool) = state.editor.tool_manager.active_tool_mut() {
        tool.reset();
    }

    let (start_pos, end_pos) = {
        let Some(road_map) = state.road_map.as_deref() else {
            return;
        };
        let start = road_map.node(start_node_id);
        let end = road_map.node(end_node_id);
        match (start, end) {
            (Some(s), Some(e)) => (s.position, e.position),
            _ => {
                log::warn!(
                    "Route-Tool mit Ankern: Nodes {}/{} nicht gefunden",
                    start_node_id,
                    end_node_id
                );
                return;
            }
        }
    };

    let (old_selected, old_anchor) =
        crate::app::handlers::helpers::capture_selection_snapshot(state);
    state.selection.ids_mut().clear();
    crate::app::handlers::helpers::record_selection_if_changed(state, old_selected, old_anchor);

    let action1 = {
        let Some(road_map) = state.road_map.as_deref() else {
            return;
        };
        let Some(tool) = state.editor.tool_manager.active_tool_mut() else {
            return;
        };
        tool.on_click(start_pos, road_map, false)
    };

    if action1 == ToolAction::ReadyToExecute {
        super::apply::execute_and_apply(state);
        return;
    }

    let action2 = {
        let Some(road_map) = state.road_map.as_deref() else {
            return;
        };
        let Some(tool) = state.editor.tool_manager.active_tool_mut() else {
            return;
        };
        tool.on_click(end_pos, road_map, false)
    };

    if action2 == ToolAction::ReadyToExecute {
        super::apply::execute_and_apply(state);
    }
}
