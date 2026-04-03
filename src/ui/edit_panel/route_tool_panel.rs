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
use crate::ui::properties::selectors::{
    render_direction_icon_selector, render_priority_icon_selector,
};

/// Route-Tool-Panel: Tool-Konfiguration plus Ausfuehren/Abbrechen.
#[allow(clippy::too_many_arguments)]
pub(super) fn render_route_tool_panel(
    ctx: &egui::Context,
    route_tool: RouteToolPanelState,
    default_direction: ConnectionDirection,
    default_priority: ConnectionPriority,
    _distance_wheel_step_m: f32,
    panel_pos: Option<egui::Pos2>,
    events: &mut Vec<AppIntent>,
) {
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
            render_route_tool_config(ui, config_state, events);
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
    events: &mut Vec<AppIntent>,
) {
    match config_state {
        RouteToolConfigState::Straight(state) => render_straight_panel(ui, state, events),
        RouteToolConfigState::Curve(state) => render_curve_panel(ui, state, events),
        RouteToolConfigState::Spline(state) => render_spline_panel(ui, state, events),
        RouteToolConfigState::SmoothCurve(state) => render_smooth_curve_panel(ui, state, events),
        RouteToolConfigState::Bypass(state) => render_bypass_panel(ui, state, events),
        RouteToolConfigState::Parking(state) => render_parking_panel(ui, state, events),
        RouteToolConfigState::FieldBoundary(state) => {
            render_field_boundary_panel(ui, state, events)
        }
        RouteToolConfigState::FieldPath(state) => render_field_path_panel(ui, state, events),
        RouteToolConfigState::RouteOffset(state) => render_route_offset_panel(ui, state, events),
        RouteToolConfigState::ColorPath(state) => render_color_path_panel(ui, state, events),
    }
}

fn render_straight_panel(
    ui: &mut egui::Ui,
    state: &StraightPanelState,
    events: &mut Vec<AppIntent>,
) {
    render_segment_config(ui, &state.segment, events, |action| {
        RouteToolPanelAction::Straight(StraightPanelAction::Segment(action))
    });
}

fn render_curve_panel(ui: &mut egui::Ui, state: &CurvePanelState, events: &mut Vec<AppIntent>) {
    ui.horizontal(|ui| {
        ui.label("Grad:");
        let mut degree = state.degree;
        egui::ComboBox::from_id_salt("curve_degree")
            .selected_text(curve_degree_label(degree))
            .show_ui(ui, |ui| {
                ui.selectable_value(
                    &mut degree,
                    CurveDegreeChoice::Quadratic,
                    curve_degree_label(CurveDegreeChoice::Quadratic),
                );
                ui.selectable_value(
                    &mut degree,
                    CurveDegreeChoice::Cubic,
                    curve_degree_label(CurveDegreeChoice::Cubic),
                );
            });
        if degree != state.degree {
            push_action(
                events,
                RouteToolPanelAction::Curve(CurvePanelAction::SetDegree(degree)),
            );
        }
    });

    if let Some(tangents) = state.tangents.as_ref() {
        ui.separator();
        render_curve_tangents(ui, tangents, events);
    }

    ui.separator();
    render_segment_config(ui, &state.segment, events, |action| {
        RouteToolPanelAction::Curve(CurvePanelAction::Segment(action))
    });
}

fn render_curve_tangents(
    ui: &mut egui::Ui,
    state: &CurveTangentsPanelState,
    events: &mut Vec<AppIntent>,
) {
    if let Some(help_text) = state.help_text.as_deref() {
        ui.small(help_text);
    }

    render_tangent_selection(ui, &state.start, events, |value| {
        RouteToolPanelAction::Curve(CurvePanelAction::SetTangentStart(value))
    });
    render_tangent_selection(ui, &state.end, events, |value| {
        RouteToolPanelAction::Curve(CurvePanelAction::SetTangentEnd(value))
    });
}

fn render_spline_panel(ui: &mut egui::Ui, state: &SplinePanelState, events: &mut Vec<AppIntent>) {
    if let Some(control_point_count) = state.control_point_count {
        ui.label(format!("Kontrollpunkte: {control_point_count}"));
    }

    if let Some(start_tangent) = state.start_tangent.as_ref() {
        ui.separator();
        render_tangent_selection(ui, start_tangent, events, |value| {
            RouteToolPanelAction::Spline(SplinePanelAction::SetTangentStart(value))
        });
    }

    if let Some(end_tangent) = state.end_tangent.as_ref() {
        render_tangent_selection(ui, end_tangent, events, |value| {
            RouteToolPanelAction::Spline(SplinePanelAction::SetTangentEnd(value))
        });
    }

    ui.separator();
    render_segment_config(ui, &state.segment, events, |action| {
        RouteToolPanelAction::Spline(SplinePanelAction::Segment(action))
    });
}

fn render_smooth_curve_panel(
    ui: &mut egui::Ui,
    state: &SmoothCurvePanelState,
    events: &mut Vec<AppIntent>,
) {
    let mut max_angle_deg = state.max_angle_deg;
    ui.horizontal(|ui| {
        ui.label("Max. Winkel:");
        if ui
            .add(
                egui::DragValue::new(&mut max_angle_deg)
                    .range(SMOOTH_CURVE_MAX_ANGLE_LIMITS.range())
                    .speed(1.0)
                    .suffix("°"),
            )
            .changed()
        {
            push_action(
                events,
                RouteToolPanelAction::SmoothCurve(SmoothCurvePanelAction::SetMaxAngleDeg(
                    max_angle_deg,
                )),
            );
        }
    });

    render_segment_distance_only(ui, &state.segment, events, |value| {
        RouteToolPanelAction::SmoothCurve(SmoothCurvePanelAction::SetMaxSegmentLength(value))
    });

    let mut min_distance = state.min_distance;
    ui.horizontal(|ui| {
        ui.label("Min. Distanz:");
        if ui
            .add(
                egui::DragValue::new(&mut min_distance)
                    .range(SMOOTH_CURVE_MIN_DISTANCE_LIMITS.range())
                    .speed(0.25)
                    .suffix(" m"),
            )
            .changed()
        {
            push_action(
                events,
                RouteToolPanelAction::SmoothCurve(SmoothCurvePanelAction::SetMinDistance(
                    min_distance,
                )),
            );
        }
    });

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

fn render_bypass_panel(ui: &mut egui::Ui, state: &BypassPanelState, events: &mut Vec<AppIntent>) {
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

    let mut offset = state.offset;
    ui.horizontal(|ui| {
        ui.label("Versatz:");
        if ui
            .add(
                egui::DragValue::new(&mut offset)
                    .range(BYPASS_OFFSET_LIMITS.range())
                    .speed(0.25)
                    .suffix(" m"),
            )
            .changed()
        {
            push_action(
                events,
                RouteToolPanelAction::Bypass(BypassPanelAction::SetOffset(offset)),
            );
        }
    });

    let mut base_spacing = state.base_spacing;
    ui.horizontal(|ui| {
        ui.label("Basisabstand:");
        if ui
            .add(
                egui::DragValue::new(&mut base_spacing)
                    .range(BYPASS_BASE_SPACING_LIMITS.range())
                    .speed(0.25)
                    .suffix(" m"),
            )
            .changed()
        {
            push_action(
                events,
                RouteToolPanelAction::Bypass(BypassPanelAction::SetBaseSpacing(base_spacing)),
            );
        }
    });
}

fn render_parking_panel(ui: &mut egui::Ui, state: &ParkingPanelState, events: &mut Vec<AppIntent>) {
    let mut num_rows = state.num_rows;
    ui.horizontal(|ui| {
        ui.label("Reihen:");
        if ui
            .add(
                egui::DragValue::new(&mut num_rows)
                    .range(PARKING_NUM_ROWS_LIMITS.range())
                    .speed(1.0),
            )
            .changed()
        {
            push_action(
                events,
                RouteToolPanelAction::Parking(ParkingPanelAction::SetNumRows(num_rows)),
            );
        }
    });

    render_parking_f32(
        ui,
        "Reihenabstand:",
        state.row_spacing,
        PARKING_ROW_SPACING_LIMITS.range(),
        " m",
        events,
        |value| RouteToolPanelAction::Parking(ParkingPanelAction::SetRowSpacing(value)),
    );
    render_parking_f32(
        ui,
        "Reihenlaenge:",
        state.bay_length,
        PARKING_BAY_LENGTH_LIMITS.range(),
        " m",
        events,
        |value| RouteToolPanelAction::Parking(ParkingPanelAction::SetBayLength(value)),
    );
    render_parking_f32(
        ui,
        "Max. Node-Abstand:",
        state.max_node_distance,
        PARKING_MAX_NODE_DISTANCE_LIMITS.range(),
        " m",
        events,
        |value| RouteToolPanelAction::Parking(ParkingPanelAction::SetMaxNodeDistance(value)),
    );
    render_parking_f32(
        ui,
        "Einfahrt t:",
        state.entry_t,
        PARKING_ENTRY_EXIT_T_LIMITS.range(),
        "",
        events,
        |value| RouteToolPanelAction::Parking(ParkingPanelAction::SetEntryT(value)),
    );
    render_parking_f32(
        ui,
        "Ausfahrt t:",
        state.exit_t,
        PARKING_ENTRY_EXIT_T_LIMITS.range(),
        "",
        events,
        |value| RouteToolPanelAction::Parking(ParkingPanelAction::SetExitT(value)),
    );
    render_parking_f32(
        ui,
        "Rampenlaenge:",
        state.ramp_length,
        PARKING_RAMP_LENGTH_LIMITS.range(),
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
    events: &mut Vec<AppIntent>,
) {
    if let Some(field_id) = state.selected_field_id {
        ui.label(format!("Feld #{field_id}"));
    } else if let Some(text) = state.empty_selection_text.as_deref() {
        ui.colored_label(egui::Color32::GRAY, text);
    }

    ui.separator();
    render_drag_f32(
        ui,
        "Node-Abstand:",
        state.node_spacing,
        1.0..=50.0,
        " m",
        events,
        |value| {
            RouteToolPanelAction::FieldBoundary(FieldBoundaryPanelAction::SetNodeSpacing(value))
        },
    );
    render_drag_f32(
        ui,
        "Versatz:",
        state.offset,
        -20.0..=20.0,
        " m",
        events,
        |value| RouteToolPanelAction::FieldBoundary(FieldBoundaryPanelAction::SetOffset(value)),
    );
    render_drag_f32(
        ui,
        "Begradigen:",
        state.straighten_tolerance,
        0.0..=10.0,
        " m",
        events,
        |value| {
            RouteToolPanelAction::FieldBoundary(FieldBoundaryPanelAction::SetStraightenTolerance(
                value,
            ))
        },
    );

    let mut corner_detection_enabled = state.corner_detection_enabled;
    if ui
        .checkbox(&mut corner_detection_enabled, "Ecken erkennen")
        .changed()
    {
        push_action(
            events,
            RouteToolPanelAction::FieldBoundary(
                FieldBoundaryPanelAction::SetCornerDetectionEnabled(corner_detection_enabled),
            ),
        );
    }

    if state.corner_detection_enabled {
        render_drag_f32(
            ui,
            "Winkel-Schwelle:",
            state.corner_angle_threshold_deg,
            10.0..=170.0,
            "°",
            events,
            |value| {
                RouteToolPanelAction::FieldBoundary(
                    FieldBoundaryPanelAction::SetCornerAngleThresholdDeg(value),
                )
            },
        );

        let mut corner_rounding_enabled = state.corner_rounding_enabled;
        if ui
            .checkbox(&mut corner_rounding_enabled, "Ecken verrunden")
            .changed()
        {
            push_action(
                events,
                RouteToolPanelAction::FieldBoundary(
                    FieldBoundaryPanelAction::SetCornerRoundingEnabled(corner_rounding_enabled),
                ),
            );
        }

        if state.corner_rounding_enabled {
            render_drag_f32(
                ui,
                "Radius:",
                state.corner_rounding_radius,
                1.0..=50.0,
                " m",
                events,
                |value| {
                    RouteToolPanelAction::FieldBoundary(
                        FieldBoundaryPanelAction::SetCornerRoundingRadius(value),
                    )
                },
            );
            render_drag_f32(
                ui,
                "Max. Winkelabw.:",
                state.corner_rounding_max_angle_deg,
                1.0..=45.0,
                "°",
                events,
                |value| {
                    RouteToolPanelAction::FieldBoundary(
                        FieldBoundaryPanelAction::SetCornerRoundingMaxAngleDeg(value),
                    )
                },
            );
        }
    }

    render_direction_selector(ui, state.direction, events, |value| {
        RouteToolPanelAction::FieldBoundary(FieldBoundaryPanelAction::SetDirection(value))
    });
    render_priority_selector(ui, state.priority, events, |value| {
        RouteToolPanelAction::FieldBoundary(FieldBoundaryPanelAction::SetPriority(value))
    });

    if let Some(hint_text) = state.hint_text.as_deref() {
        ui.small(hint_text);
    }
}

fn render_field_path_panel(
    ui: &mut egui::Ui,
    state: &FieldPathPanelState,
    events: &mut Vec<AppIntent>,
) {
    ui.horizontal(|ui| {
        ui.label("Modus:");
        let mut mode = state.mode;
        egui::ComboBox::from_id_salt("field_path_mode")
            .selected_text(field_path_mode_label(mode))
            .show_ui(ui, |ui| {
                ui.selectable_value(
                    &mut mode,
                    FieldPathModeChoice::Fields,
                    field_path_mode_label(FieldPathModeChoice::Fields),
                );
                ui.selectable_value(
                    &mut mode,
                    FieldPathModeChoice::Boundaries,
                    field_path_mode_label(FieldPathModeChoice::Boundaries),
                );
            });
        if mode != state.mode {
            push_action(
                events,
                RouteToolPanelAction::FieldPath(FieldPathPanelAction::SetMode(mode)),
            );
        }
    });

    ui.separator();
    render_field_path_selection_summary(ui, &state.side1);

    if let Some(side2) = state.side2.as_ref() {
        ui.separator();
        render_field_path_selection_summary(ui, side2);
    }

    ui.separator();
    match state.phase {
        FieldPathPanelPhase::Idle => {
            if ui.button("Starten →").clicked() {
                push_action(
                    events,
                    RouteToolPanelAction::FieldPath(FieldPathPanelAction::Start),
                );
            }
        }
        FieldPathPanelPhase::SelectingSide1 => {
            if ui
                .add_enabled(state.can_advance_to_side2, egui::Button::new("Seite 2 →"))
                .clicked()
            {
                push_action(
                    events,
                    RouteToolPanelAction::FieldPath(FieldPathPanelAction::AdvanceToSide2),
                );
            }
        }
        FieldPathPanelPhase::SelectingSide2 => {
            if ui
                .add_enabled(state.can_compute, egui::Button::new("Berechnen"))
                .clicked()
            {
                push_action(
                    events,
                    RouteToolPanelAction::FieldPath(FieldPathPanelAction::Compute),
                );
            }
            if ui.button("← Zurueck").clicked() {
                push_action(
                    events,
                    RouteToolPanelAction::FieldPath(FieldPathPanelAction::BackToSide1),
                );
            }
        }
        FieldPathPanelPhase::Preview => {
            if let Some(preview_message) = state.preview_message.as_deref() {
                ui.label(preview_message);
            }
            if ui.button("← Seite 2 neu waehlen").clicked() {
                push_action(
                    events,
                    RouteToolPanelAction::FieldPath(FieldPathPanelAction::BackToSide2),
                );
            }
        }
    }

    ui.separator();
    ui.label("Einstellungen:");
    render_drag_f32(
        ui,
        "Knotenabstand:",
        state.node_spacing,
        1.0..=50.0,
        " m",
        events,
        |value| RouteToolPanelAction::FieldPath(FieldPathPanelAction::SetNodeSpacing(value)),
    );
    render_drag_f32(
        ui,
        "Vereinfachung:",
        state.simplify_tolerance,
        0.0..=20.0,
        " m",
        events,
        |value| RouteToolPanelAction::FieldPath(FieldPathPanelAction::SetSimplifyTolerance(value)),
    );

    let mut connect_to_existing = state.connect_to_existing;
    if ui
        .checkbox(&mut connect_to_existing, "An bestehende Nodes anschl.")
        .changed()
    {
        push_action(
            events,
            RouteToolPanelAction::FieldPath(FieldPathPanelAction::SetConnectToExisting(
                connect_to_existing,
            )),
        );
    }

    if state.phase != FieldPathPanelPhase::Idle {
        ui.separator();
        if ui.button("Zuruecksetzen").clicked() {
            push_action(
                events,
                RouteToolPanelAction::FieldPath(FieldPathPanelAction::Reset),
            );
        }
    }
}

fn render_route_offset_panel(
    ui: &mut egui::Ui,
    state: &RouteOffsetPanelState,
    events: &mut Vec<AppIntent>,
) {
    if let Some(message) = state.empty_message.as_deref() {
        ui.colored_label(egui::Color32::GRAY, message);
        return;
    }

    ui.label(format!("Kette: {} Nodes", state.chain_node_count));

    let mut left_enabled = state.left_enabled;
    if ui
        .checkbox(&mut left_enabled, "Linken Versatz erzeugen")
        .changed()
    {
        push_action(
            events,
            RouteToolPanelAction::RouteOffset(RouteOffsetPanelAction::SetLeftEnabled(left_enabled)),
        );
    }
    render_drag_f32(
        ui,
        "Links-Abstand:",
        state.left_distance,
        ROUTE_OFFSET_DISTANCE_LIMITS.range(),
        " m",
        events,
        |value| RouteToolPanelAction::RouteOffset(RouteOffsetPanelAction::SetLeftDistance(value)),
    );

    let mut right_enabled = state.right_enabled;
    if ui
        .checkbox(&mut right_enabled, "Rechten Versatz erzeugen")
        .changed()
    {
        push_action(
            events,
            RouteToolPanelAction::RouteOffset(RouteOffsetPanelAction::SetRightEnabled(
                right_enabled,
            )),
        );
    }
    render_drag_f32(
        ui,
        "Rechts-Abstand:",
        state.right_distance,
        ROUTE_OFFSET_DISTANCE_LIMITS.range(),
        " m",
        events,
        |value| RouteToolPanelAction::RouteOffset(RouteOffsetPanelAction::SetRightDistance(value)),
    );
    render_drag_f32(
        ui,
        "Basisabstand:",
        state.base_spacing,
        ROUTE_OFFSET_BASE_SPACING_LIMITS.range(),
        " m",
        events,
        |value| RouteToolPanelAction::RouteOffset(RouteOffsetPanelAction::SetBaseSpacing(value)),
    );

    let mut keep_original = state.keep_original;
    if ui
        .checkbox(&mut keep_original, "Original behalten")
        .changed()
    {
        push_action(
            events,
            RouteToolPanelAction::RouteOffset(RouteOffsetPanelAction::SetKeepOriginal(
                keep_original,
            )),
        );
    }
}

fn render_color_path_panel(
    ui: &mut egui::Ui,
    state: &ColorPathPanelState,
    events: &mut Vec<AppIntent>,
) {
    let status = match state.phase {
        ColorPathPanelPhase::Idle => "Alt+Lasso fuer Farbsample",
        ColorPathPanelPhase::Sampling => "Berechnen fuer Wegenetz",
        ColorPathPanelPhase::Preview => "Ausfuehren uebernehmen, Reset setzt zurueck",
    };
    ui.colored_label(egui::Color32::LIGHT_BLUE, status);
    ui.separator();

    match state.phase {
        ColorPathPanelPhase::Idle => {
            if ui.button("Starten →").clicked() {
                push_action(
                    events,
                    RouteToolPanelAction::ColorPath(ColorPathPanelAction::StartSampling),
                );
            }
        }
        ColorPathPanelPhase::Sampling => {
            render_color_path_sampling_info(ui, state);
            ui.separator();
            if ui
                .add_enabled(state.can_compute, egui::Button::new("Berechnen →"))
                .clicked()
            {
                push_action(
                    events,
                    RouteToolPanelAction::ColorPath(ColorPathPanelAction::ComputePreview),
                );
            }
        }
        ColorPathPanelPhase::Preview => {
            if let Some(stats) = state.preview_stats {
                ui.label(format!(
                    "Kreuzungen: {}  Offene Enden: {}",
                    stats.junction_count, stats.open_end_count
                ));
                ui.label(format!(
                    "Segmente: {}  Preview-Nodes: {}",
                    stats.segment_count, stats.node_count
                ));
                if !stats.can_accept {
                    ui.small("Keine Nodes zum Einfuegen vorhanden.");
                }
            }

            ui.separator();
            if ui.button("← Zurueck").clicked() {
                push_action(
                    events,
                    RouteToolPanelAction::ColorPath(ColorPathPanelAction::BackToSampling),
                );
            }
        }
    }

    ui.separator();
    if ui.button("Reset").clicked() {
        push_action(
            events,
            RouteToolPanelAction::ColorPath(ColorPathPanelAction::Reset),
        );
    }

    ui.separator();
    ui.label("Einstellungen:");

    let mut exact_color_match = state.exact_color_match;
    if ui.checkbox(&mut exact_color_match, "Exaktmodus").changed() {
        push_action(
            events,
            RouteToolPanelAction::ColorPath(ColorPathPanelAction::SetExactColorMatch(
                exact_color_match,
            )),
        );
    }

    ui.horizontal(|ui| {
        ui.label("Farbtoleranz:");
        let mut color_tolerance = state.color_tolerance;
        let response = ui.add_enabled(
            !state.exact_color_match,
            egui::Slider::new(&mut color_tolerance, 1.0..=80.0),
        );
        if response.changed() {
            push_action(
                events,
                RouteToolPanelAction::ColorPath(ColorPathPanelAction::SetColorTolerance(
                    color_tolerance,
                )),
            );
        }
    });

    ui.horizontal(|ui| {
        ui.label("Knotenabstand:");
        let mut node_spacing = state.node_spacing;
        if ui
            .add(egui::Slider::new(&mut node_spacing, 1.0..=50.0).suffix(" m"))
            .changed()
        {
            push_action(
                events,
                RouteToolPanelAction::ColorPath(ColorPathPanelAction::SetNodeSpacing(node_spacing)),
            );
        }
    });

    ui.horizontal(|ui| {
        ui.label("Vereinfachung:");
        let mut simplify_tolerance = state.simplify_tolerance;
        if ui
            .add(egui::Slider::new(&mut simplify_tolerance, 0.0..=20.0).suffix(" m"))
            .changed()
        {
            push_action(
                events,
                RouteToolPanelAction::ColorPath(ColorPathPanelAction::SetSimplifyTolerance(
                    simplify_tolerance,
                )),
            );
        }
    });

    let mut noise_filter = state.noise_filter;
    if ui.checkbox(&mut noise_filter, "Rauschfilter").changed() {
        push_action(
            events,
            RouteToolPanelAction::ColorPath(ColorPathPanelAction::SetNoiseFilter(noise_filter)),
        );
    }

    ui.horizontal(|ui| {
        ui.label("Bestandsanschluss:");
        let mut mode = state.existing_connection_mode;
        egui::ComboBox::from_id_salt("color_path_existing_connection_mode")
            .selected_text(existing_connection_mode_label(mode))
            .show_ui(ui, |ui| {
                for choice in [
                    ExistingConnectionModeChoice::Never,
                    ExistingConnectionModeChoice::OpenEnds,
                    ExistingConnectionModeChoice::OpenEndsAndJunctions,
                ] {
                    ui.selectable_value(&mut mode, choice, existing_connection_mode_label(choice));
                }
            });
        if mode != state.existing_connection_mode {
            push_action(
                events,
                RouteToolPanelAction::ColorPath(ColorPathPanelAction::SetExistingConnectionMode(
                    mode,
                )),
            );
        }
    });
}

fn render_color_path_sampling_info(ui: &mut egui::Ui, state: &ColorPathPanelState) {
    if let Some(avg) = state.avg_color {
        ui.horizontal(|ui| {
            ui.label(format!("Samples: {}  Ø-Farbe:", state.sample_count));
            render_color_swatch(ui, avg, 16.0, 2.0);
        });
        ui.label(format!(
            "{}: {} Farben",
            state.palette_label,
            state.palette_colors.len()
        ));
        ui.horizontal_wrapped(|ui| {
            for &color in state.palette_colors.iter().take(20) {
                render_color_swatch(ui, color, 10.0, 1.0);
            }
        });
    } else {
        ui.label(format!("Samples: {}", state.sample_count));
        ui.colored_label(egui::Color32::GRAY, "Alt+Drag zum Sampeln von Farben");
    }
}

fn render_segment_config(
    ui: &mut egui::Ui,
    state: &SegmentConfigPanelState,
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
        if ui
            .add(
                egui::DragValue::new(&mut max_segment_length)
                    .range(min_segment_length..=max_segment_length_limit)
                    .speed(0.25)
                    .suffix(" m"),
            )
            .changed()
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
            if ui
                .add(
                    egui::DragValue::new(&mut node_count)
                        .range(min..=max)
                        .speed(1.0),
                )
                .changed()
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
        if ui
            .add(
                egui::DragValue::new(&mut max_segment_length)
                    .range(min_segment_length..=max_segment_length_limit)
                    .speed(0.25)
                    .suffix(" m"),
            )
            .changed()
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

fn render_parking_f32(
    ui: &mut egui::Ui,
    label: &str,
    current: f32,
    range: std::ops::RangeInclusive<f32>,
    suffix: &str,
    events: &mut Vec<AppIntent>,
    map_action: impl Fn(f32) -> RouteToolPanelAction,
) {
    ui.horizontal(|ui| {
        ui.label(label);
        let mut value = current;
        if ui
            .add(
                egui::DragValue::new(&mut value)
                    .range(range)
                    .speed(0.25)
                    .suffix(suffix),
            )
            .changed()
        {
            push_action(events, map_action(value));
        }
    });
}

fn render_drag_f32(
    ui: &mut egui::Ui,
    label: &str,
    current: f32,
    range: std::ops::RangeInclusive<f32>,
    suffix: &str,
    events: &mut Vec<AppIntent>,
    map_action: impl Fn(f32) -> RouteToolPanelAction,
) {
    ui.horizontal(|ui| {
        ui.label(label);
        let mut value = current;
        if ui
            .add(
                egui::DragValue::new(&mut value)
                    .range(range)
                    .speed(0.25)
                    .suffix(suffix),
            )
            .changed()
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
