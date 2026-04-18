//! Gemeinsame UI-Hilfsfunktionen.

use crate::app::tool_contract::RouteToolId;
use crate::app::{ConnectionDirection, ConnectionPriority, EditorTool};
use crate::shared::{BackgroundLayerKind, I18nKey, OverviewFieldDetectionSource};
use fs25_auto_drive_host_bridge::{
    HostActiveTool, HostBackgroundLayerKind, HostChromeSnapshot, HostDefaultConnectionDirection,
    HostDefaultConnectionPriority, HostRouteToolDisabledReason, HostRouteToolEntrySnapshot,
    HostRouteToolGroup, HostRouteToolId, HostRouteToolSelectionSnapshot, HostRouteToolSurface,
};

/// Schwellenwert fuer Scroll-Events – unterdrückt Rauschen bei kleinen Scroll-Bewegungen.
pub(crate) const WHEEL_THRESHOLD: f32 = 0.5;

/// Standard-Schrittweite fuer Mausrad-Anpassungen von Float-Werten.
pub(crate) const DEFAULT_FLOAT_WHEEL_STEP: f32 = 0.1;

const SCROLL_CAPTURE_EPSILON: f32 = 0.000_001;
const ALT_WHEEL_MULTIPLIER_F32: f32 = 10.0;
const CTRL_WHEEL_MULTIPLIER_F32: f32 = 0.1;
const ALT_WHEEL_MULTIPLIER_USIZE: usize = 10;

pub(crate) const OVERVIEW_FIELD_DETECTION_SOURCE_ORDER: [OverviewFieldDetectionSource; 4] = [
    OverviewFieldDetectionSource::ZipGroundGdm,
    OverviewFieldDetectionSource::FromZip,
    OverviewFieldDetectionSource::FieldTypeGrle,
    OverviewFieldDetectionSource::GroundGdm,
];

/// Erstellt ein Top-Level-`Ui`, in dem Panels via `show_inside()` gerendert werden koennen.
pub(crate) fn create_top_level_ui(ctx: &egui::Context, id_source: &'static str) -> egui::Ui {
    let mut top_ui = egui::Ui::new(
        ctx.clone(),
        egui::Id::new((ctx.viewport_id(), id_source)),
        egui::UiBuilder::new()
            .layer_id(egui::LayerId::background())
            .max_rect(ctx.content_rect()),
    );
    top_ui.set_clip_rect(ctx.content_rect());
    top_ui
}

pub(crate) fn host_background_layer_to_engine(
    kind: HostBackgroundLayerKind,
) -> BackgroundLayerKind {
    match kind {
        HostBackgroundLayerKind::Terrain => BackgroundLayerKind::Terrain,
        HostBackgroundLayerKind::Hillshade => BackgroundLayerKind::Hillshade,
        HostBackgroundLayerKind::FarmlandBorders => BackgroundLayerKind::FarmlandBorders,
        HostBackgroundLayerKind::FarmlandIds => BackgroundLayerKind::FarmlandIds,
        HostBackgroundLayerKind::PoiMarkers => BackgroundLayerKind::PoiMarkers,
        HostBackgroundLayerKind::Legend => BackgroundLayerKind::Legend,
    }
}

pub(crate) fn host_background_layer_label_key(kind: HostBackgroundLayerKind) -> I18nKey {
    match kind {
        HostBackgroundLayerKind::Terrain => I18nKey::MenuBgLayerTerrain,
        HostBackgroundLayerKind::Hillshade => I18nKey::MenuBgLayerHillshade,
        HostBackgroundLayerKind::FarmlandBorders => I18nKey::MenuBgLayerFarmlandBorders,
        HostBackgroundLayerKind::FarmlandIds => I18nKey::MenuBgLayerFarmlandIds,
        HostBackgroundLayerKind::PoiMarkers => I18nKey::MenuBgLayerPoiMarkers,
        HostBackgroundLayerKind::Legend => I18nKey::MenuBgLayerLegend,
    }
}

pub(crate) fn overview_field_detection_source_label(
    source: OverviewFieldDetectionSource,
) -> &'static str {
    match source {
        OverviewFieldDetectionSource::ZipGroundGdm => "densityMap_ground (ZIP)",
        OverviewFieldDetectionSource::FromZip => "infoLayer_farmlands (ZIP)",
        OverviewFieldDetectionSource::FieldTypeGrle => "infoLayer_fieldType (Savegame)",
        OverviewFieldDetectionSource::GroundGdm => "densityMap_ground (Savegame)",
    }
}

pub(crate) fn ordered_available_overview_field_detection_sources(
    available: &[OverviewFieldDetectionSource],
) -> Vec<OverviewFieldDetectionSource> {
    OVERVIEW_FIELD_DETECTION_SOURCE_ORDER
        .into_iter()
        .filter(|source| available.contains(source))
        .collect()
}

fn effective_float_wheel_step(step: f32, modifiers: egui::Modifiers) -> f32 {
    let mut effective_step = step;
    if modifiers.alt {
        effective_step *= ALT_WHEEL_MULTIPLIER_F32;
    }
    if modifiers.ctrl {
        effective_step *= CTRL_WHEEL_MULTIPLIER_F32;
    }
    effective_step
}

fn effective_usize_wheel_step(step: usize, modifiers: egui::Modifiers) -> usize {
    if modifiers.alt {
        step.saturating_mul(ALT_WHEEL_MULTIPLIER_USIZE)
    } else {
        step
    }
}

fn wheel_direction_from_deltas(raw: f32, _smooth: f32) -> f32 {
    if raw.abs() >= WHEEL_THRESHOLD {
        raw.signum()
    } else {
        0.0
    }
}

fn should_capture_hovered_scroll(raw: f32, smooth: f32) -> bool {
    raw.abs() > SCROLL_CAPTURE_EPSILON || smooth.abs() > SCROLL_CAPTURE_EPSILON
}

fn raw_scroll_delta_y(input: &egui::InputState) -> f32 {
    input
        .raw
        .events
        .iter()
        .filter_map(|event| match event {
            egui::Event::MouseWheel { delta, .. } => Some(delta.y),
            _ => None,
        })
        .sum()
}

/// Ermittelt die Scroll-Richtung fuer ein gehovertes Widget und konsumiert das Event.
///
/// Gibt `+1.0` (hoch), `-1.0` (runter) oder `0.0` (nicht gehovert / kein Scroll)
/// zurueck. Fuer diskrete Numerik-Anpassungen wird bewusst nur
/// der rohe `MouseWheel`-Eventstrom ausgewertet, damit ein physischer Wheel-Notch genau
/// einen Schritt ausloest (kein Mehrfach-Feuern durch Smoothing). Solange ein
/// Numeric-Widget gehovert ist, werden Wheel-Events mit Raw- oder Smooth-Delta
/// konsumiert, damit umgebende Scroll-Areas nicht gleichzeitig reagieren.
pub(crate) fn wheel_dir(ui: &egui::Ui, response: &egui::Response) -> f32 {
    if !response.hovered() {
        return 0.0;
    }

    let (raw, smooth) = ui.input(|i| (raw_scroll_delta_y(i), i.smooth_scroll_delta.y));
    let direction = wheel_direction_from_deltas(raw, smooth);
    if should_capture_hovered_scroll(raw, smooth) {
        ui.input_mut(|i| {
            i.raw
                .events
                .retain(|event| !matches!(event, egui::Event::MouseWheel { .. }));
            i.smooth_scroll_delta.y = 0.0;
        });
    }

    direction
}

/// Wendet Mausrad-Scrolling auf einen numerischen Wert an.
///
/// Wenn die Response gehovert ist und ein Scroll-Event vorliegt,
/// wird `value` um `step` in Scroll-Richtung geaendert und auf `range` geclampt.
/// `Alt` vergroessert die Schrittweite um Faktor 10, `Ctrl` reduziert sie auf 1/10.
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

/// Wendet Mausrad-Scrolling mit Ganzzahl-Schritten auf einen `usize`-Wert an.
///
/// Bei Scroll-Impuls wird der Wert um genau `1` erhoeht oder verringert,
/// anschliessend auf `range` geclampt. `Alt` vergroessert den Schritt auf `10`;
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

/// Wendet Mausrad-Scrolling mit der Standard-Schrittweite (`0.1`) auf einen Float an.
pub(crate) fn apply_wheel_step_default(
    ui: &egui::Ui,
    response: &egui::Response,
    value: &mut f32,
    range: std::ops::RangeInclusive<f32>,
) -> bool {
    apply_wheel_step(ui, response, value, DEFAULT_FLOAT_WHEEL_STEP, range)
}

/// Wendet den Float-Standardschritt (`0.1`) nur an, wenn Wheel-Anpassung aktiv ist.
pub(crate) fn apply_wheel_step_default_enabled(
    ui: &egui::Ui,
    response: &egui::Response,
    value: &mut f32,
    range: std::ops::RangeInclusive<f32>,
    wheel_enabled: bool,
) -> bool {
    if !wheel_enabled {
        return false;
    }

    apply_wheel_step_default(ui, response, value, range)
}

/// Konvertiert eine host-neutrale Route-Tool-ID in die Engine-ID fuer AppIntents.
pub(crate) fn host_route_tool_to_engine(tool: HostRouteToolId) -> RouteToolId {
    match tool {
        HostRouteToolId::Straight => RouteToolId::Straight,
        HostRouteToolId::CurveQuad => RouteToolId::CurveQuad,
        HostRouteToolId::CurveCubic => RouteToolId::CurveCubic,
        HostRouteToolId::Spline => RouteToolId::Spline,
        HostRouteToolId::Bypass => RouteToolId::Bypass,
        HostRouteToolId::SmoothCurve => RouteToolId::SmoothCurve,
        HostRouteToolId::Parking => RouteToolId::Parking,
        HostRouteToolId::FieldBoundary => RouteToolId::FieldBoundary,
        HostRouteToolId::FieldPath => RouteToolId::FieldPath,
        HostRouteToolId::RouteOffset => RouteToolId::RouteOffset,
        HostRouteToolId::ColorPath => RouteToolId::ColorPath,
    }
}

/// Konvertiert ein host-neutrales Tool in das lokale Editor-Tool.
pub(crate) fn host_active_tool_to_editor(tool: HostActiveTool) -> EditorTool {
    match tool {
        HostActiveTool::Select => EditorTool::Select,
        HostActiveTool::Connect => EditorTool::Connect,
        HostActiveTool::AddNode => EditorTool::AddNode,
        HostActiveTool::Route => EditorTool::Route,
    }
}

/// Konvertiert die host-neutrale Default-Richtung in den Engine-Typ.
pub(crate) fn host_default_direction_to_engine(
    direction: HostDefaultConnectionDirection,
) -> ConnectionDirection {
    match direction {
        HostDefaultConnectionDirection::Regular => ConnectionDirection::Regular,
        HostDefaultConnectionDirection::Dual => ConnectionDirection::Dual,
        HostDefaultConnectionDirection::Reverse => ConnectionDirection::Reverse,
    }
}

/// Konvertiert die host-neutrale Default-Prioritaet in den Engine-Typ.
pub(crate) fn host_default_priority_to_engine(
    priority: HostDefaultConnectionPriority,
) -> ConnectionPriority {
    match priority {
        HostDefaultConnectionPriority::Regular => ConnectionPriority::Regular,
        HostDefaultConnectionPriority::SubPriority => ConnectionPriority::SubPriority,
    }
}

/// Liefert den i18n-Key fuer einen host-neutralen Disabled-Grund.
pub(crate) fn host_route_tool_disabled_reason_key(reason: HostRouteToolDisabledReason) -> I18nKey {
    match reason {
        HostRouteToolDisabledReason::MissingFarmland => I18nKey::RouteToolNeedFarmland,
        HostRouteToolDisabledReason::MissingBackground => I18nKey::RouteToolNeedBackground,
        HostRouteToolDisabledReason::MissingOrderedChain => I18nKey::RouteToolNeedOrderedChain,
    }
}

/// Filtert host-neutrale Route-Tool-Eintraege nach Surface und Gruppe.
pub(crate) fn host_route_tool_entries_for<'a>(
    chrome: &'a HostChromeSnapshot,
    surface: HostRouteToolSurface,
    group: HostRouteToolGroup,
) -> impl Iterator<Item = &'a HostRouteToolEntrySnapshot> + 'a {
    chrome
        .route_tool_entries
        .iter()
        .filter(move |entry| entry.surface == surface && entry.group == group)
}

/// Liefert die zuletzt gewaehlte host-neutrale Tool-ID je Gruppe.
pub(crate) fn host_memory_tool_for_group(
    memory: HostRouteToolSelectionSnapshot,
    group: HostRouteToolGroup,
) -> HostRouteToolId {
    match group {
        HostRouteToolGroup::Basics => memory.basics,
        HostRouteToolGroup::Section => memory.section,
        HostRouteToolGroup::Analysis => memory.analysis,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn modifiers(alt: bool, ctrl: bool) -> egui::Modifiers {
        egui::Modifiers {
            alt,
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
    fn wheel_direction_ignores_smooth_delta_for_discrete_notches() {
        assert_eq!(wheel_direction_from_deltas(0.2, 2.0), 0.0);
        assert_eq!(wheel_direction_from_deltas(-0.2, -2.0), 0.0);
    }

    #[test]
    fn wheel_direction_returns_zero_for_subthreshold_noise() {
        assert_eq!(wheel_direction_from_deltas(0.49, 0.49), 0.0);
        assert_eq!(wheel_direction_from_deltas(-0.49, -0.49), 0.0);
    }

    #[test]
    fn hovered_scroll_capture_detects_raw_and_smooth_deltas() {
        assert!(should_capture_hovered_scroll(0.1, 0.0));
        assert!(should_capture_hovered_scroll(0.0, 0.1));
        assert!(should_capture_hovered_scroll(0.1, 0.1));
    }

    #[test]
    fn hovered_scroll_capture_ignores_zero_or_tiny_noise() {
        assert!(!should_capture_hovered_scroll(0.0, 0.0));
        assert!(!should_capture_hovered_scroll(0.000_000_1, 0.0));
        assert!(!should_capture_hovered_scroll(0.0, -0.000_000_1));
    }

    #[test]
    fn host_background_layer_mapping_covers_all_variants() {
        assert_eq!(
            host_background_layer_to_engine(HostBackgroundLayerKind::Terrain),
            BackgroundLayerKind::Terrain
        );
        assert_eq!(
            host_background_layer_to_engine(HostBackgroundLayerKind::Hillshade),
            BackgroundLayerKind::Hillshade
        );
        assert_eq!(
            host_background_layer_to_engine(HostBackgroundLayerKind::FarmlandBorders),
            BackgroundLayerKind::FarmlandBorders
        );
        assert_eq!(
            host_background_layer_to_engine(HostBackgroundLayerKind::FarmlandIds),
            BackgroundLayerKind::FarmlandIds
        );
        assert_eq!(
            host_background_layer_to_engine(HostBackgroundLayerKind::PoiMarkers),
            BackgroundLayerKind::PoiMarkers
        );
        assert_eq!(
            host_background_layer_to_engine(HostBackgroundLayerKind::Legend),
            BackgroundLayerKind::Legend
        );
    }

    #[test]
    fn ordered_available_overview_sources_follow_canonical_ui_order() {
        let ordered = ordered_available_overview_field_detection_sources(&[
            OverviewFieldDetectionSource::GroundGdm,
            OverviewFieldDetectionSource::FromZip,
            OverviewFieldDetectionSource::ZipGroundGdm,
        ]);

        assert_eq!(
            ordered,
            vec![
                OverviewFieldDetectionSource::ZipGroundGdm,
                OverviewFieldDetectionSource::FromZip,
                OverviewFieldDetectionSource::GroundGdm,
            ]
        );
    }

    #[test]
    fn float_wheel_step_applies_alt_and_ctrl_modifiers() {
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
    fn usize_wheel_step_ignores_ctrl_but_scales_with_alt() {
        assert_eq!(effective_usize_wheel_step(1, modifiers(false, false)), 1);
        assert_eq!(effective_usize_wheel_step(1, modifiers(false, true)), 1);
        assert_eq!(effective_usize_wheel_step(1, modifiers(true, false)), 10);
        assert_eq!(effective_usize_wheel_step(1, modifiers(true, true)), 10);
    }
}
