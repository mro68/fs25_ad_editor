use super::*;

pub(super) fn render_straight_panel(
    ui: &mut egui::Ui,
    state: &StraightPanelState,
    wheel_enabled: bool,
    events: &mut Vec<AppIntent>,
) {
    render_segment_config(ui, &state.segment, wheel_enabled, events, |action| {
        RouteToolPanelAction::Straight(StraightPanelAction::Segment(action))
    });
}

pub(super) fn render_smooth_curve_panel(
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

pub(super) fn render_bypass_panel(
    ui: &mut egui::Ui,
    state: &BypassPanelState,
    wheel_enabled: bool,
    events: &mut Vec<AppIntent>,
) {
    if !state.has_chain {
        ui.colored_label(
            egui::Color32::GRAY,
            "Kette selektieren und Route-Tool neu aktivieren.",
        );
        return;
    }

    ui.label(format!("Kette: {} Nodes", state.chain_node_count));
    if let Some(new_node_count) = state.new_node_count {
        ui.label(format!("Neue Nodes: {new_node_count}"));
    }
    if let Some(transition_length_m) = state.transition_length_m {
        ui.label(format!("Uebergang: {:.1} m", transition_length_m));
    }
    let side_text = if state.offset >= 0.0 {
        "links"
    } else {
        "rechts"
    };
    ui.label(format!("Seite: Richtung: {side_text}"));

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

pub(super) fn render_parking_panel(
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
