//! Event-Sammlung fuer Panels, Dialoge und Viewport.

use crate::app::{AppIntent, EditorTool};
use crate::ui;
use eframe::egui;
use fs25_auto_drive_host_bridge::{HostRouteToolViewportSnapshot, HostSessionAction};
use glam::Vec2;

use super::{map_intent_to_collected_event, CollectedEvent, EditorApp};

impl EditorApp {
    /// Sammelt alle UI- und Viewport-Events des aktuellen Frames.
    pub(super) fn collect_ui_events(&mut self, ctx: &egui::Context) -> Vec<CollectedEvent> {
        let mut events = Vec::new();
        let host_ui_snapshot = self.session.build_host_ui_snapshot();
        let host_chrome_snapshot = self.session.build_host_chrome_snapshot();
        let mut top_ui = ui::common::create_top_level_ui(ctx, "editor_app_top_level_panels");

        // Panels und Dialoge
        events.extend(self.collect_panel_events(
            ctx,
            &host_ui_snapshot,
            &host_chrome_snapshot,
            &mut top_ui,
        ));
        events.extend(
            self.collect_dialog_events(ctx, &host_ui_snapshot)
                .into_iter()
                .map(map_intent_to_collected_event),
        );
        let mut show_command_palette = host_ui_snapshot
            .command_palette_state()
            .is_some_and(|state| state.visible);
        events.extend(
            ui::command_palette::render_command_palette(
                ctx,
                &mut show_command_palette,
                &host_chrome_snapshot,
            )
            .into_iter()
            .map(map_intent_to_collected_event),
        );
        if show_command_palette != host_chrome_snapshot.show_command_palette {
            events.push(CollectedEvent::HostAction(
                HostSessionAction::ToggleCommandPalette,
            ));
        }

        // Zentraler Viewport (Rendering + Input + Overlays)
        egui::CentralPanel::default()
            .frame(egui::Frame::NONE)
            .show_inside(&mut top_ui, |ui| {
                let (rect, response) =
                    ui.allocate_exact_size(ui.available_size(), egui::Sense::click_and_drag());
                let viewport_size = [rect.width(), rect.height()];
                let command_palette_open = host_chrome_snapshot.show_command_palette;

                events.extend(self.collect_viewport_events(
                    ui,
                    &response,
                    viewport_size,
                    command_palette_open,
                ));
                self.render_viewport(ui, rect, viewport_size);
                let overlay_intents = self.render_overlays(ui, rect, &response, viewport_size);
                events.extend(
                    overlay_intents
                        .into_iter()
                        .map(map_intent_to_collected_event),
                );
            });

        events
    }

    /// Sammelt Input-Events aus dem Viewport (Maus, Drag, Route-Tool-Kontextmenue).
    fn collect_viewport_events(
        &mut self,
        ui: &egui::Ui,
        response: &egui::Response,
        viewport_size: [f32; 2],
        command_palette_open: bool,
    ) -> Vec<CollectedEvent> {
        let mut events = Vec::new();
        let HostRouteToolViewportSnapshot {
            drag_targets,
            has_pending_input: route_tool_is_drawing,
            segment_shortcuts_active: route_tool_segment_shortcuts_active,
            tangent_menu_data,
            needs_lasso_input,
        } = self.session.build_route_tool_viewport_snapshot();
        let viewport_state = self.session.viewport_input_context_mut();

        // ── Paste-Vorschau hat Prioritaet: normale Klicks unterdruecken ──────
        if viewport_state.paste_preview_active {
            events.push(CollectedEvent::HostAction(
                HostSessionAction::SubmitViewportInput {
                    batch: fs25_auto_drive_host_bridge::HostViewportInputBatch {
                        events: vec![
                            fs25_auto_drive_host_bridge::HostViewportInputEvent::Resize {
                                size_px: viewport_size,
                            },
                        ],
                    },
                },
            ));

            // Mauszeiger-Position → Vorschau aktualisieren
            if let Some(hover_screen) = response.hover_pos() {
                let local = hover_screen - response.rect.min;
                let vp = Vec2::new(viewport_size[0], viewport_size[1]);
                let world_pos = viewport_state
                    .camera
                    .screen_to_world(Vec2::new(local.x, local.y), vp);
                events.push(map_intent_to_collected_event(
                    AppIntent::PastePreviewMoved { world_pos },
                ));
            }

            // Linksklick → Einfuegen bestaetigen
            if response.clicked() {
                events.push(map_intent_to_collected_event(
                    AppIntent::PasteConfirmRequested,
                ));
            }

            // Esc → Vorschau abbrechen
            if ui.input(|i| i.key_pressed(egui::Key::Escape)) {
                events.push(map_intent_to_collected_event(AppIntent::PasteCancelled));
            }

            // Cursor als Fadenkreuz anzeigen
            ui.ctx().set_cursor_icon(egui::CursorIcon::Crosshair);

            return events;
        }
        // ─────────────────────────────────────────────────────────────────────

        let viewport_events = self.input.collect_viewport_events(
            ui,
            response,
            viewport_size,
            viewport_state.camera,
            viewport_state.road_map,
            viewport_state.selected_node_ids,
            viewport_state.active_tool,
            route_tool_is_drawing,
            route_tool_segment_shortcuts_active,
            viewport_state.options,
            command_palette_open,
            viewport_state.default_direction,
            viewport_state.default_priority,
            &drag_targets,
            viewport_state.distanzen,
            tangent_menu_data,
            viewport_state.clipboard_has_nodes,
            viewport_state.farmland_available,
            viewport_state.group_editing_active,
            Some(viewport_state.group_registry),
            needs_lasso_input,
        );
        events.extend(
            viewport_events
                .intents
                .into_iter()
                .map(map_intent_to_collected_event),
        );
        if let Some(host_batch) = viewport_events.host_input_batch {
            events.push(CollectedEvent::HostAction(
                HostSessionAction::SubmitViewportInput { batch: host_batch },
            ));
        }

        // Mauszeiger im Viewport je nach aktivem Werkzeug anpassen
        if response.hovered() {
            match viewport_state.active_tool {
                EditorTool::AddNode => {
                    ui.ctx().set_cursor_icon(egui::CursorIcon::Crosshair);
                }
                EditorTool::Connect => {
                    ui.ctx().set_cursor_icon(egui::CursorIcon::ResizeNwSe);
                }
                _ => {}
            }
        }

        events
    }
}

#[cfg(test)]
mod tests {}
