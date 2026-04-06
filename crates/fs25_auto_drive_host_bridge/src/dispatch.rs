use anyhow::{anyhow, bail, Result};
use fs25_auto_drive_engine::app::ui_contract::{
    dialog_result_to_intent, DialogRequest, DialogRequestKind, DialogResult, HostUiSnapshot,
    ViewportOverlaySnapshot,
};
use fs25_auto_drive_engine::app::{AppController, AppIntent, AppState, EditorTool};
use fs25_auto_drive_engine::shared::{
    RenderAssetsSnapshot, RenderConnectionDirection, RenderConnectionPriority, RenderNodeKind,
    RenderScene,
};
use glam::Vec2;

use crate::dto::{
    HostActiveTool, HostDialogRequest, HostDialogRequestKind, HostDialogResult, HostInputModifiers,
    HostPointerButton, HostSessionAction, HostTapKind, HostViewportConnectionDirection,
    HostViewportConnectionPriority, HostViewportConnectionSnapshot, HostViewportGeometrySnapshot,
    HostViewportInputBatch, HostViewportInputEvent, HostViewportMarkerSnapshot,
    HostViewportNodeKind, HostViewportNodeSnapshot,
};
use crate::session::HostRenderFrameSnapshot;

#[derive(Debug, Clone, Copy, PartialEq)]
enum HostViewportDragKind {
    CameraPan,
    SelectionMove,
    RectSelection {
        start_screen: [f32; 2],
        additive: bool,
    },
}

#[derive(Debug, Clone, Copy, PartialEq)]
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

fn start_primary_drag(
    controller: &mut AppController,
    state: &mut AppState,
    input_state: &mut HostViewportInputState,
    screen_pos: [f32; 2],
    modifiers: HostInputModifiers,
) -> Result<bool> {
    if modifiers.alt {
        input_state.active_drag = None;
        return Ok(false);
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
            tap_kind: HostTapKind::Single,
            screen_pos,
            modifiers,
        } => {
            if button != HostPointerButton::Primary {
                return Ok(false);
            }

            let viewport_size = input_state.effective_viewport_size(state);
            apply_primary_tap(controller, state, viewport_size, screen_pos, modifiers)
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

            match active_drag.kind {
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
    use fs25_auto_drive_engine::app::{AppController, AppIntent, AppState};
    use fs25_auto_drive_engine::core::{
        Connection, ConnectionDirection, ConnectionPriority, MapMarker, MapNode, NodeFlag, RoadMap,
    };
    use glam::Vec2;
    use std::sync::Arc;

    use crate::dto::{
        HostActiveTool, HostDialogRequestKind, HostDialogResult, HostSessionAction,
        HostViewportConnectionDirection, HostViewportConnectionPriority, HostViewportNodeKind,
    };

    use super::{
        apply_host_action, apply_mapped_intent, build_host_ui_snapshot, build_render_assets,
        build_render_frame, build_render_scene, build_viewport_geometry_snapshot,
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
            AppIntent::RouteToolClicked {
                world_pos: Vec2::new(1.0, 2.0),
                ctrl: false,
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
}
