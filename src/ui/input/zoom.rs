//! Scroll-Zoom auf Mausposition.

use super::{screen_pos_to_world, InputState, ViewportContext};
use crate::app::state::EditorTool;
use crate::app::AppIntent;

/// Diskrete Drehschrittweite pro Scroll-Tick fuer die Gruppen-Rotation (Grad).
const GROUP_ROTATION_STEP_DEG: f32 = 5.0;

impl InputState {
    /// Verarbeitet Scroll-Zoom auf die aktuelle Mausposition.
    pub(crate) fn handle_scroll_zoom(
        &mut self,
        ctx: &ViewportContext,
        events: &mut Vec<AppIntent>,
    ) {
        if !ctx.response.hovered() {
            // Viewport verlassen → Rotation zwingend beenden
            self.end_group_rotation_if_active(events);
            return;
        }

        // Kein Zoom wenn die Maus ueber einem Fenster/Dialog liegt (z.B. Options-Dialog,
        // Tool-Panel). layer_id_at verwendet die Memory-Areas und ist Layer-bestellungsgetreu.
        let pointer_pos = ctx.ui.input(|i| i.pointer.latest_pos());
        if let Some(pos) = pointer_pos {
            let top_layer = ctx.ui.ctx().layer_id_at(pos);
            // Background-Layer = Viewport; alles andere (Window, Tooltip, Popup) → kein Zoom
            if top_layer.is_some_and(|l| l.order != egui::Order::Background) {
                self.end_group_rotation_if_active(events);
                return;
            }
        }

        let modifiers = ctx.ui.input(|i| i.modifiers);
        // Alt+Scroll-Rotation: raw_scroll_delta verwenden (kein Smoothing → 1× pro Tick statt ~13×)
        // Normaler Zoom: smooth_scroll_delta bleibt unveraendert
        let raw_scroll = ctx.ui.input(|i| i.raw_scroll_delta.y);
        let scroll = ctx.ui.input(|i| i.smooth_scroll_delta.y);

        // Gruppen-Rotation beenden wenn Alt losgelassen wurde oder Bedingungen nicht mehr gelten.
        // Wichtig: NICHT bei scroll==0, damit kein falsches Begin/End zwischen Scroll-Ticks entsteht.
        if self.rotation_active {
            let conditions_met = modifiers.alt
                && ctx.active_tool == EditorTool::Select
                && !ctx.selected_node_ids.is_empty();
            if !conditions_met {
                self.rotation_active = false;
                events.push(AppIntent::EndRotateSelectedNodesRequested);
                // Kein return: normaler Scroll kann danach noch folgen
            }
        }

        // Alt+Scroll + Select-Tool + aktive Selektion → Gruppen-Rotation
        // raw_scroll_delta statt smooth: verhindert 13× Feuern pro Mausrad-Tick
        if modifiers.alt
            && ctx.active_tool == EditorTool::Select
            && !ctx.selected_node_ids.is_empty()
        {
            if raw_scroll.abs() >= 0.5 {
                ctx.ui.input_mut(|i| {
                    i.raw_scroll_delta.y = 0.0;
                    i.smooth_scroll_delta.y = 0.0;
                });
                if !self.rotation_active {
                    self.rotation_active = true;
                    events.push(AppIntent::BeginRotateSelectedNodesRequested);
                }
                let step_rad = GROUP_ROTATION_STEP_DEG.to_radians();
                events.push(AppIntent::RotateSelectedNodesRequested {
                    delta_angle: raw_scroll.signum() * step_rad,
                });
            }
            return;
        }

        // Alt+Scroll → Route-Tool-Rotation statt Zoom
        // raw_scroll_delta statt smooth: verhindert Mehrfach-Feuern pro Tick
        if modifiers.alt && ctx.active_tool == EditorTool::Route {
            if raw_scroll.abs() >= 0.5 {
                ctx.ui.input_mut(|i| {
                    i.raw_scroll_delta.y = 0.0;
                    i.smooth_scroll_delta.y = 0.0;
                });
                events.push(AppIntent::RouteToolScrollRotated {
                    delta: raw_scroll.signum(),
                });
            }
            return;
        }

        if scroll == 0.0 {
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

    /// Beendet die Gruppen-Rotation falls aktiv und sendet das End-Intent.
    fn end_group_rotation_if_active(&mut self, events: &mut Vec<AppIntent>) {
        if self.rotation_active {
            self.rotation_active = false;
            events.push(AppIntent::EndRotateSelectedNodesRequested);
        }
    }
}
