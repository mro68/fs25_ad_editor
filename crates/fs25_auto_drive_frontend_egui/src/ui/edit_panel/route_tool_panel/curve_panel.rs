//! Renderer fuer Kurven- und Spline-spezifische Panel-Sektionen.

use super::*;

/// Rendert den Kurven-Konfigurationsbereich im Route-Tool-Panel.
///
/// Numerische Segment-Felder erhalten `wheel_enabled`, damit der zentrale
/// Float-Standardschritt (`0.1`) aus `ui::common` angewendet wird.
pub(super) fn render_curve_panel(
    ui: &mut egui::Ui,
    state: &CurvePanelState,
    panel_ctx: &mut RouteToolPanelRenderContext<'_>,
) {
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
                panel_ctx.events,
                RouteToolPanelAction::Curve(CurvePanelAction::SetDegree(degree)),
            );
        }
    });

    if let Some(tangents) = state.tangents.as_ref() {
        ui.separator();
        render_curve_tangents(ui, tangents, panel_ctx);
    }

    ui.separator();
    render_segment_config(ui, &state.segment, panel_ctx, |action| {
        RouteToolPanelAction::Curve(CurvePanelAction::Segment(action))
    });
}

/// Rendert die Tangenten-Auswahl fuer Bezier-Kurven im Route-Tool-Panel.
pub(super) fn render_curve_tangents(
    ui: &mut egui::Ui,
    state: &CurveTangentsPanelState,
    panel_ctx: &mut RouteToolPanelRenderContext<'_>,
) {
    if let Some(hint) = state.help_hint {
        ui.small(tangent_help_hint_label(hint));
    }

    render_tangent_selection(ui, "Start-Tangente", &state.start, panel_ctx, |value| {
        RouteToolPanelAction::Curve(CurvePanelAction::SetTangentStart(value))
    });
    render_tangent_selection(ui, "End-Tangente", &state.end, panel_ctx, |value| {
        RouteToolPanelAction::Curve(CurvePanelAction::SetTangentEnd(value))
    });
}

fn tangent_help_hint_label(hint: TangentHelpHint) -> &'static str {
    match hint {
        TangentHelpHint::SetStartEnd => "Start- und Endpunkt setzen, um Tangenten auswaehlen.",
    }
}

/// Rendert den Spline-Konfigurationsbereich im Route-Tool-Panel.
///
/// Numerische Segment-Felder erhalten `wheel_enabled`, damit der zentrale
/// Float-Standardschritt (`0.1`) aus `ui::common` angewendet wird.
pub(super) fn render_spline_panel(
    ui: &mut egui::Ui,
    state: &SplinePanelState,
    panel_ctx: &mut RouteToolPanelRenderContext<'_>,
) {
    if let Some(control_point_count) = state.control_point_count {
        ui.label(format!("Kontrollpunkte: {control_point_count}"));
    }

    if let Some(start_tangent) = state.start_tangent.as_ref() {
        ui.separator();
        render_tangent_selection(ui, "Tangente Start:", start_tangent, panel_ctx, |value| {
            RouteToolPanelAction::Spline(SplinePanelAction::SetTangentStart(value))
        });
    }

    if let Some(end_tangent) = state.end_tangent.as_ref() {
        render_tangent_selection(ui, "Tangente Ende:", end_tangent, panel_ctx, |value| {
            RouteToolPanelAction::Spline(SplinePanelAction::SetTangentEnd(value))
        });
    }

    ui.separator();
    render_segment_config(ui, &state.segment, panel_ctx, |action| {
        RouteToolPanelAction::Spline(SplinePanelAction::Segment(action))
    });
}
