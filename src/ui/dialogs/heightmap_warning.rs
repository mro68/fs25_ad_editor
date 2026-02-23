use crate::app::AppIntent;

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
                ui.label("Es wurde keine Heightmap ausgewählt.");
                ui.label("Alle Y-Koordinaten werden auf 0 gesetzt (flache Map).");
                ui.add_space(10.0);

                ui.horizontal(|ui| {
                    if ui.button("Heightmap auswählen").clicked() {
                        events.push(AppIntent::HeightmapSelectionRequested);
                    }

                    if ui.button("Ohne Heightmap fortfahren").clicked() {
                        events.push(AppIntent::HeightmapWarningConfirmed);
                    }

                    if ui.button("Abbrechen").clicked() {
                        events.push(AppIntent::HeightmapWarningCancelled);
                    }
                });
            });
        });

    events
}