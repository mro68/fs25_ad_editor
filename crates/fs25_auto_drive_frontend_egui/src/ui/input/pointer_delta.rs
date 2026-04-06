//! Pointer-Delta-Verarbeitung: Kamera-Pan und Selektion-Move.

use super::{
    host_pointer_button, screen_pos_to_world, to_viewport_screen_pos, InputState, PrimaryDragMode,
    ViewportContext,
};
use crate::app::AppIntent;
use fs25_auto_drive_host_bridge::HostViewportInputEvent;

impl InputState {
    /// Verarbeitet Maus-Bewegungs-Deltas fuer Kamera-Pan und Selektion-Move.
    pub(crate) fn handle_pointer_delta(
        &mut self,
        ctx: &ViewportContext,
        local_intents: &mut Vec<AppIntent>,
        host_events: &mut Vec<HostViewportInputEvent>,
    ) {
        let pointer_delta = ctx.ui.input(|i| i.pointer.delta());
        if pointer_delta == egui::Vec2::ZERO {
            return;
        }

        if self.drag_selection.is_some() {
            if self.primary_drag_via_bridge
                && let Some(pointer_pos) = ctx
                    .response
                    .interact_pointer_pos()
                    .or_else(|| ctx.response.hover_pos())
            {
                host_events.push(HostViewportInputEvent::DragUpdate {
                    button: fs25_auto_drive_host_bridge::HostPointerButton::Primary,
                    screen_pos: to_viewport_screen_pos(pointer_pos, ctx.response),
                    delta_px: [pointer_delta.x, pointer_delta.y],
                });
            }
        } else if ctx.response.dragged_by(egui::PointerButton::Primary) {
            match self.primary_drag_mode {
                PrimaryDragMode::RouteToolPointDrag => {
                    if let Some(pointer_pos) = ctx.response.interact_pointer_pos() {
                        let world_pos = screen_pos_to_world(
                            pointer_pos,
                            ctx.response,
                            ctx.viewport_size,
                            ctx.camera,
                        );
                        local_intents.push(AppIntent::RouteToolDragUpdated { world_pos });
                    }
                }
                PrimaryDragMode::None if self.primary_drag_via_bridge => {
                    if let Some(pointer_pos) = ctx
                        .response
                        .interact_pointer_pos()
                        .or_else(|| ctx.response.hover_pos())
                    {
                        host_events.push(HostViewportInputEvent::DragUpdate {
                            button: fs25_auto_drive_host_bridge::HostPointerButton::Primary,
                            screen_pos: to_viewport_screen_pos(pointer_pos, ctx.response),
                            delta_px: [pointer_delta.x, pointer_delta.y],
                        });
                    }
                }
                PrimaryDragMode::None => {}
            }
        } else if ctx.response.dragged_by(egui::PointerButton::Middle)
            || ctx.response.dragged_by(egui::PointerButton::Secondary)
        {
            let button = if ctx.response.dragged_by(egui::PointerButton::Middle) {
                egui::PointerButton::Middle
            } else {
                egui::PointerButton::Secondary
            };

            if let Some(host_button) = host_pointer_button(button)
                && let Some(pointer_pos) = ctx
                    .response
                    .interact_pointer_pos()
                    .or_else(|| ctx.response.hover_pos())
            {
                host_events.push(HostViewportInputEvent::DragUpdate {
                    button: host_button,
                    screen_pos: to_viewport_screen_pos(pointer_pos, ctx.response),
                    delta_px: [pointer_delta.x, pointer_delta.y],
                });
            }
        }
    }
}
