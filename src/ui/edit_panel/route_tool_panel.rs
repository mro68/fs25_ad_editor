use crate::app::tool_contract::TangentSource;
use crate::app::ui_contract::{
    BypassPanelAction, BypassPanelState, ColorPathPanelAction, ColorPathPanelPhase,
    ColorPathPanelState, CurveDegreeChoice, CurvePanelAction, CurvePanelState,
    CurveTangentsPanelState, ExistingConnectionModeChoice, FieldBoundaryPanelAction,
    FieldBoundaryPanelState, FieldPathModeChoice, FieldPathPanelAction, FieldPathPanelPhase,
    FieldPathPanelState, FieldPathSelectionSummary, ParkingPanelAction, ParkingPanelState,
    ParkingRampSideChoice, RouteOffsetPanelAction, RouteOffsetPanelState, RouteToolConfigState,
    RouteToolPanelAction, RouteToolPanelState, SegmentConfigPanelAction, SegmentConfigPanelState,
    SmoothCurvePanelAction, SmoothCurvePanelState, SplinePanelAction, SplinePanelState,
    StraightPanelAction, StraightPanelState, TangentSelectionState, BYPASS_BASE_SPACING_LIMITS,
    BYPASS_OFFSET_LIMITS, PARKING_BAY_LENGTH_LIMITS, PARKING_ENTRY_EXIT_T_LIMITS,
    PARKING_MAX_NODE_DISTANCE_LIMITS, PARKING_NUM_ROWS_LIMITS, PARKING_RAMP_LENGTH_LIMITS,
    PARKING_ROTATION_STEP_LIMITS, PARKING_ROW_SPACING_LIMITS, ROUTE_OFFSET_BASE_SPACING_LIMITS,
    ROUTE_OFFSET_DISTANCE_LIMITS, SMOOTH_CURVE_MAX_ANGLE_LIMITS, SMOOTH_CURVE_MIN_DISTANCE_LIMITS,
};
use crate::app::{AppIntent, ConnectionDirection, ConnectionPriority};
use crate::ui::common::{apply_wheel_step_default_enabled, apply_wheel_step_usize};
use crate::ui::properties::selectors::{
    render_direction_icon_selector, render_priority_icon_selector,
};

mod analysis_panel;
mod curve_panel;

/// Rendert das Route-Tool-Panel mit Tool-Konfiguration sowie Ausfuehren/Abbrechen.
///
/// Ein positiver `distance_wheel_step_m` aktiviert Mausrad-Anpassungen in den
/// numerischen Unterpanels. Die konkrete Scroll-Auswertung bleibt in
/// `ui::common`, damit Route-Tool- und Analysis-Widgets dieselbe Wheel-Logik
/// verwenden.
#[allow(clippy::too_many_arguments)]
pub(super) fn render_route_tool_panel(
    ctx: &egui::Context,
    route_tool: RouteToolPanelState,
    default_direction: ConnectionDirection,
    default_priority: ConnectionPriority,
    distance_wheel_step_m: f32,
    panel_pos: Option<egui::Pos2>,
    events: &mut Vec<AppIntent>,
) {
    let wheel_enabled = distance_wheel_step_m > 0.0;

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
            events.push(AppIntent::SetDefaultDirectionRequested {
                direction: selected_dir,
            });
        }

        ui.add_space(4.0);
        let mut selected_prio = default_priority;
        render_priority_icon_selector(ui, &mut selected_prio, "route_tool_floating");
        if selected_prio != default_priority {
            events.push(AppIntent::SetDefaultPriorityRequested {
                priority: selected_prio,
            });
        }

        ui.add_space(6.0);

        if let Some(config_state) = route_tool.config_state.as_ref() {
            render_route_tool_config(ui, config_state, wheel_enabled, events);
        } else {
            ui.small("Kein Route-Tool aktiv.");
        }

        ui.add_space(8.0);
        ui.horizontal(|ui| {
            if ui
                .add_enabled(route_tool.can_execute, egui::Button::new("✓ Ausfuehren"))
                .clicked()
            {
                events.push(AppIntent::RouteToolExecuteRequested);
            }
            if ui.button("✕ Abbrechen").clicked() {
                events.push(AppIntent::RouteToolCancelled);
            }
        });
    });
}

fn render_route_tool_config(
    ui: &mut egui::Ui,
    config_state: &RouteToolConfigState,
    wheel_enabled: bool,
    events: &mut Vec<AppIntent>,
) {
    match config_state {
        RouteToolConfigState::Straight(state) => {
            render_straight_panel(ui, state, wheel_enabled, events)
        }
        RouteToolConfigState::Curve(state) => render_curve_panel(ui, state, wheel_enabled, events),
        RouteToolConfigState::Spline(state) => {
            render_spline_panel(ui, state, wheel_enabled, events)
        }
        RouteToolConfigState::SmoothCurve(state) => {
            render_smooth_curve_panel(ui, state, wheel_enabled, events)
        }
        RouteToolConfigState::Bypass(state) => {
            render_bypass_panel(ui, state, wheel_enabled, events)
        }
        RouteToolConfigState::Parking(state) => {
            render_parking_panel(ui, state, wheel_enabled, events)
        }
        RouteToolConfigState::FieldBoundary(state) => {
            render_field_boundary_panel(ui, state, wheel_enabled, events)
        }
        RouteToolConfigState::FieldPath(state) => {
            render_field_path_panel(ui, state, wheel_enabled, events)
        }
        RouteToolConfigState::RouteOffset(state) => {
            render_route_offset_panel(ui, state, wheel_enabled, events)
        }
        RouteToolConfigState::ColorPath(state) => {
            render_color_path_panel(ui, state, wheel_enabled, events)
        }
    }
}

fn render_straight_panel(
    ui: &mut egui::Ui,
    state: &StraightPanelState,
    wheel_enabled: bool,
    events: &mut Vec<AppIntent>,
) {
    render_segment_config(ui, &state.segment, wheel_enabled, events, |action| {
        RouteToolPanelAction::Straight(StraightPanelAction::Segment(action))
    });
}

fn render_curve_panel(
    ui: &mut egui::Ui,
    state: &CurvePanelState,
    wheel_enabled: bool,
    events: &mut Vec<AppIntent>,
) {
    curve_panel::render_curve_panel(ui, state, wheel_enabled, events);
}

fn render_spline_panel(
    ui: &mut egui::Ui,
    state: &SplinePanelState,
    wheel_enabled: bool,
    events: &mut Vec<AppIntent>,
) {
    curve_panel::render_spline_panel(ui, state, wheel_enabled, events);
}

fn render_smooth_curve_panel(
    ui: &mut egui::Ui,
    state: &SmoothCurvePanelState,
    wheel_enabled: bool,
    events: &mut Vec<AppIntent>,
) {
    render_drag_f32(
        ui,
        "Max. Winkel:",
        state.max_angle_deg,
        SMOOTH_CURVE_MAX_ANGLE_LIMITS.range(),
        0.1,
        "°",
        wheel_enabled,
        events,
        |value| RouteToolPanelAction::SmoothCurve(SmoothCurvePanelAction::SetMaxAngleDeg(value)),
    );

    render_segment_distance_only(ui, &state.segment, wheel_enabled, events, |value| {
        RouteToolPanelAction::SmoothCurve(SmoothCurvePanelAction::SetMaxSegmentLength(value))
    });

    render_drag_f32(
        ui,
        "Min. Distanz:",
        state.min_distance,
        SMOOTH_CURVE_MIN_DISTANCE_LIMITS.range(),
        0.1,
        " m",
        wheel_enabled,
        events,
        |value| RouteToolPanelAction::SmoothCurve(SmoothCurvePanelAction::SetMinDistance(value)),
    );

    if let Some(approach) = state.approach_steerer.as_ref() {
        ui.horizontal(|ui| {
            ui.label(format!(
                "Approach: {}{}",
                format_vec2(approach.position),
                if approach.is_manual { " (manuell)" } else { "" }
            ));
            if approach.is_manual && ui.button("Reset").clicked() {
                push_action(
                    events,
                    RouteToolPanelAction::SmoothCurve(SmoothCurvePanelAction::ResetApproachSteerer),
                );
            }
        });
    }

    if let Some(departure) = state.departure_steerer.as_ref() {
        ui.horizontal(|ui| {
            ui.label(format!(
                "Departure: {}{}",
                format_vec2(departure.position),
                if departure.is_manual {
                    " (manuell)"
                } else {
                    ""
                }
            ));
            if departure.is_manual && ui.button("Reset").clicked() {
                push_action(
                    events,
                    RouteToolPanelAction::SmoothCurve(
                        SmoothCurvePanelAction::ResetDepartureSteerer,
                    ),
                );
            }
        });
    }

    if !state.control_nodes.is_empty() {
        ui.separator();
        ui.label("Kontrollpunkte:");
        for (index, position) in state.control_nodes.iter().enumerate() {
            ui.horizontal(|ui| {
                ui.label(format!("#{} {}", index + 1, format_vec2(*position)));
                if ui.button("Entfernen").clicked() {
                    push_action(
                        events,
                        RouteToolPanelAction::SmoothCurve(
                            SmoothCurvePanelAction::RemoveControlNode { index },
                        ),
                    );
                }
            });
        }
    }

    if let Some(preview_node_count) = state.preview_node_count {
        ui.separator();
        ui.label(format!("Preview-Nodes: {preview_node_count}"));
    }
}

fn render_bypass_panel(
    ui: &mut egui::Ui,
    state: &BypassPanelState,
    wheel_enabled: bool,
    events: &mut Vec<AppIntent>,
) {
    if let Some(message) = state.empty_message.as_deref() {
        ui.colored_label(egui::Color32::GRAY, message);
        return;
    }

    ui.label(format!("Kette: {} Nodes", state.chain_node_count));
    if let Some(new_node_count) = state.new_node_count {
        ui.label(format!("Neue Nodes: {new_node_count}"));
    }
    if let Some(transition_length_m) = state.transition_length_m {
        ui.label(format!("Uebergang: {:.1} m", transition_length_m));
    }
    ui.label(format!("Seite: {}", state.side_label));

    render_drag_f32(
        ui,
        "Versatz:",
        state.offset,
        BYPASS_OFFSET_LIMITS.range(),
        0.1,
        " m",
        wheel_enabled,
        events,
        |value| RouteToolPanelAction::Bypass(BypassPanelAction::SetOffset(value)),
    );

    render_drag_f32(
        ui,
        "Basisabstand:",
        state.base_spacing,
        BYPASS_BASE_SPACING_LIMITS.range(),
        0.1,
        " m",
        wheel_enabled,
        events,
        |value| RouteToolPanelAction::Bypass(BypassPanelAction::SetBaseSpacing(value)),
    );
}

fn render_parking_panel(
    ui: &mut egui::Ui,
    state: &ParkingPanelState,
    wheel_enabled: bool,
    events: &mut Vec<AppIntent>,
) {
    render_drag_usize(
        ui,
        "Reihen:",
        state.num_rows,
        PARKING_NUM_ROWS_LIMITS.range(),
        1.0,
        wheel_enabled,
        events,
        |value| RouteToolPanelAction::Parking(ParkingPanelAction::SetNumRows(value)),
    );

    render_parking_f32(
        ui,
        "Reihenabstand:",
        state.row_spacing,
        PARKING_ROW_SPACING_LIMITS.range(),
        wheel_enabled,
        " m",
        events,
        |value| RouteToolPanelAction::Parking(ParkingPanelAction::SetRowSpacing(value)),
    );
    render_parking_f32(
        ui,
        "Reihenlaenge:",
        state.bay_length,
        PARKING_BAY_LENGTH_LIMITS.range(),
        wheel_enabled,
        " m",
        events,
        |value| RouteToolPanelAction::Parking(ParkingPanelAction::SetBayLength(value)),
    );
    render_parking_f32(
        ui,
        "Max. Node-Abstand:",
        state.max_node_distance,
        PARKING_MAX_NODE_DISTANCE_LIMITS.range(),
        wheel_enabled,
        " m",
        events,
        |value| RouteToolPanelAction::Parking(ParkingPanelAction::SetMaxNodeDistance(value)),
    );
    render_parking_f32(
        ui,
        "Einfahrt t:",
        state.entry_t,
        PARKING_ENTRY_EXIT_T_LIMITS.range(),
        wheel_enabled,
        "",
        events,
        |value| RouteToolPanelAction::Parking(ParkingPanelAction::SetEntryT(value)),
    );
    render_parking_f32(
        ui,
        "Ausfahrt t:",
        state.exit_t,
        PARKING_ENTRY_EXIT_T_LIMITS.range(),
        wheel_enabled,
        "",
        events,
        |value| RouteToolPanelAction::Parking(ParkingPanelAction::SetExitT(value)),
    );
    render_parking_f32(
        ui,
        "Rampenlaenge:",
        state.ramp_length,
        PARKING_RAMP_LENGTH_LIMITS.range(),
        wheel_enabled,
        " m",
        events,
        |value| RouteToolPanelAction::Parking(ParkingPanelAction::SetRampLength(value)),
    );

    render_parking_side_selector(ui, "Einfahrt:", state.entry_side, events, |value| {
        RouteToolPanelAction::Parking(ParkingPanelAction::SetEntrySide(value))
    });
    render_parking_side_selector(ui, "Ausfahrt:", state.exit_side, events, |value| {
        RouteToolPanelAction::Parking(ParkingPanelAction::SetExitSide(value))
    });

    let mut marker_group = state.marker_group.clone();
    ui.horizontal(|ui| {
        ui.label("Marker-Gruppe:");
        if ui.text_edit_singleline(&mut marker_group).changed() {
            push_action(
                events,
                RouteToolPanelAction::Parking(ParkingPanelAction::SetMarkerGroup(marker_group)),
            );
        }
    });

    render_parking_f32(
        ui,
        "Drehschritt:",
        state.rotation_step_deg,
        PARKING_ROTATION_STEP_LIMITS.range(),
        wheel_enabled,
        "°",
        events,
        |value| RouteToolPanelAction::Parking(ParkingPanelAction::SetRotationStepDeg(value)),
    );

    if let Some(angle_deg) = state.angle_deg {
        ui.label(format!("Winkel: {:.1}°", angle_deg));
    }
    if let Some(hint_text) = state.hint_text.as_deref() {
        ui.small(hint_text);
    }
}

fn render_field_boundary_panel(
    ui: &mut egui::Ui,
    state: &FieldBoundaryPanelState,
    wheel_enabled: bool,
    events: &mut Vec<AppIntent>,
) {
    analysis_panel::render_field_boundary_panel(ui, state, wheel_enabled, events);
}

fn render_field_path_panel(
    ui: &mut egui::Ui,
    state: &FieldPathPanelState,
    wheel_enabled: bool,
    events: &mut Vec<AppIntent>,
) {
    analysis_panel::render_field_path_panel(ui, state, wheel_enabled, events);
}

fn render_route_offset_panel(
    ui: &mut egui::Ui,
    state: &RouteOffsetPanelState,
    wheel_enabled: bool,
    events: &mut Vec<AppIntent>,
) {
    analysis_panel::render_route_offset_panel(ui, state, wheel_enabled, events);
}

fn render_color_path_panel(
    ui: &mut egui::Ui,
    state: &ColorPathPanelState,
    wheel_enabled: bool,
    events: &mut Vec<AppIntent>,
) {
    analysis_panel::render_color_path_panel(ui, state, wheel_enabled, events);
}

fn render_segment_config(
    ui: &mut egui::Ui,
    state: &SegmentConfigPanelState,
    wheel_enabled: bool,
    events: &mut Vec<AppIntent>,
    map_action: impl Fn(SegmentConfigPanelAction) -> RouteToolPanelAction,
) {
    ui.label(&state.length_label);
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
                wheel_enabled,
            )
        {
            push_action(
                events,
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
                | apply_wheel_step_usize(ui, &response, &mut node_count, range, wheel_enabled)
            {
                push_action(
                    events,
                    map_action(SegmentConfigPanelAction::SetNodeCount(node_count)),
                );
            }
        });
    }
}

fn render_segment_distance_only(
    ui: &mut egui::Ui,
    state: &SegmentConfigPanelState,
    wheel_enabled: bool,
    events: &mut Vec<AppIntent>,
    map_action: impl Fn(f32) -> RouteToolPanelAction,
) {
    ui.label(&state.length_label);
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
                wheel_enabled,
            )
        {
            push_action(events, map_action(max_segment_length));
        }
    });
}

fn render_tangent_selection(
    ui: &mut egui::Ui,
    selection: &TangentSelectionState,
    events: &mut Vec<AppIntent>,
    map_action: impl Fn(TangentSource) -> RouteToolPanelAction,
) {
    let selected_text = tangent_selection_label(selection);
    ui.horizontal(|ui| {
        ui.label(&selection.label);
        ui.add_enabled_ui(selection.enabled, |ui| {
            egui::ComboBox::from_id_salt(("tangent_selection", selection.label.as_str()))
                .selected_text(selected_text)
                .show_ui(ui, |ui| {
                    if ui
                        .selectable_label(
                            selection.current == TangentSource::None,
                            &selection.none_label,
                        )
                        .clicked()
                    {
                        push_action(events, map_action(TangentSource::None));
                    }
                    for option in &selection.options {
                        if ui
                            .selectable_label(selection.current == option.source, &option.label)
                            .clicked()
                        {
                            push_action(events, map_action(option.source));
                        }
                    }
                });
        });
    });
}

fn render_field_path_selection_summary(ui: &mut egui::Ui, summary: &FieldPathSelectionSummary) {
    ui.label(format!("── {} ──", summary.title));
    if summary.is_empty {
        ui.colored_label(egui::Color32::GRAY, &summary.text);
    } else {
        ui.label(&summary.text);
    }
}

fn render_direction_selector(
    ui: &mut egui::Ui,
    current: ConnectionDirection,
    events: &mut Vec<AppIntent>,
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
            push_action(events, map_action(value));
        }
    });
}

fn render_priority_selector(
    ui: &mut egui::Ui,
    current: ConnectionPriority,
    events: &mut Vec<AppIntent>,
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
            push_action(events, map_action(value));
        }
    });
}

fn render_parking_side_selector(
    ui: &mut egui::Ui,
    label: &str,
    current: ParkingRampSideChoice,
    events: &mut Vec<AppIntent>,
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
            push_action(events, map_action(value));
        }
    });
}

#[allow(clippy::too_many_arguments)]
fn render_parking_f32(
    ui: &mut egui::Ui,
    label: &str,
    current: f32,
    range: std::ops::RangeInclusive<f32>,
    wheel_enabled: bool,
    suffix: &str,
    events: &mut Vec<AppIntent>,
    map_action: impl Fn(f32) -> RouteToolPanelAction,
) {
    render_drag_f32(
        ui,
        label,
        current,
        range,
        0.1,
        suffix,
        wheel_enabled,
        events,
        map_action,
    );
}

#[allow(clippy::too_many_arguments)]
fn render_drag_f32(
    ui: &mut egui::Ui,
    label: &str,
    current: f32,
    range: std::ops::RangeInclusive<f32>,
    speed: f64,
    suffix: &str,
    wheel_enabled: bool,
    events: &mut Vec<AppIntent>,
    map_action: impl Fn(f32) -> RouteToolPanelAction,
) {
    ui.horizontal(|ui| {
        ui.label(label);
        let mut value = current;
        let response = ui.add(
            egui::DragValue::new(&mut value)
                .range(range.clone())
                .speed(speed)
                .suffix(suffix),
        );
        if response.changed()
            | apply_wheel_step_default_enabled(ui, &response, &mut value, range, wheel_enabled)
        {
            push_action(events, map_action(value));
        }
    });
}

#[allow(clippy::too_many_arguments)]
fn render_drag_usize(
    ui: &mut egui::Ui,
    label: &str,
    current: usize,
    range: std::ops::RangeInclusive<usize>,
    speed: f64,
    wheel_enabled: bool,
    events: &mut Vec<AppIntent>,
    map_action: impl Fn(usize) -> RouteToolPanelAction,
) {
    ui.horizontal(|ui| {
        ui.label(label);
        let mut value = current;
        let response = ui.add(
            egui::DragValue::new(&mut value)
                .range(range.clone())
                .speed(speed),
        );
        if response.changed()
            | apply_wheel_step_usize(ui, &response, &mut value, range, wheel_enabled)
        {
            push_action(events, map_action(value));
        }
    });
}

#[allow(clippy::too_many_arguments)]
fn render_slider_f32(
    ui: &mut egui::Ui,
    label: &str,
    current: f32,
    range: std::ops::RangeInclusive<f32>,
    suffix: &str,
    enabled: bool,
    wheel_enabled: bool,
    events: &mut Vec<AppIntent>,
    map_action: impl Fn(f32) -> RouteToolPanelAction,
) {
    ui.horizontal(|ui| {
        ui.label(label);
        let mut value = current;
        let response = ui.add_enabled(
            enabled,
            egui::Slider::new(&mut value, range.clone()).suffix(suffix),
        );
        if response.changed()
            | apply_wheel_step_default_enabled(
                ui,
                &response,
                &mut value,
                range,
                wheel_enabled && enabled,
            )
        {
            push_action(events, map_action(value));
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
    events.push(AppIntent::RouteToolPanelActionRequested { action });
}

fn tangent_selection_label(selection: &TangentSelectionState) -> String {
    if selection.current == TangentSource::None {
        selection.none_label.clone()
    } else {
        selection
            .options
            .iter()
            .find(|option| option.source == selection.current)
            .map(|option| option.label.clone())
            .unwrap_or_else(|| selection.none_label.clone())
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
