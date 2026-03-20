//! Schwebendes Kontextmenue fuer Werkzeuggruppen an der Mausposition.

use crate::app::segment_registry::{
    TOOL_INDEX_BYPASS, TOOL_INDEX_CURVE_CUBIC, TOOL_INDEX_CURVE_QUAD, TOOL_INDEX_PARKING,
    TOOL_INDEX_ROUTE_OFFSET, TOOL_INDEX_SMOOTH_CURVE, TOOL_INDEX_SPLINE, TOOL_INDEX_STRAIGHT,
};
use crate::app::state::FloatingMenuKind;
use crate::app::{AppIntent, AppState, ConnectionDirection, ConnectionPriority, EditorTool};
use crate::shared::{t, I18nKey};
use crate::ui::icons::{
    accent_icon_color, function_icon_color, route_tool_icon, svg_icon, ICON_SIZE,
};

/// Rendert ein schwebendes Menue an der gespeicherten Position.
/// Gibt `AppIntent`s zurueck, wenn ein Menueeintrag geklickt wurde.
/// Der boolesche Rueckgabewert signalisiert, ob das Menue geschlossen werden soll.
pub fn render_floating_menu(ctx: &egui::Context, state: &AppState) -> (Vec<AppIntent>, bool) {
    let Some(menu) = state.ui.floating_menu else {
        return (vec![], false);
    };

    let mut events = Vec::new();
    let lang = state.options.language;
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
                            t(lang, I18nKey::FloatingToolSelect),
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
                            t(lang, I18nKey::FloatingToolConnect),
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
                            t(lang, I18nKey::FloatingToolAddNode),
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
                            (TOOL_INDEX_STRAIGHT, t(lang, I18nKey::FloatingBasicStraight)),
                            (
                                TOOL_INDEX_CURVE_QUAD,
                                t(lang, I18nKey::FloatingBasicQuadratic),
                            ),
                            (TOOL_INDEX_CURVE_CUBIC, t(lang, I18nKey::FloatingBasicCubic)),
                            (TOOL_INDEX_SPLINE, t(lang, I18nKey::FloatingBasicSpline)),
                            (
                                TOOL_INDEX_SMOOTH_CURVE,
                                t(lang, I18nKey::FloatingBasicSmoothCurve),
                            ),
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
                            (TOOL_INDEX_BYPASS, t(lang, I18nKey::FloatingEditBypass)),
                            (TOOL_INDEX_PARKING, t(lang, I18nKey::FloatingEditParking)),
                            (
                                TOOL_INDEX_ROUTE_OFFSET,
                                t(lang, I18nKey::FloatingEditRouteOffset),
                            ),
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
                    FloatingMenuKind::DirectionPriority => {
                        for (direction, icon, tooltip) in [
                            (
                                ConnectionDirection::Regular,
                                egui::include_image!(
                                    "../../assets/icons/icon_direction_regular.svg"
                                ),
                                t(lang, I18nKey::FloatingDirectionRegular),
                            ),
                            (
                                ConnectionDirection::Dual,
                                egui::include_image!("../../assets/icons/icon_direction_dual.svg"),
                                t(lang, I18nKey::FloatingDirectionDual),
                            ),
                            (
                                ConnectionDirection::Reverse,
                                egui::include_image!(
                                    "../../assets/icons/icon_direction_reverse.svg"
                                ),
                                t(lang, I18nKey::FloatingDirectionReverse),
                            ),
                        ] {
                            if tool_icon_button(
                                ui,
                                icon,
                                tooltip,
                                false,
                                icon_color,
                                active_icon_color,
                            ) {
                                events.push(AppIntent::SetDefaultDirectionRequested { direction });
                            }
                        }
                        for (priority, icon, tooltip) in [
                            (
                                ConnectionPriority::Regular,
                                egui::include_image!("../../assets/icons/icon_priority_main.svg"),
                                t(lang, I18nKey::FloatingPriorityMain),
                            ),
                            (
                                ConnectionPriority::SubPriority,
                                egui::include_image!("../../assets/icons/icon_priority_side.svg"),
                                t(lang, I18nKey::FloatingPrioritySub),
                            ),
                        ] {
                            if tool_icon_button(
                                ui,
                                icon,
                                tooltip,
                                false,
                                icon_color,
                                active_icon_color,
                            ) {
                                events.push(AppIntent::SetDefaultPriorityRequested { priority });
                            }
                        }
                    }
                    FloatingMenuKind::Zoom => {
                        if tool_icon_button(
                            ui,
                            egui::include_image!("../../assets/icons/icon_zoom_full_map.svg"),
                            t(lang, I18nKey::FloatingZoomFullMap),
                            false,
                            icon_color,
                            active_icon_color,
                        ) {
                            events.push(AppIntent::ZoomToFitRequested);
                        }
                        if tool_icon_button(
                            ui,
                            egui::include_image!("../../assets/icons/icon_zoom_selection.svg"),
                            t(lang, I18nKey::FloatingZoomSelection),
                            false,
                            icon_color,
                            active_icon_color,
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
