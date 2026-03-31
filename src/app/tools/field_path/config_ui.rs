//! Sidebar-Konfiguration fuer das FieldPathTool.

use super::state::{FieldPathMode, FieldPathPhase, FieldPathTool};
use crate::app::tools::common::wheel_dir;
use crate::app::tools::RouteTool;

impl FieldPathTool {
    /// Rendert die FieldPathTool-Konfiguration im Properties-Panel.
    ///
    /// Gibt `true` zurueck wenn sich Einstellungen geaendert haben (Neuzeichnung noetig).
    pub(super) fn render_config_view(
        &mut self,
        ui: &mut egui::Ui,
        distance_wheel_step_m: f32,
    ) -> bool {
        let mut changed = false;

        ui.label("Feldweg-Erkennung");
        ui.separator();

        // ── Modus-Auswahl ────────────────────────────────────────────────────
        ui.horizontal(|ui| {
            ui.label("Modus:");
            egui::ComboBox::from_id_salt("field_path_mode")
                .selected_text(match self.mode {
                    FieldPathMode::Fields => "Felder",
                    FieldPathMode::Boundaries => "Grenzen",
                })
                .show_ui(ui, |ui| {
                    if ui
                        .selectable_value(&mut self.mode, FieldPathMode::Fields, "Felder")
                        .clicked()
                    {
                        changed = true;
                    }
                    if ui
                        .selectable_value(&mut self.mode, FieldPathMode::Boundaries, "Grenzen")
                        .clicked()
                    {
                        changed = true;
                    }
                });
        });

        ui.separator();

        // ── Seite 1 ──────────────────────────────────────────────────────────
        ui.label("── Seite 1 ──");
        match self.mode {
            FieldPathMode::Fields => {
                if self.side1_field_ids.is_empty() {
                    ui.colored_label(egui::Color32::GRAY, "Keine Felder ausgewaehlt");
                } else {
                    let labels: Vec<String> = self
                        .side1_field_ids
                        .iter()
                        .map(|id| format!("#{id}"))
                        .collect();
                    ui.label(format!("Felder: {}", labels.join(", ")));
                }
            }
            FieldPathMode::Boundaries => {
                ui.label(format!("Segmente: {}", self.side1_segments.len()));
            }
        }

        // ── Seite 2 (nur ab SelectingSide2/Preview sichtbar) ─────────────────
        if matches!(
            self.phase,
            FieldPathPhase::SelectingSide2 | FieldPathPhase::Preview
        ) {
            ui.separator();
            ui.label("── Seite 2 ──");
            match self.mode {
                FieldPathMode::Fields => {
                    if self.side2_field_ids.is_empty() {
                        ui.colored_label(egui::Color32::GRAY, "Keine Felder ausgewaehlt");
                    } else {
                        let labels: Vec<String> = self
                            .side2_field_ids
                            .iter()
                            .map(|id| format!("#{id}"))
                            .collect();
                        ui.label(format!("Felder: {}", labels.join(", ")));
                    }
                }
                FieldPathMode::Boundaries => {
                    ui.label(format!("Segmente: {}", self.side2_segments.len()));
                }
            }
        }

        ui.separator();

        // ── Phasen-Aktionen ───────────────────────────────────────────────────
        match self.phase {
            FieldPathPhase::Idle => {
                if ui.button("Starten \u{2192}").clicked() {
                    self.phase = FieldPathPhase::SelectingSide1;
                    changed = true;
                }
            }
            FieldPathPhase::SelectingSide1 => {
                let can_advance = match self.mode {
                    FieldPathMode::Fields => !self.side1_field_ids.is_empty(),
                    FieldPathMode::Boundaries => !self.side1_segments.is_empty(),
                };
                ui.add_enabled_ui(can_advance, |ui| {
                    if ui.button("Seite 2 \u{2192}").clicked() {
                        self.phase = FieldPathPhase::SelectingSide2;
                        changed = true;
                    }
                });
                if ui.button("Zuruecksetzen").clicked() {
                    self.reset();
                    changed = true;
                }
            }
            FieldPathPhase::SelectingSide2 => {
                let can_compute = match self.mode {
                    FieldPathMode::Fields => !self.side2_field_ids.is_empty(),
                    FieldPathMode::Boundaries => !self.side2_segments.is_empty(),
                };
                ui.add_enabled_ui(can_compute, |ui| {
                    if ui.button("Berechnen").clicked() {
                        self.compute_centerline();
                        changed = true;
                    }
                });
                if ui.button("\u{2190} Zurueck").clicked() {
                    self.phase = FieldPathPhase::SelectingSide1;
                    changed = true;
                }
            }
            FieldPathPhase::Preview => {
                if self.resampled_nodes.is_empty() {
                    ui.colored_label(
                        egui::Color32::from_rgb(220, 120, 0),
                        "Keine Mittellinie gefunden \u{2014} Seiten anpassen",
                    );
                } else {
                    ui.label(format!("{} Nodes generiert", self.resampled_nodes.len()));
                }
                if ui.button("\u{2190} Seite 2 neu waehlen").clicked() {
                    self.phase = FieldPathPhase::SelectingSide2;
                    self.centerline.clear();
                    self.resampled_nodes.clear();
                    changed = true;
                }
            }
        }

        ui.separator();

        // ── Einstellungen ─────────────────────────────────────────────────────
        ui.label("── Einstellungen ──");

        // Knotenabstand
        ui.horizontal(|ui| {
            ui.label("Knotenabstand:");
            let response = ui.add(
                egui::DragValue::new(&mut self.config.node_spacing)
                    .range(1.0..=50.0)
                    .speed(0.5)
                    .suffix(" m"),
            );
            let mut local_changed = response.changed();
            let wd = wheel_dir(ui, &response);
            if distance_wheel_step_m > 0.0 && wd != 0.0 {
                self.config.node_spacing =
                    (self.config.node_spacing + wd * distance_wheel_step_m).clamp(1.0, 50.0);
                local_changed = true;
            }
            if local_changed {
                changed = true;
            }
        });

        // Vereinfachungs-Toleranz (Douglas-Peucker)
        ui.horizontal(|ui| {
            ui.label("Vereinfachung:");
            let response = ui.add(
                egui::DragValue::new(&mut self.config.simplify_tolerance)
                    .range(0.0..=20.0)
                    .speed(0.1)
                    .suffix(" m"),
            );
            let mut local_changed = response.changed();
            let wd = wheel_dir(ui, &response);
            if distance_wheel_step_m > 0.0 && wd != 0.0 {
                self.config.simplify_tolerance =
                    (self.config.simplify_tolerance + wd * distance_wheel_step_m).clamp(0.0, 20.0);
                local_changed = true;
            }
            if local_changed {
                changed = true;
            }
        });

        // An bestehende Nodes anschliessen
        if ui
            .checkbox(
                &mut self.config.connect_to_existing,
                "An bestehende Nodes anschl.",
            )
            .changed()
        {
            changed = true;
        }

        // Abbrechen-Button (immer sichtbar wenn nicht Idle)
        if !matches!(self.phase, FieldPathPhase::Idle) {
            ui.separator();
            if ui.button("Abbrechen").clicked() {
                self.reset();
                changed = true;
            }
        }

        changed
    }
}
