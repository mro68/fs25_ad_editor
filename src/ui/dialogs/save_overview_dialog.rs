//! Dialog: Hintergrundbild als overview.jpg im Savegame-Verzeichnis speichern.

use crate::app::{AppIntent, UiState};

/// Zeigt den Dialog "Als overview.jpg speichern?" nach ZIP-Extraktion.
pub fn show_save_overview_dialog(ctx: &egui::Context, ui_state: &mut UiState) -> Vec<AppIntent> {
    let mut events = Vec::new();

    if !ui_state.save_overview_dialog.visible {
        return events;
    }

    egui::Window::new("Hintergrundbild speichern?")
        .collapsible(false)
        .resizable(false)
        .anchor(egui::Align2::CENTER_CENTER, [0.0, 0.0])
        .show(ctx, |ui| {
            ui.set_min_width(380.0);
            ui.vertical_centered(|ui| {
                ui.add_space(8.0);
                ui.label("Soll das Hintergrundbild als overview.jpg");
                ui.label("im Savegame-Verzeichnis gespeichert werden?");
                ui.add_space(4.0);
                ui.label(
                    egui::RichText::new(&ui_state.save_overview_dialog.target_path)
                        .weak()
                        .small(),
                );
                ui.add_space(4.0);
                ui.label(
                    egui::RichText::new(
                        "Beim nächsten Laden wird es automatisch als Hintergrund verwendet.",
                    )
                    .weak(),
                );
                ui.add_space(12.0);
                ui.horizontal(|ui| {
                    if ui.button("Ja, speichern").clicked() {
                        events.push(AppIntent::SaveBackgroundAsOverviewConfirmed);
                    }
                    if ui.button("Nein").clicked() {
                        events.push(AppIntent::SaveBackgroundAsOverviewDismissed);
                    }
                });
            });
        });

    events
}
