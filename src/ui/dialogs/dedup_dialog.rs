use crate::app::{AppIntent, UiState};

/// Zeigt den Duplikat-Bestätigungsdialog.
///
/// Wird nach dem Laden einer XML-Datei angezeigt, wenn duplizierte Nodes
/// erkannt wurden. Der Benutzer kann wählen, ob die Bereinigung durchgeführt
/// werden soll.
pub fn show_dedup_dialog(ctx: &egui::Context, ui_state: &UiState) -> Vec<AppIntent> {
    let mut events = Vec::new();

    if !ui_state.dedup_dialog.visible {
        return events;
    }

    egui::Window::new("Duplizierte Wegpunkte erkannt")
        .collapsible(false)
        .resizable(false)
        .anchor(egui::Align2::CENTER_CENTER, [0.0, 0.0])
        .show(ctx, |ui| {
            ui.set_min_width(380.0);
            ui.vertical_centered(|ui| {
                ui.add_space(8.0);
                ui.label(
                    egui::RichText::new("⚠ AutoDrive hat Teile des Netzwerks mehrfach erstellt.")
                        .color(egui::Color32::YELLOW),
                );
                ui.add_space(6.0);
                ui.label(format!(
                    "Gefunden: {} duplizierte Nodes in {} Positions-Gruppen",
                    ui_state.dedup_dialog.duplicate_count, ui_state.dedup_dialog.group_count
                ));
                ui.add_space(4.0);
                ui.label("Die Bereinigung entfernt Duplikate und leitet deren Verbindungen um.");
                ui.label("Das Original-Netzwerk bleibt vollständig erhalten.");
                ui.add_space(12.0);

                ui.horizontal(|ui| {
                    if ui.button("Bereinigen").clicked() {
                        events.push(AppIntent::DeduplicateConfirmed);
                    }
                    if ui.button("Nicht bereinigen").clicked() {
                        events.push(AppIntent::DeduplicateCancelled);
                    }
                });
            });
        });

    events
}
