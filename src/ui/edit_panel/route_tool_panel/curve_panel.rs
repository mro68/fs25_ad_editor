//! Renderer fuer Kurven- und Spline-spezifische Panel-Sektionen.

use super::*;

pub(super) fn render_curve_panel(
    ui: &mut egui::Ui,
    state: &CurvePanelState,
    events: &mut Vec<AppIntent>,
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

pub(super) fn render_curve_tangents(
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

pub(super) fn render_spline_panel(
    ui: &mut egui::Ui,
    state: &SplinePanelState,
    events: &mut Vec<AppIntent>,
) {
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
