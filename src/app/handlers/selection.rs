//! Handler für Selektions-Operationen.

use crate::app::history::Snapshot;
use crate::app::use_cases;
use crate::app::{AppState, SelectionState};
use std::sync::Arc;

/// Zeichnet einen Undo-Snapshot auf, wenn sich die Selektion geändert hat.
fn record_if_selection_changed(state: &mut AppState, old_selection: SelectionState) {
    if old_selection.selected_node_ids != state.selection.selected_node_ids {
        let snap = Snapshot {
            road_map: state.road_map.clone(),
            selection: old_selection,
        };
        state.history.record_snapshot(snap);
    }
}

/// Selektiert den nächstgelegenen Node zum Klickpunkt.
pub fn select_nearest_node(
    state: &mut AppState,
    world_pos: glam::Vec2,
    max_distance: f32,
    additive: bool,
    extend_path: bool,
) {
    let old = state.selection.clone();
    use_cases::selection::select_nearest_node(
        state,
        world_pos,
        max_distance,
        additive,
        extend_path,
    );
    record_if_selection_changed(state, old);
}

/// Selektiert das Segment zwischen den nächsten Kreuzungen.
pub fn select_segment(
    state: &mut AppState,
    world_pos: glam::Vec2,
    max_distance: f32,
    additive: bool,
) {
    let old = state.selection.clone();
    use_cases::selection::select_segment_between_nearest_intersections(
        state,
        world_pos,
        max_distance,
        additive,
    );
    record_if_selection_changed(state, old);
}

/// Selektiert Nodes innerhalb eines Rechtecks.
pub fn select_in_rect(state: &mut AppState, min: glam::Vec2, max: glam::Vec2, additive: bool) {
    let old = state.selection.clone();
    use_cases::selection::select_nodes_in_rect(state, min, max, additive);
    record_if_selection_changed(state, old);
}

/// Selektiert Nodes innerhalb eines Lasso-Polygons.
pub fn select_in_lasso(state: &mut AppState, polygon: &[glam::Vec2], additive: bool) {
    let old = state.selection.clone();
    use_cases::selection::select_nodes_in_lasso(state, polygon, additive);
    record_if_selection_changed(state, old);
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
    let old = state.selection.clone();
    use_cases::selection::clear_selection(state);
    record_if_selection_changed(state, old);
}

/// Selektiert alle Nodes der geladenen RoadMap.
pub fn select_all(state: &mut AppState) {
    if let Some(road_map) = state.road_map.as_deref() {
        let old = state.selection.clone();
        state.selection.selected_node_ids = Arc::new(road_map.nodes.keys().copied().collect());
        state.selection.selection_anchor_node_id = None;
        record_if_selection_changed(state, old);
        log::info!(
            "Alle {} Nodes selektiert",
            state.selection.selected_node_ids.len()
        );
    }
}

/// Invertiert die aktuelle Selektion (alle unselektierten werden selektiert und umgekehrt).
pub fn invert(state: &mut AppState) {
    if let Some(rm) = &state.road_map {
        let old = state.selection.clone();
        let all_nodes: std::collections::HashSet<_> = rm.nodes.keys().copied().collect();
        let inverted: std::collections::HashSet<_> = all_nodes
            .symmetric_difference(&state.selection.selected_node_ids)
            .copied()
            .collect();
        state.selection.selected_node_ids = Arc::new(inverted);
        record_if_selection_changed(state, old);
    }
}
