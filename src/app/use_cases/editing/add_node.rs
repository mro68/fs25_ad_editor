//! Use-Case: Neuen Node an einer Weltposition hinzufügen.

use crate::app::AppState;
use crate::core::{Connection, MapNode, NodeFlag};
use std::sync::Arc;

/// Fügt einen neuen Node an der gegebenen Weltposition hinzu.
///
/// Wenn genau ein Node selektiert ist, wird der neue Node automatisch
/// mit der voreingestellten Richtung und Straßenart verbunden.
/// Der neue Node wird anschließend als einziger selektiert.
pub fn add_node_at_position(state: &mut AppState, world_pos: glam::Vec2) {
    let Some(_road_map) = state.road_map.as_ref() else {
        log::warn!("Kein Node hinzufügbar: keine RoadMap geladen");
        return;
    };

    // Merke aktuell selektierten Node für Auto-Connect
    let connect_from = if state.selection.selected_node_ids.len() == 1 {
        state.selection.selected_node_ids.iter().next().copied()
    } else {
        None
    };

    // Snapshot VOR Mutation
    state.record_undo_snapshot();

    let direction = state.editor.default_direction;
    let priority = state.editor.default_priority;

    let Some(road_map_arc) = state.road_map.as_mut() else {
        log::warn!("Kein Node hinzufügbar: keine RoadMap geladen");
        return;
    };
    let road_map = Arc::make_mut(road_map_arc);
    let new_id = road_map.next_node_id();
    let node = MapNode::new(new_id, world_pos, NodeFlag::Regular);
    road_map.add_node(node);

    // Auto-Connect: Vom selektierten Node zum neuen Node verbinden
    if let Some(from_id) = connect_from {
        if road_map.nodes.contains_key(&from_id) {
            let start_pos = road_map.nodes[&from_id].position;
            let end_pos = world_pos;
            let conn = Connection::new(from_id, new_id, direction, priority, start_pos, end_pos);
            road_map.add_connection(conn);
            // Flags der betroffenen Nodes neu berechnen
            road_map.recalculate_node_flags(&[from_id, new_id]);
            log::info!(
                "Auto-Connect: {}→{} ({:?}, {:?})",
                from_id,
                new_id,
                direction,
                priority
            );
        }
    }

    // Spatial-Index nach Mutation aktualisieren
    road_map.ensure_spatial_index();

    // Neuen Node selektieren
    state.selection.ids_mut().clear();
    state.selection.ids_mut().insert(new_id);
    state.selection.selection_anchor_node_id = Some(new_id);

    log::info!(
        "Node {} an Position ({:.1}, {:.1}) hinzugefügt",
        new_id,
        world_pos.x,
        world_pos.y
    );
}
