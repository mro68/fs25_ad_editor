//! Optionen-Dialog für Farben, Größen und Breiten.

mod sections;

use crate::app::AppIntent;
use crate::shared::EditorOptions;

/// Zeigt den Options-Dialog und gibt erzeugte Events zurück.
pub fn show_options_dialog(
    ctx: &egui::Context,
    show: bool,
    options: &EditorOptions,
) -> Vec<AppIntent> {
    let mut events = Vec::new();

    if !show {
        return events;
    }

    // Arbeitskopie der Optionen für Live-Bearbeitung
    let mut opts = options.clone();
    let mut changed = false;

    egui::Window::new("Optionen")
        .collapsible(true)
        .resizable(true)
        .default_width(360.0)
        .anchor(egui::Align2::CENTER_CENTER, [0.0, 0.0])
        .show(ctx, |ui| {
            egui::ScrollArea::vertical()
                .max_height(500.0)
                .show(ui, |ui| {
                    changed |= sections::render_nodes(ui, &mut opts);
                    changed |= sections::render_tools(ui, &mut opts);
                    changed |= sections::render_selection(ui, &mut opts);
                    changed |= sections::render_connections(ui, &mut opts);
                    changed |= sections::render_markers(ui, &mut opts);
                    changed |= sections::render_camera(ui, &mut opts);
                    changed |= sections::render_background(ui, &mut opts);
                    changed |= sections::render_overview_layers(ui, &mut opts);
                    changed |= sections::render_node_behavior(ui, &mut opts);
                });

            ui.separator();

            ui.horizontal(|ui| {
                if ui.button("Standardwerte").clicked() {
                    events.push(AppIntent::ResetOptionsRequested);
                }
                if ui.button("Schließen").clicked() {
                    events.push(AppIntent::CloseOptionsDialogRequested);
                }
            });
        });

    // Änderungen sofort anwenden (Live-Preview)
    if changed {
        events.push(AppIntent::OptionsChanged { options: opts });
    }

    events
}
