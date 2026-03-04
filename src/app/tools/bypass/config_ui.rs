//! Konfigurationspanel für das Ausweichstrecken-Tool.
//!
//! Zeigt Versatz- und Abstandseinstellungen sowie eine Kurz-Info zur geladenen Kette.

use super::geometry::compute_bypass_positions;
use super::state::BypassTool;

impl BypassTool {
    /// Unterdrückt Rauschen/Restwerte, die ohne echtes Scrollen auftreten können.
    const WHEEL_DELTA_THRESHOLD: f32 = 0.5;

    /// Liest die Scroll-Richtung für ein gehovertes Widget.
    fn wheel_dir(ui: &egui::Ui, response: &egui::Response) -> f32 {
        if !response.hovered() {
            return 0.0;
        }
        let delta = ui.input(|i| i.raw_scroll_delta.y);
        if delta.abs() < Self::WHEEL_DELTA_THRESHOLD {
            0.0
        } else {
            delta.signum()
        }
    }

    /// Rendert das Konfigurationspanel im Properties-Panel.
    ///
    /// Gibt `true` zurück wenn sich eine Einstellung geändert hat.
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

        // ── Versatz ──────────────────────────────────────────────────────────
        ui.horizontal(|ui| {
            ui.label("Versatz:");
            let r = ui.add(
                egui::DragValue::new(&mut self.offset)
                    .speed(0.5)
                    .range(-200.0..=200.0)
                    .suffix(" m"),
            );
            let mut local_changed = r.changed();
            let wheel_dir = Self::wheel_dir(ui, &r);
            if distance_wheel_step_m > 0.0 && wheel_dir != 0.0 {
                self.offset =
                    (self.offset + wheel_dir * distance_wheel_step_m).clamp(-200.0, 200.0);
                local_changed = true;
            }
            if local_changed {
                changed = true;
            }
        });
        ui.label(if self.offset >= 0.0 {
            "Richtung: links"
        } else {
            "Richtung: rechts"
        });

        // ── Knotenabstand ────────────────────────────────────────────────────
        ui.horizontal(|ui| {
            ui.label("Abstand:");
            let r = ui.add(
                egui::DragValue::new(&mut self.base_spacing)
                    .speed(0.5)
                    .range(1.0..=50.0)
                    .suffix(" m"),
            );
            let mut local_changed = r.changed();
            let wheel_dir = Self::wheel_dir(ui, &r);
            if distance_wheel_step_m > 0.0 && wheel_dir != 0.0 {
                self.base_spacing =
                    (self.base_spacing + wheel_dir * distance_wheel_step_m).clamp(1.0, 50.0);
                local_changed = true;
            }
            if local_changed {
                changed = true;
            }
        });
        ui.small("S-Kurven: halber Abstand");

        // ── Cache invalidieren und Infos anzeigen ────────────────────────────
        if changed {
            self.cached_positions = None;
            self.cached_connections = None;
        }

        // Positions-Cache befüllen (damit preview() ihn nutzen kann)
        if self.cached_positions.is_none() {
            if let Some((positions, d_blend)) =
                compute_bypass_positions(&self.chain_positions, self.offset, self.base_spacing)
            {
                self.d_blend = d_blend;
                let total_nodes = positions.len() + 2; // chain_start + bypass + chain_end
                self.cached_connections = Some(
                    (0..total_nodes.saturating_sub(1))
                        .map(|i| (i, i + 1))
                        .collect(),
                );
                self.cached_positions = Some(positions);
            }
        }

        // ── Info-Zeile ───────────────────────────────────────────────────────
        ui.add_space(4.0);
        ui.separator();
        if let Some(cached) = &self.cached_positions {
            ui.label(format!("Neue Nodes: {}", cached.len()));
        }
        ui.label(format!("Kette: {} Nodes", self.chain_positions.len()));
        if self.d_blend > 0.0 {
            ui.label(format!("Übergangslänge: {:.1} m", self.d_blend));
        }

        changed
    }
}
