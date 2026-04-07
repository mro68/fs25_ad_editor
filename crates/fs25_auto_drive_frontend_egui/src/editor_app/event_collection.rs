//! Event-Sammlung fuer Panels, Dialoge und Viewport.

use crate::ui;
use eframe::egui;
use fs25_auto_drive_host_bridge::HostSessionAction;

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
                let overlay_intents =
                    self.render_overlays(ui, rect, &response, viewport_size, &host_chrome_snapshot);
                events.extend(
                    overlay_intents
                        .into_iter()
                        .map(map_intent_to_collected_event),
                );
            });

        events
    }
}

#[cfg(test)]
mod tests {}
