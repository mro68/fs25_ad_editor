//! Handler für Selektions-Operationen.

use crate::app::use_cases;
use crate::app::AppState;
use std::sync::Arc;

/// Selektiert den nächstgelegenen Node zum Klickpunkt.
pub fn select_nearest_node(
    state: &mut AppState,
    world_pos: glam::Vec2,
    max_distance: f32,
    additive: bool,
    extend_path: bool,
) {
    use_cases::selection::select_nearest_node(
        state,
        world_pos,
        max_distance,
        additive,
        extend_path,
    );
}

/// Selektiert das Segment zwischen den nächsten Kreuzungen.
pub fn select_segment(
    state: &mut AppState,
    world_pos: glam::Vec2,
    max_distance: f32,
    additive: bool,
) {
    use_cases::selection::select_segment_between_nearest_intersections(
        state,
        world_pos,
        max_distance,
        additive,
    );
}

/// Selektiert Nodes innerhalb eines Rechtecks.
pub fn select_in_rect(state: &mut AppState, min: glam::Vec2, max: glam::Vec2, additive: bool) {
    use_cases::selection::select_nodes_in_rect(state, min, max, additive);
}

/// Selektiert Nodes innerhalb eines Lasso-Polygons.
pub fn select_in_lasso(state: &mut AppState, polygon: &[glam::Vec2], additive: bool) {
    use_cases::selection::select_nodes_in_lasso(state, polygon, additive);
}

/// Verschiebt alle selektierten Nodes um ein Delta.
pub fn move_selected(state: &mut AppState, delta_world: glam::Vec2) {
    use_cases::selection::move_selected_nodes(state, delta_world);
}

/// Startet einen Move-Lifecycle (nimmt Undo-Snapshot auf).
pub fn begin_move(state: &mut AppState) {
    state.record_undo_snapshot();
}

/// Hebt die aktuelle Selektion auf.
pub fn clear(state: &mut AppState) {
    use_cases::selection::clear_selection(state);
}

/// Selektiert alle Nodes der geladenen RoadMap.
pub fn select_all(state: &mut AppState) {
    if let Some(road_map) = state.road_map.as_deref() {
        state.selection.selected_node_ids = Arc::new(road_map.nodes.keys().copied().collect());
        state.selection.selection_anchor_node_id = None;
        log::info!(
            "Alle {} Nodes selektiert",
            state.selection.selected_node_ids.len()
        );
    }
}

/// Invertiert die aktuelle Selektion (alle unselektierten werden selektiert und umgekehrt).
pub fn invert(state: &mut AppState) {
    if let Some(rm) = &state.road_map {
        let all_nodes: std::collections::HashSet<_> = rm.nodes.keys().copied().collect();
        let inverted: std::collections::HashSet<_> = all_nodes
            .symmetric_difference(&state.selection.selected_node_ids)
            .copied()
            .collect();
        state.selection.selected_node_ids = Arc::new(inverted);
    }
}
