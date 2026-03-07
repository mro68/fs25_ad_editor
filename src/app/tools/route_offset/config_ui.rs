//! Konfigurationspanel fuer das Strecken-Versatz-Tool.
//!
//! Zeigt Checkboxen fuer Links/Rechts-Versatz, Distanz-Felder,
//! "Original beibehalten"-Option sowie eine Ketten-Info-Zeile.

use super::super::common::wheel_dir;
use super::state::RouteOffsetTool;

impl RouteOffsetTool {
    /// Rendert das Konfigurationspanel im Properties-Panel.
    ///
    /// Gibt `true` zurueck wenn sich eine Einstellung geaendert hat.
    pub(super) fn render_config_view(
        &mut self,
        ui: &mut egui::Ui,
        distance_wheel_step_m: f32,
    ) -> bool {
        let mut changed = false;

        if !self.has_chain() {
            ui.label("Kette selektieren und Route-Tool neu aktivieren.");
            return false;
        }

        // ── Links-Versatz ────────────────────────────────────────────────────
        ui.horizontal(|ui| {
            if ui
                .checkbox(&mut self.config.left_enabled, "Links versetzen")
                .changed()
            {
                changed = true;
            }
        });

        if self.config.left_enabled {
            ui.horizontal(|ui| {
                ui.label("  Distanz:");
                let r = ui.add(
                    egui::DragValue::new(&mut self.config.left_distance)
                        .speed(0.5)
                        .range(0.5..=200.0)
                        .suffix(" m"),
                );
                let mut local_changed = r.changed();
                let wheel = wheel_dir(ui, &r);
                if distance_wheel_step_m > 0.0 && wheel != 0.0 {
                    self.config.left_distance = (self.config.left_distance
                        + wheel * distance_wheel_step_m)
                        .clamp(0.5, 200.0);
                    local_changed = true;
                }
                if local_changed {
                    changed = true;
                }
            });
        }

        ui.add_space(4.0);

        // ── Rechts-Versatz ───────────────────────────────────────────────────
        ui.horizontal(|ui| {
            if ui
                .checkbox(&mut self.config.right_enabled, "Rechts versetzen")
                .changed()
            {
                changed = true;
            }
        });

        if self.config.right_enabled {
            ui.horizontal(|ui| {
                ui.label("  Distanz:");
                let r = ui.add(
                    egui::DragValue::new(&mut self.config.right_distance)
                        .speed(0.5)
                        .range(0.5..=200.0)
                        .suffix(" m"),
                );
                let mut local_changed = r.changed();
                let wheel = wheel_dir(ui, &r);
                if distance_wheel_step_m > 0.0 && wheel != 0.0 {
                    self.config.right_distance = (self.config.right_distance
                        + wheel * distance_wheel_step_m)
                        .clamp(0.5, 200.0);
                    local_changed = true;
                }
                if local_changed {
                    changed = true;
                }
            });
        }

        ui.add_space(4.0);
        ui.separator();

        // ── Knotenabstand ────────────────────────────────────────────────────
        ui.horizontal(|ui| {
            ui.label("Abstand:");
            let r = ui.add(
                egui::DragValue::new(&mut self.config.base_spacing)
                    .speed(0.5)
                    .range(1.0..=50.0)
                    .suffix(" m"),
            );
            let mut local_changed = r.changed();
            let wheel = wheel_dir(ui, &r);
            if distance_wheel_step_m > 0.0 && wheel != 0.0 {
                self.config.base_spacing =
                    (self.config.base_spacing + wheel * distance_wheel_step_m).clamp(1.0, 50.0);
                local_changed = true;
            }
            if local_changed {
                changed = true;
            }
        });

        ui.add_space(4.0);
        ui.separator();

        // ── Original-Option ───────────────────────────────────────────────────
        if ui
            .checkbox(&mut self.config.keep_original, "Original beibehalten")
            .changed()
        {
            changed = true;
        }

        // ── Cache-Invalidierung bei Aenderung ─────────────────────────────────
        if changed {
            self.cached_preview = None;
        }

        // ── Info-Zeile ────────────────────────────────────────────────────────
        ui.add_space(4.0);
        ui.separator();
        ui.label(format!("Kette: {} Nodes", self.chain_positions.len()));

        changed
    }
}
