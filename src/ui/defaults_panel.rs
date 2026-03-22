//! Linkes Sidebar-Panel fuer Werkzeuge, Defaults und Hintergrund-Controls.

use crate::app::group_registry::{
    TOOL_INDEX_BYPASS, TOOL_INDEX_CURVE_CUBIC, TOOL_INDEX_CURVE_QUAD, TOOL_INDEX_PARKING,
    TOOL_INDEX_ROUTE_OFFSET, TOOL_INDEX_SMOOTH_CURVE, TOOL_INDEX_SPLINE, TOOL_INDEX_STRAIGHT,
};
use crate::app::{AppIntent, AppState, ConnectionDirection, ConnectionPriority, EditorTool};
use crate::shared::{t, I18nKey};
use crate::ui::icons::{
    accent_icon_color, function_icon_color, route_tool_icon, svg_icon, ICON_SIZE,
};
use crate::ui::long_press::{
    render_long_press_button, LongPressGroup, LongPressItem, LongPressState,
};

/// Zoom-Ziel fuer den LongPress-Button in der Zoom-Sektion.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum ZoomTarget {
    /// Gesamte Map in den Viewport einpassen.
    FullMap,
    /// Viewport auf die selektierten Nodes einpassen.
    Selection,
}

fn push_route_tool_selection(events: &mut Vec<AppIntent>, index: usize) {
    events.push(AppIntent::SetEditorToolRequested {
        tool: EditorTool::Route,
    });
    events.push(AppIntent::SelectRouteToolRequested { index });
}

fn render_long_press_with_memory<T: Clone + PartialEq>(
    ui: &mut egui::Ui,
    icon_color: egui::Color32,
    active_icon_color: egui::Color32,
    group: &LongPressGroup<'_, T>,
    display_value: &T,
    is_button_active: bool,
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
        display_value,
        is_button_active,
        &mut lp_state,
    );

    ui.ctx().data_mut(|d| d.insert_temp(key, lp_state));
    selection
}

/// Rendert die linke Sidebar mit Tool-Auswahl, Route-Tools und Defaults.
pub fn render_route_defaults_panel(ctx: &egui::Context, state: &AppState) -> Vec<AppIntent> {
    let mut events = Vec::new();
    let lang = state.options.language;
    let active_tool = state.editor.active_tool;
    let icon_color = function_icon_color(state);
    let active_icon_color = accent_icon_color(state);

    // Aktueller Route-Tool-Index – None wenn kein Route-Tool aktiv.
    let active_route_index: Option<usize> = if active_tool == EditorTool::Route {
        state.editor.tool_manager.active_index()
    } else {
        None
    };
    let is_werkzeug_active = matches!(
        active_tool,
        EditorTool::Select | EditorTool::Connect | EditorTool::AddNode
    );
    let is_basic_command_active = matches!(
        active_route_index,
        Some(i) if i == TOOL_INDEX_STRAIGHT
            || i == TOOL_INDEX_CURVE_QUAD
            || i == TOOL_INDEX_CURVE_CUBIC
            || i == TOOL_INDEX_SPLINE
            || i == TOOL_INDEX_SMOOTH_CURVE
    );
    let is_section_active = matches!(
        active_route_index,
        Some(i) if i == TOOL_INDEX_BYPASS || i == TOOL_INDEX_PARKING || i == TOOL_INDEX_ROUTE_OFFSET
    );

    let tools_items = [
        LongPressItem {
            icon: egui::include_image!("../../assets/icons/icon_select_node.svg"),
            tooltip: t(lang, I18nKey::LpToolSelect),
            value: EditorTool::Select,
        },
        LongPressItem {
            icon: egui::include_image!("../../assets/icons/icon_connect.svg"),
            tooltip: t(lang, I18nKey::LpToolConnect),
            value: EditorTool::Connect,
        },
        LongPressItem {
            icon: egui::include_image!("../../assets/icons/icon_add_node.svg"),
            tooltip: t(lang, I18nKey::LpToolAddNode),
            value: EditorTool::AddNode,
        },
    ];

    let basic_items = [
        LongPressItem {
            icon: route_tool_icon(TOOL_INDEX_STRAIGHT),
            tooltip: t(lang, I18nKey::LpStraight),
            value: TOOL_INDEX_STRAIGHT,
        },
        LongPressItem {
            icon: route_tool_icon(TOOL_INDEX_CURVE_QUAD),
            tooltip: t(lang, I18nKey::LpCurveQuad),
            value: TOOL_INDEX_CURVE_QUAD,
        },
        LongPressItem {
            icon: route_tool_icon(TOOL_INDEX_CURVE_CUBIC),
            tooltip: t(lang, I18nKey::LpCurveCubic),
            value: TOOL_INDEX_CURVE_CUBIC,
        },
        LongPressItem {
            icon: route_tool_icon(TOOL_INDEX_SPLINE),
            tooltip: t(lang, I18nKey::LpSpline),
            value: TOOL_INDEX_SPLINE,
        },
        LongPressItem {
            icon: route_tool_icon(TOOL_INDEX_SMOOTH_CURVE),
            tooltip: t(lang, I18nKey::LpSmoothCurve),
            value: TOOL_INDEX_SMOOTH_CURVE,
        },
    ];

    let section_tools_items = [
        LongPressItem {
            icon: route_tool_icon(TOOL_INDEX_BYPASS),
            tooltip: t(lang, I18nKey::LpBypass),
            value: TOOL_INDEX_BYPASS,
        },
        LongPressItem {
            icon: route_tool_icon(TOOL_INDEX_PARKING),
            tooltip: t(lang, I18nKey::LpParking),
            value: TOOL_INDEX_PARKING,
        },
        LongPressItem {
            icon: route_tool_icon(TOOL_INDEX_ROUTE_OFFSET),
            tooltip: t(lang, I18nKey::LpRouteOffset),
            value: TOOL_INDEX_ROUTE_OFFSET,
        },
    ];

    let direction_items = [
        LongPressItem {
            icon: egui::include_image!("../../assets/icons/icon_direction_regular.svg"),
            tooltip: t(lang, I18nKey::LpDirectionRegular),
            value: ConnectionDirection::Regular,
        },
        LongPressItem {
            icon: egui::include_image!("../../assets/icons/icon_direction_dual.svg"),
            tooltip: t(lang, I18nKey::LpDirectionDual),
            value: ConnectionDirection::Dual,
        },
        LongPressItem {
            icon: egui::include_image!("../../assets/icons/icon_direction_reverse.svg"),
            tooltip: t(lang, I18nKey::LpDirectionReverse),
            value: ConnectionDirection::Reverse,
        },
    ];

    let priority_items = [
        LongPressItem {
            icon: egui::include_image!("../../assets/icons/icon_priority_main.svg"),
            tooltip: t(lang, I18nKey::LpPriorityMain),
            value: ConnectionPriority::Regular,
        },
        LongPressItem {
            icon: egui::include_image!("../../assets/icons/icon_priority_side.svg"),
            tooltip: t(lang, I18nKey::LpPrioritySub),
            value: ConnectionPriority::SubPriority,
        },
    ];

    let tools_group = LongPressGroup {
        id: "werkzeuge",
        label: t(lang, I18nKey::SidebarTools),
        items: &tools_items,
    };

    let basic_commands_group = LongPressGroup {
        id: "grundbefehle",
        label: t(lang, I18nKey::SidebarBasics),
        items: &basic_items,
    };

    let section_tools_group = LongPressGroup {
        id: "tools_abschnitt",
        label: t(lang, I18nKey::RouteGroupSection),
        items: &section_tools_items,
    };

    let direction_group = LongPressGroup {
        id: "defaults_richtung",
        label: t(lang, I18nKey::SidebarDirection),
        items: &direction_items,
    };

    let priority_group = LongPressGroup {
        id: "defaults_prioritaet",
        label: t(lang, I18nKey::SidebarPriority),
        items: &priority_items,
    };

    egui::SidePanel::left("route_defaults_panel")
        .resizable(false)
        .default_width(80.0)
        .show(ctx, |ui| {
            ui.label(
                egui::RichText::new(t(lang, I18nKey::SidebarTools))
                    .small()
                    .weak(),
            );
            if let Some(tool) = render_long_press_with_memory(
                ui,
                icon_color,
                active_icon_color,
                &tools_group,
                &active_tool,
                is_werkzeug_active,
            ) {
                events.push(AppIntent::SetEditorToolRequested { tool });
            }

            ui.add_space(6.0);
            ui.separator();
            ui.add_space(6.0);

            ui.label(
                egui::RichText::new(t(lang, I18nKey::SidebarBasics))
                    .small()
                    .weak(),
            );
            if let Some(index) = render_long_press_with_memory(
                ui,
                icon_color,
                active_icon_color,
                &basic_commands_group,
                &state.editor.last_basic_command_index,
                is_basic_command_active,
            ) {
                push_route_tool_selection(&mut events, index);
            }

            ui.add_space(6.0);
            ui.separator();
            ui.add_space(6.0);

            ui.label(
                egui::RichText::new(t(lang, I18nKey::SidebarEdit))
                    .small()
                    .weak(),
            );
            if let Some(index) = render_long_press_with_memory(
                ui,
                icon_color,
                active_icon_color,
                &section_tools_group,
                &state.editor.last_section_tool_index,
                is_section_active,
            ) {
                push_route_tool_selection(&mut events, index);
            }

            ui.add_space(6.0);
            ui.separator();
            ui.add_space(6.0);

            ui.label(
                egui::RichText::new(t(lang, I18nKey::SidebarDirection))
                    .small()
                    .weak(),
            );
            if let Some(direction) = render_long_press_with_memory(
                ui,
                icon_color,
                active_icon_color,
                &direction_group,
                &state.editor.default_direction,
                false,
            ) {
                events.push(AppIntent::SetDefaultDirectionRequested { direction });
            }

            ui.add_space(6.0);
            ui.separator();
            ui.add_space(6.0);

            ui.label(
                egui::RichText::new(t(lang, I18nKey::SidebarPriority))
                    .small()
                    .weak(),
            );
            if let Some(priority) = render_long_press_with_memory(
                ui,
                icon_color,
                active_icon_color,
                &priority_group,
                &state.editor.default_priority,
                false,
            ) {
                events.push(AppIntent::SetDefaultPriorityRequested { priority });
            }

            ui.add_space(6.0);
            ui.separator();
            ui.add_space(2.0);

            ui.label(
                egui::RichText::new(t(lang, I18nKey::SidebarZoom))
                    .small()
                    .weak(),
            );

            // Zoom-Ziel-Auswahl via LongPressGroup
            let has_selection = state.selection.selected_node_ids.len() >= 2;
            let zoom_target_key = egui::Id::new("sidebar_last_zoom_target");
            let last_zoom_target = ctx
                .data_mut(|d| d.get_temp::<ZoomTarget>(zoom_target_key))
                .unwrap_or(ZoomTarget::FullMap);

            let zoom_target_items = [
                LongPressItem {
                    icon: egui::include_image!("../../assets/icons/icon_zoom_full_map.svg"),
                    tooltip: t(lang, I18nKey::ZoomFullMapHelp),
                    value: ZoomTarget::FullMap,
                },
                LongPressItem {
                    icon: egui::include_image!("../../assets/icons/icon_zoom_selection.svg"),
                    tooltip: t(lang, I18nKey::ZoomToSelectionHelp),
                    value: ZoomTarget::Selection,
                },
            ];
            let zoom_target_group = LongPressGroup {
                id: "zoom_ziel",
                label: t(lang, I18nKey::SidebarZoom),
                items: &zoom_target_items,
            };

            let zoom_target_key_lp = egui::Id::new(("defaults_panel_long_press", "zoom_ziel"));
            let mut lp_zoom_state = ui
                .ctx()
                .data_mut(|d| d.get_temp::<LongPressState>(zoom_target_key_lp).unwrap_or_default());

            let selected_zoom = render_long_press_button(
                ui,
                icon_color,
                active_icon_color,
                &zoom_target_group,
                &last_zoom_target,
                false,
                &mut lp_zoom_state,
            );
            ui.ctx()
                .data_mut(|d| d.insert_temp(zoom_target_key_lp, lp_zoom_state));

            if let Some(target) = selected_zoom {
                ctx.data_mut(|d| d.insert_temp(zoom_target_key, target));
                match target {
                    ZoomTarget::FullMap => events.push(AppIntent::ZoomToFitRequested),
                    ZoomTarget::Selection if has_selection => {
                        events.push(AppIntent::ZoomToSelectionBoundsRequested);
                    }
                    ZoomTarget::Selection => {}
                }
            }

            // Zoom-In / Zoom-Out Buttons
            ui.horizontal(|ui| {
                let zoom_in_img = svg_icon(
                    egui::include_image!("../../assets/icons/icon_zoom_in.svg"),
                    ICON_SIZE,
                )
                .tint(icon_color);
                if ui
                    .add(egui::Button::image(zoom_in_img))
                    .on_hover_text(t(lang, I18nKey::MenuZoomIn))
                    .clicked()
                {
                    events.push(AppIntent::ZoomInRequested);
                }
                let zoom_out_img = svg_icon(
                    egui::include_image!("../../assets/icons/icon_zoom_out.svg"),
                    ICON_SIZE,
                )
                .tint(icon_color);
                if ui
                    .add(egui::Button::image(zoom_out_img))
                    .on_hover_text(t(lang, I18nKey::MenuZoomOut))
                    .clicked()
                {
                    events.push(AppIntent::ZoomOutRequested);
                }
            });

            if state.view.background_map.is_some() {
                egui::CollapsingHeader::new(t(lang, I18nKey::SidebarBackground)).show(ui, |ui| {
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
                            t(lang, I18nKey::BackgroundHide)
                        } else {
                            t(lang, I18nKey::BackgroundShow)
                        })
                        .clicked()
                    {
                        events.push(AppIntent::ToggleBackgroundVisibility);
                    }

                    let scale = state.view.background_scale;
                    if ui
                        .button("-")
                        .on_hover_text(t(lang, I18nKey::BackgroundScaleDown))
                        .clicked()
                    {
                        events.push(AppIntent::ScaleBackground { factor: 0.5 });
                    }
                    ui.label(format!("x{scale:.2}"));
                    if ui
                        .button("+")
                        .on_hover_text(t(lang, I18nKey::BackgroundScaleUp))
                        .clicked()
                    {
                        events.push(AppIntent::ScaleBackground { factor: 2.0 });
                    }
                    if (scale - 1.0).abs() > f32::EPSILON
                        && ui
                            .button("1:1")
                            .on_hover_text(t(lang, I18nKey::BackgroundScaleReset))
                            .clicked()
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
