//! Gemeinsame UI-Hilfsfunktionen.

/// Schwellenwert fuer Scroll-Events – unterdrückt Rauschen bei kleinen Scroll-Bewegungen.
pub(crate) const WHEEL_THRESHOLD: f32 = 0.5;

/// Wendet Mausrad-Scrolling auf einen numerischen Wert an.
///
/// Wenn die Response gehovert ist und ein Scroll-Event vorliegt,
/// wird `value` um `step` in Scroll-Richtung geaendert und auf `range` geclampt.
/// Das Scroll-Event wird konsumiert (nullgestellt), damit uebergeordnete ScrollAreas
/// nicht gleichzeitig scrollen.
/// Gibt `true` zurueck wenn sich der Wert geaendert hat.
pub(crate) fn apply_wheel_step(
    ui: &egui::Ui,
    response: &egui::Response,
    value: &mut f32,
    step: f32,
    range: std::ops::RangeInclusive<f32>,
) -> bool {
    if !response.hovered() {
        return false;
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
    if raw.abs() < WHEEL_THRESHOLD {
        return false;
    }
    let old = *value;
    *value = (*value + raw.signum() * step).clamp(*range.start(), *range.end());
    *value != old
}
