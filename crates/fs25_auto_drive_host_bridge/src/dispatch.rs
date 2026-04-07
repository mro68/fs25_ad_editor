use anyhow::{anyhow, bail, Result};
use fs25_auto_drive_engine::app::tool_contract::{RouteToolId, TangentSource};
use fs25_auto_drive_engine::app::tools::{
    resolve_route_tool_entries, RouteToolAvailabilityContext, RouteToolDisabledReason,
    RouteToolGroup, RouteToolIconKey, RouteToolSurface,
};
use fs25_auto_drive_engine::app::ui_contract::{
    dialog_result_to_intent, DialogRequest, DialogRequestKind, DialogResult, HostUiSnapshot,
    RouteToolViewportData, TangentMenuData, TangentOptionData, ViewportOverlaySnapshot,
};
use fs25_auto_drive_engine::app::{
    AppController, AppIntent, AppState, ConnectionDirection, ConnectionPriority, EditorTool,
};
use fs25_auto_drive_engine::shared::{
    RenderAssetsSnapshot, RenderConnectionDirection, RenderConnectionPriority, RenderNodeKind,
    RenderScene,
};
use glam::Vec2;

use crate::dto::{
    HostActiveTool, HostChromeSnapshot, HostDefaultConnectionDirection,
    HostDefaultConnectionPriority, HostDialogRequest, HostDialogRequestKind, HostDialogResult,
    HostInputModifiers, HostPointerButton, HostRouteToolAction, HostRouteToolDisabledReason,
    HostRouteToolEntrySnapshot, HostRouteToolGroup, HostRouteToolIconKey, HostRouteToolId,
    HostRouteToolSelectionSnapshot, HostRouteToolSurface, HostRouteToolViewportSnapshot,
    HostSessionAction, HostTangentMenuSnapshot, HostTangentOptionSnapshot, HostTangentSource,
    HostTapKind, HostViewportConnectionDirection, HostViewportConnectionPriority,
    HostViewportConnectionSnapshot, HostViewportGeometrySnapshot, HostViewportInputBatch,
    HostViewportInputEvent, HostViewportMarkerSnapshot, HostViewportNodeKind,
    HostViewportNodeSnapshot,
};
use crate::session::HostRenderFrameSnapshot;

#[derive(Debug, Clone, PartialEq)]
enum HostViewportDragKind {
    CameraPan,
    SelectionMove,
    RectSelection {
        start_screen: [f32; 2],
        additive: bool,
    },
    LassoSelection {
        additive: bool,
        points_screen: Vec<[f32; 2]>,
    },
}

#[derive(Debug, Clone, PartialEq)]
struct HostViewportDragState {
    button: HostPointerButton,
    latest_screen: [f32; 2],
    kind: HostViewportDragKind,
}

/// Kleiner bridge-owned Input-Zustand fuer Viewport-Drag- und Resize-Lifecycles.
#[derive(Debug, Clone, Default, PartialEq)]
pub struct HostViewportInputState {
    viewport_size: [f32; 2],
    active_drag: Option<HostViewportDragState>,
}

impl HostViewportInputState {
    fn remember_viewport_size(&mut self, size: [f32; 2]) {
        self.viewport_size = size;
    }

    fn effective_viewport_size(&self, state: &AppState) -> [f32; 2] {
        if self.viewport_size[0] > 0.0 && self.viewport_size[1] > 0.0 {
            self.viewport_size
        } else {
            state.view.viewport_size
        }
    }
}

fn validate_resize_size(size: [f32; 2]) -> Result<()> {
    if !size[0].is_finite() || !size[1].is_finite() {
        bail!("viewport size must be finite");
    }
    if size[0] < 0.0 || size[1] < 0.0 {
        bail!("viewport size must not be negative");
    }
    Ok(())
}

fn require_projection_viewport_size(size: [f32; 2]) -> Result<[f32; 2]> {
    if !size[0].is_finite() || !size[1].is_finite() || size[0] <= 0.0 || size[1] <= 0.0 {
        bail!("viewport input requires a positive finite viewport size; send resize first");
    }
    Ok(size)
}

fn screen_pos_to_world(
    camera: &fs25_auto_drive_engine::app::Camera2D,
    viewport_size: [f32; 2],
    screen_pos: [f32; 2],
) -> Result<Vec2> {
    let viewport_size = require_projection_viewport_size(viewport_size)?;
    Ok(camera.screen_to_world(
        Vec2::new(screen_pos[0], screen_pos[1]),
        Vec2::new(viewport_size[0], viewport_size[1]),
    ))
}

fn delta_px_to_world(
    camera: &fs25_auto_drive_engine::app::Camera2D,
    viewport_size: [f32; 2],
    delta_px: [f32; 2],
) -> Result<Vec2> {
    let viewport_size = require_projection_viewport_size(viewport_size)?;
    let world_per_pixel = camera.world_per_pixel(viewport_size[1]);
    Ok(Vec2::new(
        delta_px[0] * world_per_pixel,
        delta_px[1] * world_per_pixel,
    ))
}

fn apply_intent(
    controller: &mut AppController,
    state: &mut AppState,
    intent: AppIntent,
) -> Result<()> {
    controller.handle_intent(state, intent)
}

fn apply_primary_tap(
    controller: &mut AppController,
    state: &mut AppState,
    viewport_size: [f32; 2],
    screen_pos: [f32; 2],
    modifiers: HostInputModifiers,
) -> Result<bool> {
    let world_pos = screen_pos_to_world(&state.view.camera, viewport_size, screen_pos)?;

    match state.editor.active_tool {
        EditorTool::Select => {
            let extend_path = modifiers.shift;
            let additive = modifiers.command || extend_path;
            apply_intent(
                controller,
                state,
                AppIntent::NodePickRequested {
                    world_pos,
                    additive,
                    extend_path,
                },
            )?;
            Ok(true)
        }
        EditorTool::AddNode => {
            apply_intent(controller, state, AppIntent::AddNodeRequested { world_pos })?;
            Ok(true)
        }
        EditorTool::Connect => {
            apply_intent(
                controller,
                state,
                AppIntent::ConnectToolNodeClicked { world_pos },
            )?;
            Ok(true)
        }
        EditorTool::Route => Ok(false),
    }
}

fn apply_primary_double_tap(
    controller: &mut AppController,
    state: &mut AppState,
    viewport_size: [f32; 2],
    screen_pos: [f32; 2],
    modifiers: HostInputModifiers,
) -> Result<bool> {
    let world_pos = screen_pos_to_world(&state.view.camera, viewport_size, screen_pos)?;
    apply_intent(
        controller,
        state,
        AppIntent::NodeSegmentBetweenIntersectionsRequested {
            world_pos,
            additive: modifiers.command,
        },
    )?;
    Ok(true)
}

fn push_lasso_point(points_screen: &mut Vec<[f32; 2]>, screen_pos: [f32; 2]) {
    let min_distance_sq = 3.0 * 3.0;
    let should_push = points_screen.last().is_none_or(|last| {
        let dx = last[0] - screen_pos[0];
        let dy = last[1] - screen_pos[1];
        (dx * dx) + (dy * dy) >= min_distance_sq
    });

    if should_push {
        points_screen.push(screen_pos);
    }
}

fn start_primary_drag(
    controller: &mut AppController,
    state: &mut AppState,
    input_state: &mut HostViewportInputState,
    screen_pos: [f32; 2],
    modifiers: HostInputModifiers,
) -> Result<bool> {
    if modifiers.alt {
        let mut points_screen = Vec::with_capacity(16);
        push_lasso_point(&mut points_screen, screen_pos);
        input_state.active_drag = Some(HostViewportDragState {
            button: HostPointerButton::Primary,
            latest_screen: screen_pos,
            kind: HostViewportDragKind::LassoSelection {
                additive: modifiers.command,
                points_screen,
            },
        });
        return Ok(true);
    }

    if state.editor.active_tool == EditorTool::Select && modifiers.shift {
        input_state.active_drag = Some(HostViewportDragState {
            button: HostPointerButton::Primary,
            latest_screen: screen_pos,
            kind: HostViewportDragKind::RectSelection {
                start_screen: screen_pos,
                additive: modifiers.command,
            },
        });
        return Ok(true);
    }

    if state.editor.active_tool == EditorTool::Select {
        let viewport_size = input_state.effective_viewport_size(state);
        let world_pos = screen_pos_to_world(&state.view.camera, viewport_size, screen_pos)?;
        let base_max_distance = state.options.hitbox_radius();
        let move_max_distance = base_max_distance * state.options.selection_size_multiplier();

        let hit = state
            .road_map
            .as_ref()
            .and_then(|road_map| road_map.nearest_node(world_pos))
            .filter(|hit| hit.distance <= move_max_distance);

        if let Some(hit) = hit {
            let already_selected = state.selection.selected_node_ids.contains(&hit.node_id);
            if !already_selected {
                apply_intent(
                    controller,
                    state,
                    AppIntent::NodePickRequested {
                        world_pos,
                        additive: modifiers.command,
                        extend_path: false,
                    },
                )?;
            }

            apply_intent(
                controller,
                state,
                AppIntent::BeginMoveSelectedNodesRequested,
            )?;
            input_state.active_drag = Some(HostViewportDragState {
                button: HostPointerButton::Primary,
                latest_screen: screen_pos,
                kind: HostViewportDragKind::SelectionMove,
            });
            return Ok(true);
        }
    }

    input_state.active_drag = Some(HostViewportDragState {
        button: HostPointerButton::Primary,
        latest_screen: screen_pos,
        kind: HostViewportDragKind::CameraPan,
    });
    Ok(true)
}

fn apply_viewport_input_event(
    controller: &mut AppController,
    state: &mut AppState,
    input_state: &mut HostViewportInputState,
    event: HostViewportInputEvent,
) -> Result<bool> {
    match event {
        HostViewportInputEvent::Resize { size_px } => {
            validate_resize_size(size_px)?;
            input_state.remember_viewport_size(size_px);
            apply_intent(
                controller,
                state,
                AppIntent::ViewportResized { size: size_px },
            )?;
            Ok(true)
        }
        HostViewportInputEvent::Tap {
            button,
            tap_kind,
            screen_pos,
            modifiers,
        } => {
            if button != HostPointerButton::Primary {
                return Ok(false);
            }

            let viewport_size = input_state.effective_viewport_size(state);
            match tap_kind {
                HostTapKind::Single => {
                    apply_primary_tap(controller, state, viewport_size, screen_pos, modifiers)
                }
                HostTapKind::Double => apply_primary_double_tap(
                    controller,
                    state,
                    viewport_size,
                    screen_pos,
                    modifiers,
                ),
            }
        }
        HostViewportInputEvent::DragStart {
            button,
            screen_pos,
            modifiers,
        } => match button {
            HostPointerButton::Primary => {
                start_primary_drag(controller, state, input_state, screen_pos, modifiers)
            }
            HostPointerButton::Middle | HostPointerButton::Secondary => {
                input_state.active_drag = Some(HostViewportDragState {
                    button,
                    latest_screen: screen_pos,
                    kind: HostViewportDragKind::CameraPan,
                });
                Ok(true)
            }
        },
        HostViewportInputEvent::DragUpdate {
            button,
            screen_pos,
            delta_px,
        } => {
            let viewport_size = input_state.effective_viewport_size(state);
            let Some(active_drag) = input_state.active_drag.as_mut() else {
                return Ok(false);
            };
            if active_drag.button != button {
                return Ok(false);
            }

            active_drag.latest_screen = screen_pos;

            match &mut active_drag.kind {
                HostViewportDragKind::CameraPan => {
                    let delta_world =
                        delta_px_to_world(&state.view.camera, viewport_size, delta_px)?;
                    apply_intent(
                        controller,
                        state,
                        AppIntent::CameraPan {
                            delta: Vec2::new(-delta_world.x, -delta_world.y),
                        },
                    )?;
                    Ok(true)
                }
                HostViewportDragKind::SelectionMove => {
                    if state.selection.selected_node_ids.is_empty() {
                        return Ok(false);
                    }

                    let delta_world =
                        delta_px_to_world(&state.view.camera, viewport_size, delta_px)?;
                    apply_intent(
                        controller,
                        state,
                        AppIntent::MoveSelectedNodesRequested { delta_world },
                    )?;
                    Ok(true)
                }
                HostViewportDragKind::RectSelection { .. } => Ok(true),
                HostViewportDragKind::LassoSelection { points_screen, .. } => {
                    push_lasso_point(points_screen, screen_pos);
                    Ok(true)
                }
            }
        }
        HostViewportInputEvent::DragEnd { button, screen_pos } => {
            let viewport_size = input_state.effective_viewport_size(state);
            let Some(active_drag) = input_state.active_drag.take() else {
                return Ok(false);
            };
            if active_drag.button != button {
                input_state.active_drag = Some(active_drag);
                return Ok(false);
            }

            let final_screen = screen_pos.unwrap_or(active_drag.latest_screen);

            match active_drag.kind {
                HostViewportDragKind::CameraPan => Ok(true),
                HostViewportDragKind::SelectionMove => {
                    apply_intent(controller, state, AppIntent::EndMoveSelectedNodesRequested)?;
                    Ok(true)
                }
                HostViewportDragKind::RectSelection {
                    start_screen,
                    additive,
                } => {
                    let min = screen_pos_to_world(&state.view.camera, viewport_size, start_screen)?;
                    let max = screen_pos_to_world(&state.view.camera, viewport_size, final_screen)?;
                    apply_intent(
                        controller,
                        state,
                        AppIntent::SelectNodesInRectRequested { min, max, additive },
                    )?;
                    Ok(true)
                }
                HostViewportDragKind::LassoSelection {
                    additive,
                    mut points_screen,
                } => {
                    push_lasso_point(&mut points_screen, final_screen);
                    if points_screen.len() < 3 {
                        return Ok(false);
                    }

                    let polygon = points_screen
                        .into_iter()
                        .map(|point| screen_pos_to_world(&state.view.camera, viewport_size, point))
                        .collect::<Result<Vec<_>>>()?;
                    apply_intent(
                        controller,
                        state,
                        AppIntent::SelectNodesInLassoRequested { polygon, additive },
                    )?;
                    Ok(true)
                }
            }
        }
        HostViewportInputEvent::Scroll {
            screen_pos,
            smooth_delta_y,
            raw_delta_y,
            modifiers,
        } => {
            if modifiers.alt {
                return Ok(false);
            }

            let scroll_delta = if smooth_delta_y != 0.0 {
                smooth_delta_y
            } else {
                raw_delta_y
            };
            if scroll_delta == 0.0 {
                return Ok(false);
            }

            let viewport_size = input_state.effective_viewport_size(state);
            let factor = if scroll_delta > 0.0 {
                state.options.camera_scroll_zoom_step
            } else {
                1.0 / state.options.camera_scroll_zoom_step
            };
            let focus_world = screen_pos
                .map(|screen_pos| {
                    screen_pos_to_world(&state.view.camera, viewport_size, screen_pos)
                })
                .transpose()?;
            apply_intent(
                controller,
                state,
                AppIntent::CameraZoom {
                    factor,
                    focus_world,
                },
            )?;
            Ok(true)
        }
    }
}

/// Wendet einen Viewport-Input-Batch ueber die bridge-owned Gesture-Seam an.
pub fn apply_viewport_input_batch(
    controller: &mut AppController,
    state: &mut AppState,
    input_state: &mut HostViewportInputState,
    batch: HostViewportInputBatch,
) -> Result<bool> {
    let mut handled = false;

    for event in batch.events {
        handled |= apply_viewport_input_event(controller, state, input_state, event)?;
    }

    Ok(handled)
}

fn apply_stateless_host_action(
    controller: &mut AppController,
    state: &mut AppState,
    action: HostSessionAction,
) -> Result<bool> {
    let Some(intent) = map_host_action_to_intent(action) else {
        return Ok(false);
    };

    controller.handle_intent(state, intent)?;
    Ok(true)
}

fn map_render_node_kind(kind: RenderNodeKind) -> HostViewportNodeKind {
    match kind {
        RenderNodeKind::Regular => HostViewportNodeKind::Regular,
        RenderNodeKind::SubPrio => HostViewportNodeKind::SubPrio,
        RenderNodeKind::Warning => HostViewportNodeKind::Warning,
    }
}

fn map_render_connection_direction(
    direction: RenderConnectionDirection,
) -> HostViewportConnectionDirection {
    match direction {
        RenderConnectionDirection::Regular => HostViewportConnectionDirection::Regular,
        RenderConnectionDirection::Dual => HostViewportConnectionDirection::Dual,
        RenderConnectionDirection::Reverse => HostViewportConnectionDirection::Reverse,
    }
}

fn map_render_connection_priority(
    priority: RenderConnectionPriority,
) -> HostViewportConnectionPriority {
    match priority {
        RenderConnectionPriority::Regular => HostViewportConnectionPriority::Regular,
        RenderConnectionPriority::SubPriority => HostViewportConnectionPriority::SubPriority,
    }
}

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

fn map_host_active_tool(tool: HostActiveTool) -> EditorTool {
    match tool {
        HostActiveTool::Select => EditorTool::Select,
        HostActiveTool::Connect => EditorTool::Connect,
        HostActiveTool::AddNode => EditorTool::AddNode,
        HostActiveTool::Route => EditorTool::Route,
    }
}

fn map_editor_tool(tool: EditorTool) -> HostActiveTool {
    match tool {
        EditorTool::Select => HostActiveTool::Select,
        EditorTool::Connect => HostActiveTool::Connect,
        EditorTool::AddNode => HostActiveTool::AddNode,
        EditorTool::Route => HostActiveTool::Route,
    }
}

fn map_connection_direction(direction: ConnectionDirection) -> HostDefaultConnectionDirection {
    match direction {
        ConnectionDirection::Regular => HostDefaultConnectionDirection::Regular,
        ConnectionDirection::Dual => HostDefaultConnectionDirection::Dual,
        ConnectionDirection::Reverse => HostDefaultConnectionDirection::Reverse,
    }
}

fn map_host_connection_direction(direction: HostDefaultConnectionDirection) -> ConnectionDirection {
    match direction {
        HostDefaultConnectionDirection::Regular => ConnectionDirection::Regular,
        HostDefaultConnectionDirection::Dual => ConnectionDirection::Dual,
        HostDefaultConnectionDirection::Reverse => ConnectionDirection::Reverse,
    }
}

fn map_connection_priority(priority: ConnectionPriority) -> HostDefaultConnectionPriority {
    match priority {
        ConnectionPriority::Regular => HostDefaultConnectionPriority::Regular,
        ConnectionPriority::SubPriority => HostDefaultConnectionPriority::SubPriority,
    }
}

fn map_host_connection_priority(priority: HostDefaultConnectionPriority) -> ConnectionPriority {
    match priority {
        HostDefaultConnectionPriority::Regular => ConnectionPriority::Regular,
        HostDefaultConnectionPriority::SubPriority => ConnectionPriority::SubPriority,
    }
}

fn map_route_tool_id(tool_id: RouteToolId) -> HostRouteToolId {
    match tool_id {
        RouteToolId::Straight => HostRouteToolId::Straight,
        RouteToolId::CurveQuad => HostRouteToolId::CurveQuad,
        RouteToolId::CurveCubic => HostRouteToolId::CurveCubic,
        RouteToolId::Spline => HostRouteToolId::Spline,
        RouteToolId::Bypass => HostRouteToolId::Bypass,
        RouteToolId::SmoothCurve => HostRouteToolId::SmoothCurve,
        RouteToolId::Parking => HostRouteToolId::Parking,
        RouteToolId::FieldBoundary => HostRouteToolId::FieldBoundary,
        RouteToolId::FieldPath => HostRouteToolId::FieldPath,
        RouteToolId::RouteOffset => HostRouteToolId::RouteOffset,
        RouteToolId::ColorPath => HostRouteToolId::ColorPath,
    }
}

fn map_host_route_tool_id(tool_id: HostRouteToolId) -> RouteToolId {
    match tool_id {
        HostRouteToolId::Straight => RouteToolId::Straight,
        HostRouteToolId::CurveQuad => RouteToolId::CurveQuad,
        HostRouteToolId::CurveCubic => RouteToolId::CurveCubic,
        HostRouteToolId::Spline => RouteToolId::Spline,
        HostRouteToolId::Bypass => RouteToolId::Bypass,
        HostRouteToolId::SmoothCurve => RouteToolId::SmoothCurve,
        HostRouteToolId::Parking => RouteToolId::Parking,
        HostRouteToolId::FieldBoundary => RouteToolId::FieldBoundary,
        HostRouteToolId::FieldPath => RouteToolId::FieldPath,
        HostRouteToolId::RouteOffset => RouteToolId::RouteOffset,
        HostRouteToolId::ColorPath => RouteToolId::ColorPath,
    }
}

fn map_tangent_source(source: TangentSource) -> HostTangentSource {
    match source {
        TangentSource::None => HostTangentSource::None,
        TangentSource::Connection { neighbor_id, angle } => {
            HostTangentSource::Connection { neighbor_id, angle }
        }
    }
}

fn map_host_tangent_source(source: HostTangentSource) -> TangentSource {
    match source {
        HostTangentSource::None => TangentSource::None,
        HostTangentSource::Connection { neighbor_id, angle } => {
            TangentSource::Connection { neighbor_id, angle }
        }
    }
}

fn map_route_tool_action_to_intent(action: HostRouteToolAction) -> AppIntent {
    match action {
        HostRouteToolAction::SelectTool { tool } => AppIntent::SelectRouteToolRequested {
            tool_id: map_host_route_tool_id(tool),
        },
        HostRouteToolAction::SelectToolWithAnchors {
            tool,
            start_node_id,
            end_node_id,
        } => AppIntent::RouteToolWithAnchorsRequested {
            tool_id: map_host_route_tool_id(tool),
            start_node_id,
            end_node_id,
        },
        HostRouteToolAction::PanelAction { action } => {
            AppIntent::RouteToolPanelActionRequested { action }
        }
        HostRouteToolAction::Execute => AppIntent::RouteToolExecuteRequested,
        HostRouteToolAction::Cancel => AppIntent::RouteToolCancelled,
        HostRouteToolAction::Recreate => AppIntent::RouteToolRecreateRequested,
        HostRouteToolAction::ApplyTangent { start, end } => AppIntent::RouteToolTangentSelected {
            start: map_host_tangent_source(start),
            end: map_host_tangent_source(end),
        },
        HostRouteToolAction::Click { world_pos, ctrl } => AppIntent::RouteToolClicked {
            world_pos: Vec2::new(world_pos[0], world_pos[1]),
            ctrl,
        },
        HostRouteToolAction::LassoCompleted { polygon } => AppIntent::RouteToolLassoCompleted {
            polygon: polygon
                .into_iter()
                .map(|point| Vec2::new(point[0], point[1]))
                .collect(),
        },
        HostRouteToolAction::DragStart { world_pos } => AppIntent::RouteToolDragStarted {
            world_pos: Vec2::new(world_pos[0], world_pos[1]),
        },
        HostRouteToolAction::DragUpdate { world_pos } => AppIntent::RouteToolDragUpdated {
            world_pos: Vec2::new(world_pos[0], world_pos[1]),
        },
        HostRouteToolAction::DragEnd => AppIntent::RouteToolDragEnded,
        HostRouteToolAction::ScrollRotate { delta } => AppIntent::RouteToolScrollRotated { delta },
        HostRouteToolAction::IncreaseNodeCount => AppIntent::IncreaseRouteToolNodeCount,
        HostRouteToolAction::DecreaseNodeCount => AppIntent::DecreaseRouteToolNodeCount,
        HostRouteToolAction::IncreaseSegmentLength => AppIntent::IncreaseRouteToolSegmentLength,
        HostRouteToolAction::DecreaseSegmentLength => AppIntent::DecreaseRouteToolSegmentLength,
    }
}

fn map_route_tool_group(group: RouteToolGroup) -> HostRouteToolGroup {
    match group {
        RouteToolGroup::Basics => HostRouteToolGroup::Basics,
        RouteToolGroup::Section => HostRouteToolGroup::Section,
        RouteToolGroup::Analysis => HostRouteToolGroup::Analysis,
    }
}

fn map_route_tool_surface(surface: RouteToolSurface) -> HostRouteToolSurface {
    match surface {
        RouteToolSurface::FloatingMenu => HostRouteToolSurface::FloatingMenu,
        RouteToolSurface::DefaultsPanel => HostRouteToolSurface::DefaultsPanel,
        RouteToolSurface::MainMenu => HostRouteToolSurface::MainMenu,
        RouteToolSurface::CommandPalette => HostRouteToolSurface::CommandPalette,
    }
}

fn map_route_tool_icon_key(icon_key: RouteToolIconKey) -> HostRouteToolIconKey {
    match icon_key {
        RouteToolIconKey::Straight => HostRouteToolIconKey::Straight,
        RouteToolIconKey::CurveQuad => HostRouteToolIconKey::CurveQuad,
        RouteToolIconKey::CurveCubic => HostRouteToolIconKey::CurveCubic,
        RouteToolIconKey::Spline => HostRouteToolIconKey::Spline,
        RouteToolIconKey::Bypass => HostRouteToolIconKey::Bypass,
        RouteToolIconKey::SmoothCurve => HostRouteToolIconKey::SmoothCurve,
        RouteToolIconKey::Parking => HostRouteToolIconKey::Parking,
        RouteToolIconKey::FieldBoundary => HostRouteToolIconKey::FieldBoundary,
        RouteToolIconKey::FieldPath => HostRouteToolIconKey::FieldPath,
        RouteToolIconKey::RouteOffset => HostRouteToolIconKey::RouteOffset,
        RouteToolIconKey::ColorPath => HostRouteToolIconKey::ColorPath,
    }
}

fn map_route_tool_disabled_reason(reason: RouteToolDisabledReason) -> HostRouteToolDisabledReason {
    match reason {
        RouteToolDisabledReason::MissingFarmland => HostRouteToolDisabledReason::MissingFarmland,
        RouteToolDisabledReason::MissingBackground => {
            HostRouteToolDisabledReason::MissingBackground
        }
        RouteToolDisabledReason::MissingOrderedChain => {
            HostRouteToolDisabledReason::MissingOrderedChain
        }
    }
}

fn route_tool_availability_context(state: &AppState) -> RouteToolAvailabilityContext {
    let has_farmland = state
        .farmland_polygons_arc()
        .is_some_and(|polygons| !polygons.is_empty());
    let has_background = state.has_background_image();
    let has_ordered_chain = state.road_map.as_deref().is_some_and(|road_map| {
        road_map
            .ordered_chain_nodes(&state.selection.selected_node_ids)
            .is_some()
    });

    RouteToolAvailabilityContext {
        has_farmland,
        has_background,
        has_ordered_chain,
    }
}

fn build_route_tool_entries_snapshot(state: &AppState) -> Vec<HostRouteToolEntrySnapshot> {
    let availability = route_tool_availability_context(state);
    let mut entries = Vec::new();

    for surface in [
        RouteToolSurface::MainMenu,
        RouteToolSurface::DefaultsPanel,
        RouteToolSurface::FloatingMenu,
        RouteToolSurface::CommandPalette,
    ] {
        for group in [
            RouteToolGroup::Basics,
            RouteToolGroup::Section,
            RouteToolGroup::Analysis,
        ] {
            for entry in resolve_route_tool_entries(surface, group, availability) {
                entries.push(HostRouteToolEntrySnapshot {
                    surface: map_route_tool_surface(surface),
                    group: map_route_tool_group(group),
                    tool: map_route_tool_id(entry.descriptor.id),
                    slot: entry.slot,
                    icon_key: map_route_tool_icon_key(entry.descriptor.icon_key),
                    enabled: entry.enabled,
                    disabled_reason: entry.disabled_reason.map(map_route_tool_disabled_reason),
                });
            }
        }
    }

    entries
}

fn build_route_tool_selection_snapshot(state: &AppState) -> HostRouteToolSelectionSnapshot {
    HostRouteToolSelectionSnapshot {
        basics: map_route_tool_id(state.editor.route_tool_memory.basics),
        section: map_route_tool_id(state.editor.route_tool_memory.section),
        analysis: map_route_tool_id(state.editor.route_tool_memory.analysis),
    }
}

fn map_tangent_option_data(option: TangentOptionData) -> HostTangentOptionSnapshot {
    HostTangentOptionSnapshot {
        source: map_tangent_source(option.source),
        label: option.label,
    }
}

fn map_tangent_menu_data(menu: TangentMenuData) -> HostTangentMenuSnapshot {
    HostTangentMenuSnapshot {
        start_options: menu
            .start_options
            .into_iter()
            .map(map_tangent_option_data)
            .collect(),
        end_options: menu
            .end_options
            .into_iter()
            .map(map_tangent_option_data)
            .collect(),
        current_start: map_tangent_source(menu.current_start),
        current_end: map_tangent_source(menu.current_end),
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

fn map_engine_dialog_request_kind(kind: DialogRequestKind) -> HostDialogRequestKind {
    match kind {
        DialogRequestKind::OpenFile => HostDialogRequestKind::OpenFile,
        DialogRequestKind::SaveFile => HostDialogRequestKind::SaveFile,
        DialogRequestKind::Heightmap => HostDialogRequestKind::Heightmap,
        DialogRequestKind::BackgroundMap => HostDialogRequestKind::BackgroundMap,
        DialogRequestKind::OverviewZip => HostDialogRequestKind::OverviewZip,
        DialogRequestKind::CurseplayImport => HostDialogRequestKind::CurseplayImport,
        DialogRequestKind::CurseplayExport => HostDialogRequestKind::CurseplayExport,
    }
}

fn map_engine_dialog_request(request: DialogRequest) -> HostDialogRequest {
    HostDialogRequest {
        kind: map_engine_dialog_request_kind(request.kind()),
        suggested_file_name: request.suggested_file_name().map(str::to_owned),
    }
}

fn map_host_dialog_request_kind(kind: HostDialogRequestKind) -> DialogRequestKind {
    match kind {
        HostDialogRequestKind::OpenFile => DialogRequestKind::OpenFile,
        HostDialogRequestKind::SaveFile => DialogRequestKind::SaveFile,
        HostDialogRequestKind::Heightmap => DialogRequestKind::Heightmap,
        HostDialogRequestKind::BackgroundMap => DialogRequestKind::BackgroundMap,
        HostDialogRequestKind::OverviewZip => DialogRequestKind::OverviewZip,
        HostDialogRequestKind::CurseplayImport => DialogRequestKind::CurseplayImport,
        HostDialogRequestKind::CurseplayExport => DialogRequestKind::CurseplayExport,
    }
}

fn map_dialog_result(result: HostDialogResult) -> DialogResult {
    match result {
        HostDialogResult::Cancelled { kind } => DialogResult::Cancelled {
            kind: map_host_dialog_request_kind(kind),
        },
        HostDialogResult::PathSelected { kind, path } => DialogResult::PathSelected {
            kind: map_host_dialog_request_kind(kind),
            path,
        },
    }
}

/// Entnimmt ausstehende Dialog-Anforderungen als Host-Bridge-DTOs.
///
/// Diese Funktion ist fuer Host-Adapter gedacht, die weiterhin einen eigenen
/// `AppController`/`AppState` besitzen, den Dialog-Lifecycle aber bereits ueber
/// die kanonischen `HostDialogRequest`-DTOs konsolidieren wollen.
pub fn take_host_dialog_requests(
    controller: &AppController,
    state: &mut AppState,
) -> Vec<HostDialogRequest> {
    controller
        .take_dialog_requests(state)
        .into_iter()
        .map(map_engine_dialog_request)
        .collect()
}

/// Mappt einen stabilen Engine-Intent auf eine explizite Host-Action.
///
/// Rueckgabewert `None` bedeutet, dass der Intent nicht zur stabilen,
/// niederfrequenten Host-Action-Surface gehoert.
pub fn map_intent_to_host_action(intent: &AppIntent) -> Option<HostSessionAction> {
    match intent {
        AppIntent::OpenFileRequested => Some(HostSessionAction::OpenFile),
        AppIntent::SaveRequested => Some(HostSessionAction::Save),
        AppIntent::SaveAsRequested => Some(HostSessionAction::SaveAs),
        AppIntent::HeightmapSelectionRequested => {
            Some(HostSessionAction::RequestHeightmapSelection)
        }
        AppIntent::BackgroundMapSelectionRequested => {
            Some(HostSessionAction::RequestBackgroundMapSelection)
        }
        AppIntent::GenerateOverviewRequested => Some(HostSessionAction::GenerateOverview),
        AppIntent::CurseplayImportRequested => Some(HostSessionAction::CurseplayImport),
        AppIntent::CurseplayExportRequested => Some(HostSessionAction::CurseplayExport),
        AppIntent::ResetCameraRequested => Some(HostSessionAction::ResetCamera),
        AppIntent::ZoomToFitRequested => Some(HostSessionAction::ZoomToFit),
        AppIntent::ZoomToSelectionBoundsRequested => Some(HostSessionAction::ZoomToSelectionBounds),
        AppIntent::ExitRequested => Some(HostSessionAction::Exit),
        AppIntent::CommandPaletteToggled => Some(HostSessionAction::ToggleCommandPalette),
        AppIntent::SetEditorToolRequested { tool } => Some(HostSessionAction::SetEditorTool {
            tool: map_editor_tool(*tool),
        }),
        AppIntent::SetDefaultDirectionRequested { direction } => {
            Some(HostSessionAction::SetDefaultDirection {
                direction: map_connection_direction(*direction),
            })
        }
        AppIntent::SetDefaultPriorityRequested { priority } => {
            Some(HostSessionAction::SetDefaultPriority {
                priority: map_connection_priority(*priority),
            })
        }
        AppIntent::OptionsChanged { options } => Some(HostSessionAction::ApplyOptions {
            options: options.clone(),
        }),
        AppIntent::ResetOptionsRequested => Some(HostSessionAction::ResetOptions),
        AppIntent::SelectRouteToolRequested { tool_id } => Some(HostSessionAction::RouteTool {
            action: HostRouteToolAction::SelectTool {
                tool: map_route_tool_id(*tool_id),
            },
        }),
        AppIntent::RouteToolWithAnchorsRequested {
            tool_id,
            start_node_id,
            end_node_id,
        } => Some(HostSessionAction::RouteTool {
            action: HostRouteToolAction::SelectToolWithAnchors {
                tool: map_route_tool_id(*tool_id),
                start_node_id: *start_node_id,
                end_node_id: *end_node_id,
            },
        }),
        AppIntent::RouteToolPanelActionRequested { action } => Some(HostSessionAction::RouteTool {
            action: HostRouteToolAction::PanelAction {
                action: action.clone(),
            },
        }),
        AppIntent::RouteToolExecuteRequested => Some(HostSessionAction::RouteTool {
            action: HostRouteToolAction::Execute,
        }),
        AppIntent::RouteToolCancelled => Some(HostSessionAction::RouteTool {
            action: HostRouteToolAction::Cancel,
        }),
        AppIntent::RouteToolConfigChanged | AppIntent::RouteToolRecreateRequested => {
            Some(HostSessionAction::RouteTool {
                action: HostRouteToolAction::Recreate,
            })
        }
        AppIntent::RouteToolTangentSelected { start, end } => Some(HostSessionAction::RouteTool {
            action: HostRouteToolAction::ApplyTangent {
                start: map_tangent_source(*start),
                end: map_tangent_source(*end),
            },
        }),
        AppIntent::RouteToolClicked { world_pos, ctrl } => Some(HostSessionAction::RouteTool {
            action: HostRouteToolAction::Click {
                world_pos: [world_pos.x, world_pos.y],
                ctrl: *ctrl,
            },
        }),
        AppIntent::RouteToolLassoCompleted { polygon } => Some(HostSessionAction::RouteTool {
            action: HostRouteToolAction::LassoCompleted {
                polygon: polygon.iter().map(|point| [point.x, point.y]).collect(),
            },
        }),
        AppIntent::RouteToolDragStarted { world_pos } => Some(HostSessionAction::RouteTool {
            action: HostRouteToolAction::DragStart {
                world_pos: [world_pos.x, world_pos.y],
            },
        }),
        AppIntent::RouteToolDragUpdated { world_pos } => Some(HostSessionAction::RouteTool {
            action: HostRouteToolAction::DragUpdate {
                world_pos: [world_pos.x, world_pos.y],
            },
        }),
        AppIntent::RouteToolDragEnded => Some(HostSessionAction::RouteTool {
            action: HostRouteToolAction::DragEnd,
        }),
        AppIntent::RouteToolScrollRotated { delta } => Some(HostSessionAction::RouteTool {
            action: HostRouteToolAction::ScrollRotate { delta: *delta },
        }),
        AppIntent::IncreaseRouteToolNodeCount => Some(HostSessionAction::RouteTool {
            action: HostRouteToolAction::IncreaseNodeCount,
        }),
        AppIntent::DecreaseRouteToolNodeCount => Some(HostSessionAction::RouteTool {
            action: HostRouteToolAction::DecreaseNodeCount,
        }),
        AppIntent::IncreaseRouteToolSegmentLength => Some(HostSessionAction::RouteTool {
            action: HostRouteToolAction::IncreaseSegmentLength,
        }),
        AppIntent::DecreaseRouteToolSegmentLength => Some(HostSessionAction::RouteTool {
            action: HostRouteToolAction::DecreaseSegmentLength,
        }),
        AppIntent::OpenOptionsDialogRequested => Some(HostSessionAction::OpenOptionsDialog),
        AppIntent::CloseOptionsDialogRequested => Some(HostSessionAction::CloseOptionsDialog),
        AppIntent::UndoRequested => Some(HostSessionAction::Undo),
        AppIntent::RedoRequested => Some(HostSessionAction::Redo),
        _ => None,
    }
}

/// Uebersetzt eine explizite Host-Action in einen stabilen Engine-Intent.
///
/// Gibt `None` zurueck, wenn die Action keinen direkten Intent erzeugt
/// (z. B. ein abgebrochenes Dialog-Ergebnis).
pub fn map_host_action_to_intent(action: HostSessionAction) -> Option<AppIntent> {
    match action {
        HostSessionAction::OpenFile => Some(AppIntent::OpenFileRequested),
        HostSessionAction::Save => Some(AppIntent::SaveRequested),
        HostSessionAction::SaveAs => Some(AppIntent::SaveAsRequested),
        HostSessionAction::RequestHeightmapSelection => {
            Some(AppIntent::HeightmapSelectionRequested)
        }
        HostSessionAction::RequestBackgroundMapSelection => {
            Some(AppIntent::BackgroundMapSelectionRequested)
        }
        HostSessionAction::GenerateOverview => Some(AppIntent::GenerateOverviewRequested),
        HostSessionAction::CurseplayImport => Some(AppIntent::CurseplayImportRequested),
        HostSessionAction::CurseplayExport => Some(AppIntent::CurseplayExportRequested),
        HostSessionAction::ResetCamera => Some(AppIntent::ResetCameraRequested),
        HostSessionAction::ZoomToFit => Some(AppIntent::ZoomToFitRequested),
        HostSessionAction::ZoomToSelectionBounds => Some(AppIntent::ZoomToSelectionBoundsRequested),
        HostSessionAction::Exit => Some(AppIntent::ExitRequested),
        HostSessionAction::ToggleCommandPalette => Some(AppIntent::CommandPaletteToggled),
        HostSessionAction::SetEditorTool { tool } => Some(AppIntent::SetEditorToolRequested {
            tool: map_host_active_tool(tool),
        }),
        HostSessionAction::RouteTool { action } => Some(map_route_tool_action_to_intent(action)),
        HostSessionAction::SetDefaultDirection { direction } => {
            Some(AppIntent::SetDefaultDirectionRequested {
                direction: map_host_connection_direction(direction),
            })
        }
        HostSessionAction::SetDefaultPriority { priority } => {
            Some(AppIntent::SetDefaultPriorityRequested {
                priority: map_host_connection_priority(priority),
            })
        }
        HostSessionAction::ApplyOptions { options } => Some(AppIntent::OptionsChanged { options }),
        HostSessionAction::ResetOptions => Some(AppIntent::ResetOptionsRequested),
        HostSessionAction::OpenOptionsDialog => Some(AppIntent::OpenOptionsDialogRequested),
        HostSessionAction::CloseOptionsDialog => Some(AppIntent::CloseOptionsDialogRequested),
        HostSessionAction::Undo => Some(AppIntent::UndoRequested),
        HostSessionAction::Redo => Some(AppIntent::RedoRequested),
        HostSessionAction::SubmitViewportInput { .. } => None,
        HostSessionAction::SubmitDialogResult { result } => {
            dialog_result_to_intent(map_dialog_result(result))
        }
    }
}

/// Wendet eine Host-Action inklusive stateful Viewport-Input auf Controller und State an.
pub fn apply_host_action_with_viewport_input_state(
    controller: &mut AppController,
    state: &mut AppState,
    input_state: &mut HostViewportInputState,
    action: HostSessionAction,
) -> Result<bool> {
    match action {
        HostSessionAction::SubmitViewportInput { batch } => {
            apply_viewport_input_batch(controller, state, input_state, batch)
        }
        other => apply_stateless_host_action(controller, state, other),
    }
}

/// Wendet die gemeinsame Rust-Host-Dispatch-Seam auf Controller und State an.
///
/// Rueckgabe:
/// - `Ok(true)`: Es wurde ein Intent erzeugt und erfolgreich verarbeitet.
/// - `Ok(false)`: Die Action war semantisch ein No-Op ohne Intent.
pub fn apply_host_action(
    controller: &mut AppController,
    state: &mut AppState,
    action: HostSessionAction,
) -> Result<bool> {
    match action {
        HostSessionAction::SubmitViewportInput { .. } => Err(anyhow!(
            "SubmitViewportInput requires HostViewportInputState; use apply_host_action_with_viewport_input_state(...) or HostBridgeSession"
        )),
        other => apply_stateless_host_action(controller, state, other),
    }
}

/// Wendet einen stabil gemappten Engine-Intent ueber die Host-Bridge-Seam an.
///
/// Rueckgabe:
/// - `Ok(true)`: Der Intent wurde auf eine Host-Action gemappt und verarbeitet.
/// - `Ok(false)`: Der Intent gehoert nicht zur stabilen Host-Action-Surface.
pub fn apply_mapped_intent(
    controller: &mut AppController,
    state: &mut AppState,
    intent: &AppIntent,
) -> Result<bool> {
    let Some(action) = map_intent_to_host_action(intent) else {
        return Ok(false);
    };

    apply_host_action(controller, state, action)
}

/// Baut den host-neutralen Panel-Snapshot fuer Hosts mit lokalem Controller/State.
pub fn build_host_ui_snapshot(controller: &AppController, state: &AppState) -> HostUiSnapshot {
    controller.build_host_ui_snapshot(state)
}

/// Baut den host-neutralen Chrome-Snapshot fuer Menues, Defaults und Status.
pub fn build_host_chrome_snapshot(state: &AppState) -> HostChromeSnapshot {
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
    controller: &AppController,
    state: &mut AppState,
    cursor_world: Option<Vec2>,
) -> ViewportOverlaySnapshot {
    controller.build_viewport_overlay_snapshot(state, cursor_world)
}

/// Baut den per-frame Render-Vertrag fuer lokale Host-Adapter.
pub fn build_render_scene(
    controller: &AppController,
    state: &AppState,
    viewport_size: [f32; 2],
) -> RenderScene {
    controller.build_render_scene(state, viewport_size)
}

/// Baut den langlebigen Render-Asset-Snapshot fuer lokale Host-Adapter.
pub fn build_render_assets(controller: &AppController, state: &AppState) -> RenderAssetsSnapshot {
    controller.build_render_assets(state)
}

/// Baut Szene und Assets als gekoppelten read-only Render-Frame fuer lokale Hosts.
pub fn build_render_frame(
    controller: &AppController,
    state: &AppState,
    viewport_size: [f32; 2],
) -> HostRenderFrameSnapshot {
    HostRenderFrameSnapshot {
        scene: build_render_scene(controller, state, viewport_size),
        assets: build_render_assets(controller, state),
    }
}

/// Baut einen minimalen, serialisierbaren Viewport-Geometry-Snapshot fuer Hosts.
pub fn build_viewport_geometry_snapshot(
    controller: &AppController,
    state: &AppState,
    viewport_size: [f32; 2],
) -> HostViewportGeometrySnapshot {
    let scene = controller.build_render_scene(state, viewport_size);
    build_viewport_geometry_snapshot_from_scene(&scene)
}

#[cfg(test)]
mod tests {
    use fs25_auto_drive_engine::app::ui_contract::{BypassPanelAction, RouteToolPanelAction};
    use fs25_auto_drive_engine::app::{AppController, AppIntent, AppState};
    use fs25_auto_drive_engine::core::{
        Connection, ConnectionDirection, ConnectionPriority, MapMarker, MapNode, NodeFlag, RoadMap,
    };
    use glam::Vec2;
    use std::sync::Arc;

    use crate::dto::{
        HostActiveTool, HostDefaultConnectionDirection, HostDefaultConnectionPriority,
        HostDialogRequestKind, HostDialogResult, HostRouteToolAction, HostRouteToolDisabledReason,
        HostRouteToolGroup, HostRouteToolIconKey, HostRouteToolId, HostRouteToolSurface,
        HostSessionAction, HostTangentSource, HostViewportConnectionDirection,
        HostViewportConnectionPriority, HostViewportNodeKind,
    };

    use super::{
        apply_host_action, apply_mapped_intent, build_host_chrome_snapshot, build_host_ui_snapshot,
        build_render_assets, build_render_frame, build_render_scene,
        build_route_tool_viewport_snapshot, build_viewport_geometry_snapshot,
        build_viewport_overlay_snapshot, map_host_action_to_intent, map_intent_to_host_action,
        take_host_dialog_requests,
    };

    fn geometry_test_map() -> RoadMap {
        let mut map = RoadMap::new(2);
        map.add_node(MapNode::new(1, Vec2::new(0.0, 0.0), NodeFlag::Regular));
        map.add_node(MapNode::new(2, Vec2::new(20.0, 10.0), NodeFlag::SubPrio));
        map.add_connection(Connection::new(
            1,
            2,
            ConnectionDirection::Dual,
            ConnectionPriority::SubPriority,
            Vec2::new(0.0, 0.0),
            Vec2::new(20.0, 10.0),
        ));
        map.add_map_marker(MapMarker::new(
            1,
            "Hof".to_string(),
            "Farmen".to_string(),
            1,
            false,
        ));
        map.ensure_spatial_index();
        map
    }

    #[test]
    fn take_host_dialog_requests_maps_and_drains_engine_queue() {
        let mut controller = AppController::new();
        let mut state = AppState::new();

        controller
            .handle_intent(&mut state, AppIntent::OpenFileRequested)
            .expect("OpenFileRequested muss Dialog-Anforderung erzeugen");
        controller
            .handle_intent(&mut state, AppIntent::HeightmapSelectionRequested)
            .expect("HeightmapSelectionRequested muss Dialog-Anforderung erzeugen");

        let requests = take_host_dialog_requests(&controller, &mut state);
        assert_eq!(requests.len(), 2);
        assert_eq!(requests[0].kind, HostDialogRequestKind::OpenFile);
        assert_eq!(requests[1].kind, HostDialogRequestKind::Heightmap);

        let drained = take_host_dialog_requests(&controller, &mut state);
        assert!(drained.is_empty());
    }

    #[test]
    fn take_host_dialog_requests_covers_save_background_and_curseplay_export_requests() {
        let mut controller = AppController::new();
        let mut state = AppState::new();

        controller
            .handle_intent(&mut state, AppIntent::SaveAsRequested)
            .expect("SaveAsRequested muss Dialog-Anforderung erzeugen");
        controller
            .handle_intent(&mut state, AppIntent::BackgroundMapSelectionRequested)
            .expect("BackgroundMapSelectionRequested muss Dialog-Anforderung erzeugen");
        controller
            .handle_intent(&mut state, AppIntent::CurseplayExportRequested)
            .expect("CurseplayExportRequested muss Dialog-Anforderung erzeugen");

        let requests = take_host_dialog_requests(&controller, &mut state);
        assert_eq!(requests.len(), 3);
        assert_eq!(requests[0].kind, HostDialogRequestKind::SaveFile);
        assert_eq!(requests[1].kind, HostDialogRequestKind::BackgroundMap);
        assert_eq!(requests[2].kind, HostDialogRequestKind::CurseplayExport);
    }

    #[test]
    fn apply_host_action_dispatches_mapped_action() {
        let mut controller = AppController::new();
        let mut state = AppState::new();

        let handled = apply_host_action(
            &mut controller,
            &mut state,
            HostSessionAction::ToggleCommandPalette,
        )
        .expect("ToggleCommandPalette muss verarbeitet werden");

        assert!(handled);
        assert!(state.ui.show_command_palette);
    }

    #[test]
    fn apply_host_action_returns_false_for_dialog_cancel_without_intent() {
        let mut controller = AppController::new();
        let mut state = AppState::new();

        let handled = apply_host_action(
            &mut controller,
            &mut state,
            HostSessionAction::SubmitDialogResult {
                result: HostDialogResult::Cancelled {
                    kind: HostDialogRequestKind::OpenFile,
                },
            },
        )
        .expect("Abgebrochene Dialoge duerfen keinen Fehler ausloesen");

        assert!(!handled);
        assert!(state.ui.dialog_requests.is_empty());
    }

    #[test]
    fn apply_host_action_dispatches_dialog_path_selected_into_state() {
        let mut controller = AppController::new();
        let mut state = AppState::new();
        let selected_path = "/tmp/test_heightmap.png".to_string();

        let handled = apply_host_action(
            &mut controller,
            &mut state,
            HostSessionAction::SubmitDialogResult {
                result: HostDialogResult::PathSelected {
                    kind: HostDialogRequestKind::Heightmap,
                    path: selected_path.clone(),
                },
            },
        )
        .expect("PathSelected muss einen Intent erzeugen und verarbeitet werden");

        assert!(handled);
        assert_eq!(state.ui.heightmap_path, Some(selected_path));
    }

    #[test]
    fn build_viewport_geometry_snapshot_exposes_minimal_geometry_transport() {
        let controller = AppController::new();
        let mut state = AppState::new();
        state.road_map = Some(Arc::new(geometry_test_map()));
        state.view.camera.position = Vec2::new(3.0, -4.0);
        state.view.camera.zoom = 2.0;
        state.selection.ids_mut().insert(2);

        let snapshot = build_viewport_geometry_snapshot(&controller, &state, [640.0, 320.0]);

        assert!(snapshot.has_map);
        assert_eq!(snapshot.viewport_size, [640.0, 320.0]);
        assert_eq!(snapshot.camera_position, [3.0, -4.0]);
        assert_eq!(snapshot.zoom, 2.0);
        assert_eq!(snapshot.nodes.len(), 2);
        assert_eq!(snapshot.connections.len(), 1);
        assert_eq!(snapshot.markers.len(), 1);
        assert_eq!(snapshot.nodes[0].id, 1);
        assert_eq!(snapshot.nodes[0].kind, HostViewportNodeKind::Regular);
        assert!(!snapshot.nodes[0].selected);
        assert_eq!(snapshot.nodes[1].id, 2);
        assert_eq!(snapshot.nodes[1].kind, HostViewportNodeKind::SubPrio);
        assert!(snapshot.nodes[1].selected);
        assert_eq!(
            snapshot.connections[0].direction,
            HostViewportConnectionDirection::Dual
        );
        assert_eq!(
            snapshot.connections[0].priority,
            HostViewportConnectionPriority::SubPriority
        );
        assert_eq!(snapshot.markers[0].position, [0.0, 0.0]);
        assert!(snapshot.world_per_pixel.is_finite());
        assert!(snapshot.world_per_pixel > 0.0);
    }

    #[test]
    fn build_render_frame_couples_scene_and_assets_for_local_hosts() {
        let controller = AppController::new();
        let mut state = AppState::new();
        state.road_map = Some(Arc::new(geometry_test_map()));

        let frame = build_render_frame(&controller, &state, [640.0, 320.0]);

        assert!(frame.scene.has_map());
        assert_eq!(frame.scene.viewport_size(), [640.0, 320.0]);
        assert_eq!(frame.assets.background_asset_revision(), 0);
        assert_eq!(frame.assets.background_transform_revision(), 0);
        assert!(frame.assets.background().is_none());
    }

    #[test]
    fn map_host_action_to_intent_covers_new_dialog_result_branches() {
        let save_intent = map_host_action_to_intent(HostSessionAction::SubmitDialogResult {
            result: HostDialogResult::PathSelected {
                kind: HostDialogRequestKind::SaveFile,
                path: "/tmp/savegame.xml".to_string(),
            },
        });
        let background_zip_intent =
            map_host_action_to_intent(HostSessionAction::SubmitDialogResult {
                result: HostDialogResult::PathSelected {
                    kind: HostDialogRequestKind::BackgroundMap,
                    path: "/tmp/map_overview.zip".to_string(),
                },
            });
        let curseplay_export_intent =
            map_host_action_to_intent(HostSessionAction::SubmitDialogResult {
                result: HostDialogResult::PathSelected {
                    kind: HostDialogRequestKind::CurseplayExport,
                    path: "/tmp/customField.xml".to_string(),
                },
            });

        assert!(matches!(
            save_intent,
            Some(AppIntent::SaveFilePathSelected { path }) if path == "/tmp/savegame.xml"
        ));
        assert!(matches!(
            background_zip_intent,
            Some(AppIntent::ZipBackgroundBrowseRequested { path }) if path == "/tmp/map_overview.zip"
        ));
        assert!(matches!(
            curseplay_export_intent,
            Some(AppIntent::CurseplayExportPathSelected { path }) if path == "/tmp/customField.xml"
        ));
    }

    #[test]
    fn map_host_action_to_intent_covers_route_tool_and_chrome_writes() {
        let route_intent = map_host_action_to_intent(HostSessionAction::RouteTool {
            action: HostRouteToolAction::ScrollRotate { delta: -1.0 },
        });
        let default_direction_intent =
            map_host_action_to_intent(HostSessionAction::SetDefaultDirection {
                direction: HostDefaultConnectionDirection::Reverse,
            });
        let default_priority_intent =
            map_host_action_to_intent(HostSessionAction::SetDefaultPriority {
                priority: HostDefaultConnectionPriority::SubPriority,
            });

        assert!(matches!(
            route_intent,
            Some(AppIntent::RouteToolScrollRotated { delta }) if (delta + 1.0).abs() < f32::EPSILON
        ));
        assert!(matches!(
            default_direction_intent,
            Some(AppIntent::SetDefaultDirectionRequested {
                direction: ConnectionDirection::Reverse
            })
        ));
        assert!(matches!(
            default_priority_intent,
            Some(AppIntent::SetDefaultPriorityRequested {
                priority: ConnectionPriority::SubPriority
            })
        ));
    }

    #[test]
    fn map_intent_to_host_action_covers_stable_bridge_intents() {
        let cases = vec![
            (AppIntent::OpenFileRequested, HostSessionAction::OpenFile),
            (AppIntent::SaveRequested, HostSessionAction::Save),
            (AppIntent::SaveAsRequested, HostSessionAction::SaveAs),
            (
                AppIntent::HeightmapSelectionRequested,
                HostSessionAction::RequestHeightmapSelection,
            ),
            (
                AppIntent::BackgroundMapSelectionRequested,
                HostSessionAction::RequestBackgroundMapSelection,
            ),
            (
                AppIntent::GenerateOverviewRequested,
                HostSessionAction::GenerateOverview,
            ),
            (
                AppIntent::CurseplayImportRequested,
                HostSessionAction::CurseplayImport,
            ),
            (
                AppIntent::CurseplayExportRequested,
                HostSessionAction::CurseplayExport,
            ),
            (
                AppIntent::ResetCameraRequested,
                HostSessionAction::ResetCamera,
            ),
            (AppIntent::ZoomToFitRequested, HostSessionAction::ZoomToFit),
            (
                AppIntent::ZoomToSelectionBoundsRequested,
                HostSessionAction::ZoomToSelectionBounds,
            ),
            (AppIntent::ExitRequested, HostSessionAction::Exit),
            (
                AppIntent::CommandPaletteToggled,
                HostSessionAction::ToggleCommandPalette,
            ),
            (
                AppIntent::SetEditorToolRequested {
                    tool: fs25_auto_drive_engine::app::EditorTool::Route,
                },
                HostSessionAction::SetEditorTool {
                    tool: HostActiveTool::Route,
                },
            ),
            (
                AppIntent::SetDefaultDirectionRequested {
                    direction: ConnectionDirection::Reverse,
                },
                HostSessionAction::SetDefaultDirection {
                    direction: HostDefaultConnectionDirection::Reverse,
                },
            ),
            (
                AppIntent::SetDefaultPriorityRequested {
                    priority: ConnectionPriority::SubPriority,
                },
                HostSessionAction::SetDefaultPriority {
                    priority: HostDefaultConnectionPriority::SubPriority,
                },
            ),
            (
                AppIntent::OptionsChanged {
                    options: Box::new(fs25_auto_drive_engine::shared::EditorOptions::default()),
                },
                HostSessionAction::ApplyOptions {
                    options: Box::new(fs25_auto_drive_engine::shared::EditorOptions::default()),
                },
            ),
            (
                AppIntent::ResetOptionsRequested,
                HostSessionAction::ResetOptions,
            ),
            (
                AppIntent::SelectRouteToolRequested {
                    tool_id: fs25_auto_drive_engine::app::tool_contract::RouteToolId::CurveCubic,
                },
                HostSessionAction::RouteTool {
                    action: HostRouteToolAction::SelectTool {
                        tool: HostRouteToolId::CurveCubic,
                    },
                },
            ),
            (
                AppIntent::RouteToolPanelActionRequested {
                    action: RouteToolPanelAction::Bypass(BypassPanelAction::SetOffset(2.5)),
                },
                HostSessionAction::RouteTool {
                    action: HostRouteToolAction::PanelAction {
                        action: RouteToolPanelAction::Bypass(BypassPanelAction::SetOffset(2.5)),
                    },
                },
            ),
            (
                AppIntent::RouteToolTangentSelected {
                    start: fs25_auto_drive_engine::app::tool_contract::TangentSource::Connection {
                        neighbor_id: 7,
                        angle: 0.5,
                    },
                    end: fs25_auto_drive_engine::app::tool_contract::TangentSource::None,
                },
                HostSessionAction::RouteTool {
                    action: HostRouteToolAction::ApplyTangent {
                        start: HostTangentSource::Connection {
                            neighbor_id: 7,
                            angle: 0.5,
                        },
                        end: HostTangentSource::None,
                    },
                },
            ),
            (
                AppIntent::RouteToolScrollRotated { delta: 1.0 },
                HostSessionAction::RouteTool {
                    action: HostRouteToolAction::ScrollRotate { delta: 1.0 },
                },
            ),
            (
                AppIntent::OpenOptionsDialogRequested,
                HostSessionAction::OpenOptionsDialog,
            ),
            (
                AppIntent::CloseOptionsDialogRequested,
                HostSessionAction::CloseOptionsDialog,
            ),
            (AppIntent::UndoRequested, HostSessionAction::Undo),
            (AppIntent::RedoRequested, HostSessionAction::Redo),
        ];

        for (intent, expected_action) in cases {
            assert_eq!(map_intent_to_host_action(&intent), Some(expected_action));
        }
    }

    #[test]
    fn map_intent_to_host_action_keeps_high_frequency_intents_unmapped() {
        let cases = vec![
            AppIntent::ViewportResized {
                size: [1920.0, 1080.0],
            },
            AppIntent::CameraPan {
                delta: Vec2::new(3.0, -2.0),
            },
            AppIntent::CameraZoom {
                factor: 1.1,
                focus_world: Some(Vec2::ZERO),
            },
            AppIntent::NodePickRequested {
                world_pos: Vec2::new(5.0, 6.0),
                additive: false,
                extend_path: false,
            },
            AppIntent::AddNodeRequested {
                world_pos: Vec2::new(9.0, 1.0),
            },
        ];

        for intent in cases {
            assert!(map_intent_to_host_action(&intent).is_none());
        }
    }

    #[test]
    fn apply_mapped_intent_dispatches_open_file_request() {
        let mut controller = AppController::new();
        let mut state = AppState::new();

        let handled =
            apply_mapped_intent(&mut controller, &mut state, &AppIntent::OpenFileRequested)
                .expect("OpenFileRequested muss ueber die Bridge-Seam verarbeitet werden");

        assert!(handled);
        assert_eq!(state.ui.dialog_requests.len(), 1);
    }

    #[test]
    fn apply_mapped_intent_returns_false_for_unmapped_intents() {
        let mut controller = AppController::new();
        let mut state = AppState::new();

        let handled = apply_mapped_intent(
            &mut controller,
            &mut state,
            &AppIntent::ViewportResized {
                size: [640.0, 480.0],
            },
        )
        .expect("Unmapped Intent darf keinen Fehler ausloesen");

        assert!(!handled);
    }

    #[test]
    fn read_helpers_delegate_to_controller_read_seams() {
        let controller = AppController::new();
        let mut state = AppState::new();

        let host_ui = build_host_ui_snapshot(&controller, &state);
        let overlay = build_viewport_overlay_snapshot(&controller, &mut state, None);
        let scene = build_render_scene(&controller, &state, [640.0, 480.0]);
        let assets = build_render_assets(&controller, &state);

        assert!(host_ui.command_palette_state().is_some());
        assert!(overlay.route_tool_preview.is_none());
        assert_eq!(scene.viewport_size(), [640.0, 480.0]);
        assert_eq!(assets.background_asset_revision(), 0);
    }

    #[test]
    fn build_route_tool_viewport_snapshot_exposes_straight_tool_flags() {
        let road_map = RoadMap::default();
        let mut state = AppState::new();

        state.editor.active_tool = fs25_auto_drive_engine::app::EditorTool::Route;
        state
            .editor
            .tool_manager
            .set_active_by_id(fs25_auto_drive_engine::app::tool_contract::RouteToolId::Straight);
        state
            .editor
            .tool_manager
            .active_tool_mut()
            .expect("Straight-Tool muss fuer den Snapshot-Test aktiv sein")
            .on_click(Vec2::new(0.0, 0.0), &road_map, false);

        let snapshot = build_route_tool_viewport_snapshot(&state);

        assert!(snapshot.has_pending_input);
        assert!(snapshot.drag_targets.is_empty());
        assert!(snapshot.segment_shortcuts_active);
        assert!(snapshot.tangent_menu_data.is_none());
        assert!(!snapshot.needs_lasso_input);
    }

    #[test]
    fn build_host_chrome_snapshot_exposes_status_defaults_and_route_tool_entries() {
        let mut state = AppState::new();
        state.ui.status_message = Some("bereit".to_string());
        state.ui.show_command_palette = true;
        state.editor.active_tool = fs25_auto_drive_engine::app::EditorTool::Route;
        state
            .editor
            .tool_manager
            .set_active_by_id(fs25_auto_drive_engine::app::tool_contract::RouteToolId::CurveCubic);
        state.editor.default_direction = ConnectionDirection::Dual;
        state.editor.default_priority = ConnectionPriority::SubPriority;

        let chrome = build_host_chrome_snapshot(&state);

        assert_eq!(chrome.status_message.as_deref(), Some("bereit"));
        assert!(chrome.show_command_palette);
        assert_eq!(chrome.active_tool, HostActiveTool::Route);
        assert_eq!(chrome.active_route_tool, Some(HostRouteToolId::CurveCubic));
        assert_eq!(
            chrome.default_direction,
            HostDefaultConnectionDirection::Dual
        );
        assert_eq!(
            chrome.default_priority,
            HostDefaultConnectionPriority::SubPriority
        );

        let defaults_entry = chrome
            .route_tool_entries
            .iter()
            .find(|entry| {
                entry.surface == HostRouteToolSurface::DefaultsPanel
                    && entry.group == HostRouteToolGroup::Basics
                    && entry.tool == HostRouteToolId::CurveCubic
            })
            .expect("Defaults-Panel muss Cubic-Tool-Eintrag enthalten");
        assert!(defaults_entry.enabled);
        assert_eq!(defaults_entry.icon_key, HostRouteToolIconKey::CurveCubic);
        assert!(defaults_entry.disabled_reason.is_none());

        let disabled_analysis_entry = chrome
            .route_tool_entries
            .iter()
            .find(|entry| {
                entry.surface == HostRouteToolSurface::MainMenu
                    && entry.group == HostRouteToolGroup::Analysis
                    && entry.tool == HostRouteToolId::FieldBoundary
            })
            .expect("MainMenu muss Analysis-Eintrag enthalten");
        assert!(!disabled_analysis_entry.enabled);
        assert_eq!(
            disabled_analysis_entry.disabled_reason,
            Some(HostRouteToolDisabledReason::MissingFarmland)
        );
    }
}
