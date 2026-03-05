//! UI-Eingabe-Hilfsfunktionen fuer Route-Tools.

/// Unterdrueckt Rauschen/Restwerte, die ohne echtes Scrollen auftreten koennen.
const WHEEL_DELTA_THRESHOLD: f32 = 0.5;

/// Ermittelt die Scroll-Richtung fuer ein gehovertes Widget.
///
/// Gibt `+1.0` (hoch), `-1.0` (runter) oder `0.0` (nicht gehovert / kein Scroll) zurueck.
pub(crate) fn wheel_dir(ui: &egui::Ui, response: &egui::Response) -> f32 {
    if !response.hovered() {
        return 0.0;
    }
    let delta = ui.input(|i| i.raw_scroll_delta.y);
    if delta.abs() < WHEEL_DELTA_THRESHOLD {
        0.0
    } else {
        delta.signum()
    }
}
