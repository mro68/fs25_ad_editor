//! Drag-Logik fuer das Geglättete-Kurve-Tool.
//!
//! Ermoeglicht das Verschieben von Start, End und Kontrollpunkten per Drag.

use super::super::ToolAnchor;
use super::state::{DragTarget, Phase, SmoothCurveTool};
use crate::core::RoadMap;
use glam::Vec2;

/// Gibt die Weltpositionen aller verschiebbaren Punkte zurueck.
///
/// Reihenfolge: Start, End, Approach-Steuerpunkt, Departure-Steuerpunkt, Kontrollpunkte.
pub(crate) fn drag_targets(tool: &SmoothCurveTool) -> Vec<Vec2> {
    if tool.phase != Phase::ControlNodes || !is_ready(tool) {
        return vec![];
    }
    let mut targets = Vec::with_capacity(4 + tool.control_nodes.len());
    if let Some(a) = &tool.start {
        targets.push(a.position());
    }
    if let Some(a) = &tool.end {
        targets.push(a.position());
    }
    if let Some(ap) = tool.approach_steerer {
        targets.push(ap);
    }
    if let Some(dp) = tool.departure_steerer {
        targets.push(dp);
    }
    for &cp in &tool.control_nodes {
        targets.push(cp);
    }
    targets
}

/// Startet einen Drag auf einem Punkt nahe `pos`.
pub(crate) fn on_drag_start(
    tool: &mut SmoothCurveTool,
    pos: Vec2,
    _road_map: &RoadMap,
    pick_radius: f32,
) -> bool {
    if tool.phase != Phase::ControlNodes || !is_ready(tool) {
        return false;
    }

    let mut candidates: Vec<(DragTarget, f32)> = Vec::new();

    if let Some(a) = &tool.start {
        candidates.push((DragTarget::Start, a.position().distance(pos)));
    }
    if let Some(a) = &tool.end {
        candidates.push((DragTarget::End, a.position().distance(pos)));
    }
    if let Some(ap) = tool.approach_steerer {
        candidates.push((DragTarget::ApproachSteerer, ap.distance(pos)));
    }
    if let Some(dp) = tool.departure_steerer {
        candidates.push((DragTarget::DepartureSteerer, dp.distance(pos)));
    }
    for (i, &cp) in tool.control_nodes.iter().enumerate() {
        candidates.push((DragTarget::Control(i), cp.distance(pos)));
    }

    candidates.sort_by(|a, b| a.1.partial_cmp(&b.1).unwrap_or(std::cmp::Ordering::Equal));

    if let Some((target, dist)) = candidates.first()
        && *dist <= pick_radius
    {
        tool.dragging = Some(*target);
        return true;
    }
    false
}

/// Aktualisiert die Position des gegriffenen Punkts waehrend eines Drags.
pub(crate) fn on_drag_update(tool: &mut SmoothCurveTool, pos: Vec2) {
    match tool.dragging {
        Some(DragTarget::Start) => {
            tool.start = Some(ToolAnchor::NewPosition(pos));
            tool.update_preview();
        }
        Some(DragTarget::End) => {
            tool.end = Some(ToolAnchor::NewPosition(pos));
            tool.update_preview();
        }
        Some(DragTarget::ApproachSteerer) => {
            tool.approach_steerer = Some(pos);
            tool.approach_manual = true;
            tool.update_preview();
        }
        Some(DragTarget::DepartureSteerer) => {
            tool.departure_steerer = Some(pos);
            tool.departure_manual = true;
            tool.update_preview();
        }
        Some(DragTarget::Control(i)) if i < tool.control_nodes.len() => {
            tool.control_nodes[i] = pos;
            tool.update_preview();
        }
        Some(DragTarget::Control(_)) => {}
        None => {}
    }
}

/// Beendet den Drag (ggf. Re-Snap auf existierenden Node).
pub(crate) fn on_drag_end(tool: &mut SmoothCurveTool, road_map: &RoadMap) {
    match tool.dragging {
        Some(DragTarget::Start) => {
            if let Some(anchor) = &tool.start {
                let re_snapped = tool.lifecycle.snap_at(anchor.position(), road_map);
                tool.start = Some(re_snapped);
                tool.start_neighbor_dirs =
                    SmoothCurveTool::collect_neighbor_dirs(&re_snapped, road_map);
                // Bei neuem Start Auto-Steuerpunkte zuruecksetzen
                tool.approach_manual = false;
            }
        }
        Some(DragTarget::End) => {
            if let Some(anchor) = &tool.end {
                let re_snapped = tool.lifecycle.snap_at(anchor.position(), road_map);
                tool.end = Some(re_snapped);
                tool.end_neighbor_dirs =
                    SmoothCurveTool::collect_neighbor_dirs(&re_snapped, road_map);
                // Bei neuem Ende Auto-Steuerpunkte zuruecksetzen
                tool.departure_manual = false;
            }
        }
        Some(DragTarget::ApproachSteerer) => {
            // Manuell positioniert — bleibt als manuell markiert
        }
        Some(DragTarget::DepartureSteerer) => {
            // Manuell positioniert — bleibt als manuell markiert
        }
        Some(DragTarget::Control(_)) => {
            // Kontrollpunkte werden nicht gesnappt
        }
        None => {}
    }
    tool.dragging = None;
    tool.sync_derived();
    tool.update_preview();
}

/// Hilfsfunktion: Start und End gesetzt?
fn is_ready(tool: &SmoothCurveTool) -> bool {
    tool.start.is_some() && tool.end.is_some()
}
