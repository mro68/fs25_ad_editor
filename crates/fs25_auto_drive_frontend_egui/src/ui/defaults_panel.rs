//! Linkes Sidebar-Panel fuer Werkzeuge, Defaults und Hintergrund-Controls.

use crate::app::tool_contract::RouteToolId;
use crate::app::tools::{
    route_tool_defaults_tooltip_key, route_tool_group_label_key, RouteToolGroup,
};
use crate::app::{AppIntent, AppState, ConnectionDirection, ConnectionPriority, EditorTool};
use crate::shared::{t, I18nKey};
use crate::ui::common::{
    host_active_tool_to_editor, host_default_direction_to_engine, host_default_priority_to_engine,
    host_memory_tool_for_group, host_route_tool_disabled_reason_key, host_route_tool_entries_for,
    host_route_tool_to_engine,
};
use crate::ui::icons::{
    accent_icon_color, function_icon_color, host_route_tool_icon, svg_icon, ICON_SIZE,
};
use crate::ui::long_press::{
    render_long_press_button, LongPressGroup, LongPressItem, LongPressState,
};
use fs25_auto_drive_host_bridge::{HostChromeSnapshot, HostRouteToolGroup, HostRouteToolSurface};

/// Zoom-Aktion fuer den LongPress-Button in der Zoom-Sektion.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum ZoomAction {
    /// Stufenweise hineinzoomen.
    ZoomIn,
    /// Stufenweise herauszoomen.
    ZoomOut,
    /// Gesamte Map in den Viewport einpassen.
    FullMap,
    /// Viewport auf die selektierten Nodes einpassen.
    Selection,
}

fn push_route_tool_selection(events: &mut Vec<AppIntent>, tool_id: RouteToolId) {
    events.push(AppIntent::SetEditorToolRequested {
        tool: EditorTool::Route,
    });
    events.push(AppIntent::SelectRouteToolRequested { tool_id });
}

fn tool_item(
    icon: egui::ImageSource<'static>,
    tooltip: &'static str,
    value: EditorTool,
) -> LongPressItem<EditorTool> {
    LongPressItem {
        icon,
        tooltip,
        value,
        enabled: true,
        disabled_tooltip: None,
    }
}

fn value_item<T: Clone>(
    icon: egui::ImageSource<'static>,
    tooltip: &'static str,
    value: T,
) -> LongPressItem<T> {
    LongPressItem {
        icon,
        tooltip,
        value,
        enabled: true,
        disabled_tooltip: None,
    }
}

fn route_tool_items_for_group(
    chrome: &HostChromeSnapshot,
    lang: crate::shared::Language,
    group: HostRouteToolGroup,
) -> Vec<LongPressItem<fs25_auto_drive_host_bridge::HostRouteToolId>> {
    host_route_tool_entries_for(chrome, HostRouteToolSurface::DefaultsPanel, group)
        .map(|entry| LongPressItem {
            icon: host_route_tool_icon(entry.icon_key),
            tooltip: t(
                lang,
                route_tool_defaults_tooltip_key(host_route_tool_to_engine(entry.tool)),
            ),
            value: entry.tool,
            enabled: entry.enabled,
            disabled_tooltip: entry
                .disabled_reason
                .map(|reason| t(lang, host_route_tool_disabled_reason_key(reason))),
        })
        .collect()
}

fn is_route_group_active(
    chrome: &HostChromeSnapshot,
    active_route_id: Option<fs25_auto_drive_host_bridge::HostRouteToolId>,
    group: HostRouteToolGroup,
) -> bool {
    active_route_id.is_some_and(|tool_id| {
        host_route_tool_entries_for(chrome, HostRouteToolSurface::DefaultsPanel, group)
            .any(|entry| entry.tool == tool_id)
    })
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
pub fn render_route_defaults_panel(
    ctx: &egui::Context,
    state: &AppState,
    host_chrome_snapshot: &HostChromeSnapshot,
) -> Vec<AppIntent> {
    let mut top_ui = crate::ui::common::create_top_level_ui(ctx, "route_defaults_panel_top_level");
    render_route_defaults_panel_inside(&mut top_ui, state, host_chrome_snapshot)
}

/// Rendert die linke Sidebar innerhalb eines bestehenden Top-Level-UIs.
pub(crate) fn render_route_defaults_panel_inside(
    ui_root: &mut egui::Ui,
    state: &AppState,
    host_chrome_snapshot: &HostChromeSnapshot,
) -> Vec<AppIntent> {
    let mut events = Vec::new();
    let lang = state.options.language;
    let active_tool = host_active_tool_to_editor(host_chrome_snapshot.active_tool);
    let icon_color = function_icon_color(state);
    let active_icon_color = accent_icon_color(state);

    let active_route_id = host_chrome_snapshot.active_route_tool;
    let is_werkzeug_active = matches!(
        active_tool,
        EditorTool::Select | EditorTool::Connect | EditorTool::AddNode
    );
    let tools_items = [
        tool_item(
            egui::include_image!("../../../../assets/icons/icon_select_node.svg"),
            t(lang, I18nKey::LpToolSelect),
            EditorTool::Select,
        ),
        tool_item(
            egui::include_image!("../../../../assets/icons/icon_connect.svg"),
            t(lang, I18nKey::LpToolConnect),
            EditorTool::Connect,
        ),
        tool_item(
            egui::include_image!("../../../../assets/icons/icon_add_node.svg"),
            t(lang, I18nKey::LpToolAddNode),
            EditorTool::AddNode,
        ),
    ];

    let basic_items =
        route_tool_items_for_group(host_chrome_snapshot, lang, HostRouteToolGroup::Basics);
    let section_items =
        route_tool_items_for_group(host_chrome_snapshot, lang, HostRouteToolGroup::Section);
    let analysis_items =
        route_tool_items_for_group(host_chrome_snapshot, lang, HostRouteToolGroup::Analysis);

    let direction_items = [
        value_item(
            egui::include_image!("../../../../assets/icons/icon_direction_regular.svg"),
            t(lang, I18nKey::LpDirectionRegular),
            ConnectionDirection::Regular,
        ),
        value_item(
            egui::include_image!("../../../../assets/icons/icon_direction_dual.svg"),
            t(lang, I18nKey::LpDirectionDual),
            ConnectionDirection::Dual,
        ),
        value_item(
            egui::include_image!("../../../../assets/icons/icon_direction_reverse.svg"),
            t(lang, I18nKey::LpDirectionReverse),
            ConnectionDirection::Reverse,
        ),
    ];

    let priority_items = [
        value_item(
            egui::include_image!("../../../../assets/icons/icon_priority_main.svg"),
            t(lang, I18nKey::LpPriorityMain),
            ConnectionPriority::Regular,
        ),
        value_item(
            egui::include_image!("../../../../assets/icons/icon_priority_side.svg"),
            t(lang, I18nKey::LpPrioritySub),
            ConnectionPriority::SubPriority,
        ),
    ];

    let tools_group = LongPressGroup {
        id: "werkzeuge",
        label: t(lang, I18nKey::SidebarTools),
        items: &tools_items,
    };

    let basic_commands_group = LongPressGroup {
        id: "grundbefehle",
        label: t(lang, route_tool_group_label_key(RouteToolGroup::Basics)),
        items: basic_items.as_slice(),
    };

    let section_tools_group = LongPressGroup {
        id: "tools_abschnitt",
        label: t(lang, route_tool_group_label_key(RouteToolGroup::Section)),
        items: section_items.as_slice(),
    };

    let analysis_tools_group = LongPressGroup {
        id: "tools_analysis",
        label: t(lang, route_tool_group_label_key(RouteToolGroup::Analysis)),
        items: analysis_items.as_slice(),
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

    egui::Panel::left("route_defaults_panel")
        .resizable(false)
        .default_size(80.0)
        .show_inside(ui_root, |ui| {
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
            let basic_tool = host_memory_tool_for_group(
                host_chrome_snapshot.route_tool_memory,
                HostRouteToolGroup::Basics,
            );
            if let Some(tool_id) = render_long_press_with_memory(
                ui,
                icon_color,
                active_icon_color,
                &basic_commands_group,
                &basic_tool,
                is_route_group_active(
                    host_chrome_snapshot,
                    active_route_id,
                    HostRouteToolGroup::Basics,
                ),
            ) {
                push_route_tool_selection(&mut events, host_route_tool_to_engine(tool_id));
            }

            ui.add_space(6.0);
            ui.separator();
            ui.add_space(6.0);

            ui.label(
                egui::RichText::new(t(lang, I18nKey::SidebarEdit))
                    .small()
                    .weak(),
            );
            let section_tool = host_memory_tool_for_group(
                host_chrome_snapshot.route_tool_memory,
                HostRouteToolGroup::Section,
            );
            if let Some(tool_id) = render_long_press_with_memory(
                ui,
                icon_color,
                active_icon_color,
                &section_tools_group,
                &section_tool,
                is_route_group_active(
                    host_chrome_snapshot,
                    active_route_id,
                    HostRouteToolGroup::Section,
                ),
            ) {
                push_route_tool_selection(&mut events, host_route_tool_to_engine(tool_id));
            }

            ui.add_space(6.0);
            ui.separator();
            ui.add_space(6.0);

            ui.label(
                egui::RichText::new(t(lang, I18nKey::SidebarAnalysis))
                    .small()
                    .weak(),
            );
            let analysis_tool = host_memory_tool_for_group(
                host_chrome_snapshot.route_tool_memory,
                HostRouteToolGroup::Analysis,
            );
            if let Some(tool_id) = render_long_press_with_memory(
                ui,
                icon_color,
                active_icon_color,
                &analysis_tools_group,
                &analysis_tool,
                is_route_group_active(
                    host_chrome_snapshot,
                    active_route_id,
                    HostRouteToolGroup::Analysis,
                ),
            ) {
                push_route_tool_selection(&mut events, host_route_tool_to_engine(tool_id));
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
                &host_default_direction_to_engine(host_chrome_snapshot.default_direction),
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
                &host_default_priority_to_engine(host_chrome_snapshot.default_priority),
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

            // Zoom-Aktion via LongPressGroup (4 Items)
            let has_selection = host_chrome_snapshot.has_selection;
            let zoom_action_key = egui::Id::new("sidebar_last_zoom_action");
            let last_zoom_action = ui
                .ctx()
                .data_mut(|d| d.get_temp::<ZoomAction>(zoom_action_key))
                .unwrap_or(ZoomAction::FullMap);

            let zoom_items = [
                value_item(
                    egui::include_image!("../../../../assets/icons/icon_zoom_in.svg"),
                    t(lang, I18nKey::ZoomInHelp),
                    ZoomAction::ZoomIn,
                ),
                value_item(
                    egui::include_image!("../../../../assets/icons/icon_zoom_out.svg"),
                    t(lang, I18nKey::ZoomOutHelp),
                    ZoomAction::ZoomOut,
                ),
                value_item(
                    egui::include_image!("../../../../assets/icons/icon_zoom_full_map.svg"),
                    t(lang, I18nKey::ZoomFullMapHelp),
                    ZoomAction::FullMap,
                ),
                LongPressItem {
                    icon: egui::include_image!("../../../../assets/icons/icon_zoom_selection.svg"),
                    tooltip: t(lang, I18nKey::ZoomToSelectionHelp),
                    value: ZoomAction::Selection,
                    enabled: has_selection,
                    disabled_tooltip: Some(t(lang, I18nKey::ZoomToSelectionHelp)),
                },
            ];
            let zoom_group = LongPressGroup {
                id: "zoom_ziel",
                label: t(lang, I18nKey::SidebarZoom),
                items: &zoom_items,
            };

            if let Some(action) = render_long_press_with_memory(
                ui,
                icon_color,
                active_icon_color,
                &zoom_group,
                &last_zoom_action,
                false,
            ) {
                ui.ctx()
                    .data_mut(|d| d.insert_temp(zoom_action_key, action));
                match action {
                    ZoomAction::ZoomIn => events.push(AppIntent::ZoomInRequested),
                    ZoomAction::ZoomOut => events.push(AppIntent::ZoomOutRequested),
                    ZoomAction::FullMap => events.push(AppIntent::ZoomToFitRequested),
                    ZoomAction::Selection if has_selection => {
                        events.push(AppIntent::ZoomToSelectionBoundsRequested);
                    }
                    ZoomAction::Selection => {}
                }
            }

            if state.view.background_map.is_some() {
                egui::CollapsingHeader::new(t(lang, I18nKey::SidebarBackground)).show(ui, |ui| {
                    let visible = state.view.background_visible;
                    let toggle_icon = if visible {
                        egui::include_image!("../../../../assets/icons/icon_visible.svg")
                    } else {
                        egui::include_image!("../../../../assets/icons/icon_hidden.svg")
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
