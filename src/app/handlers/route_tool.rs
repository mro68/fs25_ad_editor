//! Handler für Route-Tool-Operationen (Linie, Parkplatz, Kurve, …).

use crate::app::state::EditorTool;
use crate::app::tools::ToolAction;
use crate::app::use_cases;
use crate::app::AppState;

/// Verarbeitet einen Viewport-Klick im Route-Tool.
pub fn click(state: &mut AppState, world_pos: glam::Vec2, ctrl: bool) {
    let action = {
        let Some(road_map) = state.road_map.as_deref() else {
            return;
        };
        let Some(tool) = state.editor.tool_manager.active_tool_mut() else {
            return;
        };
        tool.on_click(world_pos, road_map, ctrl)
    };

    if action == ToolAction::ReadyToExecute {
        execute_and_apply(state);
    }
}

/// Führt das aktive Route-Tool aus (Enter-Bestätigung).
pub fn execute(state: &mut AppState) {
    execute_and_apply(state);
}

/// Gemeinsame Logik: Tool ausführen, Ergebnis anwenden, Tool zurücksetzen.
fn execute_and_apply(state: &mut AppState) {
    let result = match (
        state.editor.tool_manager.active_tool(),
        state.road_map.as_deref(),
    ) {
        (Some(tool), Some(rm)) => tool.execute(rm),
        _ => None,
    };

    if let Some(result) = result {
        let ids = use_cases::editing::apply_tool_result(state, result);

        // Segment in Registry speichern (für nachträgliche Bearbeitung)
        let record_id = state.segment_registry.next_id();
        if let Some(tool) = state.editor.tool_manager.active_tool() {
            if let Some(record) = tool.make_segment_record(record_id, ids.clone()) {
                state.segment_registry.register(record);
            }
        }

        if let (Some(tool), Some(rm)) = (
            state.editor.tool_manager.active_tool_mut(),
            state.road_map.as_deref(),
        ) {
            tool.set_last_created(ids, rm);
        }
    }

    if let Some(tool) = state.editor.tool_manager.active_tool_mut() {
        tool.reset();
    }
}

/// Bricht das aktive Route-Tool ab (Escape).
pub fn cancel(state: &mut AppState) {
    if let Some(tool) = state.editor.tool_manager.active_tool_mut() {
        tool.reset();
    }
}

/// Aktiviert ein Route-Tool per Index.
pub fn select(state: &mut AppState, index: usize) {
    state.editor.tool_manager.set_active(index);
    state.editor.active_tool = EditorTool::Route;
    state.editor.connect_source_node = None;
    let dir = state.editor.default_direction;
    let prio = state.editor.default_priority;
    let snap_r = state.options.snap_radius;
    if let Some(tool) = state.editor.tool_manager.active_tool_mut() {
        tool.set_direction(dir);
        tool.set_priority(prio);
        tool.set_snap_radius(snap_r);
    }
    log::info!("Route-Tool aktiviert: Index {}", index);
}

/// Löscht die letzte Strecke und erstellt sie mit neuen Parametern neu.
pub fn recreate(state: &mut AppState) {
    let old_ids = match state.editor.tool_manager.active_tool() {
        Some(tool) => tool.last_created_ids().to_vec(),
        None => return,
    };

    if old_ids.is_empty() {
        return;
    }

    // Undo-Snapshot VOR Löschung + Neuberechnung
    state.record_undo_snapshot();
    use_cases::editing::delete_nodes_by_ids(state, &old_ids);

    // Neu erstellen aus gespeicherten Ankern
    let result = match (
        state.editor.tool_manager.active_tool(),
        state.road_map.as_deref(),
    ) {
        (Some(tool), Some(rm)) => tool.execute_from_anchors(rm),
        _ => None,
    };

    if let Some(result) = result {
        let new_ids = use_cases::editing::apply_tool_result_no_snapshot(state, result);
        if let (Some(tool), Some(rm)) = (
            state.editor.tool_manager.active_tool_mut(),
            state.road_map.as_deref(),
        ) {
            tool.clear_recreate_flag();
            tool.set_last_created(new_ids, rm);
        }
    }
}

/// Startet einen Drag auf einem Steuerpunkt/Anker des aktiven Route-Tools.
pub fn drag_start(state: &mut AppState, world_pos: glam::Vec2) {
    let pick_radius = state.view.camera.pick_radius_world_scaled(
        state.view.viewport_size[1],
        state.options.selection_pick_radius_px,
    );
    let Some(road_map) = state.road_map.as_deref() else {
        return;
    };
    if let Some(tool) = state.editor.tool_manager.active_tool_mut() {
        tool.on_drag_start(world_pos, road_map, pick_radius);
    }
}

/// Aktualisiert die Position des gegriffenen Punkts während eines Drags.
pub fn drag_update(state: &mut AppState, world_pos: glam::Vec2) {
    if let Some(tool) = state.editor.tool_manager.active_tool_mut() {
        tool.on_drag_update(world_pos);
    }
}

/// Beendet den Drag (ggf. Re-Snap auf existierenden Node).
pub fn drag_end(state: &mut AppState) {
    let Some(road_map) = state.road_map.as_deref() else {
        return;
    };
    if let Some(tool) = state.editor.tool_manager.active_tool_mut() {
        tool.on_drag_end(road_map);
    }
}
