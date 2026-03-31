//! Gemeinsame UI-Hilfsfunktionen.

use crate::app::tools::RouteToolAvailabilityContext;
use crate::app::AppState;

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

/// Baut den zentralen Availability-Kontext fuer alle Route-Tool-Surfaces.
pub(crate) fn route_tool_availability_context(state: &AppState) -> RouteToolAvailabilityContext {
    let has_farmland = state
        .farmland_polygons
        .as_ref()
        .is_some_and(|polygons| !polygons.is_empty());
    let has_background = state.background_image.is_some();
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
