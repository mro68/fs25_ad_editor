//! UI-Panel fuer die ParkingTool-Konfiguration.

use super::state::{ParkingPhase, ParkingTool};

impl ParkingTool {
    /// Rendert die Parkplatz-Konfiguration im Properties-Panel.
    /// Gibt `true` zurueck wenn sich ein Wert geaendert hat.
    pub(super) fn render_config_view(&mut self, ui: &mut egui::Ui) -> bool {
        let mut changed = false;

        ui.label("Parkplatz-Konfiguration");
        ui.separator();

        // Anzahl Reihen
        ui.horizontal(|ui| {
            ui.label("Reihen:");
            let mut rows = self.config.num_rows as u32;
            if ui.add(egui::Slider::new(&mut rows, 1..=10)).changed() {
                self.config.num_rows = rows as usize;
                changed = true;
            }
        });

        // Reihenabstand
        ui.horizontal(|ui| {
            ui.label("Abstand:");
            if ui
                .add(
                    egui::Slider::new(&mut self.config.row_spacing, 4.0..=20.0)
                        .suffix(" m")
                        .fixed_decimals(1),
                )
                .changed()
            {
                changed = true;
            }
        });

        // Bucht-Laenge
        ui.horizontal(|ui| {
            ui.label("Laenge:");
            if ui
                .add(
                    egui::Slider::new(&mut self.config.bay_length, 10.0..=50.0)
                        .suffix(" m")
                        .fixed_decimals(1),
                )
                .changed()
            {
                changed = true;
            }
        });

        ui.separator();

        // Einfahrt-Position
        ui.horizontal(|ui| {
            ui.label("Einfahrt:");
            if ui
                .add(
                    egui::Slider::new(&mut self.config.entry_t, 0.0..=1.0)
                        .fixed_decimals(2)
                        .text("Ost ← → West"),
                )
                .changed()
            {
                changed = true;
            }
        });

        // Ausfahrt-Position
        ui.horizontal(|ui| {
            ui.label("Ausfahrt:");
            if ui
                .add(
                    egui::Slider::new(&mut self.config.exit_t, 0.0..=1.0)
                        .fixed_decimals(2)
                        .text("Ost ← → West"),
                )
                .changed()
            {
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
        if self.phase == ParkingPhase::Placed {
            ui.separator();
            ui.label(format!("Rotation: {:.1}°", self.angle.to_degrees()));
            ui.small("Maus bewegen zum Drehen — Klick zum Bestaetigen");
        }

        changed
    }
}
