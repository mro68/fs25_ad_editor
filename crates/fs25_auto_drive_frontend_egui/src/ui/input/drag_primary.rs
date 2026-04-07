//! Drag-Start/-Ende: Selektion-Move, Kamera-Pan, Route-Tool-Drag, Rect/Lasso-Selektion.

use super::super::drag::DragSelectionMode;
use super::{
    host_modifiers, host_pointer_button, screen_pos_to_world, to_viewport_screen_pos,
    DragSelection, InputState, PrimaryDragMode, ViewportContext,
};
use crate::app::{AppIntent, EditorTool};
use fs25_auto_drive_host_bridge::HostViewportInputEvent;

impl InputState {
    /// Erkennt Drag-Beginn und bestimmt den Drag-Modus (Pan, Move, Selektion, Route-Tool).
    pub(crate) fn handle_drag_start(
        &mut self,
        ctx: &ViewportContext,
        modifiers: egui::Modifiers,
        local_intents: &mut Vec<AppIntent>,
        host_events: &mut Vec<HostViewportInputEvent>,
    ) {
        for button in [egui::PointerButton::Middle, egui::PointerButton::Secondary] {
            if ctx.response.drag_started_by(button)
                && let Some(pointer_pos) = ctx.response.interact_pointer_pos()
                && let Some(host_button) = host_pointer_button(button)
            {
                host_events.push(HostViewportInputEvent::DragStart {
                    button: host_button,
                    screen_pos: to_viewport_screen_pos(pointer_pos, ctx.response),
                    modifiers: host_modifiers(modifiers),
                });
            }
        }

        if !ctx.response.drag_started_by(egui::PointerButton::Primary) {
            return;
        }

        if modifiers.shift || modifiers.alt {
            // Shift = Rect-Selektion, Alt = Lasso-Selektion (oder Tool-Lasso)
            // Ctrl zusaetzlich = additiv (zur bestehenden Selektion hinzufuegen)
            if let Some(pointer_pos) = ctx.response.interact_pointer_pos() {
                let mode = if modifiers.alt {
                    // Wenn das aktive Route-Tool Lasso-Input anfordert, ToolLasso verwenden
                    if ctx.tool_needs_lasso {
                        DragSelectionMode::ToolLasso
                    } else {
                        DragSelectionMode::Lasso
                    }
                } else {
                    DragSelectionMode::Rect
                };

                self.drag_selection = Some(DragSelection {
                    mode,
                    start_screen: pointer_pos,
                    points_screen: vec![pointer_pos],
                });
                self.primary_drag_mode = PrimaryDragMode::None;

                if mode == DragSelectionMode::Rect || mode == DragSelectionMode::Lasso {
                    host_events.push(HostViewportInputEvent::DragStart {
                        button: fs25_auto_drive_host_bridge::HostPointerButton::Primary,
                        screen_pos: to_viewport_screen_pos(pointer_pos, ctx.response),
                        modifiers: host_modifiers(modifiers),
                    });
                    self.primary_drag_via_bridge = true;
                } else {
                    self.primary_drag_via_bridge = false;
                }
            }
            return;
        }

        // Kein Shift/Alt: Bridge-Seam entscheidet zwischen Move-Drag und Kamera-Pan.
        let base_max_distance = ctx.options.hitbox_radius();
        let base_max_distance_sq = base_max_distance * base_max_distance;

        // Route-Tool Drag-Target Hit-Test (hat Vorrang vor Node-Move)
        let press_pos = ctx.ui.input(|i| i.pointer.press_origin());
        let route_drag_hit = if ctx.active_tool == EditorTool::Route && !ctx.drag_targets.is_empty()
        {
            press_pos.and_then(|pointer_pos| {
                let world_pos =
                    screen_pos_to_world(pointer_pos, ctx.response, ctx.viewport_size, ctx.camera);
                let hit = ctx.drag_targets.iter().any(|target| {
                    let dx = target[0] - world_pos.x;
                    let dy = target[1] - world_pos.y;
                    (dx * dx) + (dy * dy) <= base_max_distance_sq
                });
                if hit {
                    Some(world_pos)
                } else {
                    None
                }
            })
        } else {
            None
        };

        if let Some(world_pos) = route_drag_hit {
            local_intents.push(AppIntent::RouteToolDragStarted { world_pos });
            self.primary_drag_mode = PrimaryDragMode::RouteToolPointDrag;
            self.primary_drag_via_bridge = false;
        } else if let Some(pointer_pos) = press_pos.or_else(|| ctx.response.interact_pointer_pos())
        {
            host_events.push(HostViewportInputEvent::DragStart {
                button: fs25_auto_drive_host_bridge::HostPointerButton::Primary,
                screen_pos: to_viewport_screen_pos(pointer_pos, ctx.response),
                modifiers: host_modifiers(modifiers),
            });
            self.primary_drag_via_bridge = true;
            self.primary_drag_mode = PrimaryDragMode::None;
        } else {
            self.primary_drag_via_bridge = false;
            self.primary_drag_mode = PrimaryDragMode::None;
        }
    }

    /// Aktualisiert die Drag-Selektion (Rect/Lasso) waehrend des Ziehens.
    pub(crate) fn handle_drag_update(&mut self, ctx: &ViewportContext) {
        let Some(selection) = self.drag_selection.as_mut() else {
            return;
        };
        if !ctx.response.dragged_by(egui::PointerButton::Primary) {
            return;
        }
        if let Some(pointer_pos) = ctx.response.interact_pointer_pos() {
            match selection.mode {
                DragSelectionMode::Rect => {
                    if selection.points_screen.len() == 1 {
                        selection.points_screen.push(pointer_pos);
                    } else {
                        selection.points_screen[1] = pointer_pos;
                    }
                }
                DragSelectionMode::Lasso | DragSelectionMode::ToolLasso => {
                    selection.push_lasso_point(pointer_pos);
                }
            }
        }
    }

    /// Beendet einen Drag und emittiert die resultierenden Intents (Selektion, Move-Ende, etc.).
    pub(crate) fn handle_drag_end(
        &mut self,
        ctx: &ViewportContext,
        local_intents: &mut Vec<AppIntent>,
        host_events: &mut Vec<HostViewportInputEvent>,
    ) {
        if !ctx.response.drag_stopped_by(egui::PointerButton::Primary) {
            for button in [egui::PointerButton::Middle, egui::PointerButton::Secondary] {
                if ctx.response.drag_stopped_by(button)
                    && let Some(host_button) = host_pointer_button(button)
                {
                    host_events.push(HostViewportInputEvent::DragEnd {
                        button: host_button,
                        screen_pos: ctx
                            .response
                            .interact_pointer_pos()
                            .map(|pos| to_viewport_screen_pos(pos, ctx.response)),
                    });
                }
            }
            return;
        }

        if let Some(selection) = self.drag_selection.take() {
            match selection.mode {
                DragSelectionMode::Rect => {
                    // Rechteck-Selektion laeuft ueber den stateful Bridge-Drag-Lifecycle.
                }
                DragSelectionMode::Lasso => {
                    // Normales Node-Lasso laeuft ueber den stateful Bridge-Drag-Lifecycle.
                }
                DragSelectionMode::ToolLasso => {
                    if selection.points_screen.len() >= 3 {
                        let polygon = selection
                            .points_screen
                            .into_iter()
                            .map(|point| {
                                screen_pos_to_world(
                                    point,
                                    ctx.response,
                                    ctx.viewport_size,
                                    ctx.camera,
                                )
                            })
                            .collect::<Vec<_>>();

                        local_intents.push(AppIntent::RouteToolLassoCompleted { polygon });
                    }
                }
            }
        }

        if self.primary_drag_mode == PrimaryDragMode::RouteToolPointDrag {
            local_intents.push(AppIntent::RouteToolDragEnded);
        } else if self.primary_drag_via_bridge {
            host_events.push(HostViewportInputEvent::DragEnd {
                button: fs25_auto_drive_host_bridge::HostPointerButton::Primary,
                screen_pos: ctx
                    .response
                    .interact_pointer_pos()
                    .map(|pos| to_viewport_screen_pos(pos, ctx.response)),
            });
        }

        self.primary_drag_mode = PrimaryDragMode::None;
        self.primary_drag_via_bridge = false;
    }
}
