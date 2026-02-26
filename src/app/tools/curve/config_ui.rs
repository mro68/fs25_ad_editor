//! UI-Konfigurationspanel und Kontextmenü für das Bézier-Kurven-Tool.
//!
//! Enthält:
//! - `render_config_view` — Grad-Auswahl + Segment-Konfiguration im Properties-Panel
//! - `build_tangent_menu_data` — Datenaufbereitung für Tangenten-Kontextmenü
//! - `apply_tangent_from_menu` — Anwendung der Tangenten-Auswahl aus dem Kontextmenü

use super::super::common::{render_segment_config_3modes, tangent_options, TangentMenuData};
use super::super::RouteTool;
use super::geometry::{approx_length, cubic_bezier, quadratic_bezier};
use super::state::{CurveDegree, CurveTool, Phase};

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
            self.tangents.reset_tangents();
            changed = true;
        }
        ui.add_space(4.0);

        // Nachbearbeitungs-Modus
        let adjusting = !self.lifecycle.last_created_ids.is_empty()
            && self.last_start_anchor.is_some()
            && self.lifecycle.last_end_anchor.is_some()
            && self.last_control_point1.is_some();

        let length = if adjusting {
            let start_pos = self.last_start_anchor.unwrap().position();
            let end_pos = self.lifecycle.last_end_anchor.unwrap().position();
            let cp1 = self.last_control_point1.unwrap();
            let cp2 = self.last_control_point2;
            match self.degree {
                CurveDegree::Quadratic => {
                    approx_length(|t| quadratic_bezier(start_pos, cp1, end_pos, t), 64)
                }
                CurveDegree::Cubic => {
                    let cp2v = cp2.unwrap_or(cp1);
                    approx_length(|t| cubic_bezier(start_pos, cp1, cp2v, end_pos, t), 64)
                }
            }
        } else {
            self.curve_length()
        };

        let ready = self.is_ready();
        let (seg_changed, recreate) = render_segment_config_3modes(
            &mut self.seg,
            ui,
            adjusting,
            ready,
            length,
            "Kurvenlänge",
        );
        if recreate {
            self.lifecycle.recreate_needed = true;
        }
        changed |= seg_changed;

        changed
    }

    /// Liefert Tangenten-Menüdaten für das zentrale Kontextmenü (nur Daten, kein UI).
    ///
    /// Nur aktiv für kubische Kurven in `Phase::Control` oder im Adjusting-Modus,
    /// wenn Nachbarn an Start- oder Endpunkt vorhanden sind.
    pub(super) fn build_tangent_menu_data(&self) -> Option<TangentMenuData> {
        if self.degree != CurveDegree::Cubic {
            return None;
        }

        let in_control = self.phase == Phase::Control;
        let adjusting = self.lifecycle.has_last_created()
            && self.last_start_anchor.is_some()
            && self.lifecycle.last_end_anchor.is_some();
        if !in_control && !adjusting {
            return None;
        }

        let has_start = !self.tangents.start_neighbors.is_empty();
        let has_end = !self.tangents.end_neighbors.is_empty();
        if !has_start && !has_end {
            return None;
        }

        Some(TangentMenuData {
            start_options: tangent_options(&self.tangents.start_neighbors),
            end_options: tangent_options(&self.tangents.end_neighbors),
            current_start: self.tangents.tangent_start,
            current_end: self.tangents.tangent_end,
        })
    }

    /// Wendet die vom User gewählten Tangenten aus dem Kontextmenü an.
    ///
    /// Aktualisiert Kontrollpunkte, derived state und setzt ggf. das Recreate-Flag.
    pub(super) fn apply_tangent_from_menu(
        &mut self,
        start: super::super::common::TangentSource,
        end: super::super::common::TangentSource,
    ) {
        self.tangents.tangent_start = start;
        self.tangents.tangent_end = end;
        self.apply_tangent_to_cp();
        self.sync_derived();
        self.init_apex();
        if self.lifecycle.has_last_created() {
            self.lifecycle.recreate_needed = true;
        }
    }
}
