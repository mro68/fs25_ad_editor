//! Use-Case: Curseplay XML-Datei importieren und Nodes + Ring-Verbindungen anlegen.
//!
//! Liest eine Curseplay-`<customField>`-XML, erstellt fuer jeden Vertex einen MapNode
//! (Y=0.0, Position als Vec2(x, z)) und verbindet alle aufeinanderfolgenden Paare
//! bidirektional als Nebenstrassen-Ring (letzter→erster schliesst den Ring).

use crate::app::AppState;
use crate::core::{Connection, ConnectionDirection, ConnectionPriority, MapNode, NodeFlag};
use crate::xml::parse_curseplay;
use std::sync::Arc;

/// Importiert eine Curseplay-XML-Datei und fuegt Nodes + Ring-Verbindungen zur RoadMap hinzu.
///
/// - Liest die Datei, parst die Vertices
/// - Nimmt einen Undo-Snapshot vor der Mutation
/// - Erstellt einen MapNode (Regular, Y=0.0) pro Vertex
/// - Verbindet aufeinanderfolgende Nodes bidirektional (Dual/SubPriority), letzter→erster schliesst Ring
/// - Ruft recalculate_node_flags + ensure_spatial_index einmalig am Ende auf
pub fn import_curseplay(state: &mut AppState, path: &str) {
    if state.road_map.is_none() {
        log::warn!("Keine RoadMap geladen — Curseplay-Import abgebrochen");
        return;
    }

    // Datei lesen
    let content = match std::fs::read_to_string(path) {
        Ok(c) => c,
        Err(e) => {
            log::error!("Failed to read Curseplay file '{}': {}", path, e);
            return;
        }
    };

    // Parsen
    let vertices = match parse_curseplay(&content) {
        Ok(v) => v,
        Err(e) => {
            log::error!("Failed to parse Curseplay file '{}': {}", path, e);
            return;
        }
    };

    if vertices.is_empty() {
        log::warn!("Curseplay-Datei '{}' enthaelt keine Vertices", path);
        return;
    }

    // Undo-Snapshot vor Mutation
    state.record_undo_snapshot();

    let n = vertices.len();
    let mut new_ids: Vec<u64> = Vec::with_capacity(n);

    {
        let road_map = Arc::make_mut(state.road_map.as_mut().expect("road_map vorhanden"));

        // Nodes erstellen (x, z aus Curseplay → Vec2(x, z) als 2D-Position)
        for pos in &vertices {
            let id = road_map.next_node_id();
            road_map.add_node(MapNode::new(id, *pos, NodeFlag::Regular));
            new_ids.push(id);
        }

        // Ring-Verbindungen erstellen (bidirektional, Nebenstrasse)
        for i in 0..n {
            let from_id = new_ids[i];
            let to_id = new_ids[(i + 1) % n];
            let from_pos = road_map
                .node_position(from_id)
                .expect("Start-Node vorhanden");
            let to_pos = road_map.node_position(to_id).expect("End-Node vorhanden");
            let conn = Connection::new(
                from_id,
                to_id,
                ConnectionDirection::Dual,
                ConnectionPriority::SubPriority,
                from_pos,
                to_pos,
            );
            road_map.add_connection(conn);
        }

        road_map.recalculate_node_flags(&new_ids);
        road_map.ensure_spatial_index();
    }

    log::info!("Imported {} nodes from Curseplay file '{}'", n, path);
}
