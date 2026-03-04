//! UI-Konfigurationspanel fuer das Gerade-Strecke-Tool.
//!
//! Die `render_config_view`-Methode enthaelt die egui-Logik fuer:
//! - Laenge · Segment-Abstand · Node-Anzahl (Nachbearbeitungs- und Live-Modus)

use super::super::common::render_segment_config_3modes;
use super::state::StraightLineTool;

impl StraightLineTool {
    /// Rendert das Konfigurationspanel im Properties-Panel.
    ///
    /// Gibt `true` zurueck wenn sich eine Einstellung geaendert hat.
    pub(super) fn render_config_view(
        &mut self,
        ui: &mut egui::Ui,
        distance_wheel_step_m: f32,
    ) -> bool {
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
            "Streckenlaenge",
            distance_wheel_step_m,
        );
        if recreate {
            self.lifecycle.recreate_needed = true;
        }
        changed
    }
}
