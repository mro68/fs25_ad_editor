use crate::app::tool_contract::TangentSource;
use crate::app::tools::RouteToolSegmentAdjustments;
use crate::app::AppState;

/// Wendet die vom User gewaehlten Tangenten an und triggert ggf. eine Neuberechnung.
pub(super) fn apply_tangent(state: &mut AppState, start: TangentSource, end: TangentSource) {
    if let Some(tool) = state.editor.tool_manager.active_tangent_mut() {
        tool.apply_tangent_selection(start, end);
    }

    let needs_recreate = state
        .editor
        .tool_manager
        .active_recreate()
        .is_some_and(|tool| tool.needs_recreate());

    if needs_recreate {
        super::apply::recreate(state);
    }
}

/// Startet einen Drag auf einem Steuerpunkt/Anker des aktiven Route-Tools.
pub(super) fn drag_start(state: &mut AppState, world_pos: glam::Vec2) {
    let pick_radius = state.options.hitbox_radius();
    let Some(road_map) = state.road_map.as_deref() else {
        return;
    };
    if let Some(tool) = state.editor.tool_manager.active_drag_mut() {
        tool.on_drag_start(world_pos, road_map, pick_radius);
    }
}

/// Aktualisiert die Position des gegriffenen Punkts waehrend eines Drags.
pub(super) fn drag_update(state: &mut AppState, world_pos: glam::Vec2) {
    if let Some(tool) = state.editor.tool_manager.active_drag_mut() {
        tool.on_drag_update(world_pos);
    }
}

/// Beendet den Drag (ggf. Re-Snap auf existierenden Node).
pub(super) fn drag_end(state: &mut AppState) {
    let Some(road_map) = state.road_map.as_deref() else {
        return;
    };
    if let Some(tool) = state.editor.tool_manager.active_drag_mut() {
        tool.on_drag_end(road_map);
    }
}

/// Verarbeitet Alt+Scroll-Rotation fuer das aktive Route-Tool.
pub(super) fn rotate(state: &mut AppState, delta: f32) {
    if let Some(tool) = state.editor.tool_manager.active_rotate_mut() {
        tool.on_scroll_rotate(delta);
    }
}

/// Erhoeht die Anzahl der Nodes im aktiven Route-Tool um 1.
pub(super) fn increase_node_count(state: &mut AppState) {
    apply_segment_adjustment(state, "increase-node-count", |tool| {
        tool.increase_node_count()
    });
}

/// Verringert die Anzahl der Nodes im aktiven Route-Tool um 1 (min. 2).
pub(super) fn decrease_node_count(state: &mut AppState) {
    apply_segment_adjustment(state, "decrease-node-count", |tool| {
        tool.decrease_node_count()
    });
}

/// Erhoeht den minimalen Segment-Abstand im aktiven Route-Tool um 0.25m.
pub(super) fn increase_segment_length(state: &mut AppState) {
    apply_segment_adjustment(state, "increase-segment-length", |tool| {
        tool.increase_segment_length();
    });
}

/// Verringert den minimalen Segment-Abstand im aktiven Route-Tool um 0.25m (min. 0.1m).
pub(super) fn decrease_segment_length(state: &mut AppState) {
    apply_segment_adjustment(state, "decrease-segment-length", |tool| {
        tool.decrease_segment_length();
    });
}

fn apply_segment_adjustment(
    state: &mut AppState,
    shortcut_name: &'static str,
    apply: impl FnOnce(&mut dyn RouteToolSegmentAdjustments),
) {
    if state.editor.active_tool != crate::app::EditorTool::Route {
        log::debug!("Ignored route tool shortcut {shortcut_name}: editor is not in route mode");
        return;
    }

    let active_tool_id = state.editor.tool_manager.active_id();
    let Some(tool) = state.editor.tool_manager.active_segment_adjustments_mut() else {
        match active_tool_id {
            Some(tool_id) => log::debug!(
                "Ignored route tool shortcut {shortcut_name}: active tool {:?} has no segment adjustments capability",
                tool_id
            ),
            None => log::debug!(
                "Ignored route tool shortcut {shortcut_name}: no active route tool"
            ),
        }
        return;
    };

    apply(tool);
    recreate_if_needed(state);
}

fn recreate_if_needed(state: &mut AppState) {
    let needs_recreate = state
        .editor
        .tool_manager
        .active_recreate()
        .is_some_and(|tool| tool.needs_recreate());

    if needs_recreate {
        super::apply::recreate(state);
    }
}
