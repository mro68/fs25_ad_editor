//! Einstellungsdialog fuer "Alle Felder nachzeichnen".
//!
//! Oeffnet sich wenn der Nutzer im Extras-Menue "Alle Felder nachzeichnen" waehlt.
//! Zeigt Sliders fuer Nodedistanz, Versatz und Begradigung.
//! Keine Vorschau — nach Bestaetigung wird direkt gezeichnet.

use crate::app::{AppIntent, UiState};
use crate::ui::common::apply_wheel_step;

/// Rendert den Einstellungsdialog fuer die Batch-Feld-Nachzeichnung.
///
/// Solange `state.trace_all_fields_dialog.visible` gesetzt ist, wird das Fenster
/// zentriert angezeigt. Der Dialog mutiert seinen State direkt (Arbeitskopie in
/// `UiState`), damit die Werte beim naechsten Oeffnen erhalten bleiben.
pub fn show_trace_all_fields_dialog(ctx: &egui::Context, ui_state: &mut UiState) -> Vec<AppIntent> {
    let mut events = Vec::new();

    if !ui_state.trace_all_fields_dialog.visible {
        return events;
    }

    let mut confirmed = false;
    let mut cancelled = false;

    egui::Window::new("\u{1F4CD} Alle Felder nachzeichnen")
        .collapsible(false)
        .resizable(false)
        .anchor(egui::Align2::CENTER_CENTER, [0.0, 0.0])
        .show(ctx, |ui| {
            ui.set_min_width(360.0);
            let dlg = &mut ui_state.trace_all_fields_dialog;

            egui::Grid::new("trace_all_fields_grid")
                .num_columns(2)
                .spacing([16.0, 8.0])
                .show(ui, |ui| {
                    // Nodedistanz
                    ui.label("Nodedistanz (m):")
                        .on_hover_text("Abstand zwischen erzeugten Wegpunkten entlang der Feldgrenze");
                    let r = ui.add(
                        egui::DragValue::new(&mut dlg.spacing)
                            .range(1.0..=100.0)
                            .speed(0.5)
                            .suffix(" m"),
                    );
                    apply_wheel_step(ui, &r, &mut dlg.spacing, 1.0, 1.0..=100.0);
                    ui.end_row();

                    // Versatz
                    ui.label("Versatz (m):")
                        .on_hover_text("Abstand vom Feldrand — positiv = nach innen, negativ = nach aussen");
                    let r = ui.add(
                        egui::DragValue::new(&mut dlg.offset)
                            .range(-50.0..=50.0)
                            .speed(0.5)
                            .suffix(" m"),
                    );
                    apply_wheel_step(ui, &r, &mut dlg.offset, 0.5, -50.0..=50.0);
                    ui.end_row();

                    // Begradigung
                    ui.label("Begradigung (m):")
                        .on_hover_text("Douglas-Peucker-Toleranz — groessere Werte vereinfachen kurze Ausschwingungen heraus (0 = aus)");
                    let r = ui.add(
                        egui::DragValue::new(&mut dlg.tolerance)
                            .range(0.0..=20.0)
                            .speed(0.25)
                            .suffix(" m"),
                    );
                    apply_wheel_step(ui, &r, &mut dlg.tolerance, 0.1, 0.0..=20.0);
                    ui.end_row();
                });

            ui.add_space(12.0);
            ui.separator();
            ui.add_space(6.0);

            ui.horizontal(|ui| {
                if ui.button("Erstellen").clicked() {
                    confirmed = true;
                }
                if ui.button("Abbrechen").clicked() {
                    cancelled = true;
                }
            });
        });

    if confirmed {
        let dlg = &ui_state.trace_all_fields_dialog;
        events.push(AppIntent::TraceAllFieldsConfirmed {
            spacing: dlg.spacing,
            offset: dlg.offset,
            tolerance: dlg.tolerance,
        });
    } else if cancelled {
        events.push(AppIntent::TraceAllFieldsCancelled);
    }

    events
}
