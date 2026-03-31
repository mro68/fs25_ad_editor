//! Handler fuer Route-Tool-Operationen (Linie, Parkplatz, Kurve, …).

mod adjustments;
mod apply;
mod selection;

use crate::app::state::EditorTool;
use crate::app::tools::common::TangentSource;
use crate::app::tools::{RouteToolId, ToolAction};
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
        apply::execute_and_apply(state);
    }
}

/// Leitet ein abgeschlossenes Lasso-Polygon (Weltkoordinaten) an das aktive Route-Tool weiter.
///
/// Wird aufgerufen wenn der User einen Alt+Drag-Lasso abgeschlossen hat und
/// das aktive Tool `needs_lasso_input()` meldet.
pub fn lasso_completed(state: &mut AppState, polygon: Vec<glam::Vec2>) {
    let action = {
        let Some(tool) = state.editor.tool_manager.active_tool_mut() else {
            return;
        };
        tool.on_lasso_completed(polygon)
    };

    if action == ToolAction::ReadyToExecute {
        apply::execute_and_apply(state);
    }
}

/// Fuehrt das aktive Route-Tool aus (Enter-Bestaetigung).
pub fn execute(state: &mut AppState) {
    apply::execute_and_apply(state);
}

/// Bricht das aktive Route-Tool ab (Escape).
///
/// War ein Segment im Tool-Edit-Modus, wird durch Undo der Zustand vor der
/// Bearbeitung wiederhergestellt (Nodes zurueck, nicht geloescht).
pub fn cancel(state: &mut AppState) {
    if let Some(tool) = state.editor.tool_manager.active_tool_mut() {
        tool.reset();
        state.editor.active_tool = EditorTool::Select;
    }
    if state.tool_editing_record_id.take().is_some() {
        super::history::undo(state);
        // Gesicherten Record in die Registry zurueckschreiben
        if let Some(backup) = state.tool_editing_record_backup.take() {
            state.group_registry.register(backup);
        }
        log::info!("Tool-Edit abgebrochen: Undo ausgefuehrt, Record wiederhergestellt");
    }
}

/// Aktiviert ein Route-Tool per stabiler Tool-ID.
pub fn select(state: &mut AppState, tool_id: RouteToolId) {
    selection::select(state, tool_id);
}

/// Laedt die aktuelle Selektion als geordnete Kette in das aktive Tool,
/// falls dieses `needs_chain_input()` zurueckgibt.
pub fn init_chain_if_needed(state: &mut AppState) {
    selection::init_chain_if_needed(state);
}

/// Aktiviert ein Route-Tool und setzt Start/End-Anker aus zwei selektierten Nodes.
///
/// Simuliert die beiden on_click()-Aufrufe mit den Node-Positionen.
/// Bei StraightLine → ReadyToExecute → sofortige Ausfuehrung.
/// Bei Curves → Phase::Control → User platziert Kontrollpunkte.
pub fn select_with_anchors(
    state: &mut AppState,
    tool_id: RouteToolId,
    start_node_id: u64,
    end_node_id: u64,
) {
    selection::select_with_anchors(state, tool_id, start_node_id, end_node_id);
}

/// Loescht die letzte Strecke und erstellt sie mit neuen Parametern neu.
pub fn recreate(state: &mut AppState) {
    apply::recreate(state);
}

/// Wendet die vom User gewaehlten Tangenten an und triggert ggf. eine Neuberechnung.
pub fn apply_tangent(state: &mut AppState, start: TangentSource, end: TangentSource) {
    adjustments::apply_tangent(state, start, end);
}

/// Startet einen Drag auf einem Steuerpunkt/Anker des aktiven Route-Tools.
pub fn drag_start(state: &mut AppState, world_pos: glam::Vec2) {
    adjustments::drag_start(state, world_pos);
}

/// Aktualisiert die Position des gegriffenen Punkts waehrend eines Drags.
pub fn drag_update(state: &mut AppState, world_pos: glam::Vec2) {
    adjustments::drag_update(state, world_pos);
}

/// Beendet den Drag (ggf. Re-Snap auf existierenden Node).
pub fn drag_end(state: &mut AppState) {
    adjustments::drag_end(state);
}

/// Verarbeitet Alt+Scroll-Rotation fuer das aktive Route-Tool.
pub fn rotate(state: &mut AppState, delta: f32) {
    adjustments::rotate(state, delta);
}

/// Erhoeht die Anzahl der Nodes im aktiven Route-Tool um 1.
pub fn increase_node_count(state: &mut AppState) {
    adjustments::increase_node_count(state);
}

/// Verringert die Anzahl der Nodes im aktiven Route-Tool um 1 (min. 2).
pub fn decrease_node_count(state: &mut AppState) {
    adjustments::decrease_node_count(state);
}

/// Erhoeht den minimalen Segment-Abstand im aktiven Route-Tool um 0.25m.
pub fn increase_segment_length(state: &mut AppState) {
    adjustments::increase_segment_length(state);
}

/// Verringert den minimalen Segment-Abstand im aktiven Route-Tool um 0.25m (min. 0.1m).
pub fn decrease_segment_length(state: &mut AppState) {
    adjustments::decrease_segment_length(state);
}
