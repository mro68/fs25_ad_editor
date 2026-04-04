//! Builder fuer host-neutrale Viewport-Overlay-Snapshots.

use crate::app::ui_contract::{
    ClipboardOverlaySnapshot, ClipboardPreviewNode, GroupBoundaryOverlaySnapshot,
    GroupLockOverlaySnapshot, PolylineOverlaySnapshot, ViewportOverlaySnapshot,
};
use crate::app::AppState;
use crate::core::RoadMap;
use glam::Vec2;
use std::collections::{HashMap, HashSet};

/// Baut den aktuellen host-neutralen Overlay-Snapshot aus dem AppState.
pub fn build(state: &mut AppState, cursor_world: Option<Vec2>) -> ViewportOverlaySnapshot {
    let road_map_arc = state.road_map.clone();
    let road_map = road_map_arc.as_deref();

    let route_tool_preview = match (cursor_world, road_map) {
        (Some(cursor), Some(map)) => state.editor.route_tool_preview(cursor, map),
        _ => None,
    };

    let clipboard_preview = build_clipboard_preview(state);
    let distance_preview = build_distance_preview(state);

    let mut snapshot = ViewportOverlaySnapshot {
        route_tool_preview,
        clipboard_preview,
        distance_preview,
        group_locks: Vec::new(),
        group_boundaries: Vec::new(),
        show_no_file_hint: road_map.is_none(),
    };

    let Some(road_map) = road_map else {
        return snapshot;
    };

    snapshot.group_locks = build_group_lock_overlays(state, road_map);
    snapshot.group_boundaries = build_group_boundary_overlays(state, road_map);

    snapshot
}

fn build_clipboard_preview(state: &AppState) -> Option<ClipboardOverlaySnapshot> {
    let paste_pos = state.paste_preview_pos?;
    if state.clipboard.nodes.is_empty() {
        return None;
    }

    let offset = paste_pos - state.clipboard.center;

    let marker_ids: HashSet<u64> = state
        .clipboard
        .markers
        .iter()
        .map(|marker| marker.id)
        .collect();
    let mut id_to_index: HashMap<u64, usize> = HashMap::with_capacity(state.clipboard.nodes.len());
    let mut nodes = Vec::with_capacity(state.clipboard.nodes.len());

    for (index, node) in state.clipboard.nodes.iter().enumerate() {
        id_to_index.insert(node.id, index);
        nodes.push(ClipboardPreviewNode {
            world_pos: node.position + offset,
            has_marker: marker_ids.contains(&node.id),
        });
    }

    let mut connections = Vec::with_capacity(state.clipboard.connections.len());
    for connection in &state.clipboard.connections {
        let Some(&start_idx) = id_to_index.get(&connection.start_id) else {
            continue;
        };
        let Some(&end_idx) = id_to_index.get(&connection.end_id) else {
            continue;
        };
        connections.push((start_idx, end_idx));
    }

    Some(ClipboardOverlaySnapshot {
        nodes,
        connections,
        opacity: state.options.copy_preview_opacity,
    })
}

fn build_distance_preview(state: &AppState) -> Option<PolylineOverlaySnapshot> {
    if !state.ui.distanzen.active || state.ui.distanzen.preview_positions.is_empty() {
        return None;
    }

    Some(PolylineOverlaySnapshot {
        points: state.ui.distanzen.preview_positions.clone(),
    })
}

fn build_group_lock_overlays(
    state: &AppState,
    road_map: &RoadMap,
) -> Vec<GroupLockOverlaySnapshot> {
    if state.group_registry.is_empty() || state.selection.selected_node_ids.is_empty() {
        return Vec::new();
    }

    let mut overlays = Vec::new();
    let mut seen_segment_ids = HashSet::new();

    for selected_id in state.selection.selected_node_ids.iter().copied() {
        let Some(record) = state.group_registry.find_first_by_node_id(selected_id) else {
            continue;
        };
        if !seen_segment_ids.insert(record.id) {
            continue;
        }
        if !state.group_registry.is_group_valid(record, road_map) {
            continue;
        }
        let Some(node) = road_map.node(selected_id) else {
            continue;
        };

        overlays.push(GroupLockOverlaySnapshot {
            segment_id: record.id,
            world_pos: node.position,
            locked: record.locked,
        });
    }

    overlays
}

fn build_group_boundary_overlays(
    state: &mut AppState,
    road_map: &RoadMap,
) -> Vec<GroupBoundaryOverlaySnapshot> {
    if state.group_registry.is_empty() || state.selection.selected_node_ids.is_empty() {
        return Vec::new();
    }

    let show_all = state.options.show_all_group_boundaries;

    state.group_registry.warm_boundary_cache(road_map);
    let records = state
        .group_registry
        .find_by_node_ids(&state.selection.selected_node_ids);

    let mut overlays = Vec::new();

    for record in records {
        if !state.group_registry.is_group_valid(record, road_map) {
            continue;
        }

        if let Some(entry_node_id) = record.entry_node_id {
            if let Some(node) = road_map.node(entry_node_id) {
                push_unique_boundary_overlay(
                    &mut overlays,
                    GroupBoundaryOverlaySnapshot {
                        segment_id: record.id,
                        node_id: entry_node_id,
                        world_pos: node.position,
                        direction: crate::app::BoundaryDirection::Entry,
                    },
                );
            }
        }

        if let Some(exit_node_id) = record.exit_node_id {
            if let Some(node) = road_map.node(exit_node_id) {
                push_unique_boundary_overlay(
                    &mut overlays,
                    GroupBoundaryOverlaySnapshot {
                        segment_id: record.id,
                        node_id: exit_node_id,
                        world_pos: node.position,
                        direction: crate::app::BoundaryDirection::Exit,
                    },
                );
            }
        }

        if !show_all {
            continue;
        }

        let Some(boundary_infos) = state.group_registry.boundary_cache_for(record.id) else {
            continue;
        };

        for boundary in boundary_infos {
            if let Some(node) = road_map.node(boundary.node_id) {
                push_unique_boundary_overlay(
                    &mut overlays,
                    GroupBoundaryOverlaySnapshot {
                        segment_id: record.id,
                        node_id: boundary.node_id,
                        world_pos: node.position,
                        direction: boundary.direction,
                    },
                );
            }
        }
    }

    overlays
}

fn push_unique_boundary_overlay(
    overlays: &mut Vec<GroupBoundaryOverlaySnapshot>,
    candidate: GroupBoundaryOverlaySnapshot,
) {
    let exists = overlays.iter().any(|item| {
        item.segment_id == candidate.segment_id
            && item.node_id == candidate.node_id
            && item.direction == candidate.direction
    });

    if !exists {
        overlays.push(candidate);
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::app::MapNode;
    use crate::core::NodeFlag;

    #[test]
    fn build_marks_no_file_hint_without_map() {
        let mut state = AppState::new();

        let snapshot = build(&mut state, None);

        assert!(snapshot.show_no_file_hint);
        assert!(snapshot.route_tool_preview.is_none());
    }

    #[test]
    fn build_exposes_clipboard_preview_nodes_and_connections() {
        let mut state = AppState::new();
        state.clipboard.center = Vec2::new(10.0, 10.0);
        state
            .clipboard
            .nodes
            .push(MapNode::new(1, Vec2::new(8.0, 10.0), NodeFlag::Regular));
        state
            .clipboard
            .nodes
            .push(MapNode::new(2, Vec2::new(12.0, 10.0), NodeFlag::Regular));
        state
            .clipboard
            .connections
            .push(crate::app::Connection::new(
                1,
                2,
                crate::app::ConnectionDirection::Regular,
                crate::app::ConnectionPriority::Regular,
                Vec2::new(8.0, 10.0),
                Vec2::new(12.0, 10.0),
            ));
        state.paste_preview_pos = Some(Vec2::new(20.0, 20.0));

        let snapshot = build(&mut state, None);
        let clipboard = snapshot
            .clipboard_preview
            .expect("Clipboard-Preview muss vorhanden sein");

        assert_eq!(clipboard.nodes.len(), 2);
        assert_eq!(clipboard.connections, vec![(0, 1)]);
        assert_eq!(clipboard.nodes[0].world_pos, Vec2::new(18.0, 20.0));
        assert_eq!(clipboard.nodes[1].world_pos, Vec2::new(22.0, 20.0));
    }
}
