//! UI-Konfigurationspanel für das Catmull-Rom-Spline-Tool.
//!
//! Die `render_config_view`-Methode enthält die gesamte egui-Logik für:
//! - Tangenten-ComboBoxen am Start/Ende (nur Nachbearbeitungs-Modus)
//! - Länge · Segment-Abstand · Node-Anzahl (Nachbearbeitungs- und Live-Modus)

use super::super::common::{render_segment_config_3modes, render_tangent_combo};
use super::super::RouteTool;
use super::SplineTool;

impl SplineTool {
    /// Rendert das Konfigurationspanel im Properties-Panel.
    ///
    /// Gibt `true` zurück wenn sich eine Einstellung geändert hat.
    pub(super) fn render_config_view(&mut self, ui: &mut egui::Ui) -> bool {
        let mut changed = false;

        // Tangenten-Auswahl nur im Nachbearbeitungs-Modus —
        // Start/Ende stehen erst nach Enter fest
        let adjusting = !self.lifecycle.last_created_ids.is_empty() && self.last_anchors.len() >= 2;

        if adjusting {
            // Tangente Start
            if !self.tangents.start_neighbors.is_empty()
                && render_tangent_combo(
                    ui,
                    "spline_tangent_start",
                    "Tangente Start:",
                    "Standard",
                    &mut self.tangents.tangent_start,
                    &self.tangents.start_neighbors,
                )
            {
                if !self.lifecycle.last_created_ids.is_empty() {
                    self.lifecycle.recreate_needed = true;
                }
                changed = true;
            }

            // Tangente Ende
            if !self.tangents.end_neighbors.is_empty()
                && render_tangent_combo(
                    ui,
                    "spline_tangent_end",
                    "Tangente Ende:",
                    "Standard",
                    &mut self.tangents.tangent_end,
                    &self.tangents.end_neighbors,
                )
            {
                if !self.lifecycle.last_created_ids.is_empty() {
                    self.lifecycle.recreate_needed = true;
                }
                changed = true;
            }

            if !self.tangents.start_neighbors.is_empty() || !self.tangents.end_neighbors.is_empty()
            {
                ui.add_space(4.0);
            }

            // Slider für Min. Abstand und Node-Anzahl im Nachbearbeitungs-Modus
            let length = Self::spline_length_from_anchors(
                &self.last_anchors,
                self.tangents.tangent_start,
                self.tangents.tangent_end,
            );

            let (seg_changed, recreate) =
                render_segment_config_3modes(&mut self.seg, ui, true, true, length, "Spline-Länge");
            if recreate {
                self.lifecycle.recreate_needed = true;
            }
            changed |= seg_changed;
        } else if self.is_ready() {
            let length = self.spline_length();
            ui.label(format!("Kontrollpunkte: {}", self.anchors.len()));
            let (seg_changed, _) = render_segment_config_3modes(
                &mut self.seg,
                ui,
                false,
                true,
                length,
                "Spline-Länge",
            );
            changed |= seg_changed;
        } else {
            let (seg_changed, _) =
                render_segment_config_3modes(&mut self.seg, ui, false, false, 0.0, "Spline-Länge");
            changed |= seg_changed;
        }

        changed
    }
}
