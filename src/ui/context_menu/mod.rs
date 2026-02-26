//! Kontextmenü-System: echte Kontext-Befehle (Nodes, Connections, Route-Tool).
//!
//! Struktur:
//! - `mod.rs`: Router + MenuVariant enum + Helpers (button_intent, render_streckenteilung)
//! - `empty_area.rs`: Leerer Bereich (Tool-Auswahl, Streckenteilung wenn aktiv)
//! - `single_node.rs`: Einzelner Node (selektiert/nicht) — Info, Selektion, Marker
//! - `multiple_nodes.rs`: Mehrere Nodes (≥2) — Connections, Streckenteilung, Bulk-Aktionen
//! - `route_tool.rs`: Route-Tool aktiv — Ausführen/Abbrechen

mod empty_area;
mod multiple_nodes;
mod route_tool;
mod single_node;

pub use empty_area::render_empty_area_menu;
pub use multiple_nodes::render_multiple_nodes_menu;
pub use route_tool::render_route_tool_menu;
pub use single_node::{render_single_node_selected_menu, render_single_node_unselected_menu};

use crate::app::tools::common::TangentMenuData;
use crate::app::{state::DistanzenState, AppIntent, RoadMap};
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

/// Helper-Funktion: Erstellt einen Button, der bei Klick einen Intent emittiert und das Menü schließt.
pub fn button_intent(
    ui: &mut egui::Ui,
    label: &str,
    intent: AppIntent,
    events: &mut Vec<AppIntent>,
) {
    if ui.button(label).clicked() {
        events.push(intent);
        ui.close();
    }
}

/// Gemeinsame Streckenteilung-Controls (Abstand + Nodes + Übernehmen/Verwerfen).
///
/// Zeigt die Controls wenn `distanzen_state.active`, sonst den Aktivierungs-Button.
pub fn render_streckenteilung(
    ui: &mut egui::Ui,
    distanzen_state: &mut DistanzenState,
    events: &mut Vec<AppIntent>,
) {
    if distanzen_state.active {
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
    } else {
        button_intent(
            ui,
            "✂ Streckenteilung",
            AppIntent::StreckenteilungAktivieren,
            events,
        );
    }
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
        .context_menu(|ui| match variant {
            MenuVariant::EmptyArea => render_empty_area_menu(ui, distanzen_state, events),
            MenuVariant::SingleNodeUnselected { node_id } => {
                render_single_node_unselected_menu(ui, *node_id, rm, events)
            }
            MenuVariant::SingleNodeSelected { node_id } => {
                render_single_node_selected_menu(ui, *node_id, rm, events)
            }
            MenuVariant::MultipleNodesSelected => {
                render_multiple_nodes_menu(ui, selected_node_ids, rm, distanzen_state, events)
            }
            MenuVariant::RouteToolActive { tangent_data } => {
                render_route_tool_menu(ui, tangent_data.as_ref(), events)
            }
        })
        .is_some()
}
