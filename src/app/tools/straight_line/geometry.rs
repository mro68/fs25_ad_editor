//! Geometrie-Berechnungen für das Gerade-Strecke-Tool.

use super::super::{common, ToolAnchor, ToolResult};
use crate::core::{ConnectionDirection, ConnectionPriority, RoadMap};
use glam::Vec2;

/// Berechnet die gleichmäßig verteilten Zwischen-Positionen einer geraden Strecke.
///
/// Gibt `segment_count + 1` Punkte zurück (inkl. Start und Ende).
/// Bei `distance < EPSILON` wird nur `[start]` zurückgegeben.
pub fn compute_line_positions(start: Vec2, end: Vec2, max_segment_length: f32) -> Vec<Vec2> {
    let distance = start.distance(end);
    if distance < f32::EPSILON {
        return vec![start];
    }
    let segment_count = (distance / max_segment_length).ceil().max(1.0) as usize;
    (0..=segment_count)
        .map(|i| start.lerp(end, i as f32 / segment_count as f32))
        .collect()
}

/// Gemeinsame Logik für `execute()` und `execute_from_anchors()`.
///
/// Berechnet Positionen und delegiert Node-/Verbindungs-Aufbau an `assemble_tool_result`.
pub(crate) fn build_result(
    start: ToolAnchor,
    end: ToolAnchor,
    max_segment_length: f32,
    direction: ConnectionDirection,
    priority: ConnectionPriority,
    road_map: &RoadMap,
) -> Option<ToolResult> {
    let positions = compute_line_positions(start.position(), end.position(), max_segment_length);
    Some(common::assemble_tool_result(
        &positions, &start, &end, direction, priority, road_map,
    ))
}
