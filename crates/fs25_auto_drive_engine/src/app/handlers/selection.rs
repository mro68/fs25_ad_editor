//! Handler fuer Selektions-Operationen.

use crate::app::handlers::helpers;
use crate::app::use_cases;
use crate::app::AppState;
use std::sync::Arc;

/// Selektiert den naechstgelegenen Node zum Klickpunkt.
pub fn select_nearest_node(
    state: &mut AppState,
    world_pos: glam::Vec2,
    max_distance: f32,
    additive: bool,
    extend_path: bool,
) {
    let (old_selected, old_anchor) = helpers::capture_selection_snapshot(state);
    use_cases::selection::select_nearest_node(
        state,
        world_pos,
        max_distance,
        additive,
        extend_path,
    );
    helpers::record_selection_if_changed(state, old_selected, old_anchor);
}

/// Selektiert das Segment zwischen den naechsten Kreuzungen.
pub fn select_segment(
    state: &mut AppState,
    world_pos: glam::Vec2,
    max_distance: f32,
    additive: bool,
    stop_at_junction: bool,
    max_angle_deg: f32,
) {
    let (old_selected, old_anchor) = helpers::capture_selection_snapshot(state);
    use_cases::selection::select_segment_between_nearest_intersections(
        state,
        world_pos,
        max_distance,
        additive,
        stop_at_junction,
        max_angle_deg,
    );
    helpers::record_selection_if_changed(state, old_selected, old_anchor);
}

/// Selektiert alle Nodes der Gruppe, zu der der naechste Node gehoert.
pub fn select_group_nodes(
    state: &mut AppState,
    world_pos: glam::Vec2,
    max_distance: f32,
    additive: bool,
) {
    let (old_selected, old_anchor) = helpers::capture_selection_snapshot(state);
    use_cases::selection::select_group_by_nearest_node(state, world_pos, max_distance, additive);
    helpers::record_selection_if_changed(state, old_selected, old_anchor);
}

/// Selektiert Nodes innerhalb eines Rechtecks.
pub fn select_in_rect(state: &mut AppState, min: glam::Vec2, max: glam::Vec2, additive: bool) {
    let (old_selected, old_anchor) = helpers::capture_selection_snapshot(state);
    use_cases::selection::select_nodes_in_rect(state, min, max, additive);
    helpers::record_selection_if_changed(state, old_selected, old_anchor);
}

/// Selektiert Nodes innerhalb eines Lasso-Polygons.
pub fn select_in_lasso(state: &mut AppState, polygon: &[glam::Vec2], additive: bool) {
    let (old_selected, old_anchor) = helpers::capture_selection_snapshot(state);
    use_cases::selection::select_nodes_in_lasso(state, polygon, additive);
    helpers::record_selection_if_changed(state, old_selected, old_anchor);
}

/// Verschiebt alle selektierten Nodes um ein Delta.
pub fn move_selected(state: &mut AppState, delta_world: glam::Vec2) {
    use_cases::selection::move_selected_nodes(state, delta_world);
}

/// Startet einen Move-Lifecycle (nimmt Undo-Snapshot auf).
pub fn begin_move(state: &mut AppState) {
    state.record_undo_snapshot();
}

/// Beendet den Move-Lifecycle und stoesst den Spatial-Index-Rebuild an.
pub fn end_move(state: &mut AppState) {
    if let Some(road_map) = state.road_map.as_mut() {
        let road_map_mut = Arc::make_mut(road_map);
        road_map_mut.rebuild_spatial_index();
    }
}

/// Startet einen Rotation-Lifecycle (nimmt Undo-Snapshot auf).
pub fn begin_rotate(state: &mut AppState) {
    state.record_undo_snapshot();
}

/// Rotiert alle selektierten Nodes um den Delta-Winkel (Radiant).
pub fn rotate_selected(state: &mut AppState, delta_angle: f32) {
    use_cases::selection::rotate_selected_nodes(state, delta_angle);
}

/// Beendet den Rotation-Lifecycle und stoesst den Spatial-Index-Rebuild an.
pub fn end_rotate(state: &mut AppState) {
    if let Some(road_map) = state.road_map.as_mut() {
        let road_map_mut = Arc::make_mut(road_map);
        road_map_mut.rebuild_spatial_index();
    }
}

/// Hebt die aktuelle Selektion auf.
pub fn clear(state: &mut AppState) {
    let (old_selected, old_anchor) = helpers::capture_selection_snapshot(state);
    use_cases::selection::clear_selection(state);
    helpers::record_selection_if_changed(state, old_selected, old_anchor);
}

/// Selektiert alle Nodes der geladenen RoadMap.
pub fn select_all(state: &mut AppState) {
    if let Some(road_map) = state.road_map.as_deref() {
        let (old_selected, old_anchor) = helpers::capture_selection_snapshot(state);
        state.selection.selected_node_ids = Arc::new(road_map.node_ids().collect());
        state.selection.selection_anchor_node_id = None;
        helpers::record_selection_if_changed(state, old_selected, old_anchor);
        log::info!(
            "Alle {} Nodes selektiert",
            state.selection.selected_node_ids.len()
        );
    }
}

/// Invertiert die aktuelle Selektion (alle unselektierten werden selektiert und umgekehrt).
pub fn invert(state: &mut AppState) {
    if let Some(rm) = &state.road_map {
        let (old_selected, old_anchor) = helpers::capture_selection_snapshot(state);
        let current = &state.selection.selected_node_ids;
        let inverted: indexmap::IndexSet<u64> =
            rm.node_ids().filter(|id| !current.contains(id)).collect();
        state.selection.selected_node_ids = Arc::new(inverted);
        helpers::record_selection_if_changed(state, old_selected, old_anchor);
    }
}
