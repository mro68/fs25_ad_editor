//! Handler fuer Segment-Operationen (Lock-Toggle, Group-Edit).

use crate::app::segment_registry::{SegmentBase, SegmentKind, SegmentRecord};
use crate::app::state::GroupEditState;
use crate::app::tools::ToolAnchor;
use crate::app::AppState;

/// Schaltet den Lock-Zustand eines Segments um.
///
/// Ist das Segment gesperrt, werden beim Verschieben eines zugehoerigen Nodes
/// alle Segment-Nodes mitbewegt. Unbekannte IDs werden ignoriert.
pub fn toggle_lock(state: &mut AppState, segment_id: u64) {
    state.segment_registry.toggle_lock(segment_id);
}

/// Loest ein Segment auf, indem nur der Segment-Record entfernt wird.
///
/// Die zugehoerigen Nodes und Verbindungen in der RoadMap bleiben unveraendert.
/// Unbekannte IDs werden ignoriert.
pub fn dissolve(state: &mut AppState, segment_id: u64) {
    state.segment_registry.remove(segment_id);
}

/// Gruppiert die selektierten zusammenhaengenden Nodes als neues Segment.
///
/// Die Nodes werden in Ketten-Reihenfolge gespeichert. Ist die Selektion
/// keine gueltige lineare Kette, wird der Aufruf ignoriert.
pub fn group_selection(state: &mut AppState) {
    let road_map = match state.road_map.as_deref() {
        Some(rm) => rm,
        None => return,
    };
    let selected_ids = &state.selection.selected_node_ids;
    let ordered_ids = match road_map.ordered_chain_nodes(selected_ids) {
        Some(ids) => ids,
        None => return,
    };

    let start_id = *ordered_ids.first().unwrap();
    let end_id = *ordered_ids.last().unwrap();
    let start_pos = match road_map.nodes.get(&start_id) {
        Some(n) => n.position,
        None => return,
    };
    let end_pos = match road_map.nodes.get(&end_id) {
        Some(n) => n.position,
        None => return,
    };

    let original_positions: Vec<_> = ordered_ids
        .iter()
        .filter_map(|id| road_map.nodes.get(id).map(|n| n.position))
        .collect();

    let record_id = state.segment_registry.next_id();
    let record = SegmentRecord {
        id: record_id,
        node_ids: ordered_ids,
        start_anchor: ToolAnchor::ExistingNode(start_id, start_pos),
        end_anchor: ToolAnchor::ExistingNode(end_id, end_pos),
        kind: SegmentKind::Manual {
            base: SegmentBase {
                direction: state.editor.default_direction,
                priority: state.editor.default_priority,
                max_segment_length: state.options.mouse_wheel_distance_step_m.max(1.0),
            },
        },
        original_positions,
        marker_node_ids: vec![],
        locked: false,
    };
    state.segment_registry.register(record);
}

/// Startet den nicht-destruktiven Gruppen-Edit-Modus fuer einen Record.
///
/// Erstellt einen Undo-Snapshot, entsperrt den Record temporaer und
/// selektiert alle zugehoerigen Nodes. Gibt eine Warnung aus wenn kein
/// Record mit der angegebenen ID existiert oder bereits ein Edit aktiv ist.
pub fn start_group_edit(state: &mut AppState, record_id: u64) {
    // Pruefen ob Record existiert
    let (was_locked, node_ids) = {
        let record = match state.segment_registry.get(record_id) {
            Some(r) => r,
            None => {
                log::warn!("start_group_edit: record {} not found", record_id);
                return;
            }
        };
        (record.locked, record.node_ids.clone())
    };

    // Pruefen ob bereits ein Group-Edit aktiv
    if state.group_editing.is_some() {
        log::warn!("start_group_edit: already editing a group");
        return;
    }

    // Undo-Snapshot erstellen (vor jeder Aenderung)
    state.record_undo_snapshot();

    // Edit-State setzen
    state.group_editing = Some(GroupEditState {
        record_id,
        was_locked,
    });

    // Edit-Guard in Registry setzen (schuetzt vor Invalidierung)
    state.segment_registry.set_edit_guard(Some(record_id));

    // Temporaer entsperren damit Nodes einzeln bewegt werden koennen
    if was_locked {
        state.segment_registry.set_locked(record_id, false);
    }

    // Alle Nodes des Records selektieren
    {
        let ids = state.selection.ids_mut();
        ids.clear();
        for id in &node_ids {
            ids.insert(*id);
        }
    }

    log::info!("Group edit started for record {}", record_id);
}

/// Schliesst den Gruppen-Edit-Modus ab und uebernimmt alle Aenderungen.
///
/// Berechnet neue Node-IDs (Original minus geloeschte, plus selektierte neue),
/// aktualisiert den Record und stellt den Lock-Zustand wieder her.
pub fn apply_group_edit(state: &mut AppState) {
    let edit_state = match state.group_editing.take() {
        Some(es) => es,
        None => {
            log::warn!("apply_group_edit: no active group edit");
            return;
        }
    };

    let record_id = edit_state.record_id;

    // Aktuelle Node-IDs des Records lesen
    let original_node_ids: Vec<u64> = match state.segment_registry.get(record_id) {
        Some(r) => r.node_ids.clone(),
        None => {
            log::warn!("apply_group_edit: record {} not found", record_id);
            state.segment_registry.set_edit_guard(None);
            return;
        }
    };

    let road_map = match state.road_map.as_deref() {
        Some(rm) => rm,
        None => {
            state.segment_registry.set_edit_guard(None);
            return;
        }
    };

    // Neue Node-ID-Menge berechnen:
    // Original-IDs behalten (wenn noch in RoadMap vorhanden) +
    // aktuell selektierte IDs hinzufuegen + deduplizieren
    let mut new_ids: indexmap::IndexSet<u64> = indexmap::IndexSet::new();

    for id in &original_node_ids {
        if road_map.nodes.contains_key(id) {
            new_ids.insert(*id);
        }
    }

    let selected: Vec<u64> = state.selection.selected_node_ids.iter().copied().collect();

    // Iterativer Expansions-Algorithmus: selektierte Nodes nur hinzufuegen,
    // wenn sie eine Verbindung zu einem bereits erreichbaren Node haben.
    // Neue Nodes koennen Bruecken zu weiteren Nodes bilden (A→B→C).
    let original_count = new_ids.len();
    let mut reachable: indexmap::IndexSet<u64> = new_ids.clone();
    let mut changed = true;
    while changed {
        changed = false;
        for &sel_id in &selected {
            if reachable.contains(&sel_id) {
                continue;
            }
            // Pruefen ob sel_id eine Verbindung zu einem erreichbaren Node hat
            let has_connection = road_map.connections_iter().any(|conn| {
                (conn.start_id == sel_id && reachable.contains(&conn.end_id))
                    || (conn.end_id == sel_id && reachable.contains(&conn.start_id))
            });
            if has_connection {
                reachable.insert(sel_id);
                changed = true;
            }
        }
    }

    // Nur erreichbare selektierte Nodes hinzufuegen
    for &sel_id in &selected {
        if reachable.contains(&sel_id) {
            new_ids.insert(sel_id);
        }
    }

    let filtered_count = selected.len().saturating_sub(new_ids.len().saturating_sub(original_count));
    if filtered_count > 0 {
        log::info!(
            "apply_group_edit: {} nodes filtered (no connection to group)",
            filtered_count
        );
    }

    let new_node_ids: Vec<u64> = new_ids.into_iter().collect();

    // Positionen der neuen Node-IDs sammeln
    let positions: Vec<glam::Vec2> = new_node_ids
        .iter()
        .filter_map(|id| road_map.nodes.get(id).map(|n| n.position))
        .collect();

    // Record aktualisieren
    state
        .segment_registry
        .update_record(record_id, new_node_ids, positions);

    // Lock-Zustand wiederherstellen
    if edit_state.was_locked {
        state.segment_registry.set_locked(record_id, true);
    }

    // Edit-Guard aufheben
    state.segment_registry.set_edit_guard(None);

    log::info!("Group edit applied for record {}", record_id);
}

/// Bricht den Gruppen-Edit-Modus ab und stellt den Zustand via Undo wieder her.
///
/// Der Undo-Snapshot wurde in `start_group_edit` angelegt.
pub fn cancel_group_edit(state: &mut AppState) {
    if state.group_editing.is_none() {
        log::warn!("cancel_group_edit: no active group edit");
        return;
    }

    // Edit-State und Guard aufraumen
    state.group_editing = None;
    state.segment_registry.set_edit_guard(None);

    // Undo zum Snapshot vor Edit-Start
    super::history::undo(state);

    log::info!("Group edit cancelled");
}
