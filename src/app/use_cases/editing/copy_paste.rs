//! Use-Cases fuer Kopieren/Einfuegen (Copy/Paste) von Nodes, Verbindungen und Markern.

use crate::app::state::Clipboard;
use crate::app::AppState;
use crate::core::{Connection, MapMarker, MapNode};
use glam::Vec2;
use std::collections::HashMap;
use std::sync::Arc;

/// Kopiert alle selektierten Nodes, interne Verbindungen und Marker in die Zwischenablage.
///
/// "Intern" bedeutet: beide Endpunkte der Verbindung sind selektiert.
/// Das geometrische Zentrum der kopierten Nodes wird als Referenzpunkt gespeichert.
pub fn copy_selected_to_clipboard(state: &mut AppState) {
    let Some(road_map) = state.road_map.as_deref() else {
        log::warn!("Kopieren nicht moeglich: keine RoadMap geladen");
        return;
    };

    let selected_ids = &state.selection.selected_node_ids;
    if selected_ids.is_empty() {
        log::debug!("copy_selected_to_clipboard: keine Selektion");
        return;
    }

    // Nodes kopieren
    let nodes: Vec<MapNode> = selected_ids
        .iter()
        .filter_map(|&id| road_map.nodes.get(&id).cloned())
        .collect();

    if nodes.is_empty() {
        return;
    }

    // Geometrisches Zentrum berechnen
    let center = nodes.iter().fold(Vec2::ZERO, |acc, n| acc + n.position) / nodes.len() as f32;

    // Interne Verbindungen: beide Endpunkte muessen in der Selektion sein
    let selected_set: std::collections::HashSet<u64> = selected_ids.iter().copied().collect();
    let connections: Vec<Connection> = road_map
        .connections_iter()
        .filter(|c| selected_set.contains(&c.start_id) && selected_set.contains(&c.end_id))
        .cloned()
        .collect();

    // Marker: nur fuer selektierte Nodes
    let markers: Vec<MapMarker> = road_map
        .map_markers
        .iter()
        .filter(|m| selected_set.contains(&m.id))
        .cloned()
        .collect();

    let count_nodes = nodes.len();
    let count_conn = connections.len();
    let count_markers = markers.len();

    state.clipboard = Clipboard {
        nodes,
        connections,
        markers,
        center,
    };

    log::info!(
        "Zwischenablage: {} Nodes, {} Verbindungen, {} Marker kopiert (Zentrum: {:.1}/{:.1})",
        count_nodes,
        count_conn,
        count_markers,
        center.x,
        center.y
    );
}

/// Aktiviert den Einfuegen-Vorschau-Modus.
///
/// Setzt `paste_preview_pos` auf das Clipboard-Zentrum als Ausgangsposition.
/// Ohne Clipboard-Daten wird die Aktion ignoriert.
pub fn start_paste_preview(state: &mut AppState) {
    if state.clipboard.nodes.is_empty() {
        log::warn!("Paste fehlgeschlagen: Zwischenablage ist leer");
        return;
    }
    state.paste_preview_pos = Some(state.clipboard.center);
    log::debug!("Paste-Vorschau gestartet");
}

/// Aktualisiert die Vorschauposition beim Einfuegen.
pub fn update_paste_preview(state: &mut AppState, world_pos: Vec2) {
    if state.clipboard.nodes.is_empty() {
        return;
    }
    state.paste_preview_pos = Some(world_pos);
}

/// Bricht den Einfuegen-Vorschau-Modus ab.
pub fn cancel_paste_preview(state: &mut AppState) {
    state.paste_preview_pos = None;
    log::debug!("Paste-Vorschau abgebrochen");
}

/// Bestaetigt das Einfuegen an der aktuellen Vorschauposition.
///
/// - Erstellt einen Undo-Snapshot vor der Mutation.
/// - Vergibt neue IDs fortlaufend ab `next_node_id`.
/// - Berechnet den Positions-Offset: `preview_pos - clipboard.center`.
/// - Remappt alle Verbindungs-IDs und Marker-IDs auf die neuen IDs.
/// - Baut Geometrie und Spatial-Index neu auf.
/// - Selektiert die neu eingefuegten Nodes.
/// - Loescht die Vorschauposition.
pub fn confirm_paste(state: &mut AppState) {
    let preview_pos = match state.paste_preview_pos {
        Some(p) => p,
        None => {
            log::warn!("confirm_paste: keine Vorschauposition gesetzt");
            return;
        }
    };

    if state.clipboard.nodes.is_empty() {
        log::warn!("confirm_paste: Zwischenablage leer");
        state.paste_preview_pos = None;
        return;
    }

    let Some(_road_map_arc) = state.road_map.as_ref() else {
        log::warn!("confirm_paste: keine RoadMap geladen");
        state.paste_preview_pos = None;
        return;
    };

    // Undo-Snapshot vor Mutation
    state.record_undo_snapshot();

    let Some(road_map_arc) = state.road_map.as_mut() else {
        return;
    };
    let road_map = Arc::make_mut(road_map_arc);

    // Positions-Offset: Vorschauposition relativ zum gespeicherten Zentrum
    let offset = preview_pos - state.clipboard.center;

    // Neue IDs fortlaufend ab next_node_id vergeben
    let base_id = road_map.next_node_id();
    let id_map: HashMap<u64, u64> = state
        .clipboard
        .nodes
        .iter()
        .enumerate()
        .map(|(i, node)| (node.id, base_id + i as u64))
        .collect();

    // Nodes einfuegen
    let mut new_ids = Vec::with_capacity(state.clipboard.nodes.len());
    for node in &state.clipboard.nodes {
        let new_id = id_map[&node.id];
        let new_node = MapNode::new(new_id, node.position + offset, node.flag);
        road_map.add_node(new_node);
        new_ids.push(new_id);
    }

    // Verbindungen einfuegen (mit remappten IDs)
    for conn in &state.clipboard.connections {
        let Some(&new_start) = id_map.get(&conn.start_id) else {
            continue;
        };
        let Some(&new_end) = id_map.get(&conn.end_id) else {
            continue;
        };

        let start_pos = road_map
            .nodes
            .get(&new_start)
            .map(|n| n.position)
            .unwrap_or_default();
        let end_pos = road_map
            .nodes
            .get(&new_end)
            .map(|n| n.position)
            .unwrap_or_default();

        let new_conn = Connection::new(
            new_start,
            new_end,
            conn.direction,
            conn.priority,
            start_pos,
            end_pos,
        );
        road_map.add_connection(new_conn);
    }

    // Marker einfuegen (mit remappten Node-IDs)
    let next_marker_index = road_map.map_markers.len() as u32;
    for (i, marker) in state.clipboard.markers.iter().enumerate() {
        let Some(&new_node_id) = id_map.get(&marker.id) else {
            continue;
        };
        let new_marker = MapMarker {
            id: new_node_id,
            name: marker.name.clone(),
            group: marker.group.clone(),
            marker_index: next_marker_index + i as u32 + 1,
            is_debug: marker.is_debug,
        };
        road_map.add_map_marker(new_marker);
    }

    // Node-Flags und Spatial-Index aktualisieren
    road_map.recalculate_node_flags(&new_ids);
    road_map.ensure_spatial_index();

    // Neue Nodes selektieren
    state.selection.ids_mut().clear();
    for id in &new_ids {
        state.selection.ids_mut().insert(*id);
    }

    // Vorschau-Modus beenden
    state.paste_preview_pos = None;

    log::info!(
        "Paste bestaetigt: {} Nodes eingefuegt (IDs {}..{})",
        new_ids.len(),
        base_id,
        base_id + new_ids.len() as u64 - 1
    );
}
