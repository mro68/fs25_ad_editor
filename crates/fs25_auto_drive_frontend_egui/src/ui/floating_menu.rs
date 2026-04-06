//! Schwebendes Kontextmenue fuer Werkzeuggruppen an der Mausposition.

use crate::app::state::FloatingMenuKind;
use crate::app::tools::route_tool_label_key;
use crate::app::{AppIntent, AppState, ConnectionDirection, ConnectionPriority, EditorTool};
use crate::shared::{t, I18nKey};
use crate::ui::common::{
    host_active_tool_to_editor, host_route_tool_disabled_reason_key, host_route_tool_entries_for,
    host_route_tool_to_engine,
};
use crate::ui::icons::{
    accent_icon_color, function_icon_color, host_route_tool_icon, svg_icon, ICON_SIZE,
};
use fs25_auto_drive_host_bridge::{
    HostChromeSnapshot, HostRouteToolGroup, HostRouteToolIconKey, HostRouteToolSurface,
};

#[derive(Clone, Copy)]
struct IconButtonColors {
    inactive: egui::Color32,
    active: egui::Color32,
}

#[derive(Clone, Copy)]
struct IconButtonConfig {
    tooltip: &'static str,
    is_active: bool,
    enabled: bool,
    disabled_tooltip: Option<&'static str>,
    colors: IconButtonColors,
}

impl IconButtonConfig {
    fn render(self, ui: &mut egui::Ui, icon: egui::ImageSource<'static>) -> bool {
        let image = svg_icon(icon, ICON_SIZE).tint(if self.is_active {
            self.colors.active
        } else {
            self.colors.inactive
        });

        let response = ui.add_enabled(
            self.enabled,
            egui::Button::image(image).selected(self.is_active),
        );
        if self.enabled {
            response.on_hover_text(self.tooltip).clicked()
        } else {
            response
                .on_disabled_hover_text(self.disabled_tooltip.unwrap_or(self.tooltip))
                .clicked()
        }
    }
}

/// Rendert ein schwebendes Menue an der gespeicherten Position.
/// Gibt `AppIntent`s zurueck, wenn ein Menueeintrag geklickt wurde.
/// Der boolesche Rueckgabewert signalisiert, ob das Menue geschlossen werden soll.
pub fn render_floating_menu(
    ctx: &egui::Context,
    state: &AppState,
    host_chrome_snapshot: &HostChromeSnapshot,
) -> (Vec<AppIntent>, bool) {
    let Some(menu) = state.ui.floating_menu else {
        return (vec![], false);
    };

    let mut events = Vec::new();
    let lang = state.options.language;
    let active_tool = host_active_tool_to_editor(host_chrome_snapshot.active_tool);
    let active_route_id = host_chrome_snapshot.active_route_tool;
    let has_selection = host_chrome_snapshot.has_selection;

    let icon_color = function_icon_color(state);
    let active_icon_color = accent_icon_color(state);
    let button_colors = IconButtonColors {
        inactive: icon_color,
        active: active_icon_color,
    };
    let menu_pos = egui::pos2(menu.pos.x, menu.pos.y);

    let area_response = egui::Area::new(egui::Id::new(("floating_menu", menu.kind)))
        .order(egui::Order::Foreground)
        .fixed_pos(menu_pos)
        .show(ctx, |ui| {
            egui::Frame::popup(ui.style()).show(ui, |ui| {
                ui.horizontal_wrapped(|ui| match menu.kind {
                    FloatingMenuKind::Tools => {
                        if tool_icon_button(
                            ui,
                            egui::include_image!("../../../../assets/icons/icon_select_node.svg"),
                            IconButtonConfig {
                                tooltip: t(lang, I18nKey::FloatingToolSelect),
                                is_active: active_tool == EditorTool::Select,
                                enabled: true,
                                disabled_tooltip: None,
                                colors: button_colors,
                            },
                        ) {
                            events.push(AppIntent::SetEditorToolRequested {
                                tool: EditorTool::Select,
                            });
                        }

                        if tool_icon_button(
                            ui,
                            egui::include_image!("../../../../assets/icons/icon_connect.svg"),
                            IconButtonConfig {
                                tooltip: t(lang, I18nKey::FloatingToolConnect),
                                is_active: active_tool == EditorTool::Connect,
                                enabled: true,
                                disabled_tooltip: None,
                                colors: button_colors,
                            },
                        ) {
                            events.push(AppIntent::SetEditorToolRequested {
                                tool: EditorTool::Connect,
                            });
                        }

                        if tool_icon_button(
                            ui,
                            egui::include_image!("../../../../assets/icons/icon_add_node.svg"),
                            IconButtonConfig {
                                tooltip: t(lang, I18nKey::FloatingToolAddNode),
                                is_active: active_tool == EditorTool::AddNode,
                                enabled: true,
                                disabled_tooltip: None,
                                colors: button_colors,
                            },
                        ) {
                            events.push(AppIntent::SetEditorToolRequested {
                                tool: EditorTool::AddNode,
                            });
                        }
                    }
                    FloatingMenuKind::RouteTools(group) => {
                        let host_group = match group {
                            crate::app::tools::RouteToolGroup::Basics => HostRouteToolGroup::Basics,
                            crate::app::tools::RouteToolGroup::Section => {
                                HostRouteToolGroup::Section
                            }
                            crate::app::tools::RouteToolGroup::Analysis => {
                                HostRouteToolGroup::Analysis
                            }
                        };

                        for entry in host_route_tool_entries_for(
                            host_chrome_snapshot,
                            HostRouteToolSurface::FloatingMenu,
                            host_group,
                        ) {
                            let engine_tool_id = host_route_tool_to_engine(entry.tool);
                            if route_icon_button(
                                ui,
                                entry.icon_key,
                                IconButtonConfig {
                                    tooltip: t(lang, route_tool_label_key(engine_tool_id)),
                                    is_active: active_route_id == Some(entry.tool),
                                    enabled: entry.enabled,
                                    disabled_tooltip: entry.disabled_reason.map(|reason| {
                                        t(lang, host_route_tool_disabled_reason_key(reason))
                                    }),
                                    colors: button_colors,
                                },
                            ) {
                                events.push(AppIntent::SelectRouteToolRequested {
                                    tool_id: engine_tool_id,
                                });
                            }
                        }
                    }
                    FloatingMenuKind::DirectionPriority => {
                        for (direction, icon, tooltip) in [
                            (
                                ConnectionDirection::Regular,
                                egui::include_image!(
                                    "../../../../assets/icons/icon_direction_regular.svg"
                                ),
                                t(lang, I18nKey::FloatingDirectionRegular),
                            ),
                            (
                                ConnectionDirection::Dual,
                                egui::include_image!(
                                    "../../../../assets/icons/icon_direction_dual.svg"
                                ),
                                t(lang, I18nKey::FloatingDirectionDual),
                            ),
                            (
                                ConnectionDirection::Reverse,
                                egui::include_image!(
                                    "../../../../assets/icons/icon_direction_reverse.svg"
                                ),
                                t(lang, I18nKey::FloatingDirectionReverse),
                            ),
                        ] {
                            if tool_icon_button(
                                ui,
                                icon,
                                IconButtonConfig {
                                    tooltip,
                                    is_active: false,
                                    enabled: true,
                                    disabled_tooltip: None,
                                    colors: button_colors,
                                },
                            ) {
                                events.push(AppIntent::SetDefaultDirectionRequested { direction });
                            }
                        }
                        for (priority, icon, tooltip) in [
                            (
                                ConnectionPriority::Regular,
                                egui::include_image!(
                                    "../../../../assets/icons/icon_priority_main.svg"
                                ),
                                t(lang, I18nKey::FloatingPriorityMain),
                            ),
                            (
                                ConnectionPriority::SubPriority,
                                egui::include_image!(
                                    "../../../../assets/icons/icon_priority_side.svg"
                                ),
                                t(lang, I18nKey::FloatingPrioritySub),
                            ),
                        ] {
                            if tool_icon_button(
                                ui,
                                icon,
                                IconButtonConfig {
                                    tooltip,
                                    is_active: false,
                                    enabled: true,
                                    disabled_tooltip: None,
                                    colors: button_colors,
                                },
                            ) {
                                events.push(AppIntent::SetDefaultPriorityRequested { priority });
                            }
                        }
                    }
                    FloatingMenuKind::Zoom => {
                        if tool_icon_button(
                            ui,
                            egui::include_image!("../../../../assets/icons/icon_zoom_full_map.svg"),
                            IconButtonConfig {
                                tooltip: t(lang, I18nKey::FloatingZoomFullMap),
                                is_active: false,
                                enabled: true,
                                disabled_tooltip: None,
                                colors: button_colors,
                            },
                        ) {
                            events.push(AppIntent::ZoomToFitRequested);
                        }
                        if tool_icon_button(
                            ui,
                            egui::include_image!(
                                "../../../../assets/icons/icon_zoom_selection.svg"
                            ),
                            IconButtonConfig {
                                tooltip: t(lang, I18nKey::FloatingZoomSelection),
                                is_active: false,
                                enabled: has_selection,
                                disabled_tooltip: Some(t(lang, I18nKey::FloatingZoomSelection)),
                                colors: button_colors,
                            },
                        ) {
                            events.push(AppIntent::ZoomToSelectionBoundsRequested);
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
        return (events, true);
    }

    (events, false)
}

fn tool_icon_button(
    ui: &mut egui::Ui,
    icon: egui::ImageSource<'static>,
    config: IconButtonConfig,
) -> bool {
    config.render(ui, icon)
}

fn route_icon_button(
    ui: &mut egui::Ui,
    icon_key: HostRouteToolIconKey,
    config: IconButtonConfig,
) -> bool {
    config.render(ui, host_route_tool_icon(icon_key))
}
