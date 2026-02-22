//! Pointer-Delta-Verarbeitung: Kamera-Pan und Selektion-Move.

use super::{screen_pos_to_world, InputState, PrimaryDragMode, ViewportContext};
use crate::app::AppIntent;

impl InputState {
    /// Verarbeitet Maus-Bewegungs-Deltas für Kamera-Pan und Selektion-Move.
    pub(crate) fn handle_pointer_delta(
        &mut self,
        ctx: &ViewportContext,
        events: &mut Vec<AppIntent>,
    ) {
        let pointer_delta = ctx.ui.input(|i| i.pointer.delta());
        if pointer_delta == egui::Vec2::ZERO {
            return;
        }

        let wpp = ctx.camera.world_per_pixel(ctx.viewport_size[1]);

        if self.drag_selection.is_some() {
            // Während Drag-Selektion keine Pan/Move-Events senden.
        } else if ctx.response.dragged_by(egui::PointerButton::Primary) {
            match self.primary_drag_mode {
                PrimaryDragMode::SelectionMove if !ctx.selected_node_ids.is_empty() => {
                    events.push(AppIntent::MoveSelectedNodesRequested {
                        delta_world: glam::Vec2::new(pointer_delta.x * wpp, pointer_delta.y * wpp),
                    });
                }
                PrimaryDragMode::RouteToolPointDrag => {
                    if let Some(pointer_pos) = ctx.response.interact_pointer_pos() {
                        let world_pos = screen_pos_to_world(
                            pointer_pos,
                            ctx.response,
                            ctx.viewport_size,
                            ctx.camera,
                        );
                        events.push(AppIntent::RouteToolDragUpdated { world_pos });
                    }
                }
                PrimaryDragMode::CameraPan | PrimaryDragMode::None => {
                    events.push(AppIntent::CameraPan {
                        delta: glam::Vec2::new(-pointer_delta.x * wpp, -pointer_delta.y * wpp),
                    });
                }
                PrimaryDragMode::SelectionMove => {}
            }
        } else if ctx.response.dragged_by(egui::PointerButton::Middle)
            || ctx.response.dragged_by(egui::PointerButton::Secondary)
        {
            events.push(AppIntent::CameraPan {
                delta: glam::Vec2::new(-pointer_delta.x * wpp, -pointer_delta.y * wpp),
            });
        }
    }
}
