//! Use-Case: Gruppen-Selektion per Doppelklick.

use crate::AppState;
use glam::Vec2;
use indexmap::IndexSet;
use std::sync::Arc;

/// Findet den naechsten Node, prueft Gruppenzugehoerigkeit und selektiert alle Gruppen-Nodes.
///
/// Gibt alle Nodes des ersten Records zurueck, zu dem der getroffene Node gehoert.
/// Bei `additive = true` werden die Gruppen-Nodes zur bestehenden Selektion hinzugefuegt.
pub fn select_group_by_nearest_node(
    state: &mut AppState,
    world_pos: Vec2,
    max_distance: f32,
    additive: bool,
) {
    let Some(road_map) = state.road_map.as_deref() else {
        return;
    };

    let Some(hit) = road_map
        .nearest_node(world_pos)
        .filter(|h| h.distance <= max_distance)
    else {
        return;
    };

    let node_id = hit.node_id;

    let Some(record) = state.group_registry.find_first_by_node_id(node_id) else {
        return;
    };

    let group_node_ids: IndexSet<u64> = record.node_ids.iter().copied().collect();

    if additive {
        let ids = state.selection.ids_mut();
        ids.extend(group_node_ids);
    } else {
        state.selection.selected_node_ids = Arc::new(group_node_ids);
    }
    state.selection.selection_anchor_node_id = Some(node_id);
}
