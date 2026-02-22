//! Klick-Events: Einfach-/Doppel-Klick, Tool-Routing.

use super::{screen_pos_to_world, InputState, PrimaryDragMode, ViewportContext};
use crate::app::{AppIntent, EditorTool};

impl InputState {
    /// Verarbeitet Einfach- und Doppelklick-Events im Viewport.
    pub(crate) fn handle_clicks(
        &mut self,
        ctx: &ViewportContext,
        modifiers: egui::Modifiers,
        events: &mut Vec<AppIntent>,
    ) {
        if ctx.response.double_clicked_by(egui::PointerButton::Primary) {
            if let Some(pointer_pos) = ctx.response.interact_pointer_pos() {
                let world_pos =
                    screen_pos_to_world(pointer_pos, ctx.response, ctx.viewport_size, ctx.camera);

                // Ctrl + Doppelklick = Segment additiv hinzufügen
                events.push(AppIntent::NodeSegmentBetweenIntersectionsRequested {
                    world_pos,
                    additive: modifiers.command,
                });
            }

            self.primary_drag_mode = PrimaryDragMode::None;
        } else if ctx.response.clicked_by(egui::PointerButton::Primary) {
            if let Some(pointer_pos) = ctx.response.interact_pointer_pos() {
                let world_pos =
                    screen_pos_to_world(pointer_pos, ctx.response, ctx.viewport_size, ctx.camera);

                match ctx.active_tool {
                    EditorTool::Connect => {
                        events.push(AppIntent::ConnectToolNodeClicked { world_pos });
                    }
                    EditorTool::AddNode => {
                        events.push(AppIntent::AddNodeRequested { world_pos });
                    }
                    EditorTool::Select => {
                        let extend_path = modifiers.shift;
                        let additive = modifiers.command || extend_path;

                        events.push(AppIntent::NodePickRequested {
                            world_pos,
                            additive,
                            extend_path,
                        });
                    }
                    EditorTool::Route => {
                        events.push(AppIntent::RouteToolClicked {
                            world_pos,
                            ctrl: modifiers.command,
                        });
                    }
                }
            }

            self.primary_drag_mode = PrimaryDragMode::None;
        }
    }
}
