//! Viewport-Input-Zustand und Gesture-Seam fuer Rust-Hosts.

use anyhow::{bail, Result};
use fs25_auto_drive_engine::app::{AppController, AppIntent, AppState, EditorTool};
use glam::Vec2;

use crate::dto::{
    HostInputModifiers, HostPointerButton, HostTapKind, HostViewportInputBatch,
    HostViewportInputEvent,
};

#[derive(Debug, Clone, PartialEq)]
pub(super) enum HostViewportDragKind {
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
pub(super) struct HostViewportDragState {
    pub button: HostPointerButton,
    pub latest_screen: [f32; 2],
    pub kind: HostViewportDragKind,
}

/// Kleiner bridge-owned Input-Zustand fuer Viewport-Drag- und Resize-Lifecycles.
#[derive(Debug, Clone, Default, PartialEq)]
pub struct HostViewportInputState {
    pub(super) viewport_size: [f32; 2],
    pub(super) active_drag: Option<HostViewportDragState>,
}

impl HostViewportInputState {
    pub(super) fn remember_viewport_size(&mut self, size: [f32; 2]) {
        self.viewport_size = size;
    }

    pub(super) fn effective_viewport_size(&self, state: &AppState) -> [f32; 2] {
        if self.viewport_size[0] > 0.0 && self.viewport_size[1] > 0.0 {
            self.viewport_size
        } else {
            state.view.viewport_size
        }
    }
}

pub(super) fn validate_resize_size(size: [f32; 2]) -> Result<()> {
    if !size[0].is_finite() || !size[1].is_finite() {
        bail!("viewport size must be finite");
    }
    if size[0] < 0.0 || size[1] < 0.0 {
        bail!("viewport size must not be negative");
    }
    Ok(())
}

pub(super) fn require_projection_viewport_size(size: [f32; 2]) -> Result<[f32; 2]> {
    if !size[0].is_finite() || !size[1].is_finite() || size[0] <= 0.0 || size[1] <= 0.0 {
        bail!("viewport input requires a positive finite viewport size; send resize first");
    }
    Ok(size)
}

pub(super) fn screen_pos_to_world(
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

pub(super) fn delta_px_to_world(
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
