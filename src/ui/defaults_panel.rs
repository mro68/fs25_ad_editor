//! Linkes Sidebar-Panel fuer Werkzeuge, Defaults und Hintergrund-Controls.

use crate::app::segment_registry::TOOL_INDEX_FIELD_BOUNDARY;
use crate::app::{AppIntent, AppState, ConnectionDirection, ConnectionPriority, EditorTool};
use crate::ui::icons::{route_tool_icon, svg_icon, ICON_SIZE};
use crate::ui::properties::selectors::{
    render_direction_icon_selector_vertical, render_priority_icon_selector_vertical,
};

fn color32_from_rgba(color: [f32; 4]) -> egui::Color32 {
    egui::Color32::from_rgba_unmultiplied(
        (color[0].clamp(0.0, 1.0) * 255.0) as u8,
        (color[1].clamp(0.0, 1.0) * 255.0) as u8,
        (color[2].clamp(0.0, 1.0) * 255.0) as u8,
        (color[3].clamp(0.0, 1.0) * 255.0) as u8,
    )
}

fn function_icon_color(state: &AppState) -> egui::Color32 {
    match state.editor.default_priority {
        ConnectionPriority::Regular => color32_from_rgba(state.options.connection_color_regular),
        ConnectionPriority::SubPriority => color32_from_rgba(state.options.node_color_subprio),
    }
}

fn accent_icon_color(state: &AppState) -> egui::Color32 {
    match state.editor.default_direction {
        ConnectionDirection::Regular => color32_from_rgba(state.options.connection_color_regular),
        ConnectionDirection::Dual => color32_from_rgba(state.options.connection_color_dual),
        ConnectionDirection::Reverse => color32_from_rgba(state.options.connection_color_reverse),
    }
}

/// Rendert die linke Sidebar mit Tool-Auswahl, Route-Tools und Defaults.
pub fn render_route_defaults_panel(ctx: &egui::Context, state: &AppState) -> Vec<AppIntent> {
    let mut events = Vec::new();
    let active_tool = state.editor.active_tool;
    let active_route_index = if active_tool == EditorTool::Route {
        state.editor.tool_manager.active_index()
    } else {
        None
    };
    let icon_color = function_icon_color(state);
    let active_icon_color = accent_icon_color(state);

    egui::SidePanel::left("route_defaults_panel")
        .resizable(false)
        .default_width(64.0)
        .show(ctx, |ui| {
            egui::CollapsingHeader::new("Werkzeuge")
                .default_open(true)
                .show(ui, |ui| {
                    let select_icon = svg_icon(
                        egui::include_image!("../../assets/icons/icon_select_node.svg"),
                        ICON_SIZE,
                    )
                    .tint(if active_tool == EditorTool::Select {
                        active_icon_color
                    } else {
                        icon_color
                    });
                    if ui
                        .add(
                            egui::Button::image(select_icon)
                                .selected(active_tool == EditorTool::Select),
                        )
                        .on_hover_text("Select (1)")
                        .clicked()
                    {
                        events.push(AppIntent::SetEditorToolRequested {
                            tool: EditorTool::Select,
                        });
                    }

                    let connect_icon = svg_icon(
                        egui::include_image!("../../assets/icons/icon_connect.svg"),
                        ICON_SIZE,
                    )
                    .tint(if active_tool == EditorTool::Connect {
                        active_icon_color
                    } else {
                        icon_color
                    });
                    if ui
                        .add(
                            egui::Button::image(connect_icon)
                                .selected(active_tool == EditorTool::Connect),
                        )
                        .on_hover_text("Connect (2)")
                        .clicked()
                    {
                        events.push(AppIntent::SetEditorToolRequested {
                            tool: EditorTool::Connect,
                        });
                    }

                    let add_icon = svg_icon(
                        egui::include_image!("../../assets/icons/icon_add_node.svg"),
                        ICON_SIZE,
                    )
                    .tint(if active_tool == EditorTool::AddNode {
                        active_icon_color
                    } else {
                        icon_color
                    });
                    if ui
                        .add(
                            egui::Button::image(add_icon)
                                .selected(active_tool == EditorTool::AddNode),
                        )
                        .on_hover_text("Add Node (3)")
                        .clicked()
                    {
                        events.push(AppIntent::SetEditorToolRequested {
                            tool: EditorTool::AddNode,
                        });
                    }
                });

            egui::CollapsingHeader::new("Routen")
                .default_open(true)
                .show(ui, |ui| {
                    for &(idx, name, _icon_name) in &state.editor.tool_manager.tool_entries() {
                        if idx == TOOL_INDEX_FIELD_BOUNDARY {
                            continue;
                        }

                        let is_active = active_route_index == Some(idx);
                        let icon_img =
                            svg_icon(route_tool_icon(idx), ICON_SIZE).tint(if is_active {
                                active_icon_color
                            } else {
                                icon_color
                            });

                        if ui
                            .add(egui::Button::image(icon_img).selected(is_active))
                            .on_hover_text(name)
                            .clicked()
                        {
                            events.push(AppIntent::SetEditorToolRequested {
                                tool: EditorTool::Route,
                            });
                            events.push(AppIntent::SelectRouteToolRequested { index: idx });
                        }
                    }
                });

            egui::CollapsingHeader::new("Aktionen").show(ui, |ui| {
                let has_selection = !state.selection.selected_node_ids.is_empty();
                let delete_icon = svg_icon(
                    egui::include_image!("../../assets/icons/icon_delete.svg"),
                    ICON_SIZE,
                )
                .tint(icon_color);

                if ui
                    .add_enabled(has_selection, egui::Button::image(delete_icon))
                    .on_hover_text("Delete (Del)")
                    .clicked()
                {
                    events.push(AppIntent::DeleteSelectedRequested);
                }
            });

            ui.add_space(6.0);
            ui.separator();
            ui.add_space(6.0);

            let mut selected_dir = state.editor.default_direction;
            render_direction_icon_selector_vertical(ui, &mut selected_dir, "defaults_left");
            if selected_dir != state.editor.default_direction {
                events.push(AppIntent::SetDefaultDirectionRequested {
                    direction: selected_dir,
                });
            }

            ui.add_space(6.0);
            ui.separator();
            ui.add_space(6.0);

            let mut selected_prio = state.editor.default_priority;
            render_priority_icon_selector_vertical(ui, &mut selected_prio, "defaults_left");
            if selected_prio != state.editor.default_priority {
                events.push(AppIntent::SetDefaultPriorityRequested {
                    priority: selected_prio,
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
