//! Linkes Sidebar-Panel fuer Werkzeuge, Defaults und Hintergrund-Controls.

use crate::app::segment_registry::{
    TOOL_INDEX_BYPASS, TOOL_INDEX_CURVE_CUBIC, TOOL_INDEX_CURVE_QUAD, TOOL_INDEX_PARKING,
    TOOL_INDEX_ROUTE_OFFSET, TOOL_INDEX_SMOOTH_CURVE, TOOL_INDEX_SPLINE, TOOL_INDEX_STRAIGHT,
};
use crate::app::{AppIntent, AppState, ConnectionDirection, ConnectionPriority, EditorTool};
use crate::ui::icons::{
    accent_icon_color, function_icon_color, route_tool_icon, svg_icon, ICON_SIZE,
};
use crate::ui::long_press::{
    render_long_press_button, LongPressGroup, LongPressItem, LongPressState,
};

#[derive(Debug, Clone, Copy)]
enum RouteGroup {
    Straight,
    Curve,
    Section,
}

fn route_group_label(group: RouteGroup) -> &'static str {
    match group {
        RouteGroup::Straight => "Geraden",
        RouteGroup::Curve => "Kurven",
        RouteGroup::Section => "Tools",
    }
}

fn push_route_tool_selection(events: &mut Vec<AppIntent>, _group: RouteGroup, index: usize) {
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
    let is_straight_active = active_route_index == Some(TOOL_INDEX_STRAIGHT);
    let is_curve_active = matches!(
        active_route_index,
        Some(i) if i == TOOL_INDEX_CURVE_QUAD || i == TOOL_INDEX_CURVE_CUBIC || i == TOOL_INDEX_SPLINE || i == TOOL_INDEX_SMOOTH_CURVE
    );
    let is_section_active = matches!(
        active_route_index,
        Some(i) if i == TOOL_INDEX_BYPASS || i == TOOL_INDEX_PARKING || i == TOOL_INDEX_ROUTE_OFFSET
    );

    let tools_items = [
        LongPressItem {
            icon: egui::include_image!("../../assets/icons/icon_select_node.svg"),
            tooltip: "Auswahl (Taste 1)\nNodes per Klick oder Lasso selektieren.\nSelektierte Nodes per Drag verschieben.",
            value: EditorTool::Select,
        },
        LongPressItem {
            icon: egui::include_image!("../../assets/icons/icon_connect.svg"),
            tooltip: "Verbinden (Taste 2)\nVerbindung zwischen 2 Nodes erstellen (Taste C)\noder loeschen (Taste X). Pfeilrichtung = Fahrtrichtung.",
            value: EditorTool::Connect,
        },
        LongPressItem {
            icon: egui::include_image!("../../assets/icons/icon_add_node.svg"),
            tooltip: "Node hinzufuegen (Taste 3)\nNeuen Wegpunkt per Klick in die Karte setzen.",
            value: EditorTool::AddNode,
        },
    ];

    let straights_items = [LongPressItem {
        icon: route_tool_icon(TOOL_INDEX_STRAIGHT),
        tooltip: "Gerade Strecke (G)\nNodes mit gleichmaessigem Abstand entlang einer Linie platzieren.\nPfeiltasten: Abstand/Anzahl anpassen. Enter: Bestaetigen.",
        value: TOOL_INDEX_STRAIGHT,
    }];

    let curves_items = [
        LongPressItem {
            icon: route_tool_icon(TOOL_INDEX_CURVE_QUAD),
            tooltip: "Bezier-Kurve quadratisch (G)\n1 Kontrollpunkt. Einfache, gleichmaessige Kurve.",
            value: TOOL_INDEX_CURVE_QUAD,
        },
        LongPressItem {
            icon: route_tool_icon(TOOL_INDEX_CURVE_CUBIC),
            tooltip: "Bezier-Kurve kubisch (G)\n2 Kontrollpunkte fuer maximale Formkontrolle.",
            value: TOOL_INDEX_CURVE_CUBIC,
        },
        LongPressItem {
            icon: route_tool_icon(TOOL_INDEX_SPLINE),
            tooltip: "Catmull-Rom Spline (G)\nGlatte Kurve durch existierende Nodes.\nZusaetzliche Zwischenpunkte werden berechnet.",
            value: TOOL_INDEX_SPLINE,
        },
        LongPressItem {
            icon: route_tool_icon(TOOL_INDEX_SMOOTH_CURVE),
            tooltip: "Geglättete Kurve (G)\nWinkelgeglaettete Strecke zwischen zwei Nodes.\nAutomatische Ausrichtung an Strassenrasterwinkeln.",
            value: TOOL_INDEX_SMOOTH_CURVE,
        },
    ];

    let section_tools_items = [
        LongPressItem {
            icon: route_tool_icon(TOOL_INDEX_BYPASS),
            tooltip: "Ausweichstrecke (S)\nErzeugt eine parallele Umgehungsstrecke zur Selektion.",
            value: TOOL_INDEX_BYPASS,
        },
        LongPressItem {
            icon: route_tool_icon(TOOL_INDEX_PARKING),
            tooltip: "Parkplatz (S)\nGeneriert ein Parkplatz-Layout aus der Selektion.",
            value: TOOL_INDEX_PARKING,
        },
        LongPressItem {
            icon: route_tool_icon(TOOL_INDEX_ROUTE_OFFSET),
            tooltip: "Strecke versetzen (S)\nVerschiebt die selektierte Route parallel um einen konfigurierbaren Abstand.",
            value: TOOL_INDEX_ROUTE_OFFSET,
        },
    ];

    let direction_items = [
        LongPressItem {
            icon: egui::include_image!("../../assets/icons/icon_direction_regular.svg"),
            tooltip: "Einbahn vorwaerts\nFahrzeuge nutzen diese Verbindungen nur in Vorwaertsrichtung.\nStandard fuer normale gerichtete Strecken.",
            value: ConnectionDirection::Regular,
        },
        LongPressItem {
            icon: egui::include_image!("../../assets/icons/icon_direction_dual.svg"),
            tooltip: "Zweirichtung\nFahrzeuge koennen in beide Richtungen fahren.\nFuer Wege die in beiden Richtungen befahrbar sein sollen.",
            value: ConnectionDirection::Dual,
        },
        LongPressItem {
            icon: egui::include_image!("../../assets/icons/icon_direction_reverse.svg"),
            tooltip: "Einbahn rueckwaerts\nFahrzeuge fahren ausschliesslich rueckwaerts auf dieser Strecke.",
            value: ConnectionDirection::Reverse,
        },
    ];

    let priority_items = [
        LongPressItem {
            icon: egui::include_image!("../../assets/icons/icon_priority_main.svg"),
            tooltip: "Hauptstrasse\nHohe Prioritaet. AutoDrive bevorzugt diese Strecken.\nFuer Hauptverbindungen und viel genutzte Wege.",
            value: ConnectionPriority::Regular,
        },
        LongPressItem {
            icon: egui::include_image!("../../assets/icons/icon_priority_side.svg"),
            tooltip: "Nebenstrasse\nNiedrigere Prioritaet. Nur bei Bedarf genutzt.\nFuer Feldwege und selten befahrene Verbindungen.",
            value: ConnectionPriority::SubPriority,
        },
    ];

    let tools_group = LongPressGroup {
        id: "werkzeuge",
        label: "Werkzeuge",
        items: &tools_items,
    };

    let straights_group = LongPressGroup {
        id: "grundbefehle_geraden",
        label: route_group_label(RouteGroup::Straight),
        items: &straights_items,
    };

    let curves_group = LongPressGroup {
        id: "grundbefehle_kurven",
        label: route_group_label(RouteGroup::Curve),
        items: &curves_items,
    };

    let section_tools_group = LongPressGroup {
        id: "tools_abschnitt",
        label: route_group_label(RouteGroup::Section),
        items: &section_tools_items,
    };

    let direction_group = LongPressGroup {
        id: "defaults_richtung",
        label: "Voreinstellung Richtung",
        items: &direction_items,
    };

    let priority_group = LongPressGroup {
        id: "defaults_prioritaet",
        label: "Voreinstellung Strassenart",
        items: &priority_items,
    };

    egui::SidePanel::left("route_defaults_panel")
        .resizable(false)
        .default_width(80.0)
        .show(ctx, |ui| {
            ui.label(egui::RichText::new("Werkzeuge").small().weak());
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

            ui.label(egui::RichText::new("Grundbefehle").small().weak());
            if let Some(index) = render_long_press_with_memory(
                ui,
                icon_color,
                active_icon_color,
                &straights_group,
                &state.editor.last_straight_index,
                is_straight_active,
            ) {
                push_route_tool_selection(&mut events, RouteGroup::Straight, index);
            }

            if let Some(index) = render_long_press_with_memory(
                ui,
                icon_color,
                active_icon_color,
                &curves_group,
                &state.editor.last_curve_index,
                is_curve_active,
            ) {
                push_route_tool_selection(&mut events, RouteGroup::Curve, index);
            }

            ui.add_space(6.0);
            ui.separator();
            ui.add_space(6.0);

            ui.label(egui::RichText::new("Bearbeiten").small().weak());
            if let Some(index) = render_long_press_with_memory(
                ui,
                icon_color,
                active_icon_color,
                &section_tools_group,
                &state.editor.last_section_tool_index,
                is_section_active,
            ) {
                push_route_tool_selection(&mut events, RouteGroup::Section, index);
            }

            ui.add_space(6.0);
            ui.separator();
            ui.add_space(6.0);

            ui.label(egui::RichText::new("Richtung").small().weak());
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

            ui.label(egui::RichText::new("Strassenart").small().weak());
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
