//! Einstellungsdialog fuer "Alle Felder nachzeichnen".
//!
//! Oeffnet sich wenn der Nutzer im Extras-Menue "Alle Felder nachzeichnen" waehlt.
//! Zeigt Sliders fuer Nodedistanz, Versatz und Begradigung.
//! Keine Vorschau — nach Bestaetigung wird direkt gezeichnet.

use crate::app::AppIntent;
use fs25_auto_drive_host_bridge::HostLocalDialogState;
use crate::ui::common::apply_wheel_step;

/// Rendert den Einstellungsdialog fuer die Batch-Feld-Nachzeichnung.
///
/// Solange `state.trace_all_fields_dialog.visible` gesetzt ist, wird das Fenster
/// zentriert angezeigt. Der Dialog mutiert seinen State direkt (Arbeitskopie in
/// `UiState`), damit die Werte beim naechsten Oeffnen erhalten bleiben.
pub fn show_trace_all_fields_dialog(
    ctx: &egui::Context,
    ui_state: &mut HostLocalDialogState,
) -> Vec<AppIntent> {
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
                            .speed(0.1)
                            .suffix(" m"),
                    );
                    apply_wheel_step(ui, &r, &mut dlg.spacing, 0.1, 1.0..=100.0);
                    ui.end_row();

                    // Versatz
                    ui.label("Versatz (m):")
                        .on_hover_text("Abstand vom Feldrand — positiv = nach innen, negativ = nach aussen");
                    let r = ui.add(
                        egui::DragValue::new(&mut dlg.offset)
                            .range(-50.0..=50.0)
                            .speed(0.1)
                            .suffix(" m"),
                    );
                    apply_wheel_step(ui, &r, &mut dlg.offset, 0.1, -50.0..=50.0);
                    ui.end_row();

                    // Begradigung
                    ui.label("Begradigung (m):")
                        .on_hover_text("Douglas-Peucker-Toleranz — groessere Werte vereinfachen kurze Ausschwingungen heraus (0 = aus)");
                    let r = ui.add(
                        egui::DragValue::new(&mut dlg.tolerance)
                            .range(0.0..=20.0)
                            .speed(0.1)
                            .suffix(" m"),
                    );
                    apply_wheel_step(ui, &r, &mut dlg.tolerance, 0.1, 0.0..=20.0);
                    ui.end_row();

                    // Ecken-Erkennung
                    ui.label("Ecken erkennen:")
                        .on_hover_text("Eckpunkte als feste Anker beim Resampling beibehalten");
                    ui.checkbox(&mut dlg.corner_detection_enabled, "");
                    ui.end_row();

                    // Winkel-Schwelle (nur sichtbar wenn Ecken-Erkennung aktiv)
                    if dlg.corner_detection_enabled {
                        ui.label("Winkel-Schwelle:")
                            .on_hover_text("Kleinerer Winkel = aggressivere Erkennung, mehr Ecken (Standard: 90\u{b0})");
                        let r = ui.add(
                            egui::DragValue::new(&mut dlg.corner_angle_threshold_deg)
                                .range(10.0..=170.0)
                                .speed(0.1)
                                .suffix("\u{b0}"),
                        );
                        apply_wheel_step(ui, &r, &mut dlg.corner_angle_threshold_deg, 0.1, 10.0..=170.0);
                        ui.end_row();

                        // Eckenverrundung
                        ui.label("Ecken verrunden:")
                            .on_hover_text("Erkannte Ecken mit Kreisbogen abrunden");
                        ui.checkbox(&mut dlg.corner_rounding_enabled, "");
                        ui.end_row();

                        // Verrundungsradius (nur sichtbar wenn Eckenverrundung aktiv)
                        if dlg.corner_rounding_enabled {
                            ui.label("Radius (m):")
                                .on_hover_text("Radius des Kreisbogens fuer die Eckenverrundung");
                            let r = ui.add(
                                egui::DragValue::new(&mut dlg.corner_rounding_radius)
                                    .range(1.0..=50.0)
                                    .speed(0.1)
                                    .suffix(" m"),
                            );
                            apply_wheel_step(ui, &r, &mut dlg.corner_rounding_radius, 0.1, 1.0..=50.0);
                            ui.end_row();

                            // Max. Winkelabweichung (nur sichtbar wenn Verrundung aktiv)
                            ui.label("Max. Winkelabw. (°):")
                                .on_hover_text("Maximale Winkelabweichung zwischen benachbarten Bogenpunkten — kleinerer Wert = glatterer Bogen, mehr Punkte");
                            let r = ui.add(
                                egui::DragValue::new(&mut dlg.corner_rounding_max_angle_deg)
                                    .range(1.0..=45.0)
                                    .speed(0.1)
                                    .suffix("°"),
                            );
                            apply_wheel_step(ui, &r, &mut dlg.corner_rounding_max_angle_deg, 0.1, 1.0..=45.0);
                            ui.end_row();
                        }
                    }
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
            corner_angle: if dlg.corner_detection_enabled {
                Some(dlg.corner_angle_threshold_deg)
            } else {
                None
            },
            corner_rounding_radius: if dlg.corner_detection_enabled && dlg.corner_rounding_enabled {
                Some(dlg.corner_rounding_radius)
            } else {
                None
            },
            corner_rounding_max_angle_deg: if dlg.corner_detection_enabled
                && dlg.corner_rounding_enabled
            {
                Some(dlg.corner_rounding_max_angle_deg)
            } else {
                None
            },
        });
    } else if cancelled {
        events.push(AppIntent::TraceAllFieldsCancelled);
    }

    events
}
