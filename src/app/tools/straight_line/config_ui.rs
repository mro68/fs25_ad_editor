//! UI-Konfigurationspanel für das Gerade-Strecke-Tool.
//!
//! Die `render_config_view`-Methode enthält die egui-Logik für:
//! - Länge · Segment-Abstand · Node-Anzahl (Nachbearbeitungs- und Live-Modus)

use super::state::StraightLineTool;

impl StraightLineTool {
    /// Rendert das Konfigurationspanel im Properties-Panel.
    ///
    /// Gibt `true` zurück wenn sich eine Einstellung geändert hat.
    pub(super) fn render_config_view(&mut self, ui: &mut egui::Ui) -> bool {
        let adjusting = !self.lifecycle.last_created_ids.is_empty()
            && self.last_start_anchor.is_some()
            && self.lifecycle.last_end_anchor.is_some();

        if adjusting {
            let Some(start_anchor) = self.last_start_anchor else {
                return false;
            };
            let Some(end_anchor) = self.lifecycle.last_end_anchor else {
                return false;
            };
            let distance = start_anchor.position().distance(end_anchor.position());
            let (changed, recreate) = self.seg.render_adjusting(ui, distance, "Streckenlänge");
            if recreate {
                self.lifecycle.recreate_needed = true;
            }
            changed
        } else if self.start.is_some() && self.end.is_some() {
            let distance = self.total_distance();
            self.seg.render_live(ui, distance, "Streckenlänge")
        } else {
            self.seg.render_default(ui)
        }
    }
}
