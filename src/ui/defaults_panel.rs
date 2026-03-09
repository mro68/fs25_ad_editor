//! Linkes Sidebar-Panel fuer Werkzeuge, Defaults und Hintergrund-Controls.

use crate::app::segment_registry::{
    TOOL_INDEX_BYPASS, TOOL_INDEX_CONSTRAINT_ROUTE, TOOL_INDEX_CURVE_CUBIC,
    TOOL_INDEX_CURVE_QUAD, TOOL_INDEX_PARKING, TOOL_INDEX_ROUTE_OFFSET, TOOL_INDEX_SPLINE,
    TOOL_INDEX_STRAIGHT,
};
use crate::app::{AppIntent, AppState, ConnectionDirection, ConnectionPriority, EditorTool};
use crate::ui::icons::{
    accent_icon_color, function_icon_color, route_tool_icon, svg_icon, ICON_SIZE,
};
use crate::ui::long_press::{
    LongPressGroup, LongPressItem, LongPressState, render_long_press_button,
};

#[derive(Debug, Clone, Copy)]
enum RouteGroup {
    Straight,
    Curve,
    Constraint,
    Section,
}

fn route_group_label(group: RouteGroup) -> &'static str {
    match group {
        RouteGroup::Straight => "Geraden",
        RouteGroup::Curve => "Kurven",
        RouteGroup::Constraint => "Constraint",
        RouteGroup::Section => "Tools",
    }
}

fn push_route_tool_selection(
    events: &mut Vec<AppIntent>,
    _group: RouteGroup,
    index: usize,
) {
    events.push(AppIntent::SetEditorToolRequested {
        tool: EditorTool::Route,
    });
    events.push(AppIntent::SelectRouteToolRequested { index });
}

fn render_long_press_with_memory<T: Clone + PartialEq>(
    ui: &mut egui::Ui,
    icon_color: egui::Color32,
    active_icon_color: egui::Color32,
    group: &LongPressGroup<T>,
    active_value: &T,
) -> Option<T> {
    let key = egui::Id::new(("defaults_panel_long_press", group.id));
    let mut lp_state = ui
        .ctx()
        .data_mut(|d| d.get_temp::<LongPressState>(key).unwrap_or_default());

    let selection = render_long_press_button(
        ui,
        icon_color,
        active_icon_color,
        group,
        active_value,
        &mut lp_state,
    );

    ui.ctx().data_mut(|d| d.insert_temp(key, lp_state));
    selection
}

/// Rendert die linke Sidebar mit Tool-Auswahl, Route-Tools und Defaults.
pub fn render_route_defaults_panel(ctx: &egui::Context, state: &AppState) -> Vec<AppIntent> {
    let mut events = Vec::new();
    let active_tool = state.editor.active_tool;
    let icon_color = function_icon_color(state);
    let active_icon_color = accent_icon_color(state);

    let tools_group = LongPressGroup {
        id: "werkzeuge",
        label: "Werkzeuge",
        items: vec![
            LongPressItem {
                icon: egui::include_image!("../../assets/icons/icon_select_node.svg"),
                tooltip: "Auswahl (1)",
                value: EditorTool::Select,
            },
            LongPressItem {
                icon: egui::include_image!("../../assets/icons/icon_connect.svg"),
                tooltip: "Verbinden (2)",
                value: EditorTool::Connect,
            },
            LongPressItem {
                icon: egui::include_image!("../../assets/icons/icon_add_node.svg"),
                tooltip: "Node hinzufuegen (3)",
                value: EditorTool::AddNode,
            },
        ],
    };

    let straights_group = LongPressGroup {
        id: "grundbefehle_geraden",
        label: route_group_label(RouteGroup::Straight),
        items: vec![LongPressItem {
            icon: route_tool_icon(TOOL_INDEX_STRAIGHT),
            tooltip: "Gerade Strecke",
            value: TOOL_INDEX_STRAIGHT,
        }],
    };

    let curves_group = LongPressGroup {
        id: "grundbefehle_kurven",
        label: route_group_label(RouteGroup::Curve),
        items: vec![
            LongPressItem {
                icon: route_tool_icon(TOOL_INDEX_CURVE_QUAD),
                tooltip: "Bezier Grad 2",
                value: TOOL_INDEX_CURVE_QUAD,
            },
            LongPressItem {
                icon: route_tool_icon(TOOL_INDEX_CURVE_CUBIC),
                tooltip: "Bezier Grad 3",
                value: TOOL_INDEX_CURVE_CUBIC,
            },
            LongPressItem {
                icon: route_tool_icon(TOOL_INDEX_SPLINE),
                tooltip: "Spline",
                value: TOOL_INDEX_SPLINE,
            },
        ],
    };

    let constraint_group = LongPressGroup {
        id: "grundbefehle_constraint",
        label: route_group_label(RouteGroup::Constraint),
        items: vec![LongPressItem {
            icon: route_tool_icon(TOOL_INDEX_CONSTRAINT_ROUTE),
            tooltip: "Constraint-Route",
            value: TOOL_INDEX_CONSTRAINT_ROUTE,
        }],
    };

    let section_tools_group = LongPressGroup {
        id: "tools_abschnitt",
        label: route_group_label(RouteGroup::Section),
        items: vec![
            LongPressItem {
                icon: route_tool_icon(TOOL_INDEX_BYPASS),
                tooltip: "Ausweichstrecke",
                value: TOOL_INDEX_BYPASS,
            },
            LongPressItem {
                icon: route_tool_icon(TOOL_INDEX_PARKING),
                tooltip: "Parkplatz",
                value: TOOL_INDEX_PARKING,
            },
            LongPressItem {
                icon: route_tool_icon(TOOL_INDEX_ROUTE_OFFSET),
                tooltip: "Strecke versetzen",
                value: TOOL_INDEX_ROUTE_OFFSET,
            },
        ],
    };

    let direction_group = LongPressGroup {
        id: "defaults_richtung",
        label: "Voreinstellung Richtung",
        items: vec![
            LongPressItem {
                icon: egui::include_image!("../../assets/icons/icon_direction_regular.svg"),
                tooltip: "Einbahn vorwaerts",
                value: ConnectionDirection::Regular,
            },
            LongPressItem {
                icon: egui::include_image!("../../assets/icons/icon_direction_dual.svg"),
                tooltip: "Zweirichtung",
                value: ConnectionDirection::Dual,
            },
            LongPressItem {
                icon: egui::include_image!("../../assets/icons/icon_direction_reverse.svg"),
                tooltip: "Einbahn rueckwaerts",
                value: ConnectionDirection::Reverse,
            },
        ],
    };

    let priority_group = LongPressGroup {
        id: "defaults_prioritaet",
        label: "Voreinstellung Strassenart",
        items: vec![
            LongPressItem {
                icon: egui::include_image!("../../assets/icons/icon_priority_main.svg"),
                tooltip: "Hauptstrasse",
                value: ConnectionPriority::Regular,
            },
            LongPressItem {
                icon: egui::include_image!("../../assets/icons/icon_priority_side.svg"),
                tooltip: "Nebenstrasse",
                value: ConnectionPriority::SubPriority,
            },
        ],
    };

    egui::SidePanel::left("route_defaults_panel")
        .resizable(false)
        .default_width(64.0)
        .show(ctx, |ui| {
            if let Some(tool) = render_long_press_with_memory(
                ui,
                icon_color,
                active_icon_color,
                &tools_group,
                &active_tool,
            ) {
                events.push(AppIntent::SetEditorToolRequested { tool });
            }

            ui.add_space(6.0);
            ui.separator();
            ui.add_space(6.0);

            if let Some(index) = render_long_press_with_memory(
                ui,
                icon_color,
                active_icon_color,
                &straights_group,
                &state.editor.last_straight_index,
            ) {
                push_route_tool_selection(&mut events, RouteGroup::Straight, index);
            }

            if let Some(index) = render_long_press_with_memory(
                ui,
                icon_color,
                active_icon_color,
                &curves_group,
                &state.editor.last_curve_index,
            ) {
                push_route_tool_selection(&mut events, RouteGroup::Curve, index);
            }

            if let Some(index) = render_long_press_with_memory(
                ui,
                icon_color,
                active_icon_color,
                &constraint_group,
                &state.editor.last_constraint_index,
            ) {
                push_route_tool_selection(&mut events, RouteGroup::Constraint, index);
            }

            ui.add_space(6.0);
            ui.separator();
            ui.add_space(6.0);

            if let Some(index) = render_long_press_with_memory(
                ui,
                icon_color,
                active_icon_color,
                &section_tools_group,
                &state.editor.last_section_tool_index,
            ) {
                push_route_tool_selection(&mut events, RouteGroup::Section, index);
            }

            ui.add_space(6.0);
            ui.separator();
            ui.add_space(6.0);

            if let Some(direction) = render_long_press_with_memory(
                ui,
                icon_color,
                active_icon_color,
                &direction_group,
                &state.editor.default_direction,
            ) {
                events.push(AppIntent::SetDefaultDirectionRequested {
                    direction,
                });
            }

            ui.add_space(6.0);
            ui.separator();
            ui.add_space(6.0);

            if let Some(priority) = render_long_press_with_memory(
                ui,
                icon_color,
                active_icon_color,
                &priority_group,
                &state.editor.default_priority,
            ) {
                events.push(AppIntent::SetDefaultPriorityRequested {
                    priority,
                });
            }

            if state.view.background_map.is_some() {
                egui::CollapsingHeader::new("Hintergrund").show(ui, |ui| {
                    let visible = state.view.background_visible;
                    let toggle_icon = if visible {
                        egui::include_image!("../../assets/icons/icon_visible.svg")
                    } else {
                        egui::include_image!("../../assets/icons/icon_hidden.svg")
                    };
                    let toggle_img = svg_icon(toggle_icon, ICON_SIZE).tint(if visible {
                        active_icon_color
                    } else {
                        icon_color
                    });

                    if ui
                        .add(egui::Button::image(toggle_img))
                        .on_hover_text(if visible {
                            "Hintergrund ausblenden"
                        } else {
                            "Hintergrund einblenden"
                        })
                        .clicked()
                    {
                        events.push(AppIntent::ToggleBackgroundVisibility);
                    }

                    let scale = state.view.background_scale;
                    if ui
                        .button("-")
                        .on_hover_text("Ausdehnung halbieren")
                        .clicked()
                    {
                        events.push(AppIntent::ScaleBackground { factor: 0.5 });
                    }
                    ui.label(format!("x{scale:.2}"));
                    if ui
                        .button("+")
                        .on_hover_text("Ausdehnung verdoppeln")
                        .clicked()
                    {
                        events.push(AppIntent::ScaleBackground { factor: 2.0 });
                    }
                    if (scale - 1.0).abs() > f32::EPSILON
                        && ui.button("1:1").on_hover_text("Originalgroesse").clicked()
                    {
                        events.push(AppIntent::ScaleBackground {
                            factor: 1.0 / scale,
                        });
                    }
                });
            }
        });

    events
}
