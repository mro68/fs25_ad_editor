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
use crate::app::{state::DistanzenState, AppIntent, RoadMap};
use commands::{validate_entries, IntentContext, MenuCatalog, PreconditionContext, ValidatedEntry};
use std::collections::HashSet;

/// Kontextabhängige Menü-Variante basierend auf Selection und Position.
///
/// Wird beim Rechtsklick einmalig bestimmt und eingefroren, bis das Menü
/// geschlossen wird. Enthält alle Daten die zum Rendern nötig sind.
#[derive(Debug, Clone)]
pub enum MenuVariant {
    /// Rechtsklick auf leeren Bereich (kein Node gehovered)
    EmptyArea,
    /// Rechtsklick auf einzelnen Node, der noch nicht selektiert ist
    SingleNodeUnselected { node_id: u64 },
    /// Rechtsklick auf einzelnen Node, der bereits selektiert ist
    SingleNodeSelected { node_id: u64 },
    /// Mehrere Nodes selektiert (≥2)
    MultipleNodesSelected,
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

/// Bestimmt die MenuVariant basierend auf Kontext (Node-Hoverung, Selection, Route-Tool).
///
/// Wird einmal beim Rechtsklick aufgerufen und das Ergebnis eingefroren, bis das Menü
/// geschlossen wird — so verursachen Zustandsänderungen (Esc, Deselection) kein Flackern.
///
/// `tangent_data` wird vom Aufrufer aus dem aktiven Route-Tool abgefragt
/// und hier eingebettet, wenn die Variante `RouteToolActive` ist.
pub fn determine_menu_variant(
    road_map: Option<&RoadMap>,
    selected_node_ids: &HashSet<u64>,
    pointer_pos_world: Option<glam::Vec2>,
    route_tool_has_input: bool,
    tangent_data: Option<TangentMenuData>,
    snap_radius: f32,
) -> MenuVariant {
    let Some(rm) = road_map else {
        return MenuVariant::EmptyArea;
    };
    let hovered_node_id =
        pointer_pos_world.and_then(|pos| find_nearest_node_at(pos, rm, snap_radius));

    match (
        selected_node_ids.len(),
        hovered_node_id,
        route_tool_has_input,
    ) {
        // Route-Tool aktiv: eigenes Menü mit optionalen Tangenten (nur wenn kein Node gehovered)
        (_, None, true) => MenuVariant::RouteToolActive { tangent_data },
        // Rechtsklick auf leeren Bereich (kein Node gehovered) → immer EmptyArea
        // (unabhängig von vorheriger Selektion)
        (_, None, _) => MenuVariant::EmptyArea,
        // Node gehovered, der noch nicht selektiert ist
        (_, Some(id), _) if !selected_node_ids.contains(&id) => {
            MenuVariant::SingleNodeUnselected { node_id: id }
        }
        // Node gehovered, der selektiert ist (bei Einzel-Selektion)
        (1, Some(id), _) => MenuVariant::SingleNodeSelected { node_id: id },
        // Node gehovered + Multi-Selektion → Multi-Menü
        (n, Some(_), _) if n >= 2 => MenuVariant::MultipleNodesSelected,
        // Fallback
        _ => MenuVariant::EmptyArea,
    }
}

/// Rendert das Kontextmenü basierend auf der eingefrorenen MenuVariant.
///
/// Verwendet das validierte Command-System: Nur Commands mit erfüllten
/// Preconditions werden gerendert. Streckenteilung und Tangenten werden
/// als interaktive Widgets separat gehandhabt, da sie DragValues/ComboBoxes haben.
pub fn render_context_menu(
    response: &egui::Response,
    road_map: Option<&RoadMap>,
    selected_node_ids: &HashSet<u64>,
    distanzen_state: &mut DistanzenState,
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
                distanzen_active: distanzen_state.active,
            };

            match variant {
                MenuVariant::EmptyArea => {
                    let catalog = MenuCatalog::for_empty_area(distanzen_state.active);
                    let intent_ctx = IntentContext {
                        node_id: None,
                        node_position: None,
                        two_node_ids: None,
                    };
                    let entries = validate_entries(&catalog, &precondition_ctx, &intent_ctx);
                    render_validated_entries(ui, &entries, events);

                    // Streckenteilung-Widget (interaktive Controls, nicht als Command)
                    if distanzen_state.active {
                        ui.separator();
                        render_streckenteilung_widget(ui, distanzen_state, events);
                    }
                }

                MenuVariant::SingleNodeUnselected { node_id } => {
                    // Info-Header
                    render_node_info_header(ui, *node_id, rm);

                    let node_pos = rm.nodes.get(node_id).map(|n| n.position);
                    let catalog = MenuCatalog::for_single_node_unselected(*node_id);
                    let intent_ctx = IntentContext {
                        node_id: Some(*node_id),
                        node_position: node_pos,
                        two_node_ids: None,
                    };
                    let entries = validate_entries(&catalog, &precondition_ctx, &intent_ctx);
                    render_validated_entries(ui, &entries, events);
                }

                MenuVariant::SingleNodeSelected { node_id } => {
                    // Info-Header
                    if let Some(node) = rm.nodes.get(node_id) {
                        ui.label(format!("📍 Node {} ✓", node_id));
                        ui.label(format!(
                            "Pos: ({:.1}, {:.1})",
                            node.position.x, node.position.y
                        ));
                    }

                    let node_pos = rm.nodes.get(node_id).map(|n| n.position);
                    let catalog = MenuCatalog::for_single_node_selected(*node_id);
                    let intent_ctx = IntentContext {
                        node_id: Some(*node_id),
                        node_position: node_pos,
                        two_node_ids: None,
                    };
                    let entries = validate_entries(&catalog, &precondition_ctx, &intent_ctx);
                    render_validated_entries(ui, &entries, events);
                }

                MenuVariant::MultipleNodesSelected => {
                    // Info-Header
                    ui.label(format!("📍 {} Nodes selektiert", selected_node_ids.len()));

                    // Sortierte 2-Node-IDs für Route-Tool-Shortcuts
                    let two_ids = if selected_node_ids.len() == 2 {
                        let mut ids: Vec<u64> = selected_node_ids.iter().copied().collect();
                        ids.sort();
                        Some((ids[0], ids[1]))
                    } else {
                        None
                    };

                    let catalog = MenuCatalog::for_multiple_nodes_selected();
                    let intent_ctx = IntentContext {
                        node_id: None,
                        node_position: None,
                        two_node_ids: two_ids,
                    };
                    let entries = validate_entries(&catalog, &precondition_ctx, &intent_ctx);
                    render_validated_entries(ui, &entries, events);

                    // Streckenteilung als interaktives Widget ersetzen
                    // (der Command wird zum Separator — Widget wird direkt gerendert)
                    if distanzen_state.active {
                        ui.separator();
                        render_streckenteilung_widget(ui, distanzen_state, events);
                    }
                }

                MenuVariant::RouteToolActive { tangent_data } => {
                    let catalog = MenuCatalog::for_route_tool();
                    let intent_ctx = IntentContext {
                        node_id: None,
                        node_position: None,
                        two_node_ids: None,
                    };
                    let entries = validate_entries(&catalog, &precondition_ctx, &intent_ctx);
                    render_validated_entries(ui, &entries, events);

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
fn render_validated_entries(
    ui: &mut egui::Ui,
    entries: &[ValidatedEntry],
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
            ValidatedEntry::Command { label, intent, .. } => {
                if ui.button(label).clicked() {
                    events.push(intent.clone());
                    ui.close();
                }
            }
        }
    }
}

/// Info-Header für einen einzelnen Node (Position, Verbindungszähler).
fn render_node_info_header(ui: &mut egui::Ui, node_id: u64, road_map: &RoadMap) {
    if let Some(node) = road_map.nodes.get(&node_id) {
        ui.label(format!("📍 Node {}", node_id));
        ui.label(format!(
            "Pos: ({:.1}, {:.1})",
            node.position.x, node.position.y
        ));
        let in_count = road_map
            .connections_iter()
            .filter(|c| c.end_id == node_id)
            .count();
        let out_count = road_map
            .connections_iter()
            .filter(|c| c.start_id == node_id)
            .count();
        ui.label(format!("Verb.: {} ↦ {} ↤", out_count, in_count));
    }
}

/// Streckenteilung als interaktives Widget (DragValues, nicht als einfacher Button).
fn render_streckenteilung_widget(
    ui: &mut egui::Ui,
    distanzen_state: &mut DistanzenState,
    events: &mut Vec<AppIntent>,
) {
    ui.label("Streckenteilung:");

    let prev_distance = distanzen_state.distance;
    ui.horizontal(|ui| {
        ui.label("Abstand:");
        ui.add(
            egui::DragValue::new(&mut distanzen_state.distance)
                .speed(0.5)
                .range(1.0..=25.0)
                .suffix(" m"),
        );
    });
    if (distanzen_state.distance - prev_distance).abs() > f32::EPSILON {
        distanzen_state.by_count = false;
        distanzen_state.sync_from_distance();
    }

    let prev_count = distanzen_state.count;
    ui.horizontal(|ui| {
        ui.label("Nodes:");
        ui.add(
            egui::DragValue::new(&mut distanzen_state.count)
                .speed(1.0)
                .range(2..=10000),
        );
    });
    if distanzen_state.count != prev_count {
        distanzen_state.by_count = true;
        distanzen_state.sync_from_count();
        if distanzen_state.distance < 1.0 {
            distanzen_state.distance = 1.0;
            distanzen_state.sync_from_distance();
        }
    }

    ui.add_space(4.0);
    if ui.button("✓ Übernehmen").clicked() {
        events.push(AppIntent::ResamplePathRequested);
        distanzen_state.deactivate();
        ui.close();
    }
    if ui.button("✕ Verwerfen").clicked() {
        distanzen_state.deactivate();
        ui.close();
    }
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
                    start: source.clone(),
                    end: data.current_end.clone(),
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
                    start: data.current_start.clone(),
                    end: source.clone(),
                });
                ui.close();
            }
        }
    }
}
