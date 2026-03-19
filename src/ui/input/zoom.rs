//! Scroll-Zoom auf Mausposition.

use super::{screen_pos_to_world, InputState, ViewportContext};
use crate::app::state::EditorTool;
use crate::app::AppIntent;

impl InputState {
    /// Verarbeitet Scroll-Zoom auf die aktuelle Mausposition.
    pub(crate) fn handle_scroll_zoom(&self, ctx: &ViewportContext, events: &mut Vec<AppIntent>) {
        if !ctx.response.hovered() {
            return;
        }

        // Kein Zoom wenn die Maus ueber einem Fenster/Dialog liegt (z.B. Options-Dialog,
        // Tool-Panel). layer_id_at verwendet die Memory-Areas und ist Layer-bestellungsgetreu.
        let pointer_pos = ctx.ui.input(|i| i.pointer.latest_pos());
        if let Some(pos) = pointer_pos {
            let top_layer = ctx.ui.ctx().layer_id_at(pos);
            // Background-Layer = Viewport; alles andere (Window, Tooltip, Popup) → kein Zoom
            if top_layer.is_some_and(|l| l.order != egui::Order::Background) {
                return;
            }
        }

        let scroll = ctx.ui.input(|i| i.smooth_scroll_delta.y);
        if scroll == 0.0 {
            return;
        }

        // Alt+Scroll → Route-Tool-Rotation statt Zoom
        let modifiers = ctx.ui.input(|i| i.modifiers);
        if modifiers.alt && ctx.active_tool == EditorTool::Route {
            events.push(AppIntent::RouteToolScrollRotated { delta: scroll });
            return;
        }

        let step = ctx.options.camera_scroll_zoom_step;
        let factor = if scroll > 0.0 { step } else { 1.0 / step };
        let focus_world = ctx
            .response
            .hover_pos()
            .map(|pos| screen_pos_to_world(pos, ctx.response, ctx.viewport_size, ctx.camera));
        events.push(AppIntent::CameraZoom {
            factor,
            focus_world,
        });
    }
}
