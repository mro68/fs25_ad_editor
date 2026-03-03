//! Konfigurationspanel für das Ausweichstrecken-Tool.
//!
//! Zeigt Versatz- und Abstandseinstellungen sowie eine Kurz-Info zur geladenen Kette.

use super::geometry::compute_bypass_positions;
use super::state::BypassTool;

impl BypassTool {
    /// Rendert das Konfigurationspanel im Properties-Panel.
    ///
    /// Gibt `true` zurück wenn sich eine Einstellung geändert hat.
    pub(super) fn render_config_view(&mut self, ui: &mut egui::Ui) -> bool {
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
            if r.changed() {
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
            if r.changed() {
                changed = true;
            }
        });
        ui.small("S-Kurven: halber Abstand");

        // ── Cache invalidieren und Infos anzeigen ────────────────────────────
        if changed {
            self.cached_positions = None;
        }

        // Positions-Cache befüllen (damit preview() ihn nutzen kann)
        if self.cached_positions.is_none() {
            if let Some((positions, d_blend)) =
                compute_bypass_positions(&self.chain_positions, self.offset, self.base_spacing)
            {
                self.d_blend = d_blend;
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
