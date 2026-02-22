//! Drag-Logik für das Bézier-Kurven-Tool.

use super::super::{snap_to_node, ToolAnchor};
use super::state::{CurveDegree, CurveTool, DragTarget, Phase};
use crate::core::RoadMap;
use glam::Vec2;

/// Gibt die Weltpositionen aller verschiebbaren Punkte zurück.
pub(crate) fn drag_targets(tool: &CurveTool) -> Vec<Vec2> {
    if tool.phase != Phase::Control || !tool.controls_complete() {
        return vec![];
    }
    let mut targets = Vec::with_capacity(4);
    if let Some(a) = &tool.start {
        targets.push(a.position());
    }
    if let Some(a) = &tool.end {
        targets.push(a.position());
    }
    if let Some(cp) = tool.control_point1 {
        targets.push(cp);
    }
    if tool.degree == CurveDegree::Cubic {
        if let Some(cp) = tool.control_point2 {
            targets.push(cp);
        }
    }
    targets
}

/// Startet einen Drag auf einem Punkt nahe `pos`.
pub(crate) fn on_drag_start(
    tool: &mut CurveTool,
    pos: Vec2,
    _road_map: &RoadMap,
    pick_radius: f32,
) -> bool {
    if tool.phase != Phase::Control || !tool.controls_complete() {
        return false;
    }

    let mut candidates: Vec<(DragTarget, f32)> = Vec::with_capacity(4);
    if let Some(a) = &tool.start {
        candidates.push((DragTarget::Start, a.position().distance(pos)));
    }
    if let Some(a) = &tool.end {
        candidates.push((DragTarget::End, a.position().distance(pos)));
    }
    if let Some(cp) = tool.control_point1 {
        candidates.push((DragTarget::CP1, cp.distance(pos)));
    }
    if tool.degree == CurveDegree::Cubic {
        if let Some(cp) = tool.control_point2 {
            candidates.push((DragTarget::CP2, cp.distance(pos)));
        }
    }

    candidates.sort_by(|a, b| a.1.partial_cmp(&b.1).unwrap_or(std::cmp::Ordering::Equal));
    if let Some((target, dist)) = candidates.first() {
        if *dist <= pick_radius {
            tool.dragging = Some(*target);
            return true;
        }
    }
    false
}

/// Aktualisiert die Position des gegriffenen Punkts während eines Drags.
pub(crate) fn on_drag_update(tool: &mut CurveTool, pos: Vec2) {
    match tool.dragging {
        Some(DragTarget::Start) => {
            tool.start = Some(ToolAnchor::NewPosition(pos));
        }
        Some(DragTarget::End) => {
            tool.end = Some(ToolAnchor::NewPosition(pos));
        }
        Some(DragTarget::CP1) => {
            tool.control_point1 = Some(pos);
        }
        Some(DragTarget::CP2) => {
            tool.control_point2 = Some(pos);
        }
        None => {}
    }
    tool.sync_derived();
}

/// Beendet den Drag (Re-Snap auf existierenden Node).
pub(crate) fn on_drag_end(tool: &mut CurveTool, road_map: &RoadMap) {
    match tool.dragging {
        Some(DragTarget::Start) => {
            if let Some(anchor) = &tool.start {
                tool.start = Some(snap_to_node(
                    anchor.position(),
                    road_map,
                    tool.lifecycle.snap_radius,
                ));
            }
        }
        Some(DragTarget::End) => {
            if let Some(anchor) = &tool.end {
                tool.end = Some(snap_to_node(
                    anchor.position(),
                    road_map,
                    tool.lifecycle.snap_radius,
                ));
            }
        }
        _ => {}
    }
    tool.dragging = None;
}
