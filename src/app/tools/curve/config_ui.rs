//! UI-Konfigurationspanel für das Bézier-Kurven-Tool.
//!
//! Die `render_config_view`-Methode enthält die gesamte egui-Logik für:
//! - Grad-Auswahl (Quadratisch / Kubisch)
//! - Tangenten-ComboBoxen (nur Kubisch, wenn Start/Ende gesetzt)
//! - Länge · Segment-Abstand · Node-Anzahl (Nachbearbeitungs- und Live-Modus)

use super::geometry::{cubic_bezier, quadratic_bezier};
use super::super::common::{angle_to_compass, TangentSource};
use super::super::RouteTool;
use super::{CurveDegree, CurveTool, Phase};

impl CurveTool {
    /// Rendert das Konfigurationspanel im Properties-Panel.
    ///
    /// Gibt `true` zurück wenn sich eine Einstellung geändert hat.
    pub(super) fn render_config_view(&mut self, ui: &mut egui::Ui) -> bool {
        let mut changed = false;

        // Grad-Auswahl
        ui.label("Kurven-Grad:");
        let old_degree = self.degree;
        egui::ComboBox::from_id_salt("curve_degree")
            .selected_text(match self.degree {
                CurveDegree::Quadratic => "Quadratisch (Grad 2)",
                CurveDegree::Cubic => "Kubisch (Grad 3)",
            })
            .show_ui(ui, |ui| {
                ui.selectable_value(
                    &mut self.degree,
                    CurveDegree::Quadratic,
                    "Quadratisch (Grad 2)",
                );
                ui.selectable_value(&mut self.degree, CurveDegree::Cubic, "Kubisch (Grad 3)");
            });
        if self.degree != old_degree {
            // Beim Gradwechsel CP2 und Tangenten zurücksetzen
            self.control_point2 = None;
            self.tangent_start = TangentSource::None;
            self.tangent_end = TangentSource::None;
            changed = true;
        }
        ui.add_space(4.0);

        // Tangenten-Auswahl (nur Cubic, wenn Start+End gesetzt)
        if self.degree == CurveDegree::Cubic {
            let show_tangent_ui = (self.phase == Phase::Control
                || (!self.last_created_ids.is_empty()
                    && self.last_start_anchor.is_some()
                    && self.last_end_anchor.is_some()))
                && (self.start.is_some() && self.end.is_some()
                    || self.last_start_anchor.is_some() && self.last_end_anchor.is_some());

            if show_tangent_ui {
                // Tangente Start
                if !self.start_neighbors.is_empty() {
                    let old_tangent = self.tangent_start;
                    let selected_text = match self.tangent_start {
                        TangentSource::None => "Manuell".to_string(),
                        TangentSource::Connection { neighbor_id, angle } => {
                            format!("→ Node #{} ({})", neighbor_id, angle_to_compass(angle))
                        }
                    };
                    ui.label("Tangente Start:");
                    egui::ComboBox::from_id_salt("tangent_start")
                        .selected_text(selected_text)
                        .show_ui(ui, |ui| {
                            ui.selectable_value(
                                &mut self.tangent_start,
                                TangentSource::None,
                                "Manuell",
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
                        self.apply_tangent_to_cp();
                        self.sync_derived();
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
                        TangentSource::None => "Manuell".to_string(),
                        TangentSource::Connection { neighbor_id, angle } => {
                            format!("→ Node #{} ({})", neighbor_id, angle_to_compass(angle))
                        }
                    };
                    ui.label("Tangente Ende:");
                    egui::ComboBox::from_id_salt("tangent_end")
                        .selected_text(selected_text)
                        .show_ui(ui, |ui| {
                            ui.selectable_value(
                                &mut self.tangent_end,
                                TangentSource::None,
                                "Manuell",
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
                        self.apply_tangent_to_cp();
                        self.sync_derived();
                        if !self.last_created_ids.is_empty() {
                            self.recreate_needed = true;
                        }
                        changed = true;
                    }
                }

                if !self.start_neighbors.is_empty() || !self.end_neighbors.is_empty() {
                    ui.add_space(4.0);
                }
            }
        }

        // Nachbearbeitungs-Modus
        let adjusting = !self.last_created_ids.is_empty()
            && self.last_start_anchor.is_some()
            && self.last_end_anchor.is_some()
            && self.last_control_point1.is_some();

        if adjusting {
            let (Some(start_anchor), Some(end_anchor), Some(cp1)) = (
                self.last_start_anchor,
                self.last_end_anchor,
                self.last_control_point1,
            ) else {
                return changed;
            };

            let start_pos = start_anchor.position();
            let end_pos = end_anchor.position();
            let cp2 = self.last_control_point2;
            let length = match self.degree {
                CurveDegree::Quadratic => {
                    Self::approx_length(|t| quadratic_bezier(start_pos, cp1, end_pos, t), 64)
                }
                CurveDegree::Cubic => {
                    let cp2v = cp2.unwrap_or(cp1);
                    Self::approx_length(|t| cubic_bezier(start_pos, cp1, cp2v, end_pos, t), 64)
                }
            };

            let (seg_changed, recreate) =
                self.seg.render_adjusting(ui, length, "Kurvenlänge");
            if recreate {
                self.recreate_needed = true;
            }
            changed |= seg_changed;
        } else if self.is_ready() {
            let length = self.curve_length();
            changed |= self.seg.render_live(ui, length, "Kurvenlänge");
        } else {
            changed |= self.seg.render_default(ui);
        }

        changed
    }
}
