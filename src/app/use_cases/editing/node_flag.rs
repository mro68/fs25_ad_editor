//! Use-Case: Node-Flag eines bestehenden Nodes aendern.

use crate::app::AppState;
use crate::core::NodeFlag;
use std::sync::Arc;

/// Setzt das Flag eines bestehenden Nodes und erstellt davor einen Undo-Snapshot.
pub fn set_node_flag(state: &mut AppState, node_id: u64, flag: NodeFlag) {
    let Some(road_map_arc) = state.road_map.as_ref() else {
        return;
    };

    // Pruefe ob Node existiert und ob sich das Flag ueberhaupt aendert.
    let Some(node) = road_map_arc.nodes.get(&node_id) else {
        log::warn!("Node {} nicht gefunden", node_id);
        return;
    };

    if node.flag == flag {
        return;
    }

    // Snapshot VOR Mutation
    state.record_undo_snapshot();

    let Some(road_map_arc) = state.road_map.as_mut() else {
        log::warn!("Node-Flag nicht aenderbar: keine RoadMap geladen");
        return;
    };

    let road_map = Arc::make_mut(road_map_arc);
    if !road_map.set_node_flag(node_id, flag) {
        log::warn!("Node {} beim Setzen des Flags nicht gefunden", node_id);
        return;
    }

    log::info!("Node {} Flag auf {:?} gesetzt", node_id, flag);
}
