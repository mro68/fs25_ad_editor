//! Kontextmenü-System: 5 kontextabhängige Varianten für Rechtsklick im Viewport.
//!
//! Struktur:
//! - `mod.rs`: Router + MenuVariant enum + Helper
//! - `empty_area.rs`: Leerer Bereich (Datei, Werkzeug, Selektion, Ansicht)
//! - `single_node.rs`: Einzelner Node (selektiert/nicht)
//! - `multiple_nodes.rs`: Mehrere Nodes (≥2)
//! - `route_tool.rs`: Route-Tool aktiv

mod empty_area;
mod multiple_nodes;
mod route_tool;
mod single_node;

pub use empty_area::render_empty_area_menu;
pub use multiple_nodes::render_multiple_nodes_menu;
pub use route_tool::render_route_tool_menu;
pub use single_node::{render_single_node_selected_menu, render_single_node_unselected_menu};

use crate::app::{state::DistanzenState, AppIntent, RoadMap};
use std::collections::HashSet;

/// Kontextabhängige Menü-Variante basierend auf Selection und Position.
#[derive(Debug, Clone, Copy)]
pub enum MenuVariant {
    /// Rechtsklick auf leeren Bereich (kein Node in Snaprange)
    EmptyArea,
    /// Rechtsklick auf einzelnen Node, der noch nicht selektiert ist
    SingleNodeUnselected { node_id: u64 },
    /// Rechtsklick auf einzelnen Node, der bereits selektiert ist
    SingleNodeSelected { node_id: u64 },
    /// Mehrere Nodes selektiert (≥2)
    MultipleNodesSelected,
    /// Route-Tool aktiv mit pending input
    RouteToolActive,
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

/// Hilfsfunktion: Nächsten Node bei einer Weltposition finden (Snap-Range).
pub fn find_nearest_node_at(world_pos: glam::Vec2, road_map: &RoadMap) -> Option<u64> {
    const SNAP_RADIUS: f32 = 15.0; // Pixel in Welteinheiten (etwa)
    let mut nearest: Option<(u64, f32)> = None;

    for (id, node) in &road_map.nodes {
        let dist = node.position.distance(world_pos);
        if dist <= SNAP_RADIUS {
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

/// Neue Haupt-Entry-Point: Universeller Viewport-Context-Menu-Router.
///
/// Bestimmt die MenuVariant basierend auf Kontext und ruft die passende Rendering-Funktion auf.
pub fn show_viewport_context_menu(
    response: &egui::Response,
    road_map: Option<&RoadMap>,
    selected_node_ids: &HashSet<u64>,
    distanzen_state: &mut DistanzenState,
    pointer_pos_world: Option<glam::Vec2>,
    route_tool_has_input: bool,
    events: &mut Vec<AppIntent>,
) {
    // Bestimme MenuVariant
    let Some(rm) = road_map else { return };
    let hovered_node_id = pointer_pos_world.and_then(|pos| find_nearest_node_at(pos, rm));

    let variant = match (
        selected_node_ids.len(),
        hovered_node_id,
        route_tool_has_input,
    ) {
        (0, None, true) => MenuVariant::RouteToolActive,
        (0, None, _) => MenuVariant::EmptyArea,
        (0, Some(id), _) => MenuVariant::SingleNodeUnselected { node_id: id },
        (1, Some(id), _) if selected_node_ids.contains(&id) => {
            MenuVariant::SingleNodeSelected { node_id: id }
        }
        (n, _, _) if n >= 2 => MenuVariant::MultipleNodesSelected,
        _ => MenuVariant::EmptyArea,
    };

    response.context_menu(|ui| match variant {
        MenuVariant::EmptyArea => render_empty_area_menu(ui, events),
        MenuVariant::SingleNodeUnselected { node_id } => {
            render_single_node_unselected_menu(ui, node_id, rm, events)
        }
        MenuVariant::SingleNodeSelected { node_id } => {
            render_single_node_selected_menu(ui, node_id, rm, events)
        }
        MenuVariant::MultipleNodesSelected => {
            render_multiple_nodes_menu(ui, selected_node_ids, rm, distanzen_state, events)
        }
        MenuVariant::RouteToolActive => render_route_tool_menu(ui, events),
    });
}

/// Legacy-Wrapper für bestehende `show_connection_context_menu` und `show_node_marker_context_menu`
/// zur Rückwärts-Kompatibilität beim Übergang zum neuen System.
#[allow(dead_code)]
pub(super) fn show_connection_context_menu(
    response: &egui::Response,
    road_map: Option<&RoadMap>,
    selected_node_ids: &HashSet<u64>,
    distanzen_state: &mut DistanzenState,
    events: &mut Vec<AppIntent>,
) {
    if selected_node_ids.len() < 2 {
        return;
    }

    let Some(rm) = road_map else {
        return;
    };

    let connection_count = rm
        .connections_iter()
        .filter(|c| {
            selected_node_ids.contains(&c.start_id) && selected_node_ids.contains(&c.end_id)
        })
        .count();

    if connection_count == 0 && selected_node_ids.len() != 2 {
        return;
    }

    // Delegiere an neue Struktur
    show_viewport_context_menu(
        response,
        Some(rm),
        selected_node_ids,
        distanzen_state,
        None, // Keine Welt-Position für Legacy-Pfad
        false,
        events,
    );
}

#[allow(dead_code)]
pub(super) fn show_node_marker_context_menu(
    response: &egui::Response,
    road_map: Option<&RoadMap>,
    node_id: u64,
    events: &mut Vec<AppIntent>,
) {
    let Some(rm) = road_map else {
        return;
    };

    if !rm.nodes.contains_key(&node_id) {
        return;
    }

    response.context_menu(|ui| {
        render_single_node_selected_menu(ui, node_id, rm, events);
    });
}
