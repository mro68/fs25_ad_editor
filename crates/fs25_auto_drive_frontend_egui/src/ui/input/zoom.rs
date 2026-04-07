//! Scroll-Zoom auf Mausposition.

use super::{host_modifiers, to_viewport_screen_pos, InputState, ViewportContext};
use crate::app::state::EditorTool;
use crate::app::AppIntent;
use fs25_auto_drive_host_bridge::HostViewportInputEvent;

/// Diskrete Drehschrittweite pro Scroll-Tick fuer die Gruppen-Rotation (Grad).
const GROUP_ROTATION_STEP_DEG: f32 = 5.0;

fn raw_scroll_delta_from_events(events: &[egui::Event]) -> f32 {
    events
        .iter()
        .filter_map(|event| match event {
            egui::Event::MouseWheel { delta, .. } => Some(delta.y),
            _ => None,
        })
        .sum()
}

fn raw_scroll_delta_y(ui: &egui::Ui) -> f32 {
    ui.input(|i| raw_scroll_delta_from_events(&i.raw.events))
}

fn consume_scroll(ui: &egui::Ui) {
    ui.input_mut(|i| {
        i.raw
            .events
            .retain(|event| !matches!(event, egui::Event::MouseWheel { .. }));
        i.smooth_scroll_delta.y = 0.0;
    });
}

impl InputState {
    /// Verarbeitet Scroll-Zoom auf die aktuelle Mausposition.
    pub(crate) fn handle_scroll_zoom(
        &mut self,
        ctx: &ViewportContext,
        local_intents: &mut Vec<AppIntent>,
        host_events: &mut Vec<HostViewportInputEvent>,
    ) {
        if !ctx.response.hovered() {
            // Viewport verlassen → Rotation zwingend beenden
            self.end_group_rotation_if_active(local_intents);
            return;
        }

        // Kein Zoom wenn die Maus ueber einem Fenster/Dialog liegt (z.B. Options-Dialog,
        // Tool-Panel). layer_id_at verwendet die Memory-Areas und ist Layer-bestellungsgetreu.
        let pointer_pos = ctx.ui.input(|i| i.pointer.latest_pos());
        if let Some(pos) = pointer_pos {
            let top_layer = ctx.ui.ctx().layer_id_at(pos);
            // Background-Layer = Viewport; alles andere (Window, Tooltip, Popup) → kein Zoom
            if top_layer.is_some_and(|l| l.order != egui::Order::Background) {
                self.end_group_rotation_if_active(local_intents);
                return;
            }
        }

        let modifiers = ctx.ui.input(|i| i.modifiers);
        // Alt+Scroll-Rotation: rohe MouseWheel-Events verwenden (kein Smoothing → 1× pro Tick statt ~13×)
        // Normaler Zoom: smooth_scroll_delta bleibt unveraendert
        let raw_scroll = raw_scroll_delta_y(ctx.ui);
        let scroll = ctx.ui.input(|i| i.smooth_scroll_delta.y);

        // Gruppen-Rotation beenden wenn Alt losgelassen wurde oder Bedingungen nicht mehr gelten.
        // Wichtig: NICHT bei scroll==0, damit kein falsches Begin/End zwischen Scroll-Ticks entsteht.
        if self.rotation_active {
            let conditions_met = modifiers.alt
                && ctx.active_tool == EditorTool::Select
                && !ctx.selected_node_ids.is_empty();
            if !conditions_met {
                self.rotation_active = false;
                local_intents.push(AppIntent::EndRotateSelectedNodesRequested);
                // Kein return: normaler Scroll kann danach noch folgen
            }
        }

        // Alt+Scroll + Select-Tool + aktive Selektion → Gruppen-Rotation
        // Rohe MouseWheel-Events statt smooth: verhindert 13× Feuern pro Mausrad-Tick
        if modifiers.alt
            && ctx.active_tool == EditorTool::Select
            && !ctx.selected_node_ids.is_empty()
        {
            if raw_scroll.abs() >= 0.5 {
                consume_scroll(ctx.ui);
                if !self.rotation_active {
                    self.rotation_active = true;
                    local_intents.push(AppIntent::BeginRotateSelectedNodesRequested);
                }
                let step_rad = GROUP_ROTATION_STEP_DEG.to_radians();
                local_intents.push(AppIntent::RotateSelectedNodesRequested {
                    delta_angle: raw_scroll.signum() * step_rad,
                });
            }
            return;
        }

        // Alt+Scroll → Route-Tool-Rotation statt Zoom
        // Rohe MouseWheel-Events statt smooth: verhindert Mehrfach-Feuern pro Tick
        if modifiers.alt && ctx.active_tool == EditorTool::Route {
            if raw_scroll.abs() >= 0.5 {
                consume_scroll(ctx.ui);
                local_intents.push(AppIntent::RouteToolScrollRotated {
                    delta: raw_scroll.signum(),
                });
            }
            return;
        }

        if scroll == 0.0 && raw_scroll == 0.0 {
            return;
        }

        host_events.push(HostViewportInputEvent::Scroll {
            screen_pos: ctx
                .response
                .hover_pos()
                .map(|pos| to_viewport_screen_pos(pos, ctx.response)),
            smooth_delta_y: scroll,
            raw_delta_y: raw_scroll,
            modifiers: host_modifiers(modifiers),
        });
    }

    /// Beendet die Gruppen-Rotation falls aktiv und sendet das End-Intent.
    fn end_group_rotation_if_active(&mut self, local_intents: &mut Vec<AppIntent>) {
        if self.rotation_active {
            self.rotation_active = false;
            local_intents.push(AppIntent::EndRotateSelectedNodesRequested);
        }
    }
}

#[cfg(test)]
mod tests {
    use super::raw_scroll_delta_from_events;

    #[test]
    fn raw_scroll_delta_from_events_aggregates_mouse_wheel_notches() {
        let events = vec![
            egui::Event::MouseWheel {
                unit: egui::MouseWheelUnit::Line,
                delta: egui::vec2(0.0, 1.0),
                modifiers: egui::Modifiers::ALT,
                phase: egui::TouchPhase::Move,
            },
            egui::Event::PointerMoved(egui::pos2(12.0, 24.0)),
            egui::Event::MouseWheel {
                unit: egui::MouseWheelUnit::Line,
                delta: egui::vec2(0.0, -0.25),
                modifiers: egui::Modifiers::NONE,
                phase: egui::TouchPhase::Move,
            },
        ];

        assert!((raw_scroll_delta_from_events(&events) - 0.75).abs() < f32::EPSILON);
    }

    #[test]
    fn raw_scroll_delta_from_events_ignores_non_wheel_events() {
        let events = vec![
            egui::Event::PointerMoved(egui::pos2(1.0, 2.0)),
            egui::Event::Key {
                key: egui::Key::A,
                physical_key: None,
                pressed: true,
                repeat: false,
                modifiers: egui::Modifiers::ALT,
            },
        ];

        assert_eq!(raw_scroll_delta_from_events(&events), 0.0);
    }
}
