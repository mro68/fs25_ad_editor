//! UI-Konfigurationspanel für das Constraint-Route-Tool.
//!
//! Enthält:
//! - Max-Winkel-Slider (Solver-Parameter)
//! - Segment-Länge / Node-Anzahl (3-Modus-Pattern)
//! - Kontrollpunkt-Liste mit Entfernen-Button

use super::super::common::render_segment_config_3modes;
use super::state::ConstraintRouteTool;

impl ConstraintRouteTool {
    /// Rendert das Konfigurationspanel im Properties-Panel.
    ///
    /// Gibt `true` zurück wenn sich eine Einstellung geändert hat.
    pub(super) fn render_config_view(
        &mut self,
        ui: &mut egui::Ui,
        distance_wheel_step_m: f32,
    ) -> bool {
        let mut changed = false;

        // Max-Winkel-Slider
        ui.label("Max. Richtungsänderung:");
        let angle_response = ui.add(
            egui::Slider::new(&mut self.max_angle_deg, 5.0..=135.0)
                .suffix("°")
                .fixed_decimals(0),
        );
        if angle_response.changed() {
            changed = true;
            self.update_preview();
            if !self.lifecycle.last_created_ids.is_empty() {
                self.lifecycle.recreate_needed = true;
            }
        }

        ui.add_space(6.0);

        // Segment-Konfiguration (3-Modus: adjusting / live / default)
        let adjusting = !self.lifecycle.last_created_ids.is_empty()
            && self.last_start_anchor.is_some()
            && (self.last_end_anchor.is_some() || self.lifecycle.last_end_anchor.is_some());

        let length = if adjusting {
            let start = self.last_start_anchor.unwrap().position();
            let end = self
                .last_end_anchor
                .or(self.lifecycle.last_end_anchor)
                .unwrap()
                .position();
            start.distance(end)
        } else {
            self.total_distance()
        };

        let ready = self.start.is_some() && self.end.is_some();
        let (seg_changed, recreate) = render_segment_config_3modes(
            &mut self.seg,
            ui,
            adjusting,
            ready,
            length,
            "Routenlänge",
            distance_wheel_step_m,
        );
        if seg_changed {
            changed = true;
            self.update_preview();
        }
        if recreate {
            self.lifecycle.recreate_needed = true;
        }

        // Kontrollpunkt-Liste (nur anzeigen wenn welche vorhanden)
        if !self.control_nodes.is_empty() {
            ui.add_space(6.0);
            ui.label(format!("Kontrollpunkte ({})", self.control_nodes.len()));

            let mut remove_idx = None;
            for (i, cp) in self.control_nodes.iter().enumerate() {
                ui.horizontal(|ui| {
                    ui.label(format!("  #{}: ({:.1}, {:.1})", i + 1, cp.x, cp.y));
                    if ui.small_button("✕").clicked() {
                        remove_idx = Some(i);
                    }
                });
            }
            if let Some(idx) = remove_idx {
                self.control_nodes.remove(idx);
                self.sync_derived();
                self.update_preview();
                changed = true;
                if !self.lifecycle.last_created_ids.is_empty() {
                    self.lifecycle.recreate_needed = true;
                }
            }
        }

        changed
    }
}
