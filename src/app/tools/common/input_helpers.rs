//! UI-Eingabe-Hilfsfunktionen fuer Route-Tools.

/// Unterdrueckt Rauschen/Restwerte, die ohne echtes Scrollen auftreten koennen.
const WHEEL_DELTA_THRESHOLD: f32 = 0.5;

/// Ermittelt die Scroll-Richtung fuer ein gehovertes Widget und konsumiert das Event.
///
/// Gibt `+1.0` (hoch), `-1.0` (runter) oder `0.0` (nicht gehovert / kein Scroll) zurueck.
/// Wird ein Scroll-Event erkannt, wird er nullgestellt, damit uebergeordnete ScrollAreas
/// nicht gleichzeitig scrollen.
pub(crate) fn wheel_dir(ui: &egui::Ui, response: &egui::Response) -> f32 {
    if !response.hovered() {
        return 0.0;
    }
    let (raw, smooth) = ui.input(|i| (i.raw_scroll_delta.y, i.smooth_scroll_delta.y));
    // Scroll-Events konsumieren – auch Reste aus egui's Scroll-Smoothing,
    // die ueber mehrere Frames aus unprocessed_scroll_delta nachfliessen.
    if raw.abs() > 0.0 || smooth.abs() > 0.0 {
        ui.input_mut(|i| {
            i.raw_scroll_delta.y = 0.0;
            i.smooth_scroll_delta.y = 0.0;
        });
    }
    if raw.abs() < WHEEL_DELTA_THRESHOLD {
        return 0.0;
    }
    raw.signum()
}
