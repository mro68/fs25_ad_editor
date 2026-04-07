//! Renderer fuer Analyse- und Generator-Tool-Sektionen im Route-Panel.

use super::*;

/// Rendert die Konfiguration fuer das FieldBoundary-Tool.
///
/// `wheel_enabled` steuert, ob numerische Widgets ihre Mausrad-Helfer aus
/// `ui::common` anwenden.
pub(super) fn render_field_boundary_panel(
    ui: &mut egui::Ui,
    state: &FieldBoundaryPanelState,
    wheel_enabled: bool,
    events: &mut Vec<AppIntent>,
) {
    if let Some(field_id) = state.selected_field_id {
        ui.label(format!("Feld #{field_id}"));
    } else {
        ui.colored_label(
            egui::Color32::GRAY,
            "Kein Feld ausgewaehlt — in ein Feld klicken",
        );
    }

    ui.separator();
    render_drag_f32(
        ui,
        "Node-Abstand:",
        state.node_spacing,
        1.0..=50.0,
        0.1,
        " m",
        wheel_enabled,
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
        0.1,
        " m",
        wheel_enabled,
        events,
        |value| RouteToolPanelAction::FieldBoundary(FieldBoundaryPanelAction::SetOffset(value)),
    );
    render_drag_f32(
        ui,
        "Begradigen:",
        state.straighten_tolerance,
        0.0..=10.0,
        0.1,
        " m",
        wheel_enabled,
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
            0.1,
            "°",
            wheel_enabled,
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
                0.1,
                " m",
                wheel_enabled,
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
                0.1,
                "°",
                wheel_enabled,
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

    if state.show_select_hint {
        ui.small("Erneuter Klick im Viewport → anderes Feld auswählen");
    }
}

/// Rendert die Konfiguration fuer das FieldPath-Tool.
///
/// `wheel_enabled` steuert, ob numerische Widgets ihre Mausrad-Helfer aus
/// `ui::common` anwenden.
pub(super) fn render_field_path_panel(
    ui: &mut egui::Ui,
    state: &FieldPathPanelState,
    wheel_enabled: bool,
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
            if let Some(status) = state.preview_status {
                let preview_message = match status {
                    FieldPathPreviewStatus::NoMiddleLine => {
                        "Keine Mittellinie gefunden — Seiten anpassen".to_owned()
                    }
                    FieldPathPreviewStatus::Generated { node_count } => {
                        format!("{node_count} Nodes generiert")
                    }
                };
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
        0.1,
        " m",
        wheel_enabled,
        events,
        |value| RouteToolPanelAction::FieldPath(FieldPathPanelAction::SetNodeSpacing(value)),
    );
    render_drag_f32(
        ui,
        "Vereinfachung:",
        state.simplify_tolerance,
        0.0..=20.0,
        0.1,
        " m",
        wheel_enabled,
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

/// Rendert die Konfiguration fuer das RouteOffset-Tool.
///
/// `wheel_enabled` steuert, ob numerische Widgets ihre Mausrad-Helfer aus
/// `ui::common` anwenden.
pub(super) fn render_route_offset_panel(
    ui: &mut egui::Ui,
    state: &RouteOffsetPanelState,
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
        0.1,
        " m",
        wheel_enabled,
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
        0.1,
        " m",
        wheel_enabled,
        events,
        |value| RouteToolPanelAction::RouteOffset(RouteOffsetPanelAction::SetRightDistance(value)),
    );
    render_drag_f32(
        ui,
        "Basisabstand:",
        state.base_spacing,
        ROUTE_OFFSET_BASE_SPACING_LIMITS.range(),
        0.1,
        " m",
        wheel_enabled,
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

/// Rendert die Konfiguration fuer das ColorPath-Tool.
///
/// `wheel_enabled` steuert, ob numerische Widgets ihre Mausrad-Helfer aus
/// `ui::common` anwenden.
pub(super) fn render_color_path_panel(
    ui: &mut egui::Ui,
    state: &ColorPathPanelState,
    wheel_enabled: bool,
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

    render_slider_f32(
        ui,
        "Farbtoleranz:",
        state.color_tolerance,
        1.0..=80.0,
        "",
        !state.exact_color_match,
        wheel_enabled,
        events,
        |value| RouteToolPanelAction::ColorPath(ColorPathPanelAction::SetColorTolerance(value)),
    );

    render_slider_f32(
        ui,
        "Knotenabstand:",
        state.node_spacing,
        1.0..=50.0,
        " m",
        true,
        wheel_enabled,
        events,
        |value| RouteToolPanelAction::ColorPath(ColorPathPanelAction::SetNodeSpacing(value)),
    );

    render_slider_f32(
        ui,
        "Vereinfachung:",
        state.simplify_tolerance,
        0.0..=20.0,
        " m",
        true,
        wheel_enabled,
        events,
        |value| RouteToolPanelAction::ColorPath(ColorPathPanelAction::SetSimplifyTolerance(value)),
    );

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

pub(super) fn render_color_path_sampling_info(ui: &mut egui::Ui, state: &ColorPathPanelState) {
    if let Some(avg) = state.avg_color {
        ui.horizontal(|ui| {
            ui.label(format!("Samples: {}  Ø-Farbe:", state.sample_count));
            render_color_swatch(ui, avg, 16.0, 2.0);
        });
        ui.label(format!(
            "{}: {} Farben",
            if state.exact_color_match {
                "Exakte Farben"
            } else {
                "Palette"
            },
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
