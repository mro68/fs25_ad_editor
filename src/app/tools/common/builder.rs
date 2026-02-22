//! Gemeinsamer ToolResult-Aufbau aus Positions-Sequenzen.

use crate::core::{ConnectionDirection, ConnectionPriority, NodeFlag, RoadMap};
use glam::Vec2;

use super::super::{ToolAnchor, ToolResult};

/// Baut ein `ToolResult` aus einer Positions-Sequenz und Start-/End-Ankern.
///
/// Diese Funktion enthält die gemeinsame Logik aller Route-Tools:
/// 1. Neue Nodes für Positionen erzeugen (existierende Nodes überspringen)
/// 2. Interne und externe Verbindungen zwischen aufeinanderfolgenden Positionen aufbauen
///
/// Die Geometrie (Positionen) wird vorher tool-spezifisch berechnet und übergeben.
pub fn assemble_tool_result(
    positions: &[Vec2],
    start: &ToolAnchor,
    end: &ToolAnchor,
    direction: ConnectionDirection,
    priority: ConnectionPriority,
    road_map: &RoadMap,
) -> ToolResult {
    let mut new_nodes: Vec<(Vec2, NodeFlag)> = Vec::new();
    let mut internal_connections: Vec<(usize, usize, ConnectionDirection, ConnectionPriority)> =
        Vec::new();
    let mut external_connections: Vec<(usize, u64, ConnectionDirection, ConnectionPriority)> =
        Vec::new();

    // Phase 1: Positions → neue Nodes oder existierende Nodes zuordnen
    let mut pos_to_new_idx: Vec<Option<usize>> = Vec::with_capacity(positions.len());

    for (i, &pos) in positions.iter().enumerate() {
        let is_start = i == 0;
        let is_end = i == positions.len() - 1;

        let existing_id = if is_start {
            match start {
                ToolAnchor::ExistingNode(id, _) => Some(*id),
                _ => None,
            }
        } else if is_end {
            match end {
                ToolAnchor::ExistingNode(id, _) => Some(*id),
                _ => None,
            }
        } else {
            road_map
                .nearest_node(pos)
                .filter(|hit| hit.distance < 0.01)
                .map(|hit| hit.node_id)
        };

        if existing_id.is_some() {
            pos_to_new_idx.push(None);
        } else {
            let idx = new_nodes.len();
            new_nodes.push((pos, NodeFlag::Regular));
            pos_to_new_idx.push(Some(idx));
        }
    }

    // Phase 2: Verbindungen zwischen aufeinanderfolgenden Positionen aufbauen
    for i in 0..positions.len().saturating_sub(1) {
        let a_new_idx = pos_to_new_idx[i];
        let b_new_idx = pos_to_new_idx[i + 1];

        let is_start_a = i == 0;
        let is_end_b = i + 1 == positions.len() - 1;

        let a_existing = if is_start_a {
            match start {
                ToolAnchor::ExistingNode(id, _) => Some(*id),
                _ => None,
            }
        } else {
            None
        };

        let b_existing = if is_end_b {
            match end {
                ToolAnchor::ExistingNode(id, _) => Some(*id),
                _ => None,
            }
        } else if pos_to_new_idx[i + 1].is_none() {
            road_map
                .nearest_node(positions[i + 1])
                .filter(|hit| hit.distance < 0.01)
                .map(|hit| hit.node_id)
        } else {
            None
        };

        match (a_new_idx, a_existing, b_new_idx, b_existing) {
            (Some(a), _, Some(b), _) => {
                internal_connections.push((a, b, direction, priority));
            }
            (Some(a), _, None, Some(b_id)) => {
                external_connections.push((a, b_id, direction, priority));
            }
            (None, Some(a_id), Some(b), _) => {
                external_connections.push((b, a_id, direction, priority));
            }
            _ => {}
        }
    }

    ToolResult {
        new_nodes,
        internal_connections,
        external_connections,
    }
}
