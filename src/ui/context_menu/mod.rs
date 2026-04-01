//! Kontextmenü-System mit validierter Command-Architektur.
//!
//! Garantie: Nur Commands mit erfüllten Preconditions werden gerendert.
//!
//! Struktur:
//! - `commands/`: CommandId, Precondition, MenuCatalog, validate_entries()
//! - `icons.rs`: command_icon(), Farbhilfs-Funktionen
//! - `render.rs`: render_validated_entries() und Helfer
//! - `tangent_ui.rs`: render_tangent_selection(), render_node_info_submenu()
//! - `mod.rs`: MenuVariant, determine_menu_variant(), render_context_menu()

pub mod commands;
mod icons;
mod render;
mod tangent_ui;

use crate::app::tools::common::TangentMenuData;
use crate::app::{AppIntent, ConnectionDirection, ConnectionPriority, GroupRegistry, RoadMap};
use crate::shared::EditorOptions;
use commands::{validate_entries, IntentContext, MenuCatalog, PreconditionContext};
use indexmap::IndexSet;
use render::render_validated_entries;
use tangent_ui::{render_node_info_submenu, render_tangent_selection};

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
    road_map
        .nearest_node(world_pos)
        .filter(|hit| hit.distance <= snap_radius)
        .map(|hit| hit.node_id)
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
    farmland_polygons_loaded: bool,
    group_editing_active: bool,
    options: &EditorOptions,
    default_direction: ConnectionDirection,
    default_priority: ConnectionPriority,
    variant: &MenuVariant,
    group_registry: Option<&GroupRegistry>,
    events: &mut Vec<AppIntent>,
) -> bool {
    let Some(rm) = road_map else { return false };
    let lang = options.language;

    response
        .context_menu(|ui| {
            // Segment-Record-ID berechnen: Alle selektierten Nodes gehoeren zu einem validen Segment?
            let group_record_id = group_registry.and_then(|registry| {
                let records = registry.find_by_node_ids(selected_node_ids);
                if records.len() == 1 {
                    let record = records[0];
                    let all_belong = selected_node_ids
                        .iter()
                        .all(|id| record.node_ids.contains(id));
                    if all_belong && registry.is_group_valid(record, rm) {
                        return Some(record.id);
                    }
                }
                None
            });

            // Pruefen ob mindestens 1 selektierter Node zu einer Gruppe gehoert
            let selection_has_group_member = group_registry.is_some_and(|registry| {
                selected_node_ids
                    .iter()
                    .any(|&nid| !registry.groups_for_node(nid).is_empty())
            });

            // Precondition-Kontext aus aktuellem State
            let precondition_ctx = PreconditionContext {
                road_map: rm,
                selected_node_ids,
                distanzen_active,
                clipboard_has_data,
                group_record_id,
                farmland_polygons_loaded,
                group_editing_active,
                selection_has_group_member,
            };

            match variant {
                MenuVariant::EmptyArea => {
                    let catalog = MenuCatalog::for_empty_area(lang);
                    let intent_ctx = IntentContext {
                        node_id: None,
                        node_position: None,
                        two_node_ids: None,
                        group_record_id: None,
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

                    let catalog = MenuCatalog::for_selection_only(lang);
                    let intent_ctx = IntentContext {
                        node_id: None,
                        node_position: None,
                        two_node_ids: two_ids,
                        group_record_id,
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

                    let catalog = MenuCatalog::for_node_focused(*focused_node_id, lang);
                    let intent_ctx = IntentContext {
                        node_id: Some(*focused_node_id),
                        node_position: node_pos,
                        two_node_ids: two_ids,
                        group_record_id,
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
                        group_record_id: None,
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

#[cfg(test)]
mod tests {
    use super::find_nearest_node_at;
    use crate::core::{MapNode, NodeFlag, RoadMap};
    use glam::Vec2;

    fn build_test_map() -> RoadMap {
        let mut map = RoadMap::new(3);
        map.add_node(MapNode::new(1, Vec2::ZERO, NodeFlag::Regular));
        map.add_node(MapNode::new(2, Vec2::new(4.0, 0.0), NodeFlag::Regular));
        map.rebuild_spatial_index();
        map
    }

    #[test]
    fn find_nearest_node_at_prefers_the_closest_node_inside_snap_radius() {
        let map = build_test_map();

        assert_eq!(
            find_nearest_node_at(Vec2::new(3.2, 0.0), &map, 5.0),
            Some(2)
        );
    }

    #[test]
    fn find_nearest_node_at_keeps_the_snap_radius_boundary() {
        let map = build_test_map();

        assert_eq!(
            find_nearest_node_at(Vec2::new(1.0, 0.0), &map, 1.0),
            Some(1)
        );
        assert_eq!(find_nearest_node_at(Vec2::new(1.01, 0.0), &map, 1.0), None);
    }
}
