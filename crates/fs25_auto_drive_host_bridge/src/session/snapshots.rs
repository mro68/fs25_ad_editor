use fs25_auto_drive_engine::app::state::DistanzenState;
use fs25_auto_drive_engine::app::{AppState, GroupRecord, RoadMap};
use glam::Vec2;
use indexmap::IndexSet;
use std::collections::HashMap;

use super::mappings::{map_active_tool, map_route_tool_id};
use super::HostLocalDialogState;
use crate::dto::{
    HostDialogSnapshot, HostEditableGroupSummary, HostEditingOptionsSnapshot, HostEditingSnapshot,
    HostFieldDetectionSource, HostGroupBoundaryCandidateSnapshot, HostGroupEditSnapshot,
    HostOverviewLayersSnapshot, HostOverviewSourceContext, HostResampleEditSnapshot,
    HostResampleMode, HostSelectionSnapshot, HostSessionSnapshot, HostViewportSnapshot,
};

const MAX_RESAMPLE_CHAIN_NODES: usize = 500;

pub(super) fn build_snapshot(
    state: &AppState,
    chrome: &HostLocalDialogState,
    pending_dialog_request_count: usize,
) -> HostSessionSnapshot {
    HostSessionSnapshot {
        has_map: state.road_map.is_some(),
        is_dirty: state.is_dirty(),
        node_count: state.node_count(),
        connection_count: state.connection_count(),
        active_tool: map_active_tool(state.editor.active_tool),
        status_message: state.ui.status_message.clone(),
        show_command_palette: chrome.show_command_palette,
        show_options_dialog: chrome.show_options_dialog,
        can_undo: state.can_undo(),
        can_redo: state.can_redo(),
        pending_dialog_request_count,
        selection: HostSelectionSnapshot {
            selected_node_ids: state.selection.selected_node_ids.iter().copied().collect(),
        },
        viewport: HostViewportSnapshot {
            camera_position: [state.view.camera.position.x, state.view.camera.position.y],
            zoom: state.view.camera.zoom,
        },
    }
}

pub(super) fn build_dialog_snapshot(
    state: &AppState,
    chrome: &HostLocalDialogState,
) -> HostDialogSnapshot {
    let zip_browser = chrome.zip_browser.as_ref();

    HostDialogSnapshot {
        heightmap_warning: crate::dto::HostHeightmapWarningDialogSnapshot {
            visible: chrome.show_heightmap_warning,
            confirmed_for_current_save: chrome.heightmap_warning_confirmed,
        },
        marker_dialog: crate::dto::HostMarkerDialogSnapshot {
            visible: chrome.marker_dialog.visible,
            node_id: chrome.marker_dialog.node_id,
            name: chrome.marker_dialog.name.clone(),
            group: chrome.marker_dialog.group.clone(),
            is_new: chrome.marker_dialog.is_new,
        },
        dedup_dialog: crate::dto::HostDedupDialogSnapshot {
            visible: chrome.dedup_dialog.visible,
            duplicate_count: chrome.dedup_dialog.duplicate_count,
            group_count: chrome.dedup_dialog.group_count,
        },
        zip_browser: crate::dto::HostZipBrowserSnapshot {
            visible: zip_browser.is_some(),
            zip_path: zip_browser
                .map(|browser| browser.zip_path.clone())
                .unwrap_or_default(),
            entries: zip_browser
                .map(|browser| {
                    browser
                        .entries
                        .iter()
                        .map(|entry| crate::dto::HostZipImageEntrySnapshot {
                            name: entry.name.clone(),
                            size: entry.size,
                        })
                        .collect()
                })
                .unwrap_or_default(),
            selected_entry_index: zip_browser.and_then(|browser| browser.selected),
            filter_overview: zip_browser.is_some_and(|browser| browser.filter_overview),
        },
        overview_options_dialog: crate::dto::HostOverviewOptionsDialogSnapshot {
            visible: chrome.overview_options_dialog.visible,
            zip_path: chrome.overview_options_dialog.zip_path.clone(),
            layers: HostOverviewLayersSnapshot::from(&chrome.overview_options_dialog.layers),
            field_detection_source: HostFieldDetectionSource::from(
                chrome.overview_options_dialog.field_detection_source,
            ),
            available_sources: chrome
                .overview_options_dialog
                .available_sources
                .iter()
                .copied()
                .map(HostFieldDetectionSource::from)
                .collect(),
        },
        post_load_dialog: crate::dto::HostPostLoadDialogSnapshot {
            visible: chrome.post_load_dialog.visible,
            context: HostOverviewSourceContext::from(chrome.post_load_dialog.context),
            heightmap_set: chrome.post_load_dialog.heightmap_set,
            heightmap_path: chrome.post_load_dialog.heightmap_path.clone(),
            overview_loaded: chrome.post_load_dialog.overview_loaded,
            matching_zip_paths: chrome
                .post_load_dialog
                .matching_zips
                .iter()
                .map(|path| path.to_string_lossy().into_owned())
                .collect(),
            selected_zip_index: chrome.post_load_dialog.selected_zip_index,
            map_name: chrome.post_load_dialog.map_name.clone(),
        },
        save_overview_dialog: crate::dto::HostSaveOverviewDialogSnapshot {
            visible: chrome.save_overview_dialog.visible,
            target_path: chrome.save_overview_dialog.target_path.clone(),
            is_overwrite: chrome.save_overview_dialog.is_overwrite,
        },
        trace_all_fields_dialog: crate::dto::HostTraceAllFieldsDialogSnapshot {
            visible: chrome.trace_all_fields_dialog.visible,
            spacing: chrome.trace_all_fields_dialog.spacing,
            offset: chrome.trace_all_fields_dialog.offset,
            tolerance: chrome.trace_all_fields_dialog.tolerance,
            corner_detection_enabled: chrome.trace_all_fields_dialog.corner_detection_enabled,
            corner_angle_threshold_deg: chrome.trace_all_fields_dialog.corner_angle_threshold_deg,
            corner_rounding_enabled: chrome.trace_all_fields_dialog.corner_rounding_enabled,
            corner_rounding_radius: chrome.trace_all_fields_dialog.corner_rounding_radius,
            corner_rounding_max_angle_deg: chrome
                .trace_all_fields_dialog
                .corner_rounding_max_angle_deg,
        },
        group_settings_popup: crate::dto::HostGroupSettingsDialogSnapshot {
            visible: chrome.group_settings_popup.visible,
            world_pos: [
                chrome.group_settings_popup.world_pos.x,
                chrome.group_settings_popup.world_pos.y,
            ],
            segment_stop_at_junction: state.options.segment_stop_at_junction,
            segment_max_angle_deg: state.options.segment_max_angle_deg,
        },
        confirm_dissolve_group: crate::dto::HostConfirmDissolveDialogSnapshot {
            visible: chrome.confirm_dissolve_group_id.is_some(),
            segment_id: chrome.confirm_dissolve_group_id,
        },
    }
}

pub(super) fn build_editing_snapshot(state: &AppState) -> HostEditingSnapshot {
    HostEditingSnapshot {
        editable_groups: build_editable_group_summaries(state),
        group_edit: build_group_edit_snapshot(state),
        resample: build_resample_snapshot(state),
        options: HostEditingOptionsSnapshot {
            render_quality: state.view.render_quality,
            background_visible: state.view.background_visible,
            background_scale: state.view.background_scale,
            show_all_group_boundaries: state.options.show_all_group_boundaries,
            segment_stop_at_junction: state.options.segment_stop_at_junction,
            segment_max_angle_deg: state.options.segment_max_angle_deg,
            mouse_wheel_distance_step_m: state.options.mouse_wheel_distance_step_m,
        },
    }
}

fn build_editable_group_summaries(state: &AppState) -> Vec<HostEditableGroupSummary> {
    let mut groups: Vec<HostEditableGroupSummary> = state
        .group_registry
        .find_by_node_ids(&state.selection.selected_node_ids)
        .into_iter()
        .map(|record| HostEditableGroupSummary {
            record_id: record.id,
            node_count: record.node_ids.len(),
            locked: record.locked,
            tool_id: state
                .tool_edit_store
                .tool_id_for(record.id)
                .map(map_route_tool_id),
            has_tool_edit: state.tool_edit_store.contains(record.id),
            entry_node_id: record.entry_node_id,
            exit_node_id: record.exit_node_id,
        })
        .collect();
    groups.sort_by_key(|group| group.record_id);
    groups
}

fn build_group_edit_snapshot(state: &AppState) -> Option<HostGroupEditSnapshot> {
    let edit_state = state.group_editing.as_ref()?;
    let record = state.group_registry.get(edit_state.record_id);

    Some(HostGroupEditSnapshot {
        record_id: edit_state.record_id,
        locked: record.is_some_and(|record| record.locked),
        was_locked_before_edit: edit_state.was_locked,
        node_count: record.map_or(0, |record| record.node_ids.len()),
        tool_id: state
            .tool_edit_store
            .tool_id_for(edit_state.record_id)
            .map(map_route_tool_id),
        has_tool_edit: state.tool_edit_store.contains(edit_state.record_id),
        entry_node_id: record.and_then(|record| record.entry_node_id),
        exit_node_id: record.and_then(|record| record.exit_node_id),
        boundary_candidates: record
            .map(|record| build_group_boundary_candidates(record, state.road_map.as_deref()))
            .unwrap_or_default(),
    })
}

fn build_group_boundary_candidates(
    record: &GroupRecord,
    road_map: Option<&RoadMap>,
) -> Vec<HostGroupBoundaryCandidateSnapshot> {
    let boundary_nodes_by_id: HashMap<u64, fs25_auto_drive_engine::core::BoundaryNode> = road_map
        .map(|road_map| {
            let group_ids: IndexSet<u64> = record.node_ids.iter().copied().collect();
            road_map
                .boundary_nodes(&group_ids)
                .into_iter()
                .map(|boundary| (boundary.node_id, boundary))
                .collect()
        })
        .unwrap_or_default();

    let mut seen = std::collections::HashSet::new();
    let mut candidates = Vec::with_capacity(record.node_ids.len());

    for &node_id in &record.node_ids {
        if !seen.insert(node_id) {
            continue;
        }

        let boundary = boundary_nodes_by_id.get(&node_id);
        candidates.push(HostGroupBoundaryCandidateSnapshot {
            node_id,
            position: road_map
                .and_then(|road_map| road_map.node(node_id))
                .map(|node| [node.position.x, node.position.y]),
            has_external_incoming: boundary.is_some_and(|boundary| boundary.has_external_incoming),
            has_external_outgoing: boundary.is_some_and(|boundary| boundary.has_external_outgoing),
        });
    }

    for node_id in [record.entry_node_id, record.exit_node_id]
        .into_iter()
        .flatten()
    {
        if !seen.insert(node_id) {
            continue;
        }

        let boundary = boundary_nodes_by_id.get(&node_id);
        candidates.push(HostGroupBoundaryCandidateSnapshot {
            node_id,
            position: road_map
                .and_then(|road_map| road_map.node(node_id))
                .map(|node| [node.position.x, node.position.y]),
            has_external_incoming: boundary.is_some_and(|boundary| boundary.has_external_incoming),
            has_external_outgoing: boundary.is_some_and(|boundary| boundary.has_external_outgoing),
        });
    }

    candidates
}

fn build_resample_snapshot(state: &AppState) -> HostResampleEditSnapshot {
    let distanzen = &state.ui.distanzen;
    let selected_node_count = state.selection.selected_node_ids.len();
    let mut can_resample_current_selection = false;
    let mut path_length = 0.0;
    let mut preview_count = 0usize;

    if let Some(road_map) = state.road_map.as_deref()
        && let Some((computed_path_length, computed_preview_count)) =
            compute_resample_chain_metrics(road_map, &state.selection.selected_node_ids, distanzen)
    {
        can_resample_current_selection = true;
        path_length = computed_path_length;
        if distanzen.active {
            preview_count = computed_preview_count;
        }
    }

    HostResampleEditSnapshot {
        active: distanzen.active,
        can_resample_current_selection,
        selected_node_count,
        mode: if distanzen.by_count {
            HostResampleMode::Count
        } else {
            HostResampleMode::Distance
        },
        distance: distanzen.distance,
        count: distanzen.count,
        path_length,
        hide_original: distanzen.hide_original,
        preview_count,
    }
}

fn compute_resample_chain_metrics(
    road_map: &RoadMap,
    selected_node_ids: &IndexSet<u64>,
    distanzen: &DistanzenState,
) -> Option<(f32, usize)> {
    use fs25_auto_drive_engine::shared::spline_geometry::{
        catmull_rom_chain_with_tangents, polyline_length, resample_by_distance,
    };

    if !(2..=MAX_RESAMPLE_CHAIN_NODES).contains(&selected_node_ids.len()) {
        return None;
    }

    let ordered = road_map.ordered_chain_nodes(selected_node_ids)?;
    let positions: Vec<Vec2> = ordered
        .iter()
        .filter_map(|node_id| road_map.node(*node_id).map(|node| node.position))
        .collect();
    if positions.len() < 2 {
        return None;
    }

    let dense = catmull_rom_chain_with_tangents(&positions, 16, None, None);
    let path_length = polyline_length(&dense);
    let preview_count = if path_length <= f32::EPSILON {
        positions.len()
    } else if distanzen.by_count {
        let count = distanzen.count.max(2) as usize;
        let step = path_length / (count - 1) as f32;
        resample_by_distance(&dense, step).len()
    } else {
        resample_by_distance(&dense, distanzen.distance.max(0.1)).len()
    };

    Some((path_length, preview_count))
}
