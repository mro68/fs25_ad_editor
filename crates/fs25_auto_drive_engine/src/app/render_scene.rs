//! Builder fuer Render-Szenen aus dem AppState.
//!
//! Dieses Modul ist verantwortlich fuer die Transformation des internen AppState
//! in den expliziten Render-Vertrag `RenderScene`. Die gebaute Szene enthaelt alle
//! Informationen, die der Render-Layer benoetigt, ohne den State direkt zu koppeln.

use crate::app::{AppState, GroupRegistry};
use crate::core::{ConnectionDirection, ConnectionPriority, NodeFlag, RoadMap};
use crate::shared::{
    RenderCamera, RenderConnection, RenderConnectionDirection, RenderConnectionPriority, RenderMap,
    RenderMarker, RenderNode, RenderNodeKind, RenderScene, RenderSceneFrameData,
};
use indexmap::IndexSet;
use std::collections::HashMap;
use std::mem::size_of;
use std::sync::{Arc, OnceLock};

/// Gibt einen Arc auf eine leere, statisch initialisierte `IndexSet<u64>` zurueck.
///
/// Verhindert eine Heap-Allokation pro Frame, wenn kein Node ausgeblendet werden soll.
/// Die Instanz wird beim ersten Aufruf lazy erstellt und danach wiederverwendet.
fn empty_hidden_ids() -> Arc<IndexSet<u64>> {
    static EMPTY: OnceLock<Arc<IndexSet<u64>>> = OnceLock::new();
    Arc::clone(EMPTY.get_or_init(|| Arc::new(IndexSet::new())))
}

/// Berechnet die zu dimmenden Node-IDs fuer einen Frame.
///
/// Fuer alle selektierten Nodes werden die betroffenen Segmente ermittelt.
/// Alle Segment-Nodes, die NICHT selektiert sind, werden in die Dimm-Menge aufgenommen.
/// Bei leerer Selektion oder wenn kein Node zu einem Segment gehoert, wird eine
/// leere Menge zurueckgegeben.
///
/// Implementierung als einziger Pass ueber alle Records statt pro-Node-Lookup —
/// effizienter fuer den Frame-Hot-Path bei vielen selektierten Nodes.
fn compute_dimmed_ids(
    registry: &GroupRegistry,
    selected: &Arc<IndexSet<u64>>,
) -> Arc<IndexSet<u64>> {
    if selected.is_empty() {
        return empty_hidden_ids();
    }
    let mut dimmed = IndexSet::new();
    for record in registry.records() {
        if record.node_ids.iter().any(|id| selected.contains(id)) {
            for &id in &record.node_ids {
                if !selected.contains(&id) {
                    dimmed.insert(id);
                }
            }
        }
    }
    if dimmed.is_empty() {
        empty_hidden_ids()
    } else {
        Arc::new(dimmed)
    }
}

fn build_render_map_snapshot(road_map: &RoadMap) -> RenderMap {
    let mut nodes = HashMap::with_capacity(road_map.node_count());
    for (&id, node) in road_map.nodes() {
        let kind = match node.flag {
            NodeFlag::SubPrio => RenderNodeKind::SubPrio,
            NodeFlag::Warning => RenderNodeKind::Warning,
            _ => RenderNodeKind::Regular,
        };

        nodes.insert(
            id,
            RenderNode {
                id,
                position: node.position,
                kind,
                preserve_when_decimating: node.flag == NodeFlag::RoundedCorner,
            },
        );
    }

    let mut connections = Vec::with_capacity(road_map.connection_count());
    for connection in road_map.connections_iter() {
        let Some(start_pos) = road_map.node_position(connection.start_id) else {
            continue;
        };
        let Some(end_pos) = road_map.node_position(connection.end_id) else {
            continue;
        };

        let direction = match connection.direction {
            ConnectionDirection::Regular => RenderConnectionDirection::Regular,
            ConnectionDirection::Dual => RenderConnectionDirection::Dual,
            ConnectionDirection::Reverse => RenderConnectionDirection::Reverse,
        };
        let priority = match connection.priority {
            ConnectionPriority::Regular => RenderConnectionPriority::Regular,
            ConnectionPriority::SubPriority => RenderConnectionPriority::SubPriority,
        };

        connections.push(RenderConnection {
            start_id: connection.start_id,
            end_id: connection.end_id,
            start_pos,
            end_pos,
            direction,
            priority,
        });
    }

    let mut markers = Vec::with_capacity(road_map.marker_count());
    for marker in road_map.map_markers() {
        if let Some(position) = road_map.node_position(marker.id) {
            markers.push(RenderMarker { position });
        }
    }

    RenderMap::new(nodes, connections, markers)
}

fn estimate_render_snapshot_bytes(snapshot: &RenderMap) -> usize {
    let node_bytes = snapshot.node_count() * size_of::<RenderNode>();
    let connection_bytes = snapshot.connection_count() * size_of::<RenderConnection>();
    let marker_bytes = snapshot.marker_count() * size_of::<RenderMarker>();
    node_bytes + connection_bytes + marker_bytes
}

fn render_map_snapshot(state: &AppState) -> Option<Arc<RenderMap>> {
    let road_map = state.road_map.as_deref()?;
    let (instance_id, revision) = road_map.render_cache_key();
    let mut cache = state.render_map_cache.borrow_mut();

    match cache.as_ref() {
        Some((cached_instance_id, cached_revision, snapshot))
            if *cached_instance_id == instance_id && *cached_revision == revision =>
        {
            Some(Arc::clone(snapshot))
        }
        _ => {
            let snapshot = Arc::new(build_render_map_snapshot(road_map));
            log::debug!(
                "RenderMap-Snapshot neu aufgebaut: nodes={}, connections={}, markers={}, approx_bytes={}",
                snapshot.node_count(),
                snapshot.connection_count(),
                snapshot.marker_count(),
                estimate_render_snapshot_bytes(snapshot.as_ref())
            );
            *cache = Some((instance_id, revision, Arc::clone(&snapshot)));
            Some(snapshot)
        }
    }
}

/// Baut eine RenderScene aus dem aktuellen AppState.
///
/// Diese Funktion extrahiert die notwendigen Daten aus dem `AppState` und
/// montiert sie in das explizite `RenderScene`-Datenmodell. Die Szene ist
/// der Render-Layer-Vertrag und deckt folgende Bereiche ab:
///
/// - **Geometrie**: render-seitiger Snapshot der Karte, `selected_node_ids`
/// - **Sichtbarkeit**: Hintergrundstatus, `background_visible`, `hidden_node_ids`
/// - **Viewport**: Kamera, Groesse der Anzeige, Render-Qualitaet
/// - **Konfiguration**: `options_arc` (EditorOptions als shared Arc)
///
/// # Besonderheiten
///
/// - `hidden_node_ids` wird automatisch mit selektierten Nodes gefuellt,
///   wenn die Distanzen-Vorschau aktiv ist und "Original ausblenden" aktiviert wurde.
/// - `options_arc` ist ein Arc-Clone von `state.options_arc()` — das ermoeglicht
///   CoW-Updates ohne per-Frame Allokationen.
/// - Der Karten-Snapshot wird ueber `RoadMap::render_cache_key()` lazy gecacht,
///   damit App und Render keinen Core-Typenvertrag mehr teilen muessen.
///
/// # Parameter
/// - `state` – Referenz zum aktuellen AppState
/// - `viewport_size` – Fenstergroesse in Pixeln als `[width, height]`
///
/// # Rueckgabe
/// Eine vollstaendige `RenderScene`, bereit zum Rendering.
pub fn build(state: &AppState, viewport_size: [f32; 2]) -> RenderScene {
    // Arc einmal klonen — wiederverwendet fuer selected_node_ids UND hidden_node_ids
    let selected_arc = state.selection.selected_node_ids.clone();

    // Wenn Distanzen-Vorschau aktiv + hide_original → selektierte Nodes ausblenden.
    // Statt nochmals zu klonen verwenden wir den gleichen Arc (billiger O(1)-Clone).
    let hidden_node_ids = if state.ui.distanzen.should_hide_original() {
        Arc::clone(&selected_arc)
    } else {
        empty_hidden_ids()
    };

    // Gedimmte Nodes: alle anderen Nodes des Segments wenn 1 Segment-Node selektiert.
    // Cache-Hit wenn weder Selektion noch Registry seit dem letzten Build geaendert haben.
    let dimmed_node_ids = {
        let sel_gen = state.selection.generation;
        let reg_gen = state.group_registry.dimmed_generation;
        let mut cache = state.dimmed_ids_cache.borrow_mut();
        match cache.as_ref() {
            Some((s, r, result)) if *s == sel_gen && *r == reg_gen => Arc::clone(result),
            _ => {
                let result = compute_dimmed_ids(&state.group_registry, &selected_arc);
                *cache = Some((sel_gen, reg_gen, Arc::clone(&result)));
                result
            }
        }
    };

    RenderScene::new(
        render_map_snapshot(state),
        RenderSceneFrameData {
            camera: RenderCamera::new(state.view.camera.position, state.view.camera.zoom),
            viewport_size,
            render_quality: state.view.render_quality,
            selected_node_ids: selected_arc,
            has_background: state.view.background_map.is_some(),
            background_visible: state.view.background_visible,
            options: state.options_arc(),
            hidden_node_ids,
            dimmed_node_ids,
        },
    )
}

#[cfg(test)]
mod tests {
    use super::render_map_snapshot;
    use crate::app::use_cases::{editing, selection};
    use crate::app::AppState;
    use crate::core::{MapNode, NodeFlag, RoadMap};
    use glam::Vec2;
    use std::sync::Arc;

    fn make_map() -> RoadMap {
        let mut map = RoadMap::new(3);
        map.add_node(MapNode::new(1, Vec2::new(0.0, 0.0), NodeFlag::Regular));
        map.add_node(MapNode::new(2, Vec2::new(10.0, 0.0), NodeFlag::Regular));
        map
    }

    fn make_state() -> AppState {
        let mut state = AppState::new();
        state.road_map = Some(Arc::new(make_map()));
        state
    }

    #[test]
    fn build_render_scene_without_map_is_stable() {
        let state = AppState::new();
        let scene = super::build(&state, [1280.0, 720.0]);

        assert!(!scene.has_map());
        assert!(!scene.has_background());
    }

    #[test]
    fn render_map_snapshot_hits_cache_when_road_map_is_unchanged() {
        let state = make_state();

        let first = render_map_snapshot(&state).expect("Snapshot vorhanden");
        let second = render_map_snapshot(&state).expect("Snapshot vorhanden");

        assert!(Arc::ptr_eq(&first, &second));
    }

    #[test]
    fn render_map_snapshot_misses_cache_for_new_road_map_instance() {
        let mut state = make_state();
        let first = render_map_snapshot(&state).expect("Snapshot vorhanden");

        state.road_map = Some(Arc::new(make_map()));

        let second = render_map_snapshot(&state).expect("Snapshot vorhanden");
        assert!(!Arc::ptr_eq(&first, &second));
    }

    #[test]
    fn render_map_snapshot_invalidates_after_move_selected_nodes() {
        let mut state = make_state();
        state.selection.ids_mut().insert(1);

        let first = render_map_snapshot(&state).expect("Snapshot vorhanden");

        selection::move_selected_nodes(&mut state, Vec2::new(5.0, 3.0));

        let second = render_map_snapshot(&state).expect("Snapshot vorhanden");
        assert!(!Arc::ptr_eq(&first, &second));
        assert_eq!(
            second.node(&1).expect("Render-Node vorhanden").position,
            Vec2::new(5.0, 3.0)
        );
    }

    #[test]
    fn render_map_snapshot_invalidates_after_rotate_selected_nodes() {
        let mut state = make_state();
        state.selection.ids_mut().insert(1);
        state.selection.ids_mut().insert(2);

        let first = render_map_snapshot(&state).expect("Snapshot vorhanden");

        selection::rotate_selected_nodes(&mut state, std::f32::consts::FRAC_PI_2);

        let second = render_map_snapshot(&state).expect("Snapshot vorhanden");
        assert!(!Arc::ptr_eq(&first, &second));

        let rotated = second.node(&1).expect("Render-Node vorhanden").position;
        assert!((rotated.x - 5.0).abs() < 1e-5);
        assert!((rotated.y + 5.0).abs() < 1e-5);
    }

    #[test]
    fn render_map_snapshot_invalidates_after_marker_update() {
        let mut state = make_state();
        editing::create_marker(&mut state, 1, "Alt", "All");

        let first = render_map_snapshot(&state).expect("Snapshot vorhanden");

        editing::update_marker(&mut state, 1, "Neu", "Hub");

        let second = render_map_snapshot(&state).expect("Snapshot vorhanden");
        assert!(!Arc::ptr_eq(&first, &second));
        assert_eq!(second.marker_count(), 1);
    }
}
