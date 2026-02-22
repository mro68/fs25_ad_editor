//! UI-Konfigurationspanel für das Catmull-Rom-Spline-Tool.
//!
//! Die `render_config_view`-Methode enthält die gesamte egui-Logik für:
//! - Tangenten-ComboBoxen am Start/Ende (nur Nachbearbeitungs-Modus)
//! - Länge · Segment-Abstand · Node-Anzahl (Nachbearbeitungs- und Live-Modus)

use super::super::common::{
    angle_to_compass, node_count_from_length, segment_length_from_count, LastEdited,
    TangentSource,
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
        let adjusting = !self.last_created_ids.is_empty() && self.last_anchors.len() >= 2;

        if adjusting {
            // Tangente Start
            if !self.start_neighbors.is_empty() {
                let old_tangent = self.tangent_start;
                let selected_text = match self.tangent_start {
                    TangentSource::None => "Standard".to_string(),
                    TangentSource::Connection { neighbor_id, angle } => {
                        format!("→ Node #{} ({})", neighbor_id, angle_to_compass(angle))
                    }
                };
                ui.label("Tangente Start:");
                egui::ComboBox::from_id_salt("spline_tangent_start")
                    .selected_text(selected_text)
                    .show_ui(ui, |ui| {
                        ui.selectable_value(
                            &mut self.tangent_start,
                            TangentSource::None,
                            "Standard",
                        );
                        for neighbor in &self.start_neighbors {
                            let label = format!(
                                "→ Node #{} ({})",
                                neighbor.neighbor_id,
                                angle_to_compass(neighbor.angle)
                            );
                            ui.selectable_value(
                                &mut self.tangent_start,
                                TangentSource::Connection {
                                    neighbor_id: neighbor.neighbor_id,
                                    angle: neighbor.angle,
                                },
                                label,
                            );
                        }
                    });
                if self.tangent_start != old_tangent {
                    if !self.last_created_ids.is_empty() {
                        self.recreate_needed = true;
                    }
                    changed = true;
                }
            }

            // Tangente Ende
            if !self.end_neighbors.is_empty() {
                let old_tangent = self.tangent_end;
                let selected_text = match self.tangent_end {
                    TangentSource::None => "Standard".to_string(),
                    TangentSource::Connection { neighbor_id, angle } => {
                        format!("→ Node #{} ({})", neighbor_id, angle_to_compass(angle))
                    }
                };
                ui.label("Tangente Ende:");
                egui::ComboBox::from_id_salt("spline_tangent_end")
                    .selected_text(selected_text)
                    .show_ui(ui, |ui| {
                        ui.selectable_value(
                            &mut self.tangent_end,
                            TangentSource::None,
                            "Standard",
                        );
                        for neighbor in &self.end_neighbors {
                            let label = format!(
                                "→ Node #{} ({})",
                                neighbor.neighbor_id,
                                angle_to_compass(neighbor.angle)
                            );
                            ui.selectable_value(
                                &mut self.tangent_end,
                                TangentSource::Connection {
                                    neighbor_id: neighbor.neighbor_id,
                                    angle: neighbor.angle,
                                },
                                label,
                            );
                        }
                    });
                if self.tangent_end != old_tangent {
                    if !self.last_created_ids.is_empty() {
                        self.recreate_needed = true;
                    }
                    changed = true;
                }
            }

            if !self.start_neighbors.is_empty() || !self.end_neighbors.is_empty() {
                ui.add_space(4.0);
            }

            // Slider für Min. Abstand und Node-Anzahl im Nachbearbeitungs-Modus
            let length = Self::spline_length_from_anchors(
                &self.last_anchors,
                self.tangent_start,
                self.tangent_end,
            );

            ui.label(format!("Spline-Länge: {:.1} m", length));
            ui.add_space(4.0);

            ui.label("Min. Abstand:");
            let max_seg = length.max(1.0);
            if ui
                .add(egui::Slider::new(&mut self.seg.max_segment_length, 1.0..=max_seg).suffix(" m"))
                .changed()
            {
                self.seg.last_edited = LastEdited::Distance;
                self.seg.node_count = node_count_from_length(length, self.seg.max_segment_length);
                self.recreate_needed = true;
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
                self.seg.max_segment_length = segment_length_from_count(length, self.seg.node_count);
                self.recreate_needed = true;
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
                .add(egui::Slider::new(&mut self.seg.max_segment_length, 1.0..=max_seg).suffix(" m"))
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
