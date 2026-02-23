//! UI-Konfigurationspanel für das Gerade-Strecke-Tool.
//!
//! Die `render_config_view`-Methode enthält die egui-Logik für:
//! - Länge · Segment-Abstand · Node-Anzahl (Nachbearbeitungs- und Live-Modus)

use super::super::common::render_segment_config_3modes;
use super::state::StraightLineTool;

impl StraightLineTool {
    /// Rendert das Konfigurationspanel im Properties-Panel.
    ///
    /// Gibt `true` zurück wenn sich eine Einstellung geändert hat.
    pub(super) fn render_config_view(&mut self, ui: &mut egui::Ui) -> bool {
        let adjusting = !self.lifecycle.last_created_ids.is_empty()
            && self.last_start_anchor.is_some()
            && self.lifecycle.last_end_anchor.is_some();

        let length = if adjusting {
            let start = self.last_start_anchor.unwrap().position();
            let end = self.lifecycle.last_end_anchor.unwrap().position();
            start.distance(end)
        } else {
            self.total_distance()
        };

        let ready = self.start.is_some() && self.end.is_some();
        let (changed, recreate) = render_segment_config_3modes(
            &mut self.seg,
            ui,
            adjusting,
            ready,
            length,
            "Streckenlänge",
        );
        if recreate {
            self.lifecycle.recreate_needed = true;
        }
        changed
    }
}
