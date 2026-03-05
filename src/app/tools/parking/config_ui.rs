//! UI-Panel fuer die ParkingTool-Konfiguration.

use super::super::common::wheel_dir;
use super::state::{ParkingPhase, ParkingTool, RampSide};

impl ParkingTool {
    /// Rendert die Parkplatz-Konfiguration im Properties-Panel.
    /// Gibt `true` zurueck wenn sich ein Wert geaendert hat.
    pub(super) fn render_config_view(
        &mut self,
        ui: &mut egui::Ui,
        distance_wheel_step_m: f32,
    ) -> bool {
        let mut changed = false;

        ui.label("Parkplatz-Konfiguration");
        ui.separator();

        // Anzahl Reihen
        ui.horizontal(|ui| {
            ui.label("Reihen:");
            let mut rows = self.config.num_rows as u32;
            let response = ui.add(egui::Slider::new(&mut rows, 1..=10));
            let mut local_changed = response.changed();
            let wd = wheel_dir(ui, &response);
            if wd != 0.0 {
                rows = (rows as i32 + wd as i32).clamp(1, 10) as u32;
                local_changed = true;
            }
            if local_changed {
                self.config.num_rows = rows as usize;
                changed = true;
            }
        });

        // Reihenabstand
        ui.horizontal(|ui| {
            ui.label("Abstand:");
            let response = ui.add(
                egui::Slider::new(&mut self.config.row_spacing, 4.0..=20.0)
                    .suffix(" m")
                    .fixed_decimals(1),
            );
            let mut local_changed = response.changed();
            let wd = wheel_dir(ui, &response);
            if distance_wheel_step_m > 0.0 && wd != 0.0 {
                self.config.row_spacing =
                    (self.config.row_spacing + wd * distance_wheel_step_m).clamp(4.0, 20.0);
                local_changed = true;
            }
            if local_changed {
                changed = true;
            }
        });

        // Bucht-Laenge
        ui.horizontal(|ui| {
            ui.label("Laenge:");
            let response = ui.add(
                egui::Slider::new(&mut self.config.bay_length, 10.0..=50.0)
                    .suffix(" m")
                    .fixed_decimals(1),
            );
            let mut local_changed = response.changed();
            let wd = wheel_dir(ui, &response);
            if distance_wheel_step_m > 0.0 && wd != 0.0 {
                self.config.bay_length =
                    (self.config.bay_length + wd * distance_wheel_step_m).clamp(10.0, 50.0);
                local_changed = true;
            }
            if local_changed {
                changed = true;
            }
        });

        ui.separator();

        // Einfahrt-Position
        ui.horizontal(|ui| {
            ui.label("Einfahrt:");
            let response = ui.add(
                egui::Slider::new(&mut self.config.entry_t, 0.0..=1.0)
                    .fixed_decimals(2)
                    .text("Ost \u{2190} \u{2192} West"),
            );
            let mut local_changed = response.changed();
            let wd = wheel_dir(ui, &response);
            if wd != 0.0 {
                self.config.entry_t = (self.config.entry_t + wd * 0.01).clamp(0.0, 1.0);
                local_changed = true;
            }
            if local_changed {
                changed = true;
            }
        });

        // Ausfahrt-Position
        ui.horizontal(|ui| {
            ui.label("Ausfahrt:");
            let response = ui.add(
                egui::Slider::new(&mut self.config.exit_t, 0.0..=1.0)
                    .fixed_decimals(2)
                    .text("Ost \u{2190} \u{2192} West"),
            );
            let mut local_changed = response.changed();
            let wd = wheel_dir(ui, &response);
            if wd != 0.0 {
                self.config.exit_t = (self.config.exit_t + wd * 0.01).clamp(0.0, 1.0);
                local_changed = true;
            }
            if local_changed {
                changed = true;
            }
        });

        // Rampenlaenge
        ui.horizontal(|ui| {
            ui.label("Rampenlaenge:");
            let response = ui.add(
                egui::Slider::new(&mut self.config.ramp_length, 2.0..=20.0)
                    .suffix(" m")
                    .fixed_decimals(1),
            );
            let mut local_changed = response.changed();
            let wd = wheel_dir(ui, &response);
            if distance_wheel_step_m > 0.0 && wd != 0.0 {
                self.config.ramp_length =
                    (self.config.ramp_length + wd * distance_wheel_step_m).clamp(2.0, 20.0);
                local_changed = true;
            }
            if local_changed {
                changed = true;
            }
        });

        // Einfahrt-Seite
        ui.horizontal(|ui| {
            ui.label("Einfahrt-Seite:");
            let mut side = self.config.entry_side;
            egui::ComboBox::from_id_salt("parking_entry_side")
                .selected_text(match side {
                    RampSide::Left => "Links (Marker-Sicht)",
                    RampSide::Right => "Rechts (Marker-Sicht)",
                })
                .show_ui(ui, |ui| {
                    ui.selectable_value(&mut side, RampSide::Left, "Links (Marker-Sicht)");
                    ui.selectable_value(&mut side, RampSide::Right, "Rechts (Marker-Sicht)");
                });
            if side != self.config.entry_side {
                self.config.entry_side = side;
                changed = true;
            }
        });

        // Ausfahrt-Seite
        ui.horizontal(|ui| {
            ui.label("Ausfahrt-Seite:");
            let mut side = self.config.exit_side;
            egui::ComboBox::from_id_salt("parking_exit_side")
                .selected_text(match side {
                    RampSide::Left => "Links (Marker-Sicht)",
                    RampSide::Right => "Rechts (Marker-Sicht)",
                })
                .show_ui(ui, |ui| {
                    ui.selectable_value(&mut side, RampSide::Left, "Links (Marker-Sicht)");
                    ui.selectable_value(&mut side, RampSide::Right, "Rechts (Marker-Sicht)");
                });
            if side != self.config.exit_side {
                self.config.exit_side = side;
                changed = true;
            }
        });

        ui.separator();

        // Marker-Gruppe
        ui.horizontal(|ui| {
            ui.label("Gruppe:");
            if ui
                .text_edit_singleline(&mut self.config.marker_group)
                .changed()
            {
                changed = true;
            }
        });

        // Rotation-Anzeige
        if self.origin.is_some() {
            ui.separator();
            ui.label(format!("Rotation: {:.1}°", self.angle.to_degrees()));
            match self.phase {
                ParkingPhase::Idle => {
                    ui.small("Alt+Mausrad zum Drehen");
                }
                ParkingPhase::Configuring => {
                    ui.small("Position fixiert — Viewport-Klick zum Verschieben");
                }
                ParkingPhase::Adjusting => {
                    ui.small("Klicken zum Fixieren — Alt+Mausrad zum Drehen");
                }
            };
        }

        changed
    }
}
