//! Kontextmenü-System mit validierter Command-Architektur.
//!
//! Garantie: Nur Commands mit erfüllten Preconditions werden gerendert.
//!
//! Struktur:
//! - `commands.rs`: CommandId, Precondition, MenuCatalog, validate_entries()
//! - `mod.rs`: MenuVariant, determine_menu_variant(), render_context_menu()
//! - `empty_area.rs`, `single_node.rs`, `multiple_nodes.rs`, `route_tool.rs`: Legacy (nicht mehr verwendet)

pub mod commands;

use crate::app::tools::common::TangentMenuData;
use crate::app::{AppIntent, ConnectionDirection, ConnectionPriority, RoadMap};
use crate::shared::EditorOptions;
use commands::{
    validate_entries, CommandId, IntentContext, MenuCatalog, PreconditionContext, ValidatedEntry,
};
use indexmap::IndexSet;

/// Icon-Größe für SVG-Icons im Kontextmenü.
const CM_ICON_SIZE: egui::Vec2 = egui::Vec2::new(16.0, 16.0);
const CM_CHOICE_ICON_SIZE: egui::Vec2 = egui::Vec2::new(32.0, 32.0);

fn is_direction_or_priority(id: CommandId) -> bool {
    matches!(
        id,
        CommandId::DirectionRegular
            | CommandId::DirectionDual
            | CommandId::DirectionReverse
            | CommandId::PriorityRegular
            | CommandId::PrioritySub
    )
}

fn direction_or_priority_tooltip(id: CommandId) -> &'static str {
    match id {
        CommandId::DirectionRegular => "Einbahn vorwaerts",
        CommandId::DirectionDual => "Zweirichtungsverkehr",
        CommandId::DirectionReverse => "Einbahn rueckwaerts",
        CommandId::PriorityRegular => "Hauptstrasse",
        CommandId::PrioritySub => "Nebenstrasse",
        _ => "",
    }
}

fn color32_from_rgba(color: [f32; 4]) -> egui::Color32 {
    egui::Color32::from_rgba_unmultiplied(
        (color[0].clamp(0.0, 1.0) * 255.0) as u8,
        (color[1].clamp(0.0, 1.0) * 255.0) as u8,
        (color[2].clamp(0.0, 1.0) * 255.0) as u8,
        (color[3].clamp(0.0, 1.0) * 255.0) as u8,
    )
}

fn function_icon_color(options: &EditorOptions, priority: ConnectionPriority) -> egui::Color32 {
    match priority {
        ConnectionPriority::Regular => color32_from_rgba(options.connection_color_regular),
        ConnectionPriority::SubPriority => color32_from_rgba(options.node_color_subprio),
    }
}

fn direction_icon_color(options: &EditorOptions, direction: ConnectionDirection) -> egui::Color32 {
    match direction {
        ConnectionDirection::Regular => color32_from_rgba(options.connection_color_regular),
        ConnectionDirection::Dual => color32_from_rgba(options.connection_color_dual),
        ConnectionDirection::Reverse => color32_from_rgba(options.connection_color_reverse),
    }
}

/// Gibt das SVG-Icon für einen Command zurück (dieselben wie in der Toolbar).
fn command_icon(
    id: CommandId,
    options: &EditorOptions,
    default_direction: ConnectionDirection,
    default_priority: ConnectionPriority,
) -> Option<egui::Image<'static>> {
    let source: egui::ImageSource<'static> = match id {
        CommandId::SetToolSelect => {
            egui::include_image!("../../../assets/icon_select_node.svg")
        }
        CommandId::SetToolConnect => {
            egui::include_image!("../../../assets/icon_connect.svg")
        }
        CommandId::SetToolAddNode => {
            egui::include_image!("../../../assets/icon_add_node.svg")
        }
        CommandId::SetToolRouteStraight | CommandId::RouteStraight => {
            egui::include_image!("../../../assets/icon_straight_road.svg")
        }
        CommandId::SetToolRouteConstraint | CommandId::RouteConstraint => {
            egui::include_image!("../../../assets/icon_constraint_route.svg")
        }
        CommandId::SetToolRouteQuadratic | CommandId::RouteQuadratic => {
            egui::include_image!("../../../assets/icon_bezier_quadratic.svg")
        }
        CommandId::SetToolRouteCubic | CommandId::RouteCubic => {
            egui::include_image!("../../../assets/icon_bezier_cubic.svg")
        }
        CommandId::DirectionRegular => {
            egui::include_image!("../../../assets/icon_direction_regular.svg")
        }
        CommandId::DirectionDual => {
            egui::include_image!("../../../assets/icon_direction_dual.svg")
        }
        CommandId::DirectionReverse => {
            egui::include_image!("../../../assets/icon_direction_reverse.svg")
        }
        CommandId::PriorityRegular => {
            egui::include_image!("../../../assets/icon_priority_main.svg")
        }
        CommandId::PrioritySub => {
            egui::include_image!("../../../assets/icon_priority_side.svg")
        }
        _ => return None,
    };

    let tint = match id {
        CommandId::DirectionRegular => direction_icon_color(options, ConnectionDirection::Regular),
        CommandId::DirectionDual => direction_icon_color(options, ConnectionDirection::Dual),
        CommandId::DirectionReverse => direction_icon_color(options, ConnectionDirection::Reverse),
        CommandId::PriorityRegular => function_icon_color(options, ConnectionPriority::Regular),
        CommandId::PrioritySub => function_icon_color(options, ConnectionPriority::SubPriority),
        _ => {
            let _accent = direction_icon_color(options, default_direction);
            function_icon_color(options, default_priority)
        }
    };

    Some(
        egui::Image::new(source)
            .fit_to_exact_size(CM_ICON_SIZE)
            .tint(tint),
    )
}

/// Kontextabhängige Menü-Variante basierend auf Selection und Fokus-Node.
///
/// Wird beim Rechtsklick einmalig bestimmt und eingefroren, bis das Menü
/// geschlossen wird. Enthält alle Daten die zum Rendern nötig sind.
#[derive(Debug, Clone)]
pub enum MenuVariant {
    /// Rechtsklick auf leeren Bereich ohne Selektion → Tool-Auswahl
    EmptyArea,
    /// Nodes selektiert, Rechtsklick auf leeren Bereich → Befehle für Selektion
    SelectionOnly,
    /// Rechtsklick auf spezifischen Node → Einzelnode-Befehle oben + Selektions-Befehle unten
    NodeFocused {
        /// Der fokussierte Node (unter Mausposition)
        focused_node_id: u64,
    },
    /// Route-Tool aktiv mit pending input, optional mit Tangenten-Auswahl
    RouteToolActive {
        /// Tangenten-Menüdaten (nur bei kubischer Kurve mit Nachbarn)
        tangent_data: Option<TangentMenuData>,
    },
}

/// Hilfsfunktion: Nächsten Node bei einer Weltposition finden (Snap-Range).
pub fn find_nearest_node_at(
    world_pos: glam::Vec2,
    road_map: &RoadMap,
    snap_radius: f32,
) -> Option<u64> {
    let mut nearest: Option<(u64, f32)> = None;

    for (id, node) in &road_map.nodes {
        let dist = node.position.distance(world_pos);
        if dist <= snap_radius {
            if let Some((_, best_dist)) = nearest {
                if dist < best_dist {
                    nearest = Some((*id, dist));
                }
            } else {
                nearest = Some((*id, dist));
            }
        }
    }

    nearest.map(|(id, _)| id)
}

// =============================================================================
// NEUE GENERALISIERTE CONTEXT-MENU-STRUKTUR
// =============================================================================

/// Bestimmt die MenuVariant basierend auf Fokus-Node, Selektion und Route-Tool-Status.
///
/// Wird einmal beim Rechtsklick aufgerufen und das Ergebnis eingefroren, bis das Menü
/// geschlossen wird — so verursachen Zustandsänderungen (Esc, Deselection) kein Flackern.
pub fn determine_menu_variant(
    selected_node_ids: &IndexSet<u64>,
    focused_node_id: Option<u64>,
    route_tool_has_input: bool,
    tangent_data: Option<TangentMenuData>,
) -> MenuVariant {
    // Route-Tool hat Priorität (nur wenn kein Node fokussiert)
    if route_tool_has_input && focused_node_id.is_none() {
        return MenuVariant::RouteToolActive { tangent_data };
    }

    // Fokussierter Node → NodeFocused (Einzelnode + Selektions-Befehle)
    if let Some(nid) = focused_node_id {
        return MenuVariant::NodeFocused {
            focused_node_id: nid,
        };
    }

    // Selektion vorhanden → SelectionOnly
    if !selected_node_ids.is_empty() {
        return MenuVariant::SelectionOnly;
    }

    // Nichts fokussiert, nichts selektiert → EmptyArea
    MenuVariant::EmptyArea
}

/// Rendert das Kontextmenü basierend auf der eingefrorenen MenuVariant.
///
/// Verwendet das validierte Command-System: Nur Commands mit erfüllten
/// Preconditions werden gerendert. Tangenten werden
/// als interaktive Widgets separat gehandhabt, da sie ComboBoxes haben.
#[allow(clippy::too_many_arguments)]
pub fn render_context_menu(
    response: &egui::Response,
    road_map: Option<&RoadMap>,
    selected_node_ids: &IndexSet<u64>,
    distanzen_active: bool,
    clipboard_has_data: bool,
    options: &EditorOptions,
    default_direction: ConnectionDirection,
    default_priority: ConnectionPriority,
    variant: &MenuVariant,
    events: &mut Vec<AppIntent>,
) -> bool {
    let Some(rm) = road_map else { return false };

    response
        .context_menu(|ui| {
            // Precondition-Kontext aus aktuellem State
            let precondition_ctx = PreconditionContext {
                road_map: rm,
                selected_node_ids,
                distanzen_active,
                clipboard_has_data,
            };

            match variant {
                MenuVariant::EmptyArea => {
                    let catalog = MenuCatalog::for_empty_area();
                    let intent_ctx = IntentContext {
                        node_id: None,
                        node_position: None,
                        two_node_ids: None,
                    };
                    let entries = validate_entries(&catalog, &precondition_ctx, &intent_ctx);
                    render_validated_entries(
                        ui,
                        &entries,
                        options,
                        default_direction,
                        default_priority,
                        events,
                    );
                }

                MenuVariant::SelectionOnly => {
                    // 2-Node-IDs in Selektionsreihenfolge (erster Klick = from, zweiter = to)
                    let two_ids = if selected_node_ids.len() == 2 {
                        let ids: Vec<u64> = selected_node_ids.iter().copied().collect();
                        Some((ids[0], ids[1]))
                    } else {
                        None
                    };

                    let catalog = MenuCatalog::for_selection_only();
                    let intent_ctx = IntentContext {
                        node_id: None,
                        node_position: None,
                        two_node_ids: two_ids,
                    };
                    let entries = validate_entries(&catalog, &precondition_ctx, &intent_ctx);
                    render_validated_entries(
                        ui,
                        &entries,
                        options,
                        default_direction,
                        default_priority,
                        events,
                    );
                }

                MenuVariant::NodeFocused { focused_node_id } => {
                    let node_pos = rm.nodes.get(focused_node_id).map(|n| n.position);

                    // 2-Node-IDs in Selektionsreihenfolge (erster Klick = from, zweiter = to)
                    let two_ids = if selected_node_ids.len() == 2 {
                        let ids: Vec<u64> = selected_node_ids.iter().copied().collect();
                        Some((ids[0], ids[1]))
                    } else {
                        None
                    };

                    let catalog = MenuCatalog::for_node_focused(*focused_node_id);
                    let intent_ctx = IntentContext {
                        node_id: Some(*focused_node_id),
                        node_position: node_pos,
                        two_node_ids: two_ids,
                    };
                    let entries = validate_entries(&catalog, &precondition_ctx, &intent_ctx);
                    render_validated_entries(
                        ui,
                        &entries,
                        options,
                        default_direction,
                        default_priority,
                        events,
                    );

                    // ── Info-Submenu (ganz unten, öffnet bei Hover) ───
                    ui.separator();
                    render_node_info_submenu(ui, *focused_node_id, rm);
                }

                MenuVariant::RouteToolActive { tangent_data } => {
                    let catalog = MenuCatalog::for_route_tool();
                    let intent_ctx = IntentContext {
                        node_id: None,
                        node_position: None,
                        two_node_ids: None,
                    };
                    let entries = validate_entries(&catalog, &precondition_ctx, &intent_ctx);
                    render_validated_entries(
                        ui,
                        &entries,
                        options,
                        default_direction,
                        default_priority,
                        events,
                    );

                    // Tangenten-Auswahl (dynamisch, nicht als Command)
                    if let Some(data) = tangent_data {
                        render_tangent_selection(ui, data, events);
                    }
                }
            }
        })
        .is_some()
}

/// Rendert die validierten Einträge als egui-Elemente.
///
/// Submenüs werden als einklappbare `menu_button` gerendert,
/// die erst bei Hover aufklappen (natives egui-Submenu-Verhalten).
fn render_validated_entries(
    ui: &mut egui::Ui,
    entries: &[ValidatedEntry],
    options: &EditorOptions,
    default_direction: ConnectionDirection,
    default_priority: ConnectionPriority,
    events: &mut Vec<AppIntent>,
) {
    for entry in entries {
        match entry {
            ValidatedEntry::Label(text) => {
                ui.label(text);
            }
            ValidatedEntry::Separator => {
                ui.separator();
            }
            ValidatedEntry::Command {
                id, label, intent, ..
            } => {
                let clicked = if let Some(icon) =
                    command_icon(*id, options, default_direction, default_priority)
                {
                    if is_direction_or_priority(*id) {
                        ui.add(egui::Button::image(
                            icon.fit_to_exact_size(CM_CHOICE_ICON_SIZE),
                        ))
                        .on_hover_text(direction_or_priority_tooltip(*id))
                        .clicked()
                    } else {
                        ui.add(egui::Button::image_and_text(icon, label)).clicked()
                    }
                } else {
                    ui.button(label).clicked()
                };
                if clicked {
                    events.push(*intent.clone());
                    ui.close();
                }
            }
            ValidatedEntry::Submenu {
                label,
                entries: children,
            } => {
                ui.menu_button(label, |ui| {
                    render_validated_entries(
                        ui,
                        children,
                        options,
                        default_direction,
                        default_priority,
                        events,
                    );
                });
            }
        }
    }
}

/// Info-Submenu für einen Node (öffnet bei Hover, zeigt Details).
fn render_node_info_submenu(ui: &mut egui::Ui, node_id: u64, road_map: &RoadMap) {
    ui.menu_button("ℹ Info", |ui| {
        if let Some(node) = road_map.nodes.get(&node_id) {
            ui.label(format!("📍 Node {}", node_id));
            ui.label(format!(
                "Position: ({:.1}, {:.1})",
                node.position.x, node.position.y
            ));
            ui.label(format!("Flag: {:?}", node.flag));
            ui.separator();
            let out_count = road_map
                .connections_iter()
                .filter(|c| c.start_id == node_id)
                .count();
            let in_count = road_map
                .connections_iter()
                .filter(|c| c.end_id == node_id)
                .count();
            ui.label(format!("Ausgehend: {}", out_count));
            ui.label(format!("Eingehend: {}", in_count));
            if let Some(marker) = road_map.find_marker_by_node_id(node_id) {
                ui.separator();
                ui.label(format!("🗺 Marker: {}", marker.name));
                ui.label(format!("Gruppe: {}", marker.group));
            }
        } else {
            ui.label("Node nicht gefunden");
        }
    });
}

/// Tangenten-Auswahl für Route-Tool (ComboBox, nicht als Command).
fn render_tangent_selection(
    ui: &mut egui::Ui,
    data: &TangentMenuData,
    events: &mut Vec<AppIntent>,
) {
    let has_start = !data.start_options.is_empty();
    let has_end = !data.end_options.is_empty();

    if !has_start && !has_end {
        return;
    }

    ui.separator();
    ui.label("🎯 Tangenten");

    if has_start {
        ui.label("Start:");
        for (source, label) in &data.start_options {
            let is_sel = *source == data.current_start;
            if ui.selectable_label(is_sel, label).clicked() {
                events.push(AppIntent::RouteToolTangentSelected {
                    start: *source,
                    end: data.current_end,
                });
                ui.close();
            }
        }
    }

    if has_start && has_end {
        ui.separator();
    }

    if has_end {
        ui.label("Ende:");
        for (source, label) in &data.end_options {
            let is_sel = *source == data.current_end;
            if ui.selectable_label(is_sel, label).clicked() {
                events.push(AppIntent::RouteToolTangentSelected {
                    start: data.current_start,
                    end: *source,
                });
                ui.close();
            }
        }
    }
}
