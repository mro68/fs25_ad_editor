//! Übersichtskarten-Options-Dialog: Layer-Auswahl vor der Generierung.

use crate::app::state::OverviewOptionsDialogState;
use crate::app::AppIntent;

/// Zeigt den Übersichtskarten-Options-Dialog und gibt erzeugte Events zurück.
pub fn show_overview_options_dialog(
    ctx: &egui::Context,
    dialog_state: &mut OverviewOptionsDialogState,
) -> Vec<AppIntent> {
    let mut events = Vec::new();

    if !dialog_state.visible {
        return events;
    }

    let mut open = true;
    egui::Window::new("Übersichtskarte – Layer-Optionen")
        .open(&mut open)
        .collapsible(false)
        .resizable(false)
        .default_width(320.0)
        .anchor(egui::Align2::CENTER_CENTER, [0.0, 0.0])
        .show(ctx, |ui| {
            ui.label(format!("ZIP: {}", &dialog_state.zip_path));
            ui.separator();

            ui.label("Sichtbare Layer:");
            ui.add_space(4.0);

            ui.checkbox(
                &mut dialog_state.layers.hillshade,
                "Hillshade (Geländeschattierung)",
            );
            ui.checkbox(&mut dialog_state.layers.farmlands, "Farmland-Grenzen");
            ui.checkbox(&mut dialog_state.layers.farmland_ids, "Farmland-ID-Nummern");
            ui.checkbox(
                &mut dialog_state.layers.pois,
                "POI-Marker (Verkaufsstellen etc.)",
            );
            ui.checkbox(&mut dialog_state.layers.legend, "Legende");

            ui.separator();

            ui.horizontal(|ui| {
                if ui.button("Generieren").clicked() {
                    events.push(AppIntent::OverviewOptionsConfirmed);
                }
                if ui.button("Abbrechen").clicked() {
                    events.push(AppIntent::OverviewOptionsCancelled);
                }
            });
        });

    // Fenster über X geschlossen
    if !open {
        events.push(AppIntent::OverviewOptionsCancelled);
    }

    events
}
