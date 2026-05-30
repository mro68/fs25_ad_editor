use crate::app::AppIntent;
use crate::ui::dialogs::{dialog_three_action_row, DialogThreeAction};

/// Zeigt die Heightmap-Warnung als modales Fenster.
pub fn show_heightmap_warning(ctx: &egui::Context, show: bool) -> Vec<AppIntent> {
    let mut events = Vec::new();

    if !show {
        return events;
    }

    egui::Window::new("Heightmap nicht vorhanden")
        .collapsible(false)
        .resizable(false)
        .anchor(egui::Align2::CENTER_CENTER, [0.0, 0.0])
        .show(ctx, |ui| {
            ui.vertical_centered(|ui| {
                ui.add_space(10.0);
                ui.label("Es wurde keine Heightmap ausgewaehlt.");
                ui.label("Alle Y-Koordinaten werden auf 0 gesetzt (flache Map).");
                ui.add_space(10.0);

                if let Some(action) = dialog_three_action_row(
                    ui,
                    "Heightmap auswaehlen",
                    "Ohne Heightmap fortfahren",
                    "Abbrechen",
                ) {
                    match action {
                        DialogThreeAction::Primary => {
                            events.push(AppIntent::HeightmapSelectionRequested);
                        }
                        DialogThreeAction::Secondary => {
                            events.push(AppIntent::HeightmapWarningConfirmed);
                        }
                        DialogThreeAction::Tertiary => {
                            events.push(AppIntent::HeightmapWarningCancelled);
                        }
                    }
                }
            });
        });

    events
}
