//! Segment-Einstellungs-Popup (erscheint nach Doppelklick auf einen Segment-Node).
//!
//! Erlaubt Live-Anpassung der Segment-Selektionsparameter:
//! - `segment_stop_at_junction`: Ob die Selektion an Kreuzungen stoppt.
//! - `segment_max_angle_deg`: Maximale Winkelabweichung fuer Segment-Erkennung.
//!
//! Bei Aenderung wird die Selektion automatisch mit den neuen Parametern neu berechnet.

use crate::app::{AppIntent, SegmentSettingsPopupState};
use crate::shared::EditorOptions;

/// Zeigt das Segment-Einstellungs-Popup nach einem Doppelklick auf einen Segment-Node.
///
/// Bei Aenderung eines Parameters wird `NodeSegmentBetweenIntersectionsRequested`
/// zurueckgegeben, damit die Selektion sofort mit den neuen Einstellungen neu berechnet wird.
/// Das Popup schliesst sich automatisch wenn der User das Schliessen-X anklickt.
pub fn show_segment_settings_popup(
    ctx: &egui::Context,
    popup_state: &mut SegmentSettingsPopupState,
    opts: &mut EditorOptions,
) -> Vec<AppIntent> {
    let mut intents = Vec::new();
    if !popup_state.visible {
        return intents;
    }

    let mut open = popup_state.visible;
    let mut changed = false;

    egui::Window::new("Segment-Einstellungen")
        .collapsible(false)
        .resizable(false)
        .anchor(egui::Align2::CENTER_CENTER, [0.0, 0.0])
        .open(&mut open)
        .show(ctx, |ui| {
            ui.set_min_width(260.0);

            changed |= ui
                .checkbox(&mut opts.segment_stop_at_junction, "Bei Kreuzung stoppen")
                .changed();

            ui.add_space(4.0);

            ui.horizontal(|ui| {
                ui.label("Max. Winkel (°):");
                changed |= ui
                    .add(
                        egui::DragValue::new(&mut opts.segment_max_angle_deg)
                            .range(0.0..=180.0)
                            .speed(1.0),
                    )
                    .changed();
                if opts.segment_max_angle_deg == 0.0 {
                    ui.weak("(deaktiviert)");
                }
            });
        });

    if !open {
        popup_state.visible = false;
    }

    if changed {
        // Selektion mit neuen Einstellungen neu berechnen
        intents.push(AppIntent::NodeSegmentBetweenIntersectionsRequested {
            world_pos: popup_state.world_pos,
            additive: false,
        });
    }

    intents
}
