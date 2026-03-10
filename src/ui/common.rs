//! Gemeinsame UI-Hilfsfunktionen.

/// Schwellenwert fuer Scroll-Events – unterdrückt Rauschen bei kleinen Scroll-Bewegungen.
pub(crate) const WHEEL_THRESHOLD: f32 = 0.5;

/// Wendet Mausrad-Scrolling auf einen numerischen Wert an.
///
/// Wenn die Response gehovert ist und ein Scroll-Event vorliegt,
/// wird `value` um `step` in Scroll-Richtung geaendert und auf `range` geclampt.
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
    let delta = ui.input(|i| i.raw_scroll_delta.y);
    if delta.abs() < WHEEL_THRESHOLD {
        return false;
    }
    let old = *value;
    *value = (*value + delta.signum() * step).clamp(*range.start(), *range.end());
    *value != old
}
