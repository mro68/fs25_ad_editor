//! Orchestrierung fuer Tool-Edit-Persistenz und Rehydrierung.

use std::collections::HashSet;

use crate::app::group_registry::GroupRecord;
use crate::app::state::EditorTool;
use crate::app::tool_contract::RouteToolId;
use crate::app::tools::{route_tool_descriptor, ToolHostContext};
use crate::app::use_cases;
use crate::app::AppState;

use super::{ActiveToolEditSession, RouteToolEditPayload, ToolEditRecord};

/// Registriert einen gruppenbasierten Session-Record plus Payload im Store.
pub(crate) fn register_persisted_group(
    state: &mut AppState,
    record_id: Option<u64>,
    tool_id: RouteToolId,
    payload: RouteToolEditPayload,
    node_ids: &[u64],
    marker_node_ids: Vec<u64>,
) -> Option<u64> {
    let road_map = state.road_map.as_deref()?;
    let group_id = record_id.unwrap_or_else(|| state.group_registry.next_id());
    let defaults = payload.group_record_defaults(node_ids);
    let original_positions = node_ids
        .iter()
        .filter_map(|id| road_map.node(*id).map(|node| node.position))
        .collect();

    state.group_registry.register(GroupRecord {
        id: group_id,
        node_ids: node_ids.to_vec(),
        original_positions,
        marker_node_ids,
        locked: defaults.locked,
        entry_node_id: defaults.entry_node_id,
        exit_node_id: defaults.exit_node_id,
    });
    state.tool_edit_store.insert(ToolEditRecord {
        group_id,
        tool_id,
        payload,
    });
    Some(group_id)
}

/// Persistiert die aktuelle Tool-Ausfuehrung als GroupRecord plus ToolEditRecord.
pub(crate) fn persist_after_apply(
    state: &mut AppState,
    node_ids: &[u64],
    marker_indices: &[usize],
) {
    let Some(tool_id) = state.editor.tool_manager.active_id() else {
        state.active_tool_edit_session = None;
        return;
    };
    let Some(payload) = state
        .editor
        .tool_manager
        .active_group_edit()
        .and_then(|tool| tool.build_edit_payload())
    else {
        state.active_tool_edit_session = None;
        return;
    };

    let marker_node_ids = marker_indices
        .iter()
        .filter_map(|idx| node_ids.get(*idx).copied())
        .collect();
    let reused_record_id = state
        .active_tool_edit_session
        .as_ref()
        .map(|session| session.record_id);

    let _ = register_persisted_group(
        state,
        reused_record_id,
        tool_id,
        payload,
        node_ids,
        marker_node_ids,
    );
    state.active_tool_edit_session = None;
}

/// Startet den destruktiven Tool-Edit-Flow fuer eine gruppenbasierte Payload.
pub(crate) fn begin_edit(state: &mut AppState, record_id: u64) {
    if state.active_tool_edit_session.is_some() {
        log::warn!(
            "Tool-Edit bereits aktiv, neuer Start fuer Record {} ignoriert",
            record_id
        );
        return;
    }

    let Some(group_record) = state.group_registry.get(record_id).cloned() else {
        log::warn!("Segment-Record {} nicht gefunden", record_id);
        return;
    };
    let Some(tool_edit_backup) = state.tool_edit_store.get(record_id).cloned() else {
        log::warn!(
            "Segment {} hat keinen Tool-Edit-Snapshot, Bearbeitung nicht moeglich",
            record_id
        );
        return;
    };

    activate_tool_for_edit(state, tool_edit_backup.tool_id);
    if state.editor.tool_manager.active_group_edit_mut().is_none() {
        log::warn!(
            "Aktives Tool {:?} bietet keine Group-Edit-Capability",
            tool_edit_backup.tool_id
        );
        return;
    }

    state.record_undo_snapshot();

    if !group_record.marker_node_ids.is_empty() {
        use std::sync::Arc;
        if let Some(road_map_arc) = state.road_map.as_mut() {
            let road_map = Arc::make_mut(road_map_arc);
            for &node_id in &group_record.marker_node_ids {
                road_map.remove_marker(node_id);
            }
        }
    }

    let protected_anchor_ids: HashSet<u64> = tool_edit_backup
        .payload
        .protected_anchor_ids()
        .into_iter()
        .collect();
    let inner_ids: Vec<u64> = group_record
        .node_ids
        .iter()
        .copied()
        .filter(|id| !protected_anchor_ids.contains(id))
        .collect();

    state.tool_edit_store.remove(record_id);
    use_cases::editing::delete_nodes_by_ids(state, &inner_ids);
    state.group_registry.remove(record_id);

    if let Some(tool) = state.editor.tool_manager.active_group_edit_mut() {
        tool.restore_edit_payload(&tool_edit_backup.payload);
    }

    state.active_tool_edit_session = Some(ActiveToolEditSession {
        record_id,
        group_record_backup: group_record,
        tool_edit_backup: tool_edit_backup.clone(),
    });

    log::info!(
        "Segment {} geladen fuer Bearbeitung ({:?}, Gruppe {:?})",
        record_id,
        tool_edit_backup.tool_id,
        route_tool_descriptor(tool_edit_backup.tool_id).group
    );
}

/// Bricht einen aktiven Tool-Edit ab und stellt Registry plus Payload-Store wieder her.
pub(crate) fn cancel_active_edit(state: &mut AppState) {
    let Some(session) = state.active_tool_edit_session.take() else {
        return;
    };

    if let Some(tool) = state.editor.tool_manager.active_tool_mut() {
        tool.reset();
    }
    state.editor.active_tool = EditorTool::Select;
    state.editor.connect_source_node = None;

    if !crate::app::handlers::history::restore_last_snapshot_without_redo(state) {
        log::warn!(
            "Tool-Edit-Abbruch ohne Undo-Snapshot fuer Record {}",
            session.record_id
        );
    }

    state.group_registry.register(session.group_record_backup);
    state.tool_edit_store.insert(session.tool_edit_backup);
    log::info!("Tool-Edit abgebrochen: Snapshot, Registry und Payload-Store wiederhergestellt");
}

fn activate_tool_for_edit(state: &mut AppState, tool_id: RouteToolId) {
    let descriptor = route_tool_descriptor(tool_id);
    state.editor.tool_manager.set_active_by_id(tool_id);
    state.editor.remember_route_tool(descriptor.group, tool_id);
    state.editor.active_tool = EditorTool::Route;
    state.editor.connect_source_node = None;

    let host_context = ToolHostContext {
        direction: state.editor.default_direction,
        priority: state.editor.default_priority,
        snap_radius: state.options.snap_radius(),
        farmland_data: state.farmland_polygons_arc(),
        farmland_grid: state.farmland_grid_arc(),
        background_image: state.background_image_arc(),
    };
    state.editor.tool_manager.sync_active_host(&host_context);
}
