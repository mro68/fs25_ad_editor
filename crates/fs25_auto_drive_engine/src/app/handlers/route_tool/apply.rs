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
        let ids = if state.active_tool_edit_session.is_some() {
            use_cases::editing::apply_tool_result_no_snapshot(state, result)
        } else {
            use_cases::editing::apply_tool_result(state, result)
        };

        if let (Some(tool), Some(rm)) = (
            state.editor.tool_manager.active_recreate_mut(),
            state.road_map.as_deref(),
        ) {
            tool.on_applied(&ids, rm);
        }

        crate::app::tool_editing::persist_after_apply(state, &ids, &marker_indices);
    }

    if let Some(tool) = state.editor.tool_manager.active_tool_mut() {
        tool.reset();
    }
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

    if state.active_tool_edit_session.is_none() {
        state.record_undo_snapshot();
    }
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
        crate::app::tool_editing::persist_after_apply(state, &new_ids, &[]);
    }
}
