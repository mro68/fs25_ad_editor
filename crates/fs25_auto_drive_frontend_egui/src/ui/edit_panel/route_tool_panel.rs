use crate::app::tool_contract::TangentSource;
use crate::app::ui_contract::{
    panel_action_to_intent, BypassPanelAction, BypassPanelState, ColorPathPanelAction,
    ColorPathPanelPhase, ColorPathPanelState, CurveDegreeChoice, CurvePanelAction, CurvePanelState,
    CurveTangentsPanelState, ExistingConnectionModeChoice, FieldBoundaryPanelAction,
    FieldBoundaryPanelState, FieldPathModeChoice, FieldPathPanelAction, FieldPathPanelPhase,
    FieldPathPanelState, FieldPathPreviewStatus, FieldPathSelectionSummary, PanelAction,
    ParkingPanelAction, ParkingPanelState, ParkingRampSideChoice, RouteOffsetPanelAction,
    RouteOffsetPanelState, RouteToolConfigState, RouteToolPanelAction, RouteToolPanelState,
    SegmentConfigPanelAction, SegmentConfigPanelState, SegmentLengthKind, SmoothCurvePanelAction,
    SmoothCurvePanelState, SplinePanelAction, SplinePanelState, StraightPanelAction,
    StraightPanelState, TangentHelpHint, TangentNoneReason, TangentSelectionState,
    BYPASS_BASE_SPACING_LIMITS, BYPASS_OFFSET_LIMITS, PARKING_BAY_LENGTH_LIMITS,
    PARKING_ENTRY_EXIT_T_LIMITS, PARKING_MAX_NODE_DISTANCE_LIMITS, PARKING_NUM_ROWS_LIMITS,
    PARKING_RAMP_LENGTH_LIMITS, PARKING_ROTATION_STEP_LIMITS, PARKING_ROW_SPACING_LIMITS,
    ROUTE_OFFSET_BASE_SPACING_LIMITS, ROUTE_OFFSET_DISTANCE_LIMITS, SMOOTH_CURVE_MAX_ANGLE_LIMITS,
    SMOOTH_CURVE_MIN_DISTANCE_LIMITS,
};
use crate::app::{AppIntent, ConnectionDirection, ConnectionPriority};
use crate::shared::{t, Language};
use crate::ui::common::{apply_wheel_step_default_enabled, apply_wheel_step_usize};
use crate::ui::properties::selectors::{
    render_direction_icon_selector, render_priority_icon_selector,
};

mod analysis_panel;
mod curve_panel;
mod generator_panel;

/// Eingabeparameter fuer das Rendern des Route-Tool-Panels.
pub(super) struct RouteToolPanelProps {
    pub(super) route_tool: RouteToolPanelState,
    pub(super) default_direction: ConnectionDirection,
    pub(super) default_priority: ConnectionPriority,
    pub(super) distance_wheel_step_m: f32,
    pub(super) panel_pos: Option<egui::Pos2>,
    pub(super) lang: Language,
}

struct RouteToolPanelRenderContext<'a> {
    wheel_enabled: bool,
    events: &'a mut Vec<AppIntent>,
}

/// Rendert das Route-Tool-Panel mit Tool-Konfiguration sowie Ausfuehren/Abbrechen.
///
/// Ein positiver `distance_wheel_step_m` aktiviert Mausrad-Anpassungen in den
/// numerischen Unterpanels. Die konkrete Scroll-Auswertung bleibt in
/// `ui::common`, damit Route-Tool- und Analysis-Widgets dieselbe Wheel-Logik
/// verwenden.
pub(super) fn render_route_tool_panel(
    ctx: &egui::Context,
    props: RouteToolPanelProps,
    events: &mut Vec<AppIntent>,
) {
    let RouteToolPanelProps {
        route_tool,
        default_direction,
        default_priority,
        distance_wheel_step_m,
        panel_pos,
        lang,
    } = props;
    let mut panel_ctx = RouteToolPanelRenderContext {
        wheel_enabled: distance_wheel_step_m > 0.0,
        events,
    };

    let mut window = egui::Window::new("📐 Route-Tool")
        .collapsible(false)
        .resizable(false)
        .default_width(360.0)
        .min_width(320.0)
        .max_width(420.0)
        .auto_sized();

    if let Some(pos) = panel_pos {
        window = window.default_pos(pos);
    }

    window.show(ctx, |ui| {
        ui.set_min_width(320.0);
        ui.set_max_width(420.0);

        if let Some(status_text) = route_tool.status_text.as_deref() {
            ui.label(status_text);
        }

        ui.add_space(6.0);
        let mut selected_dir = default_direction;
        render_direction_icon_selector(ui, &mut selected_dir, "route_tool_floating");
        if selected_dir != default_direction {
            push_panel_action(
                panel_ctx.events,
                PanelAction::SetDefaultDirection {
                    direction: selected_dir,
                },
            );
        }

        ui.add_space(4.0);
        let mut selected_prio = default_priority;
        render_priority_icon_selector(ui, &mut selected_prio, "route_tool_floating");
        if selected_prio != default_priority {
            push_panel_action(
                panel_ctx.events,
                PanelAction::SetDefaultPriority {
                    priority: selected_prio,
                },
            );
        }

        ui.add_space(6.0);

        if let Some(config_state) = route_tool.config_state.as_ref() {
            render_route_tool_config(ui, config_state, lang, &mut panel_ctx);
        } else {
            ui.small("Kein Route-Tool aktiv.");
        }

        ui.add_space(8.0);
        ui.horizontal(|ui| {
            if ui
                .add_enabled(route_tool.can_execute, egui::Button::new("✓ Ausfuehren"))
                .clicked()
            {
                push_panel_action(panel_ctx.events, PanelAction::RouteToolExecute);
            }
            if ui.button("✕ Abbrechen").clicked() {
                push_panel_action(panel_ctx.events, PanelAction::RouteToolCancel);
            }
        });
    });
}

fn render_route_tool_config(
    ui: &mut egui::Ui,
    config_state: &RouteToolConfigState,
    lang: Language,
    panel_ctx: &mut RouteToolPanelRenderContext<'_>,
) {
    match config_state {
        RouteToolConfigState::Straight(state) => render_straight_panel(ui, state, panel_ctx),
        RouteToolConfigState::Curve(state) => render_curve_panel(ui, state, panel_ctx),
        RouteToolConfigState::Spline(state) => render_spline_panel(ui, state, panel_ctx),
        RouteToolConfigState::SmoothCurve(state) => render_smooth_curve_panel(ui, state, panel_ctx),
        RouteToolConfigState::Bypass(state) => render_bypass_panel(ui, state, panel_ctx),
        RouteToolConfigState::Parking(state) => render_parking_panel(ui, state, lang, panel_ctx),
        RouteToolConfigState::FieldBoundary(state) => {
            render_field_boundary_panel(ui, state, panel_ctx)
        }
        RouteToolConfigState::FieldPath(state) => {
            render_field_path_panel(ui, state, lang, panel_ctx)
        }
        RouteToolConfigState::RouteOffset(state) => render_route_offset_panel(ui, state, panel_ctx),
        RouteToolConfigState::ColorPath(state) => render_color_path_panel(ui, state, panel_ctx),
    }
}

fn render_straight_panel(
    ui: &mut egui::Ui,
    state: &StraightPanelState,
    panel_ctx: &mut RouteToolPanelRenderContext<'_>,
) {
    generator_panel::render_straight_panel(ui, state, panel_ctx);
}

fn render_curve_panel(
    ui: &mut egui::Ui,
    state: &CurvePanelState,
    panel_ctx: &mut RouteToolPanelRenderContext<'_>,
) {
    curve_panel::render_curve_panel(ui, state, panel_ctx);
}

fn render_spline_panel(
    ui: &mut egui::Ui,
    state: &SplinePanelState,
    panel_ctx: &mut RouteToolPanelRenderContext<'_>,
) {
    curve_panel::render_spline_panel(ui, state, panel_ctx);
}

fn render_smooth_curve_panel(
    ui: &mut egui::Ui,
    state: &SmoothCurvePanelState,
    panel_ctx: &mut RouteToolPanelRenderContext<'_>,
) {
    generator_panel::render_smooth_curve_panel(ui, state, panel_ctx);
}

fn render_bypass_panel(
    ui: &mut egui::Ui,
    state: &BypassPanelState,
    panel_ctx: &mut RouteToolPanelRenderContext<'_>,
) {
    generator_panel::render_bypass_panel(ui, state, panel_ctx);
}

fn render_parking_panel(
    ui: &mut egui::Ui,
    state: &ParkingPanelState,
    lang: Language,
    panel_ctx: &mut RouteToolPanelRenderContext<'_>,
) {
    generator_panel::render_parking_panel(ui, state, lang, panel_ctx);
}

fn render_field_boundary_panel(
    ui: &mut egui::Ui,
    state: &FieldBoundaryPanelState,
    panel_ctx: &mut RouteToolPanelRenderContext<'_>,
) {
    analysis_panel::render_field_boundary_panel(ui, state, panel_ctx);
}

fn render_field_path_panel(
    ui: &mut egui::Ui,
    state: &FieldPathPanelState,
    lang: Language,
    panel_ctx: &mut RouteToolPanelRenderContext<'_>,
) {
    analysis_panel::render_field_path_panel(ui, state, lang, panel_ctx);
}

fn render_route_offset_panel(
    ui: &mut egui::Ui,
    state: &RouteOffsetPanelState,
    panel_ctx: &mut RouteToolPanelRenderContext<'_>,
) {
    analysis_panel::render_route_offset_panel(ui, state, panel_ctx);
}

fn render_color_path_panel(
    ui: &mut egui::Ui,
    state: &ColorPathPanelState,
    panel_ctx: &mut RouteToolPanelRenderContext<'_>,
) {
    analysis_panel::render_color_path_panel(ui, state, panel_ctx);
}

fn render_segment_config(
    ui: &mut egui::Ui,
    state: &SegmentConfigPanelState,
    panel_ctx: &mut RouteToolPanelRenderContext<'_>,
    map_action: impl Fn(SegmentConfigPanelAction) -> RouteToolPanelAction,
) {
    ui.label(segment_length_kind_label(state.length_kind));
    if let Some(length_m) = state.length_m {
        ui.small(format!("Laenge: {:.1} m", length_m));
    }

    let mut max_segment_length = state.max_segment_length;
    let min_segment_length = state.max_segment_length_min;
    let max_segment_length_limit = state.max_segment_length_max;
    ui.horizontal(|ui| {
        ui.label("Max. Segmentlaenge:");
        let range = min_segment_length..=max_segment_length_limit;
        let response = ui.add(
            egui::DragValue::new(&mut max_segment_length)
                .range(range.clone())
                .speed(0.1)
                .suffix(" m"),
        );
        if response.changed()
            | apply_wheel_step_default_enabled(
                ui,
                &response,
                &mut max_segment_length,
                range,
                panel_ctx.wheel_enabled,
            )
        {
            push_action(
                panel_ctx.events,
                map_action(SegmentConfigPanelAction::SetMaxSegmentLength(
                    max_segment_length,
                )),
            );
        }
    });

    if let Some(node_count) = state.node_count {
        let mut node_count = node_count;
        ui.horizontal(|ui| {
            ui.label("Node-Anzahl:");
            let min = state.node_count_min.unwrap_or(2);
            let max = state.node_count_max.unwrap_or(node_count.max(2));
            let range = min..=max;
            let response = ui.add(
                egui::DragValue::new(&mut node_count)
                    .range(range.clone())
                    .speed(1.0),
            );
            if response.changed()
                | apply_wheel_step_usize(
                    ui,
                    &response,
                    &mut node_count,
                    range,
                    panel_ctx.wheel_enabled,
                )
            {
                push_action(
                    panel_ctx.events,
                    map_action(SegmentConfigPanelAction::SetNodeCount(node_count)),
                );
            }
        });
    }
}

fn render_segment_distance_only(
    ui: &mut egui::Ui,
    state: &SegmentConfigPanelState,
    panel_ctx: &mut RouteToolPanelRenderContext<'_>,
    map_action: impl Fn(f32) -> RouteToolPanelAction,
) {
    ui.label(segment_length_kind_label(state.length_kind));
    if let Some(length_m) = state.length_m {
        ui.small(format!("Laenge: {:.1} m", length_m));
    }
    let mut max_segment_length = state.max_segment_length;
    let min_segment_length = state.max_segment_length_min;
    let max_segment_length_limit = state.max_segment_length_max;
    ui.horizontal(|ui| {
        ui.label("Max. Segmentlaenge:");
        let range = min_segment_length..=max_segment_length_limit;
        let response = ui.add(
            egui::DragValue::new(&mut max_segment_length)
                .range(range.clone())
                .speed(0.1)
                .suffix(" m"),
        );
        if response.changed()
            | apply_wheel_step_default_enabled(
                ui,
                &response,
                &mut max_segment_length,
                range,
                panel_ctx.wheel_enabled,
            )
        {
            push_action(panel_ctx.events, map_action(max_segment_length));
        }
    });
}

fn render_tangent_selection(
    ui: &mut egui::Ui,
    label: &str,
    selection: &TangentSelectionState,
    panel_ctx: &mut RouteToolPanelRenderContext<'_>,
    map_action: impl Fn(TangentSource) -> RouteToolPanelAction,
) {
    let selected_text = tangent_selection_label(selection);
    ui.horizontal(|ui| {
        ui.label(label);
        ui.add_enabled_ui(selection.enabled, |ui| {
            egui::ComboBox::from_id_salt(("tangent_selection", label))
                .selected_text(selected_text)
                .show_ui(ui, |ui| {
                    if ui
                        .selectable_label(
                            selection.current == TangentSource::None,
                            tangent_none_reason_label(selection.none_reason),
                        )
                        .clicked()
                    {
                        push_action(panel_ctx.events, map_action(TangentSource::None));
                    }
                    for option in &selection.options {
                        if ui
                            .selectable_label(selection.current == option.source, &option.label)
                            .clicked()
                        {
                            push_action(panel_ctx.events, map_action(option.source));
                        }
                    }
                });
        });
    });
}

fn render_field_path_selection_summary(
    ui: &mut egui::Ui,
    summary: &FieldPathSelectionSummary,
    lang: Language,
) {
    ui.label(format!("── {} ──", t(lang, summary.title)));
    if summary.is_empty {
        let label = summary
            .empty_hint
            .map(|k| t(lang, k))
            .unwrap_or(summary.text.as_str());
        ui.colored_label(egui::Color32::GRAY, label);
    } else {
        ui.label(&summary.text);
    }
}

fn render_direction_selector(
    ui: &mut egui::Ui,
    current: ConnectionDirection,
    panel_ctx: &mut RouteToolPanelRenderContext<'_>,
    map_action: impl Fn(ConnectionDirection) -> RouteToolPanelAction,
) {
    ui.horizontal(|ui| {
        ui.label("Richtung:");
        let mut value = current;
        egui::ComboBox::from_id_salt("field_boundary_direction")
            .selected_text(direction_label(value))
            .show_ui(ui, |ui| {
                for choice in [
                    ConnectionDirection::Regular,
                    ConnectionDirection::Dual,
                    ConnectionDirection::Reverse,
                ] {
                    ui.selectable_value(&mut value, choice, direction_label(choice));
                }
            });
        if value != current {
            push_action(panel_ctx.events, map_action(value));
        }
    });
}

fn render_priority_selector(
    ui: &mut egui::Ui,
    current: ConnectionPriority,
    panel_ctx: &mut RouteToolPanelRenderContext<'_>,
    map_action: impl Fn(ConnectionPriority) -> RouteToolPanelAction,
) {
    ui.horizontal(|ui| {
        ui.label("Strassenart:");
        let mut value = current;
        egui::ComboBox::from_id_salt("field_boundary_priority")
            .selected_text(priority_label(value))
            .show_ui(ui, |ui| {
                for choice in [ConnectionPriority::Regular, ConnectionPriority::SubPriority] {
                    ui.selectable_value(&mut value, choice, priority_label(choice));
                }
            });
        if value != current {
            push_action(panel_ctx.events, map_action(value));
        }
    });
}

fn render_parking_side_selector(
    ui: &mut egui::Ui,
    label: &str,
    current: ParkingRampSideChoice,
    panel_ctx: &mut RouteToolPanelRenderContext<'_>,
    map_action: impl Fn(ParkingRampSideChoice) -> RouteToolPanelAction,
) {
    ui.horizontal(|ui| {
        ui.label(label);
        let mut value = current;
        egui::ComboBox::from_id_salt(("parking_side", label))
            .selected_text(parking_side_label(value))
            .show_ui(ui, |ui| {
                for choice in [ParkingRampSideChoice::Left, ParkingRampSideChoice::Right] {
                    ui.selectable_value(&mut value, choice, parking_side_label(choice));
                }
            });
        if value != current {
            push_action(panel_ctx.events, map_action(value));
        }
    });
}

fn render_parking_f32(
    panel_ctx: &mut RouteToolPanelRenderContext<'_>,
    props: DragF32Props<'_>,
    map_action: impl Fn(f32) -> RouteToolPanelAction,
) {
    render_drag_f32(
        panel_ctx,
        DragF32Props {
            speed: 0.1,
            ..props
        },
        map_action,
    );
}

struct DragF32Props<'a> {
    ui: &'a mut egui::Ui,
    label: &'a str,
    current: f32,
    range: std::ops::RangeInclusive<f32>,
    speed: f64,
    suffix: &'a str,
}

fn render_drag_f32(
    panel_ctx: &mut RouteToolPanelRenderContext<'_>,
    props: DragF32Props<'_>,
    map_action: impl Fn(f32) -> RouteToolPanelAction,
) {
    props.ui.horizontal(|ui| {
        ui.label(props.label);
        let mut value = props.current;
        let response = ui.add(
            egui::DragValue::new(&mut value)
                .range(props.range.clone())
                .speed(props.speed)
                .suffix(props.suffix),
        );
        if response.changed()
            | apply_wheel_step_default_enabled(
                ui,
                &response,
                &mut value,
                props.range,
                panel_ctx.wheel_enabled,
            )
        {
            push_action(panel_ctx.events, map_action(value));
        }
    });
}

struct DragUsizeProps<'a> {
    ui: &'a mut egui::Ui,
    label: &'a str,
    current: usize,
    range: std::ops::RangeInclusive<usize>,
    speed: f64,
}

fn render_drag_usize(
    panel_ctx: &mut RouteToolPanelRenderContext<'_>,
    props: DragUsizeProps<'_>,
    map_action: impl Fn(usize) -> RouteToolPanelAction,
) {
    props.ui.horizontal(|ui| {
        ui.label(props.label);
        let mut value = props.current;
        let response = ui.add(
            egui::DragValue::new(&mut value)
                .range(props.range.clone())
                .speed(props.speed),
        );
        if response.changed()
            | apply_wheel_step_usize(
                ui,
                &response,
                &mut value,
                props.range,
                panel_ctx.wheel_enabled,
            )
        {
            push_action(panel_ctx.events, map_action(value));
        }
    });
}

struct SliderF32Props<'a> {
    ui: &'a mut egui::Ui,
    label: &'a str,
    current: f32,
    range: std::ops::RangeInclusive<f32>,
    suffix: &'a str,
    enabled: bool,
}

fn render_slider_f32(
    panel_ctx: &mut RouteToolPanelRenderContext<'_>,
    props: SliderF32Props<'_>,
    map_action: impl Fn(f32) -> RouteToolPanelAction,
) {
    props.ui.horizontal(|ui| {
        ui.label(props.label);
        let mut value = props.current;
        let response = ui.add_enabled(
            props.enabled,
            egui::Slider::new(&mut value, props.range.clone()).suffix(props.suffix),
        );
        if response.changed()
            | apply_wheel_step_default_enabled(
                ui,
                &response,
                &mut value,
                props.range,
                panel_ctx.wheel_enabled && props.enabled,
            )
        {
            push_action(panel_ctx.events, map_action(value));
        }
    });
}

fn render_color_swatch(ui: &mut egui::Ui, color: [u8; 3], size: f32, rounding: f32) {
    let (rect, _) = ui.allocate_exact_size(egui::Vec2::splat(size), egui::Sense::hover());
    ui.painter().rect_filled(
        rect,
        rounding,
        egui::Color32::from_rgb(color[0], color[1], color[2]),
    );
}

fn push_action(events: &mut Vec<AppIntent>, action: RouteToolPanelAction) {
    push_panel_action(events, PanelAction::RouteTool(action));
}

fn push_panel_action(events: &mut Vec<AppIntent>, action: PanelAction) {
    events.push(panel_action_to_intent(action));
}

fn tangent_selection_label(selection: &TangentSelectionState) -> String {
    if selection.current == TangentSource::None {
        tangent_none_reason_label(selection.none_reason).to_owned()
    } else {
        selection
            .options
            .iter()
            .find(|option| option.source == selection.current)
            .map(|option| option.label.clone())
            .unwrap_or_else(|| tangent_none_reason_label(selection.none_reason).to_owned())
    }
}

fn tangent_none_reason_label(reason: TangentNoneReason) -> &'static str {
    match reason {
        TangentNoneReason::NoConnection => "Keine Verbindung",
        TangentNoneReason::NoTangent => "Keine Tangente",
        TangentNoneReason::UseDefault => "Standard",
    }
}

fn segment_length_kind_label(kind: SegmentLengthKind) -> &'static str {
    match kind {
        SegmentLengthKind::StraightLine => "Streckenlaenge",
        SegmentLengthKind::Curve => "Kurvenlaenge",
        SegmentLengthKind::CatmullRomSpline => "Spline-Laenge",
        SegmentLengthKind::SmoothRoute => "Routenlaenge",
    }
}

fn format_vec2(value: glam::Vec2) -> String {
    format!("({:.1}, {:.1})", value.x, value.y)
}

fn curve_degree_label(value: CurveDegreeChoice) -> &'static str {
    match value {
        CurveDegreeChoice::Quadratic => "Quadratisch",
        CurveDegreeChoice::Cubic => "Kubisch",
    }
}

fn field_path_mode_label(value: FieldPathModeChoice) -> &'static str {
    match value {
        FieldPathModeChoice::Fields => "Felder",
        FieldPathModeChoice::Boundaries => "Grenzen",
    }
}

fn existing_connection_mode_label(value: ExistingConnectionModeChoice) -> &'static str {
    match value {
        ExistingConnectionModeChoice::Never => "Nie",
        ExistingConnectionModeChoice::OpenEnds => "Nur offene Enden",
        ExistingConnectionModeChoice::OpenEndsAndJunctions => "Offene Enden + Kreuzungen",
    }
}

fn parking_side_label(value: ParkingRampSideChoice) -> &'static str {
    match value {
        ParkingRampSideChoice::Left => "Links",
        ParkingRampSideChoice::Right => "Rechts",
    }
}

fn direction_label(value: ConnectionDirection) -> &'static str {
    match value {
        ConnectionDirection::Regular => "Einbahnstrasse",
        ConnectionDirection::Dual => "Beidseitig",
        ConnectionDirection::Reverse => "Rueckwaerts",
    }
}

fn priority_label(value: ConnectionPriority) -> &'static str {
    match value {
        ConnectionPriority::Regular => "Normal",
        ConnectionPriority::SubPriority => "Nebenstrecke",
    }
}
