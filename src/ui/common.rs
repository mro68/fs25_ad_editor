//! Gemeinsame UI-Hilfsfunktionen.

use crate::app::tools::RouteToolAvailabilityContext;
use crate::app::AppState;

/// Schwellenwert fuer Scroll-Events – unterdrückt Rauschen bei kleinen Scroll-Bewegungen.
pub(crate) const WHEEL_THRESHOLD: f32 = 0.5;

/// Standard-Schrittweite fuer Mausrad-Anpassungen von Float-Werten.
pub(crate) const DEFAULT_FLOAT_WHEEL_STEP: f32 = 0.1;

const MEDIUM_FLOAT_WHEEL_STEP: f32 = 0.01;
const FINE_FLOAT_WHEEL_STEP: f32 = 0.001;
const SHIFT_WHEEL_MULTIPLIER_F32: f32 = 10.0;
const CTRL_WHEEL_MULTIPLIER_F32: f32 = 0.1;
const SHIFT_WHEEL_MULTIPLIER_USIZE: usize = 10;

fn effective_float_wheel_step(step: f32, modifiers: egui::Modifiers) -> f32 {
    let mut effective_step = step;
    if modifiers.shift {
        effective_step *= SHIFT_WHEEL_MULTIPLIER_F32;
    }
    if modifiers.ctrl {
        effective_step *= CTRL_WHEEL_MULTIPLIER_F32;
    }
    effective_step
}

fn effective_usize_wheel_step(step: usize, modifiers: egui::Modifiers) -> usize {
    if modifiers.shift {
        step.saturating_mul(SHIFT_WHEEL_MULTIPLIER_USIZE)
    } else {
        step
    }
}

fn wheel_direction_from_deltas(raw: f32, smooth: f32) -> f32 {
    if raw.abs() >= WHEEL_THRESHOLD {
        raw.signum()
    } else if smooth.abs() >= WHEEL_THRESHOLD {
        smooth.signum()
    } else {
        0.0
    }
}

/// Ermittelt die Scroll-Richtung fuer ein gehovertes Widget und konsumiert das Event.
///
/// Gibt `+1.0` (hoch), `-1.0` (runter) oder `0.0` (nicht gehovert / kein Scroll)
/// zurueck. Die Richtung wird zuerst aus `raw_scroll_delta` und bei Bedarf aus
/// `smooth_scroll_delta` abgeleitet.
/// Nur wenn ein wirksamer Scroll-Impuls erkannt wurde, wird das Event nullgestellt,
/// damit umgebende Scroll-Areas nicht gleichzeitig reagieren.
pub(crate) fn wheel_dir(ui: &egui::Ui, response: &egui::Response) -> f32 {
    if !response.hovered() {
        return 0.0;
    }

    let (raw, smooth) = ui.input(|i| (i.raw_scroll_delta.y, i.smooth_scroll_delta.y));
    let direction = wheel_direction_from_deltas(raw, smooth);
    if direction != 0.0 {
        ui.input_mut(|i| {
            i.raw_scroll_delta.y = 0.0;
            i.smooth_scroll_delta.y = 0.0;
        });
    }

    direction
}

/// Wendet Mausrad-Scrolling auf einen numerischen Wert an.
///
/// Wenn die Response gehovert ist und ein Scroll-Event vorliegt,
/// wird `value` um `step` in Scroll-Richtung geaendert und auf `range` geclampt.
/// `Shift` vergroessert die Schrittweite um Faktor 10, `Ctrl` reduziert sie auf 1/10.
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
    let direction = wheel_dir(ui, response);
    if direction == 0.0 {
        return false;
    }
    let modifiers = ui.input(|i| i.modifiers);
    let step = effective_float_wheel_step(step, modifiers);

    let old = *value;
    *value = (*value + direction * step).clamp(*range.start(), *range.end());
    *value != old
}

/// Liefert eine adaptive Mausrad-Schrittweite fuer Float-Werte.
///
/// Standard ist `0.1`; bei kleinen Werten wird feiner auf `0.01` bzw. `0.001`
/// reduziert.
pub(crate) fn adaptive_float_wheel_step(value: f32) -> f32 {
    let abs = value.abs();
    if abs < 0.1 {
        FINE_FLOAT_WHEEL_STEP
    } else if abs < 1.0 {
        MEDIUM_FLOAT_WHEEL_STEP
    } else {
        DEFAULT_FLOAT_WHEEL_STEP
    }
}

/// Wendet adaptives Mausrad-Scrolling auf einen Float-Wert an.
///
/// Die Schrittweite wird ueber `adaptive_float_wheel_step()` bestimmt.
/// Modifier wirken auf den adaptiven Basisschritt (`Shift` x10, `Ctrl` x0.1).
/// Ist `wheel_enabled` `false`, wird keine Aenderung vorgenommen.
pub(crate) fn apply_wheel_step_adaptive(
    ui: &egui::Ui,
    response: &egui::Response,
    value: &mut f32,
    range: std::ops::RangeInclusive<f32>,
    wheel_enabled: bool,
) -> bool {
    if !wheel_enabled {
        return false;
    }

    let step = adaptive_float_wheel_step(*value);
    apply_wheel_step(ui, response, value, step, range)
}

/// Wendet Mausrad-Scrolling mit Ganzzahl-Schritten auf einen `usize`-Wert an.
///
/// Bei Scroll-Impuls wird der Wert um genau `1` erhoeht oder verringert,
/// anschliessend auf `range` geclampt. `Shift` vergroessert den Schritt auf `10`;
/// `Ctrl` wird bei Ganzzahlen bewusst ignoriert. Ist `wheel_enabled` `false`, passiert nichts.
pub(crate) fn apply_wheel_step_usize(
    ui: &egui::Ui,
    response: &egui::Response,
    value: &mut usize,
    range: std::ops::RangeInclusive<usize>,
    wheel_enabled: bool,
) -> bool {
    if !wheel_enabled {
        return false;
    }

    let direction = wheel_dir(ui, response);
    if direction == 0.0 {
        return false;
    }
    let modifiers = ui.input(|i| i.modifiers);
    let step = effective_usize_wheel_step(1, modifiers);

    let old = *value;
    if direction > 0.0 {
        *value = value.saturating_add(step);
    } else {
        *value = value.saturating_sub(step);
    }
    *value = (*value).clamp(*range.start(), *range.end());

    *value != old
}

/// Baut den zentralen Availability-Kontext fuer alle Route-Tool-Surfaces.
pub(crate) fn route_tool_availability_context(state: &AppState) -> RouteToolAvailabilityContext {
    let has_farmland = state
        .farmland_polygons_arc()
        .is_some_and(|polygons| !polygons.is_empty());
    let has_background = state.has_background_image();
    let has_ordered_chain = state.road_map.as_deref().is_some_and(|road_map| {
        road_map
            .ordered_chain_nodes(&state.selection.selected_node_ids)
            .is_some()
    });

    RouteToolAvailabilityContext {
        has_farmland,
        has_background,
        has_ordered_chain,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn modifiers(shift: bool, ctrl: bool) -> egui::Modifiers {
        egui::Modifiers {
            shift,
            ctrl,
            ..egui::Modifiers::default()
        }
    }

    fn assert_f32_approx_eq(actual: f32, expected: f32) {
        assert!((actual - expected).abs() < 1e-6);
    }

    #[test]
    fn wheel_direction_prefers_raw_when_threshold_reached() {
        assert_eq!(wheel_direction_from_deltas(1.2, -2.0), 1.0);
        assert_eq!(wheel_direction_from_deltas(-1.2, 2.0), -1.0);
    }

    #[test]
    fn wheel_direction_falls_back_to_smooth_when_raw_is_noise() {
        assert_eq!(wheel_direction_from_deltas(0.2, 0.8), 1.0);
        assert_eq!(wheel_direction_from_deltas(-0.2, -0.8), -1.0);
    }

    #[test]
    fn wheel_direction_returns_zero_for_subthreshold_noise() {
        assert_eq!(wheel_direction_from_deltas(0.49, 0.49), 0.0);
        assert_eq!(wheel_direction_from_deltas(-0.49, -0.49), 0.0);
    }

    #[test]
    fn adaptive_float_wheel_step_uses_expected_bands() {
        assert_eq!(adaptive_float_wheel_step(2.0), DEFAULT_FLOAT_WHEEL_STEP);
        assert_eq!(adaptive_float_wheel_step(0.5), MEDIUM_FLOAT_WHEEL_STEP);
        assert_eq!(adaptive_float_wheel_step(0.05), FINE_FLOAT_WHEEL_STEP);
        assert_eq!(adaptive_float_wheel_step(-0.05), FINE_FLOAT_WHEEL_STEP);
    }

    #[test]
    fn float_wheel_step_applies_shift_and_ctrl_modifiers() {
        let base_step = 0.1;

        assert_f32_approx_eq(
            effective_float_wheel_step(base_step, modifiers(false, false)),
            0.1,
        );
        assert_f32_approx_eq(
            effective_float_wheel_step(base_step, modifiers(true, false)),
            1.0,
        );
        assert_f32_approx_eq(
            effective_float_wheel_step(base_step, modifiers(false, true)),
            0.01,
        );
        assert_f32_approx_eq(
            effective_float_wheel_step(base_step, modifiers(true, true)),
            0.1,
        );
    }

    #[test]
    fn adaptive_float_step_combines_with_modifiers() {
        let adaptive_base = adaptive_float_wheel_step(0.05);
        assert_eq!(adaptive_base, FINE_FLOAT_WHEEL_STEP);

        assert_f32_approx_eq(
            effective_float_wheel_step(adaptive_base, modifiers(true, false)),
            0.01,
        );
        assert_f32_approx_eq(
            effective_float_wheel_step(adaptive_base, modifiers(false, true)),
            0.0001,
        );
    }

    #[test]
    fn usize_wheel_step_ignores_ctrl_but_scales_with_shift() {
        assert_eq!(effective_usize_wheel_step(1, modifiers(false, false)), 1);
        assert_eq!(effective_usize_wheel_step(1, modifiers(false, true)), 1);
        assert_eq!(effective_usize_wheel_step(1, modifiers(true, false)), 10);
        assert_eq!(effective_usize_wheel_step(1, modifiers(true, true)), 10);
    }
}
