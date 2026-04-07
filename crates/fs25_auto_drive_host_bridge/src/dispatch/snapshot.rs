//! Snapshot-Builder-Funktionen fuer host-neutrale Read-Snapshots.

use fs25_auto_drive_engine::app::projections;
use fs25_auto_drive_engine::app::ui_contract::{RouteToolViewportData, ViewportOverlaySnapshot};
use fs25_auto_drive_engine::app::AppState;
use fs25_auto_drive_engine::shared::{RenderAssetsSnapshot, RenderScene};
use glam::Vec2;

use crate::dto::{
    HostChromeSnapshot, HostRouteToolViewportSnapshot, HostViewportConnectionSnapshot,
    HostViewportGeometrySnapshot, HostViewportMarkerSnapshot, HostViewportNodeSnapshot,
};
use crate::session::HostRenderFrameSnapshot;

use super::mappings::{
    build_route_tool_entries_snapshot, build_route_tool_selection_snapshot,
    map_connection_direction, map_connection_priority, map_editor_tool,
    map_render_connection_direction, map_render_connection_priority, map_render_node_kind,
    map_route_tool_id, map_tangent_menu_data,
};

fn build_viewport_geometry_snapshot_from_scene(
    scene: &RenderScene,
) -> HostViewportGeometrySnapshot {
    let camera = scene.camera();
    let viewport_size = scene.viewport_size();
    let selected_node_ids = scene.selected_node_ids();
    let hidden_node_ids = scene.hidden_node_ids();
    let dimmed_node_ids = scene.dimmed_node_ids();

    let mut nodes = Vec::new();
    let mut connections = Vec::new();
    let mut markers = Vec::new();

    if let Some(map) = scene.map() {
        nodes = map
            .nodes()
            .map(|node| HostViewportNodeSnapshot {
                id: node.id,
                position: [node.position.x, node.position.y],
                kind: map_render_node_kind(node.kind),
                preserve_when_decimating: node.preserve_when_decimating,
                selected: selected_node_ids.contains(&node.id),
                hidden: hidden_node_ids.contains(&node.id),
                dimmed: dimmed_node_ids.contains(&node.id),
            })
            .collect();
        nodes.sort_by_key(|node| node.id);

        connections = map
            .connections()
            .iter()
            .map(|connection| {
                let hidden = hidden_node_ids.contains(&connection.start_id)
                    || hidden_node_ids.contains(&connection.end_id);
                let dimmed = !hidden
                    && (dimmed_node_ids.contains(&connection.start_id)
                        || dimmed_node_ids.contains(&connection.end_id));

                HostViewportConnectionSnapshot {
                    start_id: connection.start_id,
                    end_id: connection.end_id,
                    start_position: [connection.start_pos.x, connection.start_pos.y],
                    end_position: [connection.end_pos.x, connection.end_pos.y],
                    direction: map_render_connection_direction(connection.direction),
                    priority: map_render_connection_priority(connection.priority),
                    hidden,
                    dimmed,
                }
            })
            .collect();
        connections.sort_by_key(|connection| (connection.start_id, connection.end_id));

        markers = map
            .markers()
            .iter()
            .map(|marker| HostViewportMarkerSnapshot {
                position: [marker.position.x, marker.position.y],
            })
            .collect();
        markers.sort_by(|left, right| {
            left.position[0]
                .total_cmp(&right.position[0])
                .then(left.position[1].total_cmp(&right.position[1]))
        });
    }

    HostViewportGeometrySnapshot {
        has_map: scene.has_map(),
        viewport_size,
        camera_position: [camera.position.x, camera.position.y],
        zoom: camera.zoom,
        world_per_pixel: camera.world_per_pixel(viewport_size[1]),
        has_background: scene.has_background(),
        background_visible: scene.background_visible(),
        nodes,
        connections,
        markers,
    }
}

fn map_route_tool_viewport_data(data: RouteToolViewportData) -> HostRouteToolViewportSnapshot {
    HostRouteToolViewportSnapshot {
        drag_targets: data
            .drag_targets
            .into_iter()
            .map(|point| [point.x, point.y])
            .collect(),
        has_pending_input: data.has_pending_input,
        segment_shortcuts_active: data.segment_shortcuts_active,
        tangent_menu_data: data.tangent_menu_data.map(map_tangent_menu_data),
        needs_lasso_input: data.needs_lasso_input,
    }
}

/// Baut den host-neutralen Panel-Snapshot fuer Hosts mit lokalem State.
pub fn build_host_ui_snapshot(
    state: &AppState,
) -> fs25_auto_drive_engine::app::ui_contract::HostUiSnapshot {
    projections::build_host_ui_snapshot(state)
}

/// Baut den host-neutralen Chrome-Snapshot fuer Menues, Defaults und Status.
pub fn build_host_chrome_snapshot(state: &AppState) -> HostChromeSnapshot {
    let (node_count, connection_count, marker_count, map_name) = state
        .road_map
        .as_ref()
        .map(|rm| {
            (
                rm.node_count(),
                rm.connection_count(),
                rm.marker_count(),
                rm.map_name.clone(),
            )
        })
        .unwrap_or((0, 0, 0, None));
    let selection_count = state.selection.selected_node_ids.len();
    let selection_example_id = state
        .selection
        .selected_node_ids
        .iter()
        .next()
        .copied();
    HostChromeSnapshot {
        status_message: state.ui.status_message.clone(),
        show_command_palette: state.ui.show_command_palette,
        show_options_dialog: state.ui.show_options_dialog,
        has_map: state.road_map.is_some(),
        has_selection: !state.selection.selected_node_ids.is_empty(),
        has_clipboard: !state.clipboard.nodes.is_empty(),
        can_undo: state.can_undo(),
        can_redo: state.can_redo(),
        active_tool: map_editor_tool(state.editor.active_tool),
        active_route_tool: state.active_route_tool_id().map(map_route_tool_id),
        default_direction: map_connection_direction(state.editor.default_direction),
        default_priority: map_connection_priority(state.editor.default_priority),
        route_tool_memory: build_route_tool_selection_snapshot(state),
        options: state.options.clone(),
        route_tool_entries: build_route_tool_entries_snapshot(state),
        node_count,
        connection_count,
        marker_count,
        map_name,
        camera_zoom: state.view.camera.zoom,
        camera_position: [state.view.camera.position.x, state.view.camera.position.y],
        heightmap_path: state.ui.heightmap_path.clone(),
        selection_count,
        selection_example_id,
        background_map_loaded: state.view.background_map.is_some(),
        render_quality: state.view.render_quality,
        has_farmland: state.has_farmland_polygons(),
        background_visible: state.view.background_visible,
        background_scale: state.view.background_scale,
    }
}

/// Baut den host-neutralen Route-Tool-Viewport-Snapshot fuer lokale Host-Adapter.
pub fn build_route_tool_viewport_snapshot(state: &AppState) -> HostRouteToolViewportSnapshot {
    map_route_tool_viewport_data(state.editor.route_tool_viewport_data())
}

/// Baut den host-neutralen Viewport-Overlay-Snapshot fuer lokale Host-Adapter.
///
/// Die mutable State-Referenz bleibt noetig, weil beim Aufbau Caches im
/// `AppState` aufgewaermt werden koennen.
pub fn build_viewport_overlay_snapshot(
    state: &mut AppState,
    cursor_world: Option<Vec2>,
) -> ViewportOverlaySnapshot {
    projections::build_viewport_overlay_snapshot(state, cursor_world)
}

/// Baut den per-frame Render-Vertrag fuer lokale Host-Adapter.
pub fn build_render_scene(
    state: &AppState,
    viewport_size: [f32; 2],
) -> RenderScene {
    projections::build_render_scene(state, viewport_size)
}

/// Baut den langlebigen Render-Asset-Snapshot fuer lokale Host-Adapter.
pub fn build_render_assets(state: &AppState) -> RenderAssetsSnapshot {
    projections::build_render_assets(state)
}

/// Baut Szene und Assets als gekoppelten read-only Render-Frame fuer lokale Hosts.
pub fn build_render_frame(
    state: &AppState,
    viewport_size: [f32; 2],
) -> HostRenderFrameSnapshot {
    HostRenderFrameSnapshot {
        scene: build_render_scene(state, viewport_size),
        assets: build_render_assets(state),
    }
}

/// Baut einen minimalen, serialisierbaren Viewport-Geometry-Snapshot fuer Hosts.
pub fn build_viewport_geometry_snapshot(
    state: &AppState,
    viewport_size: [f32; 2],
) -> HostViewportGeometrySnapshot {
    let scene = projections::build_render_scene(state, viewport_size);
    build_viewport_geometry_snapshot_from_scene(&scene)
}
