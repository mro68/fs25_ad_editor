//! Klick-Events: Einfach-/Doppel-Klick, Tool-Routing.

use super::{
    host_modifiers, host_tap_kind, screen_pos_to_world, to_viewport_screen_pos, InputState,
    PrimaryDragMode, ViewportContext,
};
use crate::app::{AppIntent, EditorTool};
use fs25_auto_drive_host_bridge::HostViewportInputEvent;

impl InputState {
    /// Verarbeitet Einfach- und Doppelklick-Events im Viewport.
    pub(crate) fn handle_clicks(
        &mut self,
        ctx: &ViewportContext,
        modifiers: egui::Modifiers,
        local_intents: &mut Vec<AppIntent>,
        host_events: &mut Vec<HostViewportInputEvent>,
    ) {
        if ctx.response.double_clicked_by(egui::PointerButton::Primary) {
            if let Some(pointer_pos) = ctx.response.interact_pointer_pos() {
                host_events.push(HostViewportInputEvent::Tap {
                    button: fs25_auto_drive_host_bridge::HostPointerButton::Primary,
                    tap_kind: host_tap_kind(true),
                    screen_pos: to_viewport_screen_pos(pointer_pos, ctx.response),
                    modifiers: host_modifiers(modifiers),
                });
            }

            self.primary_drag_mode = PrimaryDragMode::None;
            self.primary_drag_via_bridge = false;
        } else if ctx.response.clicked_by(egui::PointerButton::Primary) {
            if let Some(pointer_pos) = ctx.response.interact_pointer_pos() {
                match ctx.active_tool {
                    EditorTool::Connect => {
                        host_events.push(HostViewportInputEvent::Tap {
                            button: fs25_auto_drive_host_bridge::HostPointerButton::Primary,
                            tap_kind: host_tap_kind(false),
                            screen_pos: to_viewport_screen_pos(pointer_pos, ctx.response),
                            modifiers: host_modifiers(modifiers),
                        });
                    }
                    EditorTool::AddNode => {
                        host_events.push(HostViewportInputEvent::Tap {
                            button: fs25_auto_drive_host_bridge::HostPointerButton::Primary,
                            tap_kind: host_tap_kind(false),
                            screen_pos: to_viewport_screen_pos(pointer_pos, ctx.response),
                            modifiers: host_modifiers(modifiers),
                        });
                    }
                    EditorTool::Select => {
                        host_events.push(HostViewportInputEvent::Tap {
                            button: fs25_auto_drive_host_bridge::HostPointerButton::Primary,
                            tap_kind: host_tap_kind(false),
                            screen_pos: to_viewport_screen_pos(pointer_pos, ctx.response),
                            modifiers: host_modifiers(modifiers),
                        });
                    }
                    EditorTool::Route => {
                        let world_pos = screen_pos_to_world(
                            pointer_pos,
                            ctx.response,
                            ctx.viewport_size,
                            ctx.camera,
                        );
                        local_intents.push(AppIntent::RouteToolClicked {
                            world_pos,
                            ctrl: modifiers.command,
                        });
                    }
                }
            }

            self.primary_drag_mode = PrimaryDragMode::None;
            self.primary_drag_via_bridge = false;
        }
    }
}
