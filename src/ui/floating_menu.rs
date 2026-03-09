//! Schwebendes Kontextmenue fuer Werkzeuggruppen an der Mausposition.

use crate::app::segment_registry::{
    TOOL_INDEX_BYPASS, TOOL_INDEX_CONSTRAINT_ROUTE, TOOL_INDEX_CURVE_CUBIC,
    TOOL_INDEX_CURVE_QUAD, TOOL_INDEX_PARKING, TOOL_INDEX_ROUTE_OFFSET, TOOL_INDEX_SPLINE,
    TOOL_INDEX_STRAIGHT,
};
use crate::app::state::FloatingMenuKind;
use crate::app::{AppIntent, AppState, EditorTool};
use crate::ui::icons::{
    ICON_SIZE, accent_icon_color, function_icon_color, route_tool_icon, svg_icon,
};

/// Rendert ein schwebendes Menue an der gespeicherten Position.
/// Gibt `AppIntent`s zurueck, wenn ein Menueeintrag geklickt wurde.
pub fn render_floating_menu(ctx: &egui::Context, state: &mut AppState) -> Vec<AppIntent> {
    let Some(menu) = state.ui.floating_menu else {
        return vec![];
    };

    let mut events = Vec::new();
    let active_tool = state.editor.active_tool;
    let active_route_index = if active_tool == EditorTool::Route {
        state.editor.tool_manager.active_index()
    } else {
        None
    };

    let icon_color = function_icon_color(state);
    let active_icon_color = accent_icon_color(state);

    let area_response = egui::Area::new(egui::Id::new(("floating_menu", menu.kind)))
        .order(egui::Order::Foreground)
        .fixed_pos(menu.pos)
        .show(ctx, |ui| {
            egui::Frame::popup(ui.style()).show(ui, |ui| {
                ui.horizontal_wrapped(|ui| match menu.kind {
                    FloatingMenuKind::Tools => {
                        if tool_icon_button(
                            ui,
                            egui::include_image!("../../assets/icons/icon_select_node.svg"),
                            "Select (1)",
                            active_tool == EditorTool::Select,
                            icon_color,
                            active_icon_color,
                        ) {
                            events.push(AppIntent::SetEditorToolRequested {
                                tool: EditorTool::Select,
                            });
                        }

                        if tool_icon_button(
                            ui,
                            egui::include_image!("../../assets/icons/icon_connect.svg"),
                            "Connect (2)",
                            active_tool == EditorTool::Connect,
                            icon_color,
                            active_icon_color,
                        ) {
                            events.push(AppIntent::SetEditorToolRequested {
                                tool: EditorTool::Connect,
                            });
                        }

                        if tool_icon_button(
                            ui,
                            egui::include_image!("../../assets/icons/icon_add_node.svg"),
                            "Add Node (3)",
                            active_tool == EditorTool::AddNode,
                            icon_color,
                            active_icon_color,
                        ) {
                            events.push(AppIntent::SetEditorToolRequested {
                                tool: EditorTool::AddNode,
                            });
                        }
                    }
                    FloatingMenuKind::Basics => {
                        for &(index, tooltip) in &[
                            (TOOL_INDEX_STRAIGHT, "Gerade Strecke"),
                            (TOOL_INDEX_CURVE_QUAD, "Bezier Grad 2"),
                            (TOOL_INDEX_CURVE_CUBIC, "Bezier Grad 3"),
                            (TOOL_INDEX_SPLINE, "Spline"),
                            (TOOL_INDEX_CONSTRAINT_ROUTE, "Constraint-Route"),
                        ] {
                            if route_icon_button(
                                ui,
                                index,
                                tooltip,
                                active_route_index == Some(index),
                                icon_color,
                                active_icon_color,
                            ) {
                                events.push(AppIntent::SetEditorToolRequested {
                                    tool: EditorTool::Route,
                                });
                                events.push(AppIntent::SelectRouteToolRequested { index });
                            }
                        }
                    }
                    FloatingMenuKind::SectionTools => {
                        for &(index, tooltip) in &[
                            (TOOL_INDEX_BYPASS, "Ausweichstrecke"),
                            (TOOL_INDEX_PARKING, "Parkplatz"),
                            (TOOL_INDEX_ROUTE_OFFSET, "Strecke versetzen"),
                        ] {
                            if route_icon_button(
                                ui,
                                index,
                                tooltip,
                                active_route_index == Some(index),
                                icon_color,
                                active_icon_color,
                            ) {
                                events.push(AppIntent::SetEditorToolRequested {
                                    tool: EditorTool::Route,
                                });
                                events.push(AppIntent::SelectRouteToolRequested { index });
                            }
                        }
                    }
                });
            });
        });

    let clicked_outside = ctx.input(|i| {
        let pointer_pos = i.pointer.interact_pos().or(i.pointer.hover_pos());
        i.pointer.any_pressed()
            && pointer_pos
                .map(|pos| !area_response.response.rect.contains(pos))
                .unwrap_or(false)
    });

    if !events.is_empty() || clicked_outside {
        state.ui.floating_menu = None;
    }

    events
}

fn tool_icon_button(
    ui: &mut egui::Ui,
    icon: egui::ImageSource<'static>,
    tooltip: &'static str,
    is_active: bool,
    icon_color: egui::Color32,
    active_icon_color: egui::Color32,
) -> bool {
    let image = svg_icon(icon, ICON_SIZE).tint(if is_active {
        active_icon_color
    } else {
        icon_color
    });

    ui.add(egui::Button::image(image).selected(is_active))
        .on_hover_text(tooltip)
        .clicked()
}

fn route_icon_button(
    ui: &mut egui::Ui,
    index: usize,
    tooltip: &'static str,
    is_active: bool,
    icon_color: egui::Color32,
    active_icon_color: egui::Color32,
) -> bool {
    let image = svg_icon(route_tool_icon(index), ICON_SIZE).tint(if is_active {
        active_icon_color
    } else {
        icon_color
    });

    ui.add(egui::Button::image(image).selected(is_active))
        .on_hover_text(tooltip)
        .clicked()
}
