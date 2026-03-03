//! Drag-Logik für das Constraint-Route-Tool.
//!
//! Ermöglicht das Verschieben von Start, End und Kontrollpunkten per Drag.

use super::super::snap_to_node;
use super::super::ToolAnchor;
use super::state::{ConstraintRouteTool, DragTarget, Phase};
use crate::core::RoadMap;
use glam::Vec2;

impl ConstraintRouteTool {
    /// Gibt die Weltpositionen aller verschiebbaren Punkte zurück.
    ///
    /// Reihenfolge: Start, End, Approach-Steuerpunkt, Departure-Steuerpunkt, Kontrollpunkte.
    pub(crate) fn get_drag_targets(&self) -> Vec<Vec2> {
        if self.phase != Phase::ControlNodes || !self.is_ready_internal() {
            return vec![];
        }
        let mut targets = Vec::with_capacity(4 + self.control_nodes.len());
        if let Some(a) = &self.start {
            targets.push(a.position());
        }
        if let Some(a) = &self.end {
            targets.push(a.position());
        }
        if let Some(ap) = self.approach_steerer {
            targets.push(ap);
        }
        if let Some(dp) = self.departure_steerer {
            targets.push(dp);
        }
        for &cp in &self.control_nodes {
            targets.push(cp);
        }
        targets
    }

    /// Startet einen Drag auf einem Punkt nahe `pos`.
    pub(crate) fn handle_drag_start(
        &mut self,
        pos: Vec2,
        _road_map: &RoadMap,
        pick_radius: f32,
    ) -> bool {
        if self.phase != Phase::ControlNodes || !self.is_ready_internal() {
            return false;
        }

        let mut candidates: Vec<(DragTarget, f32)> = Vec::new();

        if let Some(a) = &self.start {
            candidates.push((DragTarget::Start, a.position().distance(pos)));
        }
        if let Some(a) = &self.end {
            candidates.push((DragTarget::End, a.position().distance(pos)));
        }
        if let Some(ap) = self.approach_steerer {
            candidates.push((DragTarget::ApproachSteerer, ap.distance(pos)));
        }
        if let Some(dp) = self.departure_steerer {
            candidates.push((DragTarget::DepartureSteerer, dp.distance(pos)));
        }
        for (i, &cp) in self.control_nodes.iter().enumerate() {
            candidates.push((DragTarget::Control(i), cp.distance(pos)));
        }

        candidates.sort_by(|a, b| a.1.partial_cmp(&b.1).unwrap_or(std::cmp::Ordering::Equal));

        if let Some((target, dist)) = candidates.first() {
            if *dist <= pick_radius {
                self.dragging = Some(*target);
                return true;
            }
        }
        false
    }

    /// Aktualisiert die Position des gegriffenen Punkts während eines Drags.
    pub(crate) fn handle_drag_update(&mut self, pos: Vec2) {
        match self.dragging {
            Some(DragTarget::Start) => {
                self.start = Some(ToolAnchor::NewPosition(pos));
                self.update_preview();
            }
            Some(DragTarget::End) => {
                self.end = Some(ToolAnchor::NewPosition(pos));
                self.update_preview();
            }
            Some(DragTarget::ApproachSteerer) => {
                self.approach_steerer = Some(pos);
                self.approach_manual = true;
                self.update_preview();
            }
            Some(DragTarget::DepartureSteerer) => {
                self.departure_steerer = Some(pos);
                self.departure_manual = true;
                self.update_preview();
            }
            Some(DragTarget::Control(i)) => {
                if i < self.control_nodes.len() {
                    self.control_nodes[i] = pos;
                    self.update_preview();
                }
            }
            None => {}
        }
    }

    /// Beendet den Drag (ggf. Re-Snap auf existierenden Node).
    pub(crate) fn handle_drag_end(&mut self, road_map: &RoadMap) {
        match self.dragging {
            Some(DragTarget::Start) => {
                if let Some(anchor) = &self.start {
                    let re_snapped =
                        snap_to_node(anchor.position(), road_map, self.lifecycle.snap_radius);
                    self.start = Some(re_snapped);
                    self.start_neighbor_dirs =
                        ConstraintRouteTool::collect_neighbor_dirs(&re_snapped, road_map);
                    // Bei neuem Start Auto-Steuerpunkte zurücksetzen
                    self.approach_manual = false;
                }
            }
            Some(DragTarget::End) => {
                if let Some(anchor) = &self.end {
                    let re_snapped =
                        snap_to_node(anchor.position(), road_map, self.lifecycle.snap_radius);
                    self.end = Some(re_snapped);
                    self.end_neighbor_dirs =
                        ConstraintRouteTool::collect_neighbor_dirs(&re_snapped, road_map);
                    // Bei neuem Ende Auto-Steuerpunkte zurücksetzen
                    self.departure_manual = false;
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
        self.dragging = None;
        self.sync_derived();
        self.update_preview();
    }

    /// Hilfsmethode: Start und End gesetzt?
    fn is_ready_internal(&self) -> bool {
        self.start.is_some() && self.end.is_some()
    }
}
