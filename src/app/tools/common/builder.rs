//! Gemeinsamer ToolResult-Aufbau aus Positions-Sequenzen.

use crate::core::{ConnectionDirection, ConnectionPriority, NodeFlag, RoadMap};
use glam::Vec2;

use super::super::{ToolAnchor, ToolResult};

/// Spiegelt die Verbindungsrichtung, wenn Start/Ende einer Kante vertauscht werden.
fn invert_direction(direction: ConnectionDirection) -> ConnectionDirection {
    match direction {
        ConnectionDirection::Regular => ConnectionDirection::Reverse,
        ConnectionDirection::Reverse => ConnectionDirection::Regular,
        ConnectionDirection::Dual => ConnectionDirection::Dual,
    }
}

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
                // Externe Kanten speichern immer (new_idx, existing_id).
                // Für den Start-Anker sind die Endpunkte daher gegenüber der
                // Positions-Reihenfolge vertauscht und die Richtung muss gespiegelt werden.
                external_connections.push((b, a_id, invert_direction(direction), priority));
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

#[cfg(test)]
mod tests {
    use super::*;
    use crate::core::{MapNode, NodeFlag};

    #[test]
    fn start_anchor_external_connection_inverts_direction() {
        let mut road_map = RoadMap::new(2);
        road_map.add_node(MapNode::new(1, Vec2::ZERO, NodeFlag::Regular));
        road_map.ensure_spatial_index();

        let positions = [Vec2::ZERO, Vec2::new(10.0, 0.0)];
        let result = assemble_tool_result(
            &positions,
            &ToolAnchor::ExistingNode(1, Vec2::ZERO),
            &ToolAnchor::NewPosition(Vec2::new(10.0, 0.0)),
            ConnectionDirection::Regular,
            ConnectionPriority::Regular,
            &road_map,
        );

        assert_eq!(result.new_nodes.len(), 1);
        assert_eq!(result.external_connections.len(), 1);
        let (new_idx, existing_id, direction, _) = result.external_connections[0];
        assert_eq!(new_idx, 0);
        assert_eq!(existing_id, 1);
        assert_eq!(direction, ConnectionDirection::Reverse);
    }

    #[test]
    fn end_anchor_external_connection_keeps_direction() {
        let mut road_map = RoadMap::new(2);
        road_map.add_node(MapNode::new(2, Vec2::new(10.0, 0.0), NodeFlag::Regular));
        road_map.ensure_spatial_index();

        let positions = [Vec2::ZERO, Vec2::new(10.0, 0.0)];
        let result = assemble_tool_result(
            &positions,
            &ToolAnchor::NewPosition(Vec2::ZERO),
            &ToolAnchor::ExistingNode(2, Vec2::new(10.0, 0.0)),
            ConnectionDirection::Regular,
            ConnectionPriority::Regular,
            &road_map,
        );

        assert_eq!(result.new_nodes.len(), 1);
        assert_eq!(result.external_connections.len(), 1);
        let (new_idx, existing_id, direction, _) = result.external_connections[0];
        assert_eq!(new_idx, 0);
        assert_eq!(existing_id, 2);
        assert_eq!(direction, ConnectionDirection::Regular);
    }
}
