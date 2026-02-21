//! Kurven-Tool: Zeichnet Bézier-Kurven 2. oder 3. Grades.
//!
//! **Quadratisch (Grad 2):** Start → End → 1 Steuerpunkt (Klick repositioniert) → Enter
//! **Kubisch (Grad 3):** Start → End → CP1 per Linksklick, CP2 per Ctrl+Linksklick → Enter
//!
//! Grad wird über `render_config` umgeschaltet (UI-Dropdown).

use super::{RouteTool, ToolAction, ToolAnchor, ToolPreview, ToolResult};
use crate::core::{ConnectionDirection, ConnectionPriority, NodeFlag, RoadMap};
use glam::Vec2;

/// Snap-Distanz: Klick innerhalb dieses Radius rastet auf existierenden Node ein.
const SNAP_RADIUS: f32 = 3.0;

/// Welcher Wert wurde zuletzt vom User geändert?
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum LastEdited {
    Distance,
    NodeCount,
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
    /// Steuerpunkt 2 (nur bei kubisch, Ctrl+Klick)
    control_point2: Option<Vec2>,
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
}

impl CurveTool {
    pub fn new() -> Self {
        Self {
            phase: Phase::Start,
            start: None,
            end: None,
            control_point1: None,
            control_point2: None,
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
        }
    }

    /// Approximierte Kurvenlänge über Polylinien-Segmente.
    fn approx_length(positions_fn: impl Fn(f32) -> Vec2, samples: usize) -> f32 {
        let mut length = 0.0;
        let mut prev = positions_fn(0.0);
        for i in 1..=samples {
            let t = i as f32 / samples as f32;
            let p = positions_fn(t);
            length += prev.distance(p);
            prev = p;
        }
        length
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
                let segments = (length / self.max_segment_length).ceil().max(1.0) as usize;
                self.node_count = segments + 1;
            }
            LastEdited::NodeCount => {
                let segments = (self.node_count.max(2) - 1) as f32;
                self.max_segment_length = length / segments;
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
}

impl Default for CurveTool {
    fn default() -> Self {
        Self::new()
    }
}

/// B(t) = (1-t)²·P0 + 2(1-t)t·P1 + t²·P2
fn quadratic_bezier(p0: Vec2, p1: Vec2, p2: Vec2, t: f32) -> Vec2 {
    let inv = 1.0 - t;
    inv * inv * p0 + 2.0 * inv * t * p1 + t * t * p2
}

/// B(t) = (1-t)³·P0 + 3(1-t)²t·P1 + 3(1-t)t²·P2 + t³·P3
fn cubic_bezier(p0: Vec2, p1: Vec2, p2: Vec2, p3: Vec2, t: f32) -> Vec2 {
    let inv = 1.0 - t;
    let inv2 = inv * inv;
    let t2 = t * t;
    inv2 * inv * p0 + 3.0 * inv2 * t * p1 + 3.0 * inv * t2 * p2 + t2 * t * p3
}

/// Gleichmäßig verteilte Punkte entlang einer parametrischen Kurve (Arc-Length).
fn compute_curve_positions(eval: impl Fn(f32) -> Vec2, max_segment_length: f32) -> Vec<Vec2> {
    let start = eval(0.0);
    let total_length = CurveTool::approx_length(&eval, 128);
    if total_length < f32::EPSILON {
        return vec![start];
    }

    let segment_count = (total_length / max_segment_length).ceil().max(1.0) as usize;
    let target_spacing = total_length / segment_count as f32;

    // Arc-Length-LUT
    let lut_samples = 256;
    let mut arc_lengths = Vec::with_capacity(lut_samples + 1);
    let mut prev = start;
    let mut cumulative = 0.0f32;
    arc_lengths.push(0.0f32);
    for i in 1..=lut_samples {
        let t = i as f32 / lut_samples as f32;
        let p = eval(t);
        cumulative += prev.distance(p);
        arc_lengths.push(cumulative);
        prev = p;
    }

    let mut positions = Vec::with_capacity(segment_count + 1);
    positions.push(start);

    for seg in 1..segment_count {
        let target_length = seg as f32 * target_spacing;
        let idx = arc_lengths
            .partition_point(|&len| len < target_length)
            .min(lut_samples)
            .max(1);

        let len_before = arc_lengths[idx - 1];
        let len_after = arc_lengths[idx];
        let frac = if (len_after - len_before).abs() > f32::EPSILON {
            (target_length - len_before) / (len_after - len_before)
        } else {
            0.0
        };

        let t = ((idx - 1) as f32 + frac) / lut_samples as f32;
        positions.push(eval(t));
    }

    positions.push(eval(1.0));
    positions
}

fn snap_to_node(pos: Vec2, road_map: &RoadMap) -> ToolAnchor {
    if let Some(hit) = road_map.nearest_node(pos) {
        if hit.distance <= SNAP_RADIUS {
            if let Some(node) = road_map.nodes.get(&hit.node_id) {
                return ToolAnchor::ExistingNode(hit.node_id, node.position);
            }
        }
    }
    ToolAnchor::NewPosition(pos)
}

/// Evaluiert die Kurvenposition für den aktuellen Grad.
fn eval_curve(
    degree: CurveDegree,
    start: Vec2,
    end: Vec2,
    cp1: Vec2,
    cp2: Option<Vec2>,
    t: f32,
) -> Vec2 {
    match degree {
        CurveDegree::Quadratic => quadratic_bezier(start, cp1, end, t),
        CurveDegree::Cubic => cubic_bezier(start, cp1, cp2.unwrap_or(cp1), end, t),
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
                        "Steuerpunkt verschieben oder Enter zum Bestätigen"
                    } else {
                        "Steuerpunkt klicken"
                    }
                }
                CurveDegree::Cubic => {
                    let has1 = self.control_point1.is_some();
                    let has2 = self.control_point2.is_some();
                    if has1 && has2 {
                        "Linksklick=CP1, Ctrl+Klick=CP2 verschieben — Enter bestätigt"
                    } else if has1 {
                        "Ctrl+Klick für 2. Steuerpunkt — Enter bestätigt"
                    } else {
                        "Linksklick für 1. Steuerpunkt"
                    }
                }
            },
        }
    }

    fn on_click(&mut self, pos: Vec2, road_map: &RoadMap, ctrl: bool) -> ToolAction {
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
                    self.end = Some(snap_to_node(pos, road_map));
                    self.phase = Phase::Control;
                    ToolAction::Continue
                } else {
                    self.start = Some(snap_to_node(pos, road_map));
                    self.phase = Phase::End;
                    ToolAction::Continue
                }
            }
            Phase::End => {
                self.end = Some(snap_to_node(pos, road_map));
                self.phase = Phase::Control;
                ToolAction::Continue
            }
            Phase::Control => {
                match self.degree {
                    CurveDegree::Quadratic => {
                        // Quadratisch: jeder Klick setzt/verschiebt CP1
                        self.control_point1 = Some(pos);
                    }
                    CurveDegree::Cubic => {
                        if ctrl {
                            // Ctrl+Klick → CP2
                            self.control_point2 = Some(pos);
                        } else {
                            // Normaler Klick → CP1
                            self.control_point1 = Some(pos);
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
                let end_pos = snap_to_node(cursor_pos, road_map).position();
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
        let mut changed = false;

        // Grad-Auswahl
        ui.label("Kurven-Grad:");
        let old_degree = self.degree;
        egui::ComboBox::from_id_salt("curve_degree")
            .selected_text(match self.degree {
                CurveDegree::Quadratic => "Quadratisch (Grad 2)",
                CurveDegree::Cubic => "Kubisch (Grad 3)",
            })
            .show_ui(ui, |ui| {
                ui.selectable_value(
                    &mut self.degree,
                    CurveDegree::Quadratic,
                    "Quadratisch (Grad 2)",
                );
                ui.selectable_value(&mut self.degree, CurveDegree::Cubic, "Kubisch (Grad 3)");
            });
        if self.degree != old_degree {
            // Beim Gradwechsel CP2 zurücksetzen
            self.control_point2 = None;
            changed = true;
        }
        ui.add_space(4.0);

        // Nachbearbeitungs-Modus
        let adjusting = !self.last_created_ids.is_empty()
            && self.last_start_anchor.is_some()
            && self.last_end_anchor.is_some()
            && self.last_control_point1.is_some();

        if adjusting {
            let start_pos = self.last_start_anchor.unwrap().position();
            let end_pos = self.last_end_anchor.unwrap().position();
            let cp1 = self.last_control_point1.unwrap();
            let cp2 = self.last_control_point2;
            let length = match self.degree {
                CurveDegree::Quadratic => {
                    CurveTool::approx_length(|t| quadratic_bezier(start_pos, cp1, end_pos, t), 64)
                }
                CurveDegree::Cubic => {
                    let cp2v = cp2.unwrap_or(cp1);
                    CurveTool::approx_length(|t| cubic_bezier(start_pos, cp1, cp2v, end_pos, t), 64)
                }
            };

            ui.label(format!("Kurvenlänge: {:.1} m", length));
            ui.add_space(4.0);

            ui.label("Min. Abstand:");
            let max_seg = length.max(1.0);
            if ui
                .add(egui::Slider::new(&mut self.max_segment_length, 1.0..=max_seg).suffix(" m"))
                .changed()
            {
                self.last_edited = LastEdited::Distance;
                let segments = (length / self.max_segment_length).ceil().max(1.0) as usize;
                self.node_count = segments + 1;
                self.recreate_needed = true;
                changed = true;
            }

            ui.add_space(4.0);

            ui.label("Anzahl Nodes:");
            let max_nodes = (length / 1.0).ceil().max(2.0) as usize;
            if ui
                .add(egui::Slider::new(&mut self.node_count, 2..=max_nodes))
                .changed()
            {
                self.last_edited = LastEdited::NodeCount;
                let segments = (self.node_count.max(2) - 1) as f32;
                self.max_segment_length = length / segments;
                self.recreate_needed = true;
                changed = true;
            }
        } else if self.is_ready() {
            let length = self.curve_length();
            ui.label(format!("Kurvenlänge: {:.1} m", length));
            ui.add_space(4.0);

            ui.label("Min. Abstand:");
            let max_seg = length.max(1.0);
            if ui
                .add(egui::Slider::new(&mut self.max_segment_length, 1.0..=max_seg).suffix(" m"))
                .changed()
            {
                self.last_edited = LastEdited::Distance;
                self.sync_derived();
                changed = true;
            }

            ui.add_space(4.0);

            ui.label("Anzahl Nodes:");
            let max_nodes = (length / 1.0).ceil().max(2.0) as usize;
            if ui
                .add(egui::Slider::new(&mut self.node_count, 2..=max_nodes))
                .changed()
            {
                self.last_edited = LastEdited::NodeCount;
                self.sync_derived();
                changed = true;
            }
        } else {
            ui.label("Max. Segment-Länge:");
            if ui
                .add(egui::Slider::new(&mut self.max_segment_length, 1.0..=50.0).suffix(" m"))
                .changed()
            {
                self.last_edited = LastEdited::Distance;
                changed = true;
            }
        }

        changed
    }

    fn execute(&self, road_map: &RoadMap) -> Option<ToolResult> {
        let start = self.start.as_ref()?;
        let end = self.end.as_ref()?;
        let cp1 = self.control_point1?;
        let cp2 = self.control_point2;

        build_tool_result(
            start,
            end,
            self.degree,
            cp1,
            cp2,
            self.max_segment_length,
            self.direction,
            self.priority,
            road_map,
        )
    }

    fn reset(&mut self) {
        self.start = None;
        self.end = None;
        self.control_point1 = None;
        self.control_point2 = None;
        self.phase = Phase::Start;
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

    fn set_last_created(&mut self, ids: Vec<u64>) {
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
        let cp1 = self.last_control_point1?;
        let cp2 = self.last_control_point2;

        build_tool_result(
            start,
            end,
            self.degree,
            cp1,
            cp2,
            self.max_segment_length,
            self.direction,
            self.priority,
            road_map,
        )
    }
}

/// Gemeinsame Logik für execute() und execute_from_anchors().
fn build_tool_result(
    start: &ToolAnchor,
    end: &ToolAnchor,
    degree: CurveDegree,
    cp1: Vec2,
    cp2: Option<Vec2>,
    max_segment_length: f32,
    direction: ConnectionDirection,
    priority: ConnectionPriority,
    road_map: &RoadMap,
) -> Option<ToolResult> {
    let start_pos = start.position();
    let end_pos = end.position();

    let positions = compute_curve_positions(
        |t| eval_curve(degree, start_pos, end_pos, cp1, cp2, t),
        max_segment_length,
    );

    let mut new_nodes: Vec<(Vec2, NodeFlag)> = Vec::new();
    let mut internal_connections: Vec<(usize, usize, ConnectionDirection, ConnectionPriority)> =
        Vec::new();
    let mut external_connections: Vec<(usize, u64, ConnectionDirection, ConnectionPriority)> =
        Vec::new();

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

    Some(ToolResult {
        new_nodes,
        internal_connections,
        external_connections,
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    // ── Quadratische Bézier ──

    #[test]
    fn test_quadratic_bezier_endpoints() {
        let p0 = Vec2::new(0.0, 0.0);
        let p1 = Vec2::new(5.0, 10.0);
        let p2 = Vec2::new(10.0, 0.0);

        let start = quadratic_bezier(p0, p1, p2, 0.0);
        let end = quadratic_bezier(p0, p1, p2, 1.0);
        let mid = quadratic_bezier(p0, p1, p2, 0.5);

        assert!((start - p0).length() < 0.001);
        assert!((end - p2).length() < 0.001);
        assert!((mid - Vec2::new(5.0, 5.0)).length() < 0.001);
    }

    #[test]
    fn test_quadratic_curve_positions_count() {
        let start = Vec2::new(0.0, 0.0);
        let control = Vec2::new(5.0, 10.0);
        let end = Vec2::new(10.0, 0.0);

        let positions = compute_curve_positions(|t| quadratic_bezier(start, control, end, t), 2.0);
        assert!(positions.len() >= 3);
        assert!((positions[0] - start).length() < 0.01);
        assert!((*positions.last().unwrap() - end).length() < 0.01);
    }

    #[test]
    fn test_quadratic_curve_spacing() {
        let start = Vec2::new(0.0, 0.0);
        let control = Vec2::new(5.0, 10.0);
        let end = Vec2::new(10.0, 0.0);

        let positions = compute_curve_positions(|t| quadratic_bezier(start, control, end, t), 2.0);
        for i in 0..positions.len() - 1 {
            let dist = positions[i].distance(positions[i + 1]);
            assert!(dist < 2.5, "Segment {} hat Abstand {:.2}m", i, dist);
        }
    }

    // ── Kubische Bézier ──

    #[test]
    fn test_cubic_bezier_endpoints() {
        let p0 = Vec2::new(0.0, 0.0);
        let p1 = Vec2::new(3.0, 10.0);
        let p2 = Vec2::new(7.0, 10.0);
        let p3 = Vec2::new(10.0, 0.0);

        let start = cubic_bezier(p0, p1, p2, p3, 0.0);
        let end = cubic_bezier(p0, p1, p2, p3, 1.0);

        assert!((start - p0).length() < 0.001);
        assert!((end - p3).length() < 0.001);
    }

    #[test]
    fn test_cubic_bezier_symmetry() {
        // Symmetrische S-Kurve → Mittelpunkt bei (5, 5)
        let p0 = Vec2::new(0.0, 0.0);
        let p1 = Vec2::new(0.0, 10.0);
        let p2 = Vec2::new(10.0, 0.0);
        let p3 = Vec2::new(10.0, 10.0);

        let mid = cubic_bezier(p0, p1, p2, p3, 0.5);
        // B(0.5) = 0.125*P0 + 0.375*P1 + 0.375*P2 + 0.125*P3 = (5, 5)
        assert!((mid - Vec2::new(5.0, 5.0)).length() < 0.001);
    }

    #[test]
    fn test_cubic_curve_positions_count() {
        let start = Vec2::new(0.0, 0.0);
        let cp1 = Vec2::new(3.0, 10.0);
        let cp2 = Vec2::new(7.0, 10.0);
        let end = Vec2::new(10.0, 0.0);

        let positions = compute_curve_positions(|t| cubic_bezier(start, cp1, cp2, end, t), 2.0);
        assert!(positions.len() >= 3);
        assert!((positions[0] - start).length() < 0.01);
        assert!((*positions.last().unwrap() - end).length() < 0.01);
    }

    // ── Tool-Flow quadratisch ──

    #[test]
    fn test_tool_quadratic_click_flow() {
        let mut tool = CurveTool::new();
        tool.degree = CurveDegree::Quadratic;
        let road_map = RoadMap::new(3);

        assert!(!tool.is_ready());
        assert_eq!(tool.status_text(), "Startpunkt klicken");

        let action = tool.on_click(Vec2::ZERO, &road_map, false);
        assert_eq!(action, ToolAction::Continue);

        let action = tool.on_click(Vec2::new(10.0, 0.0), &road_map, false);
        assert_eq!(action, ToolAction::Continue);
        assert!(tool.status_text().contains("Steuerpunkt"));

        let action = tool.on_click(Vec2::new(5.0, 8.0), &road_map, false);
        assert_eq!(action, ToolAction::UpdatePreview);
        assert!(tool.is_ready());

        // Repositionieren
        let action = tool.on_click(Vec2::new(5.0, 12.0), &road_map, false);
        assert_eq!(action, ToolAction::UpdatePreview);
        assert_eq!(tool.control_point1, Some(Vec2::new(5.0, 12.0)));
    }

    // ── Tool-Flow kubisch ──

    #[test]
    fn test_tool_cubic_click_flow() {
        let mut tool = CurveTool::new();
        tool.degree = CurveDegree::Cubic;
        let road_map = RoadMap::new(3);

        // Start
        tool.on_click(Vec2::ZERO, &road_map, false);
        // End
        tool.on_click(Vec2::new(10.0, 0.0), &road_map, false);
        // CP1 per normalem Klick
        let action = tool.on_click(Vec2::new(3.0, 8.0), &road_map, false);
        assert_eq!(action, ToolAction::UpdatePreview);
        assert!(!tool.is_ready()); // CP2 fehlt noch

        // CP2 per Ctrl+Klick
        let action = tool.on_click(Vec2::new(7.0, 8.0), &road_map, true);
        assert_eq!(action, ToolAction::UpdatePreview);
        assert!(tool.is_ready());
        assert_eq!(tool.control_point1, Some(Vec2::new(3.0, 8.0)));
        assert_eq!(tool.control_point2, Some(Vec2::new(7.0, 8.0)));
    }

    #[test]
    fn test_tool_cubic_repositions() {
        let mut tool = CurveTool::new();
        tool.degree = CurveDegree::Cubic;
        let road_map = RoadMap::new(3);

        tool.on_click(Vec2::ZERO, &road_map, false);
        tool.on_click(Vec2::new(10.0, 0.0), &road_map, false);
        tool.on_click(Vec2::new(3.0, 8.0), &road_map, false); // CP1
        tool.on_click(Vec2::new(7.0, 8.0), &road_map, true); // CP2

        // CP1 nochmal verschieben
        tool.on_click(Vec2::new(2.0, 6.0), &road_map, false);
        assert_eq!(tool.control_point1, Some(Vec2::new(2.0, 6.0)));
        // CP2 bleibt
        assert_eq!(tool.control_point2, Some(Vec2::new(7.0, 8.0)));

        // CP2 nochmal verschieben
        tool.on_click(Vec2::new(8.0, 6.0), &road_map, true);
        assert_eq!(tool.control_point2, Some(Vec2::new(8.0, 6.0)));
    }

    #[test]
    fn test_tool_cubic_execute() {
        let mut tool = CurveTool::new();
        tool.degree = CurveDegree::Cubic;
        tool.max_segment_length = 2.0;
        let road_map = RoadMap::new(3);

        tool.on_click(Vec2::ZERO, &road_map, false);
        tool.on_click(Vec2::new(10.0, 0.0), &road_map, false);
        tool.on_click(Vec2::new(3.0, 8.0), &road_map, false);
        tool.on_click(Vec2::new(7.0, 8.0), &road_map, true);

        let result = tool.execute(&road_map).expect("Ergebnis erwartet");
        assert!(result.new_nodes.len() >= 3);
        assert_eq!(
            result.internal_connections.len(),
            result.new_nodes.len() - 1,
        );
    }

    #[test]
    fn test_tool_execute_quadratic() {
        let mut tool = CurveTool::new();
        tool.max_segment_length = 2.0;
        let road_map = RoadMap::new(3);

        tool.on_click(Vec2::ZERO, &road_map, false);
        tool.on_click(Vec2::new(10.0, 0.0), &road_map, false);
        tool.on_click(Vec2::new(5.0, 8.0), &road_map, false);

        let result = tool.execute(&road_map).expect("Ergebnis erwartet");
        assert!(result.new_nodes.len() >= 3);
    }

    #[test]
    fn test_tool_reset() {
        let mut tool = CurveTool::new();
        tool.degree = CurveDegree::Cubic;
        let road_map = RoadMap::new(3);

        tool.on_click(Vec2::ZERO, &road_map, false);
        tool.on_click(Vec2::new(10.0, 0.0), &road_map, false);
        tool.on_click(Vec2::new(3.0, 8.0), &road_map, false);
        tool.on_click(Vec2::new(7.0, 8.0), &road_map, true);
        assert!(tool.is_ready());

        tool.reset();
        assert!(!tool.is_ready());
        assert_eq!(tool.phase, Phase::Start);
        assert!(tool.control_point1.is_none());
        assert!(tool.control_point2.is_none());
    }

    #[test]
    fn test_chaining_uses_last_end_as_start() {
        let mut tool = CurveTool::new();
        let road_map = RoadMap::new(3);

        tool.on_click(Vec2::ZERO, &road_map, false);
        tool.on_click(Vec2::new(10.0, 0.0), &road_map, false);
        tool.on_click(Vec2::new(5.0, 8.0), &road_map, false);
        tool.set_last_created(vec![100, 101, 102]);
        tool.reset();

        let action = tool.on_click(Vec2::new(20.0, 0.0), &road_map, false);
        assert_eq!(action, ToolAction::Continue);
        assert!(tool.start.is_some());
        assert!(tool.end.is_some());
        assert_eq!(tool.phase, Phase::Control);
    }

    #[test]
    fn test_execute_from_anchors() {
        let mut tool = CurveTool::new();
        tool.max_segment_length = 2.0;
        let road_map = RoadMap::new(3);

        tool.on_click(Vec2::ZERO, &road_map, false);
        tool.on_click(Vec2::new(10.0, 0.0), &road_map, false);
        tool.on_click(Vec2::new(5.0, 8.0), &road_map, false);
        let original = tool.execute(&road_map).unwrap();
        tool.set_last_created(vec![1, 2, 3, 4, 5]);
        tool.reset();

        tool.max_segment_length = 5.0;
        let result = tool
            .execute_from_anchors(&road_map)
            .expect("Ergebnis erwartet");
        assert!(result.new_nodes.len() < original.new_nodes.len());
    }

    #[test]
    fn test_approx_length_straight_line() {
        let length = CurveTool::approx_length(|t| Vec2::new(t * 10.0, 0.0), 128);
        assert!((length - 10.0).abs() < 0.1);
    }

    #[test]
    fn test_straight_control_point_gives_straight_line() {
        let start = Vec2::ZERO;
        let end = Vec2::new(10.0, 0.0);
        let control = Vec2::new(5.0, 0.0);

        let positions = compute_curve_positions(|t| quadratic_bezier(start, control, end, t), 2.0);
        for (i, pos) in positions.iter().enumerate() {
            assert!(
                pos.y.abs() < 0.01,
                "Node {} hat y={:.3}, erwartet 0",
                i,
                pos.y
            );
        }
    }
}
