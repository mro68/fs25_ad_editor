//! Use-Cases für Map-Marker-Operationen.

use crate::app::AppState;
use crate::core::MapMarker;
use std::sync::Arc;

/// Öffnet den Marker-Dialog für einen Node (neu oder bearbeiten)
pub fn open_marker_dialog(state: &mut AppState, node_id: u64, is_new: bool) {
    state.ui.show_marker_dialog = true;
    state.ui.marker_dialog_node_id = Some(node_id);
    state.ui.marker_dialog_is_new = is_new;

    if is_new {
        state.ui.marker_dialog_name = format!("Marker {}", node_id);
        state.ui.marker_dialog_group = "All".to_string();
    } else if let Some(rm) = state.road_map.as_ref() {
        if let Some(marker) = rm.find_marker_by_node_id(node_id) {
            state.ui.marker_dialog_name = marker.name.clone();
            state.ui.marker_dialog_group = marker.group.clone();
        }
    }
}

/// Erstellt einen Map-Marker für einen Node mit den angegebenen Werten
pub fn create_marker(state: &mut AppState, node_id: u64, name: &str, group: &str) {
    let Some(road_map_arc) = state.road_map.as_ref() else {
        log::warn!("Kein RoadMap geladen");
        return;
    };

    if !road_map_arc.nodes.contains_key(&node_id) {
        log::warn!("Node {} existiert nicht", node_id);
        return;
    }

    if road_map_arc.has_marker(node_id) {
        log::warn!("Node {} hat bereits einen Marker", node_id);
        return;
    }

    state.record_undo_snapshot();

    let Some(road_map_arc) = state.road_map.as_mut() else {
        log::warn!("Marker-Erstellung abgebrochen: keine RoadMap geladen");
        return;
    };
    let road_map = Arc::make_mut(road_map_arc);
    let marker_index = road_map.map_markers.len() as u32 + 1;

    let marker = MapMarker {
        id: node_id,
        name: name.to_string(),
        group: group.to_string(),
        marker_index,
        is_debug: false,
    };

    road_map.add_map_marker(marker);
    log::info!(
        "Marker für Node {} erstellt (Name: {}, Gruppe: {})",
        node_id,
        name,
        group
    );
}

/// Aktualisiert einen bestehenden Map-Marker
pub fn update_marker(state: &mut AppState, node_id: u64, name: &str, group: &str) {
    let Some(road_map_arc) = state.road_map.as_ref() else {
        log::warn!("Kein RoadMap geladen");
        return;
    };

    if !road_map_arc.has_marker(node_id) {
        log::warn!("Kein Marker bei Node {}", node_id);
        return;
    }

    state.record_undo_snapshot();

    let Some(road_map_arc) = state.road_map.as_mut() else {
        log::warn!("Marker-Update abgebrochen: keine RoadMap geladen");
        return;
    };
    let road_map = Arc::make_mut(road_map_arc);
    if let Some(marker) = road_map.map_markers.iter_mut().find(|m| m.id == node_id) {
        marker.name = name.to_string();
        marker.group = group.to_string();
        log::info!(
            "Marker bei Node {} aktualisiert (Name: {}, Gruppe: {})",
            node_id,
            name,
            group
        );
    }
}

/// Entfernt den Map-Marker eines Nodes
pub fn remove_marker(state: &mut AppState, node_id: u64) {
    let Some(road_map_arc) = state.road_map.as_ref() else {
        log::warn!("Kein RoadMap geladen");
        return;
    };

    if !road_map_arc.has_marker(node_id) {
        log::debug!("Kein Marker bei Node {}", node_id);
        return;
    }

    // Undo-Snapshot VOR Mutation
    state.record_undo_snapshot();

    // RoadMap mutieren
    let Some(road_map_arc) = state.road_map.as_mut() else {
        log::warn!("Marker-Entfernen abgebrochen: keine RoadMap geladen");
        return;
    };
    let road_map = Arc::make_mut(road_map_arc);

    if road_map.remove_marker(node_id) {
        log::info!("Marker bei Node {} entfernt", node_id);
    }
}
