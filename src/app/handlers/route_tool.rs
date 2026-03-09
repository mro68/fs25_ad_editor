//! Handler fuer Route-Tool-Operationen (Linie, Parkplatz, Kurve, …).

use crate::app::state::EditorTool;
use crate::app::tools::common::TangentSource;
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

/// Fuehrt das aktive Route-Tool aus (Enter-Bestaetigung).
pub fn execute(state: &mut AppState) {
    execute_and_apply(state);
}

/// Gemeinsame Logik: Tool ausfuehren, Ergebnis anwenden, Tool zuruecksetzen.
fn execute_and_apply(state: &mut AppState) {
    let result = match (
        state.editor.tool_manager.active_tool(),
        state.road_map.as_deref(),
    ) {
        (Some(tool), Some(rm)) => tool.execute(rm),
        _ => None,
    };

    if let Some(result) = result {
        // Marker-Indizes vor Consume von result sichern (fuer marker_node_ids im Record)
        let marker_indices: Vec<usize> = result.markers.iter().map(|(idx, _, _)| *idx).collect();

        let ids = use_cases::editing::apply_tool_result(state, result);

        // Zuerst last_*-Felder setzen (fuer make_segment_record)
        if let (Some(tool), Some(rm)) = (
            state.editor.tool_manager.active_tool_mut(),
            state.road_map.as_deref(),
        ) {
            tool.set_last_created(&ids, rm);
        }

        // Segment in Registry speichern (fuer nachtraegliche Bearbeitung)
        let record_id = state.segment_registry.next_id();
        if let Some(tool) = state.editor.tool_manager.active_tool() {
            if let Some(mut record) = tool.make_segment_record(record_id, &ids) {
                // Positionen aus RoadMap sammeln
                record.original_positions = record
                    .node_ids
                    .iter()
                    .filter_map(|id| state.road_map.as_ref()?.nodes.get(id).map(|n| n.position))
                    .collect();
                // Marker-Node-IDs fuer spaeteres Cleanup beim Edit
                record.marker_node_ids = marker_indices
                    .iter()
                    .filter_map(|idx| ids.get(*idx).copied())
                    .collect();
                state.segment_registry.register(record);
            }
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
        state.editor.active_tool = EditorTool::Select;
    }
}

/// Aktiviert ein Route-Tool per Index.
pub fn select(state: &mut AppState, index: usize) {
    state.editor.tool_manager.set_active(index);
    state.editor.active_tool = EditorTool::Route;
    state.editor.connect_source_node = None;
    let dir = state.editor.default_direction;
    let prio = state.editor.default_priority;
    let snap_r = state.options.snap_radius();
    let farmland = state.farmland_polygons.clone();
    if let Some(tool) = state.editor.tool_manager.active_tool_mut() {
        tool.set_direction(dir);
        tool.set_priority(prio);
        tool.set_snap_radius(snap_r);
        tool.set_farmland_data(farmland);
    }

    // Kette in chain-basierte Tools laden (z.B. BypassTool)
    init_chain_if_needed(state);

    log::info!("Route-Tool aktiviert: Index {}", index);
}

/// Laedt die aktuelle Selektion als geordnete Kette in das aktive Tool,
/// falls dieses `needs_chain_input()` zurueckgibt.
pub fn init_chain_if_needed(state: &mut AppState) {
    let needs_chain = state
        .editor
        .tool_manager
        .active_tool()
        .is_some_and(|t| t.needs_chain_input());
    if !needs_chain {
        return;
    }

    let Some(road_map) = state.road_map.as_deref() else {
        return;
    };

    let Some(ordered_ids) = road_map.ordered_chain_nodes(&state.selection.selected_node_ids) else {
        log::debug!("Bypass-Tool: Selektion bildet keine lineare Kette");
        return;
    };

    let positions: Vec<glam::Vec2> = ordered_ids
        .iter()
        .filter_map(|id| road_map.nodes.get(id).map(|n| n.position))
        .collect();

    if positions.len() < 2 {
        return;
    }

    let start_id = *ordered_ids.first().unwrap();
    let end_id = *ordered_ids.last().unwrap();

    if let Some(tool) = state.editor.tool_manager.active_tool_mut() {
        tool.load_chain(positions, start_id, end_id);
        // Innere Node-IDs fuer "Original entfernen" explizit setzen (ueberschreibt Inferenz)
        let n = ordered_ids.len();
        if n > 2 {
            let inner_ids: Vec<u64> = ordered_ids[1..n - 1].to_vec();
            tool.set_chain_inner_ids(inner_ids);
        }
        log::info!(
            "Route-Tool Kette geladen: {} Nodes ({} → {})",
            ordered_ids.len(),
            start_id,
            end_id
        );
    }
}

/// Aktiviert ein Route-Tool und setzt Start/End-Anker aus zwei selektierten Nodes.
///
/// Simuliert die beiden on_click()-Aufrufe mit den Node-Positionen.
/// Bei StraightLine → ReadyToExecute → sofortige Ausfuehrung.
/// Bei Curves → Phase::Control → User platziert Kontrollpunkte.
pub fn select_with_anchors(
    state: &mut AppState,
    index: usize,
    start_node_id: u64,
    end_node_id: u64,
) {
    // Tool aktivieren (inkl. Direction/Priority/SnapRadius)
    select(state, index);

    // Immer mit frischem Zustand starten, auch wenn dasselbe Tool bereits aktiv war.
    // So ist das Verhalten identisch zum manuellen Flow
    // (Tool waehlen -> Start klicken -> Ende klicken).
    if let Some(tool) = state.editor.tool_manager.active_tool_mut() {
        tool.reset();
    }

    // Node-Positionen holen
    let (start_pos, end_pos) = {
        let Some(road_map) = state.road_map.as_deref() else {
            return;
        };
        let start = road_map.nodes.get(&start_node_id);
        let end = road_map.nodes.get(&end_node_id);
        match (start, end) {
            (Some(s), Some(e)) => (s.position, e.position),
            _ => {
                log::warn!(
                    "Route-Tool mit Ankern: Nodes {}/{} nicht gefunden",
                    start_node_id,
                    end_node_id
                );
                return;
            }
        }
    };

    // Selektion aufheben (User ist jetzt im Route-Tool)
    let (old_selected, old_anchor) =
        crate::app::handlers::helpers::capture_selection_snapshot(state);
    state.selection.ids_mut().clear();
    crate::app::handlers::helpers::record_selection_if_changed(state, old_selected, old_anchor);

    // Ersten Klick simulieren (Start-Anker)
    let action1 = {
        let Some(road_map) = state.road_map.as_deref() else {
            return;
        };
        let Some(tool) = state.editor.tool_manager.active_tool_mut() else {
            return;
        };
        tool.on_click(start_pos, road_map, false)
    };

    if action1 == ToolAction::ReadyToExecute {
        execute_and_apply(state);
        return;
    }

    // Zweiten Klick simulieren (End-Anker)
    let action2 = {
        let Some(road_map) = state.road_map.as_deref() else {
            return;
        };
        let Some(tool) = state.editor.tool_manager.active_tool_mut() else {
            return;
        };
        tool.on_click(end_pos, road_map, false)
    };

    if action2 == ToolAction::ReadyToExecute {
        execute_and_apply(state);
    }
    // Sonst: Curve-Tool in Phase::Control → User platziert Kontrollpunkte
}

/// Loescht die letzte Strecke und erstellt sie mit neuen Parametern neu.
pub fn recreate(state: &mut AppState) {
    let old_ids = match state.editor.tool_manager.active_tool() {
        Some(tool) => {
            let ids = tool.last_created_ids();
            if ids.is_empty() {
                return;
            }
            ids.to_vec()
        }
        None => return,
    };

    // Undo-Snapshot VOR Loeschung + Neuberechnung
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
            tool.set_last_created(&new_ids, rm);
        }
    }
}

/// Wendet die vom User gewaehlten Tangenten an und triggert ggf. eine Neuberechnung.
pub fn apply_tangent(state: &mut AppState, start: TangentSource, end: TangentSource) {
    let needs_recreate = if let Some(tool) = state.editor.tool_manager.active_tool_mut() {
        tool.apply_tangent_selection(start, end);
        tool.needs_recreate()
    } else {
        false
    };

    if needs_recreate {
        recreate(state);
    }
}

/// Startet einen Drag auf einem Steuerpunkt/Anker des aktiven Route-Tools.
pub fn drag_start(state: &mut AppState, world_pos: glam::Vec2) {
    let pick_radius = state.options.hitbox_radius();
    let Some(road_map) = state.road_map.as_deref() else {
        return;
    };
    if let Some(tool) = state.editor.tool_manager.active_tool_mut() {
        tool.on_drag_start(world_pos, road_map, pick_radius);
    }
}

/// Aktualisiert die Position des gegriffenen Punkts waehrend eines Drags.
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

/// Verarbeitet Alt+Scroll-Rotation fuer das aktive Route-Tool.
pub fn rotate(state: &mut AppState, delta: f32) {
    if let Some(tool) = state.editor.tool_manager.active_tool_mut() {
        tool.on_scroll_rotate(delta);
    }
}

/// Erhoeht die Anzahl der Nodes im aktiven Route-Tool um 1.
pub fn increase_node_count(state: &mut AppState) {
    if let Some(tool) = state.editor.tool_manager.active_tool_mut() {
        tool.increase_node_count();
    }

    // Wenn das Tool sagt dass Recreate noetig ist → Segment neu erstellen
    let needs_recreate = state
        .editor
        .tool_manager
        .active_tool()
        .map(|t| t.needs_recreate())
        .unwrap_or(false);

    if needs_recreate {
        recreate(state);
    }
}

/// Verringert die Anzahl der Nodes im aktiven Route-Tool um 1 (min. 2).
pub fn decrease_node_count(state: &mut AppState) {
    if let Some(tool) = state.editor.tool_manager.active_tool_mut() {
        tool.decrease_node_count();
    }

    let needs_recreate = state
        .editor
        .tool_manager
        .active_tool()
        .map(|t| t.needs_recreate())
        .unwrap_or(false);

    if needs_recreate {
        recreate(state);
    }
}

/// Erhoeht den minimalen Segment-Abstand im aktiven Route-Tool um 0.25m.
pub fn increase_segment_length(state: &mut AppState) {
    if let Some(tool) = state.editor.tool_manager.active_tool_mut() {
        tool.increase_segment_length();
    }

    let needs_recreate = state
        .editor
        .tool_manager
        .active_tool()
        .map(|t| t.needs_recreate())
        .unwrap_or(false);

    if needs_recreate {
        recreate(state);
    }
}

/// Verringert den minimalen Segment-Abstand im aktiven Route-Tool um 0.25m (min. 0.1m).
pub fn decrease_segment_length(state: &mut AppState) {
    if let Some(tool) = state.editor.tool_manager.active_tool_mut() {
        tool.decrease_segment_length();
    }

    let needs_recreate = state
        .editor
        .tool_manager
        .active_tool()
        .map(|t| t.needs_recreate())
        .unwrap_or(false);

    if needs_recreate {
        recreate(state);
    }
}
