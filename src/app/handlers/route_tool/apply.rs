use crate::app::use_cases;
use crate::app::AppState;

/// Fuehrt das aktive Route-Tool aus, wendet das Ergebnis an und registriert ggf. die Gruppe neu.
pub(super) fn execute_and_apply(state: &mut AppState) {
    let result = match (
        state.editor.tool_manager.active_tool(),
        state.road_map.as_deref(),
    ) {
        (Some(tool), Some(rm)) => tool.execute(rm),
        _ => None,
    };

    if let Some(result) = result {
        let marker_indices: Vec<usize> = result.markers.iter().map(|(idx, _, _)| *idx).collect();
        let ids = use_cases::editing::apply_tool_result(state, result);

        if let (Some(tool), Some(rm)) = (
            state.editor.tool_manager.active_recreate_mut(),
            state.road_map.as_deref(),
        ) {
            tool.on_applied(&ids, rm);
        }

        let record_id = state.group_registry.next_id();
        if let Some(tool) = state.editor.tool_manager.active_tool() {
            if let Some(mut record) = tool.make_group_record(record_id, &ids) {
                record.original_positions = record
                    .node_ids
                    .iter()
                    .filter_map(|id| state.road_map.as_ref()?.node(*id).map(|n| n.position))
                    .collect();
                record.marker_node_ids = marker_indices
                    .iter()
                    .filter_map(|idx| ids.get(*idx).copied())
                    .collect();
                state.group_registry.register(record);
            }
        }
    }

    if let Some(tool) = state.editor.tool_manager.active_tool_mut() {
        tool.reset();
    }
    state.tool_editing_record_id = None;
    state.tool_editing_record_backup = None;
}

/// Loescht die letzte Strecke und erstellt sie mit neuen Parametern neu.
pub(super) fn recreate(state: &mut AppState) {
    let old_ids = match state.editor.tool_manager.active_recreate() {
        Some(tool) => {
            let ids = tool.last_created_ids();
            if ids.is_empty() {
                return;
            }
            ids.to_vec()
        }
        None => return,
    };

    state.record_undo_snapshot();
    use_cases::editing::delete_nodes_by_ids(state, &old_ids);

    let result = match (
        state.editor.tool_manager.active_recreate(),
        state.road_map.as_deref(),
    ) {
        (Some(tool), Some(rm)) => tool.execute_from_anchors(rm),
        _ => None,
    };

    if let Some(result) = result {
        let new_ids = use_cases::editing::apply_tool_result_no_snapshot(state, result);
        if let (Some(tool), Some(rm)) = (
            state.editor.tool_manager.active_recreate_mut(),
            state.road_map.as_deref(),
        ) {
            tool.clear_recreate_flag();
            tool.on_applied(&new_ids, rm);
        }
        let record_id = state.group_registry.next_id();
        if let Some(tool) = state.editor.tool_manager.active_tool() {
            if let Some(mut record) = tool.make_group_record(record_id, &new_ids) {
                record.original_positions = record
                    .node_ids
                    .iter()
                    .filter_map(|id| state.road_map.as_ref()?.node(*id).map(|n| n.position))
                    .collect();
                record.marker_node_ids = Vec::new();
                state.group_registry.register(record);
            }
        }
    }
}
