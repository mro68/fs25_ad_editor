//! Drag-Logik für das Bézier-Kurven-Tool.

use super::super::{snap_to_node, ToolAnchor};
use super::geometry::{
    cp1_from_apex, cp2_from_apex, cps_from_apex_symmetric, project_onto_tangent_line,
    solve_cps_from_apex_both_tangents,
};
use super::state::{CurveDegree, CurveTool, DragTarget, Phase};
use crate::app::tools::common::TangentSource;
use crate::core::RoadMap;
use glam::Vec2;

/// Gibt die Weltpositionen aller verschiebbaren Punkte zurück.
pub(crate) fn drag_targets(tool: &CurveTool) -> Vec<Vec2> {
    if tool.phase != Phase::Control || !tool.controls_complete() {
        return vec![];
    }
    let mut targets = Vec::with_capacity(5);
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
        // Virtueller Scheitelpunkt B(0.5) als zusätzliches Handle
        if let Some(apex) = tool.virtual_apex {
            targets.push(apex);
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
    if tool.degree == CurveDegree::Cubic {
        if let Some(apex) = tool.virtual_apex {
            candidates.push((DragTarget::Apex, apex.distance(pos)));
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
            // Ist eine Start-Tangente fixiert, darf CP1 nur entlang der Tangenten-Linie gleiten.
            if let TangentSource::Connection { angle, .. } = tool.tangents.tangent_start {
                if let Some(start) = tool.start {
                    let clamped = project_onto_tangent_line(start.position(), angle, pos, true);
                    tool.control_point1 = Some(clamped);
                }
            } else {
                tool.control_point1 = Some(pos);
            }
            // Apex nach CP-Änderung synchronisieren
            tool.init_apex();
        }
        Some(DragTarget::CP2) => {
            // Ist eine End-Tangente fixiert, darf CP2 nur entlang der Tangenten-Linie gleiten.
            if let TangentSource::Connection { angle, .. } = tool.tangents.tangent_end {
                if let Some(end) = tool.end {
                    let clamped = project_onto_tangent_line(end.position(), angle, pos, false);
                    tool.control_point2 = Some(clamped);
                }
            } else {
                tool.control_point2 = Some(pos);
            }
            // Apex nach CP-Änderung synchronisieren
            tool.init_apex();
        }
        Some(DragTarget::Apex) => {
            // Steuerpunkte so anpassen, dass B(0.5) dem Apex folgt.
            // Beide Tangenten fixiert  → 2×2-Gleichungssystem (asymmetrisches Biegen).
            // Nur Start-Tangente      → CP2 frei berechnen (cp2_from_apex).
            // Nur End-Tangente        → CP1 frei berechnen (cp1_from_apex).
            // Keine Tangente          → beide CPs symmetrisch aus Apex.
            if let (Some(start), Some(end)) = (tool.start, tool.end) {
                let p0 = start.position();
                let p3 = end.position();
                let has_start_t = matches!(
                    tool.tangents.tangent_start,
                    TangentSource::Connection { .. }
                );
                let has_end_t =
                    matches!(tool.tangents.tangent_end, TangentSource::Connection { .. });

                if has_start_t && has_end_t {
                    // Beide Tangenten fixiert: Löse 2×2-System
                    if let (
                        TangentSource::Connection { angle: a1, .. },
                        TangentSource::Connection { angle: a2, .. },
                    ) = (tool.tangents.tangent_start, tool.tangents.tangent_end)
                    {
                        let dir1 = glam::Vec2::from_angle(a1 + std::f32::consts::PI);
                        let dir2 = glam::Vec2::from_angle(a2);
                        if let Some((cp1, cp2)) =
                            solve_cps_from_apex_both_tangents(p0, p3, dir1, dir2, pos)
                        {
                            tool.control_point1 = Some(cp1);
                            tool.control_point2 = Some(cp2);
                        } else {
                            // Parallele Tangenten: Fallback auf Start-Tangente-only
                            if let Some(cp1) = tool.control_point1 {
                                tool.control_point2 = Some(cp2_from_apex(p0, cp1, pos, p3));
                            }
                        }
                    }
                } else if has_start_t {
                    if let Some(cp1) = tool.control_point1 {
                        tool.control_point2 = Some(cp2_from_apex(p0, cp1, pos, p3));
                    }
                } else if has_end_t {
                    if let Some(cp2) = tool.control_point2 {
                        tool.control_point1 = Some(cp1_from_apex(p0, pos, cp2, p3));
                    }
                } else {
                    let (c1, c2) = cps_from_apex_symmetric(p0, p3, pos);
                    tool.control_point1 = Some(c1);
                    tool.control_point2 = Some(c2);
                }
                tool.virtual_apex = Some(pos);
            }
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
        // Apex und CPs benötigen keinen Re-Snap
        Some(DragTarget::CP1) | Some(DragTarget::CP2) | Some(DragTarget::Apex) | None => {}
    }
    tool.dragging = None;
}
