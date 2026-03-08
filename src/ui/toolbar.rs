//! Schwebende Tool-Palette fuer Editor-Werkzeugauswahl.

use crate::app::segment_registry::TOOL_INDEX_FIELD_BOUNDARY;
use crate::app::{AppIntent, AppState, ConnectionDirection, ConnectionPriority, EditorTool};
use crate::ui::icons::{route_tool_icon, svg_icon, ICON_SIZE};

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

/// Rendert die freie Tool-Palette als schwebendes Fenster ohne Titelleiste.
/// Wird bei 'T'-Taste an der Mausposition angezeigt.
/// Gibt `(events, should_close)` zurueck.
pub fn render_tool_palette(ctx: &egui::Context, state: &AppState) -> (Vec<AppIntent>, bool) {
    let mut events = Vec::new();
    let mut should_close = false;
    let active = state.editor.active_tool;
    let icon_color = function_icon_color(state);
    let active_icon_color = accent_icon_color(state);
    let pos = state
        .ui
        .tool_palette_pos
        .unwrap_or(egui::pos2(300.0, 300.0));

    let window_response = egui::Window::new("tool_palette")
        .id(egui::Id::new("tool_palette_window"))
        .title_bar(false)
        .collapsible(false)
        .resizable(false)
        .fixed_pos(pos)
        .show(ctx, |ui| {
            ui.horizontal_wrapped(|ui| {
                let select_icon = svg_icon(
                    egui::include_image!("../../assets/icons/icon_select_node.svg"),
                    ICON_SIZE,
                )
                .tint(if active == EditorTool::Select {
                    active_icon_color
                } else {
                    icon_color
                });
                if ui
                    .add(egui::Button::image(select_icon).selected(active == EditorTool::Select))
                    .on_hover_text("Select (1)")
                    .clicked()
                {
                    events.push(AppIntent::SetEditorToolRequested {
                        tool: EditorTool::Select,
                    });
                    should_close = true;
                }

                let connect_icon = svg_icon(
                    egui::include_image!("../../assets/icons/icon_connect.svg"),
                    ICON_SIZE,
                )
                .tint(if active == EditorTool::Connect {
                    active_icon_color
                } else {
                    icon_color
                });
                if ui
                    .add(egui::Button::image(connect_icon).selected(active == EditorTool::Connect))
                    .on_hover_text("Connect (2)")
                    .clicked()
                {
                    events.push(AppIntent::SetEditorToolRequested {
                        tool: EditorTool::Connect,
                    });
                    should_close = true;
                }

                let add_icon = svg_icon(
                    egui::include_image!("../../assets/icons/icon_add_node.svg"),
                    ICON_SIZE,
                )
                .tint(if active == EditorTool::AddNode {
                    active_icon_color
                } else {
                    icon_color
                });
                if ui
                    .add(egui::Button::image(add_icon).selected(active == EditorTool::AddNode))
                    .on_hover_text("Add Node (3)")
                    .clicked()
                {
                    events.push(AppIntent::SetEditorToolRequested {
                        tool: EditorTool::AddNode,
                    });
                    should_close = true;
                }

                ui.separator();

                let active_route_index = if active == EditorTool::Route {
                    state.editor.tool_manager.active_index()
                } else {
                    None
                };
                for &(idx, name, _icon_name) in &state.editor.tool_manager.tool_entries() {
                    if idx == TOOL_INDEX_FIELD_BOUNDARY {
                        continue;
                    }

                    let is_active = active_route_index == Some(idx);
                    let icon_img = svg_icon(route_tool_icon(idx), ICON_SIZE).tint(if is_active {
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
                        should_close = true;
                    }
                }
            });
        });

    if let Some(inner) = &window_response {
        let palette_rect = inner.response.rect;
        let clicked_outside = ctx.input(|i| {
            let pointer_pos = i.pointer.interact_pos().or(i.pointer.hover_pos());
            i.pointer.any_pressed()
                && pointer_pos
                    .map(|pos| !palette_rect.contains(pos))
                    .unwrap_or(false)
        });
        if clicked_outside {
            should_close = true;
        }
    }

    (events, should_close)
}
