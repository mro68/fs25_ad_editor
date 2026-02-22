//! UI-Konfigurationspanel für das Catmull-Rom-Spline-Tool.
//!
//! Die `render_config_view`-Methode enthält die gesamte egui-Logik für:
//! - Tangenten-ComboBoxen am Start/Ende (nur Nachbearbeitungs-Modus)
//! - Länge · Segment-Abstand · Node-Anzahl (Nachbearbeitungs- und Live-Modus)

use super::super::common::{
    node_count_from_length, render_tangent_combo, segment_length_from_count, LastEdited,
};
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

            ui.label(format!("Spline-Länge: {:.1} m", length));
            ui.add_space(4.0);

            ui.label("Min. Abstand:");
            let max_seg = length.max(1.0);
            if ui
                .add(
                    egui::Slider::new(&mut self.seg.max_segment_length, 1.0..=max_seg).suffix(" m"),
                )
                .changed()
            {
                self.seg.last_edited = LastEdited::Distance;
                self.seg.node_count = node_count_from_length(length, self.seg.max_segment_length);
                self.lifecycle.recreate_needed = true;
                changed = true;
            }

            ui.add_space(4.0);

            ui.label("Anzahl Nodes:");
            let max_nodes = (length / 1.0).ceil().max(2.0) as usize;
            if ui
                .add(egui::Slider::new(&mut self.seg.node_count, 2..=max_nodes))
                .changed()
            {
                self.seg.last_edited = LastEdited::NodeCount;
                self.seg.max_segment_length =
                    segment_length_from_count(length, self.seg.node_count);
                self.lifecycle.recreate_needed = true;
                changed = true;
            }
        } else if self.is_ready() {
            let length = self.spline_length();
            ui.label(format!("Spline-Länge: {:.1} m", length));
            ui.label(format!("Kontrollpunkte: {}", self.anchors.len()));
            ui.add_space(4.0);

            ui.label("Min. Abstand:");
            let max_seg = length.max(1.0);
            if ui
                .add(
                    egui::Slider::new(&mut self.seg.max_segment_length, 1.0..=max_seg).suffix(" m"),
                )
                .changed()
            {
                self.seg.last_edited = LastEdited::Distance;
                self.sync_derived();
                changed = true;
            }

            ui.add_space(4.0);

            ui.label("Anzahl Nodes:");
            let max_nodes = (length / 1.0).ceil().max(2.0) as usize;
            if ui
                .add(egui::Slider::new(&mut self.seg.node_count, 2..=max_nodes))
                .changed()
            {
                self.seg.last_edited = LastEdited::NodeCount;
                self.sync_derived();
                changed = true;
            }
        } else {
            ui.label("Max. Segment-Länge:");
            if ui
                .add(egui::Slider::new(&mut self.seg.max_segment_length, 1.0..=50.0).suffix(" m"))
                .changed()
            {
                self.seg.last_edited = LastEdited::Distance;
                changed = true;
            }
        }

        changed
    }
}
