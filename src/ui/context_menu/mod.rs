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
    selected_node_ids: &HashSet<u64>,
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
                    let catalog = MenuCatalog::for_empty_area();
                    let intent_ctx = IntentContext {
                        node_id: None,
                        node_position: None,
                        two_node_ids: None,
                    };
                    let entries = validate_entries(&catalog, &precondition_ctx, &intent_ctx);
                    render_validated_entries(ui, &entries, events);
                }

                MenuVariant::SelectionOnly => {
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

                    let catalog = MenuCatalog::for_selection_only();
                    let intent_ctx = IntentContext {
                        node_id: None,
                        node_position: None,
                        two_node_ids: two_ids,
                    };
                    let entries = validate_entries(&catalog, &precondition_ctx, &intent_ctx);
                    render_validated_entries(ui, &entries, events);

                    // Streckenteilung nur wenn Selektion eine gültige Kette bildet
                    if rm.is_resampleable_chain(selected_node_ids) {
                        ui.separator();
                        render_streckenteilung_widget(ui, distanzen_state, events);
                    }
                }

                MenuVariant::NodeFocused { focused_node_id } => {
                    // Info-Header für fokussierten Node
                    render_node_info_header(ui, *focused_node_id, rm);

                    let node_pos = rm.nodes.get(focused_node_id).map(|n| n.position);

                    // Sortierte 2-Node-IDs für Route-Tool-Shortcuts
                    let two_ids = if selected_node_ids.len() == 2 {
                        let mut ids: Vec<u64> = selected_node_ids.iter().copied().collect();
                        ids.sort();
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
                    render_validated_entries(ui, &entries, events);

                    // Streckenteilung nur wenn Selektion eine gültige Kette bildet
                    if rm.is_resampleable_chain(selected_node_ids) {
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
                    events.push(*intent.clone());
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
