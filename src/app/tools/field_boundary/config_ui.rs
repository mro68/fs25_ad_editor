//! UI-Panel fuer die FieldBoundaryTool-Konfiguration.

use super::super::common::wheel_dir;
use super::state::{FieldBoundaryPhase, FieldBoundaryTool};
use crate::core::{ConnectionDirection, ConnectionPriority};

impl FieldBoundaryTool {
    /// Rendert die FieldBoundaryTool-Konfiguration im Properties-Panel.
    /// Gibt `true` zurueck wenn sich ein Wert geaendert hat.
    pub(super) fn render_config_view(
        &mut self,
        ui: &mut egui::Ui,
        distance_wheel_step_m: f32,
    ) -> bool {
        let mut changed = false;

        ui.label("Feldgrenz-Konfiguration");
        ui.separator();

        // Feldinfo
        if let Some(polygon) = &self.selected_polygon {
            ui.label(format!("Feld #{}", polygon.id));
        } else {
            ui.colored_label(
                egui::Color32::GRAY,
                "Kein Feld ausgewaehlt \u{2014} in ein Feld klicken",
            );
        }

        ui.separator();

        // Node-Abstand
        ui.horizontal(|ui| {
            ui.label("Node-Abstand:");
            let response = ui.add(
                egui::DragValue::new(&mut self.node_spacing)
                    .range(1.0..=50.0)
                    .speed(0.5)
                    .suffix(" m"),
            );
            let mut local_changed = response.changed();
            let wd = wheel_dir(ui, &response);
            if distance_wheel_step_m > 0.0 && wd != 0.0 {
                self.node_spacing =
                    (self.node_spacing + wd * distance_wheel_step_m).clamp(1.0, 50.0);
                local_changed = true;
            }
            if local_changed {
                changed = true;
            }
        });

        // Versatz
        ui.horizontal(|ui| {
            ui.label("Versatz:");
            let response = ui.add(
                egui::DragValue::new(&mut self.offset)
                    .range(-20.0..=20.0)
                    .speed(0.5)
                    .suffix(" m"),
            );
            let mut local_changed = response.changed();
            let wd = wheel_dir(ui, &response);
            if distance_wheel_step_m > 0.0 && wd != 0.0 {
                self.offset = (self.offset + wd * distance_wheel_step_m).clamp(-20.0, 20.0);
                local_changed = true;
            }
            if local_changed {
                changed = true;
            }
        });

        // Begradigen (Douglas-Peucker-Toleranz)
        ui.horizontal(|ui| {
            ui.label("Begradigen:");
            let response = ui.add(
                egui::DragValue::new(&mut self.straighten_tolerance)
                    .range(0.0..=10.0)
                    .speed(0.1)
                    .suffix(" m"),
            );
            let mut local_changed = response.changed();
            let wd = wheel_dir(ui, &response);
            if wd != 0.0 {
                self.straighten_tolerance =
                    (self.straighten_tolerance + wd * 0.1).clamp(0.0, 10.0);
                local_changed = true;
            }
            if local_changed {
                changed = true;
            }
        });

        ui.separator();

        // Verbindungsrichtung
        ui.horizontal(|ui| {
            ui.label("Richtung:");
            let mut dir = self.direction;
            egui::ComboBox::from_id_salt("field_boundary_direction")
                .selected_text(match dir {
                    ConnectionDirection::Regular => "Einbahnstrasse",
                    ConnectionDirection::Dual => "Beidseitig",
                    ConnectionDirection::Reverse => "R\u{fc}ckw\u{e4}rts",
                })
                .show_ui(ui, |ui| {
                    ui.selectable_value(
                        &mut dir,
                        ConnectionDirection::Regular,
                        "Einbahnstrasse",
                    );
                    ui.selectable_value(&mut dir, ConnectionDirection::Dual, "Beidseitig");
                    ui.selectable_value(
                        &mut dir,
                        ConnectionDirection::Reverse,
                        "R\u{fc}ckw\u{e4}rts",
                    );
                });
            if dir != self.direction {
                self.direction = dir;
                changed = true;
            }
        });

        // Strassenart (Prioritaet)
        ui.horizontal(|ui| {
            ui.label("Strassenart:");
            let mut prio = self.priority;
            egui::ComboBox::from_id_salt("field_boundary_priority")
                .selected_text(match prio {
                    ConnectionPriority::Regular => "Normal",
                    ConnectionPriority::SubPriority => "Nebenstrecke",
                })
                .show_ui(ui, |ui| {
                    ui.selectable_value(&mut prio, ConnectionPriority::Regular, "Normal");
                    ui.selectable_value(
                        &mut prio,
                        ConnectionPriority::SubPriority,
                        "Nebenstrecke",
                    );
                });
            if prio != self.priority {
                self.priority = prio;
                changed = true;
            }
        });

        if self.phase == FieldBoundaryPhase::Configuring {
            ui.small("Erneuter Klick im Viewport \u{2192} anderes Feld ausw\u{e4}hlen");
        }

        changed
    }
}
