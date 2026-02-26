//! Use-Case: Neuen Node an einer Weltposition hinzufügen.

use crate::app::AppState;
use crate::core::{Connection, ConnectionDirection, ConnectionPriority, MapNode, NodeFlag};
use glam::Vec2;
use std::sync::Arc;

/// Ergebnis von `add_node_at_position`.
#[derive(Debug)]
pub enum AddNodeResult {
    /// Keine RoadMap geladen
    NoMap,
    /// Existierender Node wurde selektiert (kein neuer Node erstellt)
    SelectedExisting(u64),
    /// Neuer Node wurde erstellt
    Created(u64),
}

/// Berechnet den minimalen Abstand von Punkt `pt` zum Liniensegment `a→b`.
fn point_to_segment_dist(pt: Vec2, a: Vec2, b: Vec2) -> f32 {
    let ab = b - a;
    let len_sq = ab.dot(ab);
    if len_sq < f32::EPSILON {
        return pt.distance(a);
    }
    let t = ((pt - a).dot(ab) / len_sq).clamp(0.0, 1.0);
    pt.distance(a + t * ab)
}

/// Sucht die Verbindung, die der Klickposition am nächsten liegt (innerhalb `threshold`).
/// Gibt `(start_id, end_id, direction, priority)` zurück.
fn find_nearest_connection(
    road_map: &crate::core::RoadMap,
    pt: Vec2,
    threshold: f32,
) -> Option<(u64, u64, ConnectionDirection, ConnectionPriority)> {
    let mut best_dist = threshold;
    let mut best = None;

    for conn in road_map.connections_iter() {
        let Some(s_node) = road_map.nodes.get(&conn.start_id) else {
            continue;
        };
        let Some(e_node) = road_map.nodes.get(&conn.end_id) else {
            continue;
        };
        let dist = point_to_segment_dist(pt, s_node.position, e_node.position);
        if dist < best_dist {
            best_dist = dist;
            best = Some((conn.start_id, conn.end_id, conn.direction, conn.priority));
        }
    }
    best
}

/// Fügt einen neuen Node an der gegebenen Weltposition hinzu.
///
/// Wenn genau ein Node selektiert ist, wird der neue Node automatisch
/// mit der voreingestellten Richtung und Straßenart verbunden.
/// Der neue Node wird anschließend als einziger selektiert.
///
/// Wenn `options.split_connection_on_place` aktiviert ist und die Klickposition
/// nahe einer bestehenden Verbindung liegt, wird diese Verbindung gesplittet
/// und der neue Node dazwischen eingefügt (anstelle des normalen Auto-Connects).
///
/// Trifft der Klick einen existierenden Node (innerhalb snap_radius),
/// wird dieser stattdessen nur selektiert (keine Neuerstellung).
pub fn add_node_at_position(state: &mut AppState, world_pos: Vec2) -> AddNodeResult {
    let Some(road_map_ref) = state.road_map.as_ref() else {
        log::warn!("Kein Node hinzufügbar: keine RoadMap geladen");
        return AddNodeResult::NoMap;
    };

    // Prüfe ob ein existierender Node direkt getroffen wurde → nur selektieren
    // Hitbox ist auf die visuelle Node-Größe begrenzt (nicht snap_radius),
    // damit zwischen eng platzierten Nodes noch neue Nodes gesetzt werden können.
    if let Some(hit) = road_map_ref.nearest_node(world_pos) {
        if hit.distance <= state.options.hitbox_radius() {
            state.selection.ids_mut().clear();
            state.selection.ids_mut().insert(hit.node_id);
            state.selection.selection_anchor_node_id = Some(hit.node_id);
            log::info!(
                "AddNode: Existierender Node {} selektiert (Snap)",
                hit.node_id
            );
            return AddNodeResult::SelectedExisting(hit.node_id);
        }
    }

    // Merke aktuell selektierten Node für Auto-Connect
    let connect_from = if state.selection.selected_node_ids.len() == 1 {
        state.selection.selected_node_ids.iter().next().copied()
    } else {
        None
    };

    // Verbindung zum Splitten suchen (nur wenn Option aktiv)
    let snap_threshold = state.options.snap_radius() * 1.5;
    let split_target: Option<(u64, u64, ConnectionDirection, ConnectionPriority)> =
        if state.options.split_connection_on_place {
            find_nearest_connection(state.road_map.as_ref().unwrap(), world_pos, snap_threshold)
        } else {
            None
        };

    // Snapshot VOR Mutation
    state.record_undo_snapshot();

    let direction = state.editor.default_direction;
    let priority = state.editor.default_priority;

    let Some(road_map_arc) = state.road_map.as_mut() else {
        log::warn!("Kein Node hinzufügbar: keine RoadMap geladen");
        return AddNodeResult::NoMap;
    };
    let road_map = Arc::make_mut(road_map_arc);
    let new_id = road_map.next_node_id();
    let node = MapNode::new(new_id, world_pos, NodeFlag::Regular);
    road_map.add_node(node);

    if let Some((split_start, split_end, split_dir, split_prio)) = split_target {
        // Split-Modus: alte Verbindung entfernen, zwei neue einfügen
        road_map.remove_connection(split_start, split_end);

        if let Some(s_node) = road_map.nodes.get(&split_start) {
            let s_pos = s_node.position;
            let conn1 =
                Connection::new(split_start, new_id, split_dir, split_prio, s_pos, world_pos);
            road_map.add_connection(conn1);
        }
        if let Some(e_node) = road_map.nodes.get(&split_end) {
            let e_pos = e_node.position;
            let conn2 = Connection::new(new_id, split_end, split_dir, split_prio, world_pos, e_pos);
            road_map.add_connection(conn2);
        }

        road_map.recalculate_node_flags(&[split_start, new_id, split_end]);
        log::info!(
            "Verbindung {}→{} gesplittet durch Node {}",
            split_start,
            split_end,
            new_id
        );
    } else {
        // Normaler Auto-Connect: Vom selektierten Node zum neuen Node verbinden
        if let Some(from_id) = connect_from {
            if road_map.nodes.contains_key(&from_id) {
                let start_pos = road_map.nodes[&from_id].position;
                let end_pos = world_pos;
                let conn =
                    Connection::new(from_id, new_id, direction, priority, start_pos, end_pos);
                road_map.add_connection(conn);
                road_map.recalculate_node_flags(&[from_id, new_id]);
                log::info!(
                    "Auto-Connect: {}→{} ({:?}, {:?})",
                    from_id,
                    new_id,
                    direction,
                    priority
                );
            }
        }
    }

    // Spatial-Index nach Mutation aktualisieren
    road_map.ensure_spatial_index();

    // Neuen Node selektieren
    state.selection.ids_mut().clear();
    state.selection.ids_mut().insert(new_id);
    state.selection.selection_anchor_node_id = Some(new_id);

    log::info!(
        "Node {} an Position ({:.1}, {:.1}) hinzugefügt",
        new_id,
        world_pos.x,
        world_pos.y
    );

    AddNodeResult::Created(new_id)
}
