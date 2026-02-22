//! Kurven-Tool: Zeichnet Bézier-Kurven 2. oder 3. Grades.
//!
//! **Quadratisch (Grad 2):** Start → End → 1 Steuerpunkt (Klick) → Drag-Anpassung → Enter
//! **Kubisch (Grad 3):** Start → End → CP1 (Klick) → CP2 (Klick) → Drag-Anpassung → Enter
//!
//! Nach Platzierung aller Punkte können Start, End und Steuerpunkte
//! per Drag verschoben werden. Start/Ende rasten beim Loslassen auf
//! nahe existierende Nodes ein (Re-Snap).
//!
//! Grad wird über `render_config` umgeschaltet (UI-Dropdown).

use super::{
    common::{
        node_count_from_length, populate_neighbors, segment_length_from_count, LastEdited,
        TangentSource,
    },
    snap_to_node, RouteTool, ToolAction, ToolAnchor, ToolPreview, ToolResult,
};
use crate::core::{ConnectedNeighbor, ConnectionDirection, ConnectionPriority, RoadMap};
use crate::shared::SNAP_RADIUS;
use glam::Vec2;

mod config_ui;
mod geometry;
use geometry::{
    approx_length, build_tool_result, compute_curve_positions, compute_tangent_cp, cubic_bezier,
    quadratic_bezier, CurveParams,
};

/// Welcher Punkt wird gerade per Drag verschoben?
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum DragTarget {
    Start,
    End,
    CP1,
    CP2,
}

/// Grad der Bézier-Kurve
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CurveDegree {
    /// Quadratisch: 1 Steuerpunkt
    Quadratic,
    /// Kubisch: 2 Steuerpunkte
    Cubic,
}

/// Phasen des Kurven-Tools
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum Phase {
    /// Startpunkt wählen
    Start,
    /// Endpunkt wählen
    End,
    /// Steuerpunkt(e) wählen / verschieben (Klick aktualisiert, Enter bestätigt)
    Control,
}

/// Bézier-Kurven-Tool (Grad 2 oder 3)
pub struct CurveTool {
    phase: Phase,
    start: Option<ToolAnchor>,
    end: Option<ToolAnchor>,
    /// Steuerpunkt 1 (frei positionierbar)
    control_point1: Option<Vec2>,
    /// Steuerpunkt 2 (nur bei kubisch)
    control_point2: Option<Vec2>,
    /// Gerade per Drag verschobener Punkt
    dragging: Option<DragTarget>,
    /// Grad der Kurve
    pub degree: CurveDegree,
    /// Maximaler Abstand zwischen Zwischen-Nodes (Standard: 2m)
    pub max_segment_length: f32,
    /// Gewünschte Anzahl Nodes (inkl. Start+End)
    pub node_count: usize,
    last_edited: LastEdited,
    pub direction: ConnectionDirection,
    pub priority: ConnectionPriority,
    last_created_ids: Vec<u64>,
    last_start_anchor: Option<ToolAnchor>,
    last_end_anchor: Option<ToolAnchor>,
    last_control_point1: Option<Vec2>,
    last_control_point2: Option<Vec2>,
    recreate_needed: bool,
    /// Gewählte Tangente am Startpunkt (nur Cubic)
    tangent_start: TangentSource,
    /// Gewählte Tangente am Endpunkt (nur Cubic)
    tangent_end: TangentSource,
    /// Verfügbare Nachbarn am Startpunkt (Cache)
    start_neighbors: Vec<ConnectedNeighbor>,
    /// Verfügbare Nachbarn am Endpunkt (Cache)
    end_neighbors: Vec<ConnectedNeighbor>,
    /// Tangente Start der letzten Erstellung (für Recreation)
    last_tangent_start: TangentSource,
    /// Tangente Ende der letzten Erstellung (für Recreation)
    last_tangent_end: TangentSource,
    /// Snap-Radius in Welteinheiten (aus EditorOptions)
    snap_radius: f32,
}

impl CurveTool {
    /// Erstellt ein neues Kurven-Tool mit Standardparametern.
    pub fn new() -> Self {
        Self {
            phase: Phase::Start,
            start: None,
            end: None,
            control_point1: None,
            control_point2: None,
            dragging: None,
            degree: CurveDegree::Quadratic,
            max_segment_length: 2.0,
            node_count: 2,
            last_edited: LastEdited::Distance,
            direction: ConnectionDirection::Dual,
            priority: ConnectionPriority::Regular,
            last_created_ids: Vec::new(),
            last_start_anchor: None,
            last_end_anchor: None,
            last_control_point1: None,
            last_control_point2: None,
            recreate_needed: false,
            tangent_start: TangentSource::None,
            tangent_end: TangentSource::None,
            start_neighbors: Vec::new(),
            end_neighbors: Vec::new(),
            last_tangent_start: TangentSource::None,
            last_tangent_end: TangentSource::None,
            snap_radius: SNAP_RADIUS,
        }
    }

    /// Approximierte Kurvenlänge über Polylinien-Segmente.
    fn approx_length(positions_fn: impl Fn(f32) -> Vec2, samples: usize) -> f32 {
        approx_length(positions_fn, samples)
    }

    /// Kurvenlänge je nach Grad.
    fn curve_length(&self) -> f32 {
        let s = self.start.as_ref().map(|a| a.position());
        let e = self.end.as_ref().map(|a| a.position());
        match self.degree {
            CurveDegree::Quadratic => {
                let (Some(start), Some(end), Some(cp)) = (s, e, self.control_point1) else {
                    return 0.0;
                };
                Self::approx_length(|t| quadratic_bezier(start, cp, end, t), 64)
            }
            CurveDegree::Cubic => {
                let (Some(start), Some(end), Some(cp1), Some(cp2)) =
                    (s, e, self.control_point1, self.control_point2)
                else {
                    return 0.0;
                };
                Self::approx_length(|t| cubic_bezier(start, cp1, cp2, end, t), 64)
            }
        }
    }

    fn sync_derived(&mut self) {
        let length = self.curve_length();
        if length < f32::EPSILON {
            return;
        }
        match self.last_edited {
            LastEdited::Distance => {
                self.node_count = node_count_from_length(length, self.max_segment_length);
            }
            LastEdited::NodeCount => {
                self.max_segment_length = segment_length_from_count(length, self.node_count);
            }
        }
    }

    /// True wenn alle Steuerpunkte für den aktuellen Grad gesetzt sind.
    fn controls_complete(&self) -> bool {
        match self.degree {
            CurveDegree::Quadratic => self.control_point1.is_some(),
            CurveDegree::Cubic => self.control_point1.is_some() && self.control_point2.is_some(),
        }
    }

    /// Wendet die gewählten Tangenten auf die Steuerpunkte an (nur Cubic).
    ///
    /// Setzt CP1/CP2 basierend auf der Verbindungs-Richtung, sofern eine
    /// Tangente ausgewählt ist. Klick in Phase::Control überschreibt danach manuell.
    fn apply_tangent_to_cp(&mut self) {
        if self.degree != CurveDegree::Cubic {
            return;
        }
        let (Some(start), Some(end)) = (self.start, self.end) else {
            return;
        };

        if let TangentSource::Connection { angle, .. } = self.tangent_start {
            self.control_point1 = Some(compute_tangent_cp(
                start.position(),
                angle,
                end.position(),
                true,
            ));
        }
        if let TangentSource::Connection { angle, .. } = self.tangent_end {
            self.control_point2 = Some(compute_tangent_cp(
                end.position(),
                angle,
                start.position(),
                false,
            ));
        }
    }

}

impl Default for CurveTool {
    fn default() -> Self {
        Self::new()
    }
}

impl RouteTool for CurveTool {
    fn name(&self) -> &str {
        "🔀 Kurve"
    }

    fn description(&self) -> &str {
        "Zeichnet eine Bézier-Kurve (Grad 2 oder 3) mit Steuerpunkten"
    }

    fn status_text(&self) -> &str {
        match self.phase {
            Phase::Start => "Startpunkt klicken",
            Phase::End => "Endpunkt klicken",
            Phase::Control => match self.degree {
                CurveDegree::Quadratic => {
                    if self.control_point1.is_some() {
                        "Punkte per Drag anpassen — Enter bestätigt"
                    } else {
                        "Steuerpunkt klicken"
                    }
                }
                CurveDegree::Cubic => {
                    let has1 = self.control_point1.is_some();
                    let has2 = self.control_point2.is_some();
                    if has1 && has2 {
                        "Punkte per Drag anpassen — Enter bestätigt"
                    } else if has1 {
                        "2. Steuerpunkt klicken"
                    } else {
                        "1. Steuerpunkt klicken"
                    }
                }
            },
        }
    }

    fn on_click(&mut self, pos: Vec2, road_map: &RoadMap, _ctrl: bool) -> ToolAction {
        match self.phase {
            Phase::Start => {
                // Verkettung: letzten Endpunkt als Start verwenden
                if let Some(last_end) = self.last_end_anchor {
                    self.last_created_ids.clear();
                    self.last_start_anchor = None;
                    self.last_end_anchor = None;
                    self.last_control_point1 = None;
                    self.last_control_point2 = None;
                    self.recreate_needed = false;
                    self.start = Some(last_end);
                    self.start_neighbors = populate_neighbors(&last_end, road_map);
                    let end_anchor = snap_to_node(pos, road_map, SNAP_RADIUS);
                    self.end_neighbors = populate_neighbors(&end_anchor, road_map);
                    self.end = Some(end_anchor);
                    self.tangent_start = TangentSource::None;
                    self.tangent_end = TangentSource::None;
                    self.phase = Phase::Control;
                    self.apply_tangent_to_cp();
                    ToolAction::Continue
                } else {
                    let start_anchor = snap_to_node(pos, road_map, SNAP_RADIUS);
                    self.start_neighbors = populate_neighbors(&start_anchor, road_map);
                    self.tangent_start = TangentSource::None;
                    self.start = Some(start_anchor);
                    self.phase = Phase::End;
                    ToolAction::Continue
                }
            }
            Phase::End => {
                let end_anchor = snap_to_node(pos, road_map, SNAP_RADIUS);
                self.end_neighbors = populate_neighbors(&end_anchor, road_map);
                self.tangent_end = TangentSource::None;
                self.end = Some(end_anchor);
                self.phase = Phase::Control;
                self.apply_tangent_to_cp();
                ToolAction::Continue
            }
            Phase::Control => {
                match self.degree {
                    CurveDegree::Quadratic => {
                        if self.control_point1.is_none() {
                            self.control_point1 = Some(pos);
                        }
                    }
                    CurveDegree::Cubic => {
                        if self.control_point1.is_none() {
                            self.control_point1 = Some(pos);
                            // Tangente-Start wird durch manuellen Klick überschrieben
                            self.tangent_start = TangentSource::None;
                        } else if self.control_point2.is_none() {
                            self.control_point2 = Some(pos);
                            // Tangente-Ende wird durch manuellen Klick überschrieben
                            self.tangent_end = TangentSource::None;
                        }
                    }
                }
                self.sync_derived();
                ToolAction::UpdatePreview
            }
        }
    }

    fn preview(&self, cursor_pos: Vec2, road_map: &RoadMap) -> ToolPreview {
        let start_pos = match &self.start {
            Some(anchor) => anchor.position(),
            None => return ToolPreview::default(),
        };

        match self.phase {
            Phase::End => {
                let end_pos = snap_to_node(cursor_pos, road_map, SNAP_RADIUS).position();
                ToolPreview {
                    nodes: vec![start_pos, end_pos],
                    connections: vec![(0, 1)],
                }
            }
            Phase::Control => {
                let end_pos = match &self.end {
                    Some(anchor) => anchor.position(),
                    None => return ToolPreview::default(),
                };

                let cp1 = self.control_point1.unwrap_or(cursor_pos);

                let positions = match self.degree {
                    CurveDegree::Quadratic => compute_curve_positions(
                        |t| quadratic_bezier(start_pos, cp1, end_pos, t),
                        self.max_segment_length,
                    ),
                    CurveDegree::Cubic => {
                        let cp2 = self.control_point2.unwrap_or(cursor_pos);
                        compute_curve_positions(
                            |t| cubic_bezier(start_pos, cp1, cp2, end_pos, t),
                            self.max_segment_length,
                        )
                    }
                };

                let connections: Vec<(usize, usize)> = (0..positions.len().saturating_sub(1))
                    .map(|i| (i, i + 1))
                    .collect();

                // Steuerpunkte als zusätzliche Vorschau-Nodes
                let mut nodes = positions;
                nodes.push(cp1);
                if self.degree == CurveDegree::Cubic {
                    let cp2 = self.control_point2.unwrap_or(cursor_pos);
                    nodes.push(cp2);
                }

                ToolPreview { nodes, connections }
            }
            _ => ToolPreview::default(),
        }
    }

    fn render_config(&mut self, ui: &mut egui::Ui) -> bool {
        self.render_config_view(ui)
    }

    fn execute(&self, road_map: &RoadMap) -> Option<ToolResult> {
        let start = self.start.as_ref()?;
        let end = self.end.as_ref()?;
        let params = CurveParams {
            degree: self.degree,
            cp1: self.control_point1?,
            cp2: self.control_point2,
            max_segment_length: self.max_segment_length,
            direction: self.direction,
            priority: self.priority,
        };
        build_tool_result(start, end, &params, road_map)
    }

    fn reset(&mut self) {
        self.start = None;
        self.end = None;
        self.control_point1 = None;
        self.control_point2 = None;
        self.phase = Phase::Start;
        self.tangent_start = TangentSource::None;
        self.tangent_end = TangentSource::None;
        self.start_neighbors.clear();
        self.end_neighbors.clear();
    }

    fn is_ready(&self) -> bool {
        self.start.is_some() && self.end.is_some() && self.controls_complete()
    }

    fn set_direction(&mut self, dir: ConnectionDirection) {
        self.direction = dir;
    }

    fn set_priority(&mut self, prio: ConnectionPriority) {
        self.priority = prio;
    }

    fn set_snap_radius(&mut self, radius: f32) {
        self.snap_radius = radius;
    }

    fn set_last_created(&mut self, ids: Vec<u64>, _road_map: &RoadMap) {
        if self.start.is_some() {
            self.last_start_anchor = self.start;
        }
        if self.end.is_some() {
            self.last_end_anchor = self.end;
        }
        if self.control_point1.is_some() {
            self.last_control_point1 = self.control_point1;
        }
        if self.control_point2.is_some() {
            self.last_control_point2 = self.control_point2;
        }
        self.last_tangent_start = self.tangent_start;
        self.last_tangent_end = self.tangent_end;
        self.last_created_ids = ids;
        self.recreate_needed = false;
    }

    fn last_created_ids(&self) -> &[u64] {
        &self.last_created_ids
    }

    fn last_end_anchor(&self) -> Option<ToolAnchor> {
        self.last_end_anchor
    }

    fn needs_recreate(&self) -> bool {
        self.recreate_needed
    }

    fn clear_recreate_flag(&mut self) {
        self.recreate_needed = false;
    }

    fn execute_from_anchors(&self, road_map: &RoadMap) -> Option<ToolResult> {
        let start = self.last_start_anchor.as_ref()?;
        let end = self.last_end_anchor.as_ref()?;
        let params = CurveParams {
            degree: self.degree,
            cp1: self.last_control_point1?,
            cp2: self.last_control_point2,
            max_segment_length: self.max_segment_length,
            direction: self.direction,
            priority: self.priority,
        };
        build_tool_result(start, end, &params, road_map)
    }

    fn drag_targets(&self) -> Vec<Vec2> {
        if self.phase != Phase::Control || !self.controls_complete() {
            return vec![];
        }
        let mut targets = Vec::with_capacity(4);
        if let Some(a) = &self.start {
            targets.push(a.position());
        }
        if let Some(a) = &self.end {
            targets.push(a.position());
        }
        if let Some(cp) = self.control_point1 {
            targets.push(cp);
        }
        if self.degree == CurveDegree::Cubic {
            if let Some(cp) = self.control_point2 {
                targets.push(cp);
            }
        }
        targets
    }

    fn on_drag_start(&mut self, pos: Vec2, _road_map: &RoadMap, pick_radius: f32) -> bool {
        if self.phase != Phase::Control || !self.controls_complete() {
            return false;
        }

        // Alle Kandidaten mit Abstand sammeln
        let mut candidates: Vec<(DragTarget, f32)> = Vec::with_capacity(4);
        if let Some(a) = &self.start {
            candidates.push((DragTarget::Start, a.position().distance(pos)));
        }
        if let Some(a) = &self.end {
            candidates.push((DragTarget::End, a.position().distance(pos)));
        }
        if let Some(cp) = self.control_point1 {
            candidates.push((DragTarget::CP1, cp.distance(pos)));
        }
        if self.degree == CurveDegree::Cubic {
            if let Some(cp) = self.control_point2 {
                candidates.push((DragTarget::CP2, cp.distance(pos)));
            }
        }

        // Nächsten Punkt innerhalb pick_radius finden
        candidates.sort_by(|a, b| a.1.partial_cmp(&b.1).unwrap_or(std::cmp::Ordering::Equal));
        if let Some((target, dist)) = candidates.first() {
            if *dist <= pick_radius {
                self.dragging = Some(*target);
                return true;
            }
        }
        false
    }

    fn on_drag_update(&mut self, pos: Vec2) {
        match self.dragging {
            Some(DragTarget::Start) => {
                self.start = Some(ToolAnchor::NewPosition(pos));
            }
            Some(DragTarget::End) => {
                self.end = Some(ToolAnchor::NewPosition(pos));
            }
            Some(DragTarget::CP1) => {
                self.control_point1 = Some(pos);
            }
            Some(DragTarget::CP2) => {
                self.control_point2 = Some(pos);
            }
            None => {}
        }
        self.sync_derived();
    }

    fn on_drag_end(&mut self, road_map: &RoadMap) {
        // Start/Ende: Re-Snap auf existierenden Node
        match self.dragging {
            Some(DragTarget::Start) => {
                if let Some(anchor) = &self.start {
                    self.start = Some(snap_to_node(anchor.position(), road_map, SNAP_RADIUS));
                }
            }
            Some(DragTarget::End) => {
                if let Some(anchor) = &self.end {
                    self.end = Some(snap_to_node(anchor.position(), road_map, SNAP_RADIUS));
                }
            }
            _ => {}
        }
        self.dragging = None;
    }
}

#[cfg(test)]
mod tests;
