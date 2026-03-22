//! Handler fuer Segment-Operationen (Lock-Toggle, Group-Edit).

use crate::app::group_registry::{GroupBase, GroupKind, GroupRecord};
use crate::app::state::GroupEditState;
use crate::app::tools::ToolAnchor;
use crate::app::AppState;

/// Schaltet den Lock-Zustand eines Segments um.
///
/// Ist das Segment gesperrt, werden beim Verschieben eines zugehoerigen Nodes
/// alle Segment-Nodes mitbewegt. Unbekannte IDs werden ignoriert.
pub fn toggle_lock(state: &mut AppState, segment_id: u64) {
    state.group_registry.toggle_lock(segment_id);
}

/// Loest ein Segment auf, indem nur der Segment-Record entfernt wird.
///
/// Die zugehoerigen Nodes und Verbindungen in der RoadMap bleiben unveraendert.
/// Unbekannte IDs werden ignoriert.
pub fn dissolve(state: &mut AppState, segment_id: u64) {
    state.group_registry.remove(segment_id);
}

/// Entfernt alle selektierten Nodes aus ihren Gruppen.
///
/// Nodes und Verbindungen in der RoadMap bleiben unveraendert.
/// Gruppen mit weniger als 2 verbleibenden Nodes werden automatisch aufgeloest.
/// Ist keine Selektion aktiv, wird die Funktion ohne Effekt beendet.
pub fn remove_selected_from_groups(state: &mut AppState) {
    let selected: Vec<u64> = state.selection.selected_node_ids.iter().copied().collect();
    if selected.is_empty() {
        return;
    }

    // Alle betroffenen Record-IDs sammeln
    let mut affected_records: std::collections::HashSet<u64> = std::collections::HashSet::new();
    for &nid in &selected {
        for rid in state.group_registry.groups_for_node(nid) {
            affected_records.insert(rid);
        }
    }

    if affected_records.is_empty() {
        return;
    }

    state.record_undo_snapshot();

    // Pro Record: Betroffene Nodes entfernen
    for rid in affected_records {
        state
            .group_registry
            .remove_nodes_from_record(rid, &selected);
    }
}

/// Gruppiert die selektierten Nodes als neues Segment.
///
/// Voraussetzung: Die Nodes muessen ein zusammenhaengendes Netzwerk bilden
/// (Kreuzungen und Verzweigungen sind erlaubt). Ist die Selektion kein
/// zusammenhaengender Subgraph, wird der Aufruf ignoriert.
pub fn group_selection(state: &mut AppState) {
    let road_map = match state.road_map.as_deref() {
        Some(rm) => rm,
        None => return,
    };
    let selected_ids = &state.selection.selected_node_ids;

    // Nodes muessen einen zusammenhaengenden Subgraphen bilden (keine lineare Kette noetig)
    if !road_map.is_connected_subgraph(selected_ids) {
        return;
    }

    let node_ids: Vec<u64> = selected_ids.iter().copied().collect();

    let original_positions: Vec<_> = node_ids
        .iter()
        .filter_map(|id| road_map.nodes.get(id).map(|n| n.position))
        .collect();

    let record_id = state.group_registry.next_id();
    let record = GroupRecord {
        id: record_id,
        node_ids,
        start_anchor: ToolAnchor::NewPosition(glam::Vec2::ZERO),
        end_anchor: ToolAnchor::NewPosition(glam::Vec2::ZERO),
        kind: GroupKind::Manual {
            base: GroupBase {
                direction: state.editor.default_direction,
                priority: state.editor.default_priority,
                max_segment_length: state.options.mouse_wheel_distance_step_m.max(1.0),
            },
        },
        original_positions,
        marker_node_ids: vec![],
        locked: false,
        entry_node_id: None,
        exit_node_id: None,
    };
    state.group_registry.register(record);
}

/// Setzt Einfahrt- und Ausfahrt-Node-IDs fuer eine Gruppe.
///
/// Delegiert an `GroupRegistry::set_entry_exit()`. Gibt eine Warnung aus
/// wenn die angegebenen Node-IDs nicht zur Gruppe gehoeren.
pub fn set_boundary_nodes(
    state: &mut AppState,
    record_id: u64,
    entry: Option<u64>,
    exit: Option<u64>,
) {
    if !state.group_registry.set_entry_exit(record_id, entry, exit) {
        log::warn!(
            "set_boundary_nodes: Validierung fehlgeschlagen fuer Record {}",
            record_id
        );
    }
}

/// Startet den nicht-destruktiven Gruppen-Edit-Modus fuer einen Record.
///
/// Erstellt einen Undo-Snapshot, entsperrt den Record temporaer und
/// selektiert alle zugehoerigen Nodes. Gibt eine Warnung aus wenn kein
/// Record mit der angegebenen ID existiert oder bereits ein Edit aktiv ist.
pub fn start_group_edit(state: &mut AppState, record_id: u64) {
    // Pruefen ob Record existiert
    let (was_locked, node_ids) = {
        let record = match state.group_registry.get(record_id) {
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
    state.group_registry.set_edit_guard(Some(record_id));

    // Temporaer entsperren damit Nodes einzeln bewegt werden koennen
    if was_locked {
        state.group_registry.set_locked(record_id, false);
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
    let original_node_ids: Vec<u64> = match state.group_registry.get(record_id) {
        Some(r) => r.node_ids.clone(),
        None => {
            log::warn!("apply_group_edit: record {} not found", record_id);
            state.group_registry.set_edit_guard(None);
            return;
        }
    };

    let road_map = match state.road_map.as_deref() {
        Some(rm) => rm,
        None => {
            state.group_registry.set_edit_guard(None);
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

    let filtered_count = selected
        .len()
        .saturating_sub(new_ids.len().saturating_sub(original_count));
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
        .group_registry
        .update_record(record_id, new_node_ids, positions);

    // Lock-Zustand wiederherstellen
    if edit_state.was_locked {
        state.group_registry.set_locked(record_id, true);
    }

    // Edit-Guard aufheben
    state.group_registry.set_edit_guard(None);

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
    state.group_registry.set_edit_guard(None);

    // Undo zum Snapshot vor Edit-Start
    super::history::undo(state);

    log::info!("Group edit cancelled");
}
