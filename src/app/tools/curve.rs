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

use super::{RouteTool, ToolAction, ToolAnchor, ToolPreview, ToolResult};
use crate::core::{ConnectedNeighbor, ConnectionDirection, ConnectionPriority, RoadMap};
use glam::Vec2;

mod geometry;
use geometry::{
    approx_length, build_tool_result, compute_curve_positions, compute_tangent_cp, cubic_bezier,
    quadratic_bezier, snap_to_node, CurveParams,
};

/// Snap-Distanz: Klick innerhalb dieses Radius rastet auf existierenden Node ein.
const SNAP_RADIUS: f32 = 3.0;

/// Welcher Punkt wird gerade per Drag verschoben?
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum DragTarget {
    Start,
    End,
    CP1,
    CP2,
}

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

/// Quelle einer Tangente am Start- oder Endpunkt (nur Cubic).
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum TangentSource {
    /// Kein Tangenten-Vorschlag — CP wird manuell gesetzt
    None,
    /// Tangente aus bestehender Verbindung
    Connection { neighbor_id: u64, angle: f32 },
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

    /// Befüllt die Nachbar-Liste für einen Snap-Node.
    fn populate_neighbors(anchor: &ToolAnchor, road_map: &RoadMap) -> Vec<ConnectedNeighbor> {
        match anchor {
            ToolAnchor::ExistingNode(id, _) => road_map.connected_neighbors(*id),
            ToolAnchor::NewPosition(_) => Vec::new(),
        }
    }
}

impl Default for CurveTool {
    fn default() -> Self {
        Self::new()
    }
}

/// Wandelt einen Winkel (Radiant) in eine Kompass-Richtung um.
///
/// FS25-Koordinatensystem: +X = Ost, +Z = Süd in der Draufsicht.
fn angle_to_compass(angle: f32) -> &'static str {
    let deg = angle.to_degrees().rem_euclid(360.0) as u32;
    match deg {
        0..=22 | 338..=360 => "O",
        23..=67 => "SO",
        68..=112 => "S",
        113..=157 => "SW",
        158..=202 => "W",
        203..=247 => "NW",
        248..=292 => "N",
        293..=337 => "NO",
        _ => "?",
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
                    self.start_neighbors = Self::populate_neighbors(&last_end, road_map);
                    let end_anchor = snap_to_node(pos, road_map, SNAP_RADIUS);
                    self.end_neighbors = Self::populate_neighbors(&end_anchor, road_map);
                    self.end = Some(end_anchor);
                    self.tangent_start = TangentSource::None;
                    self.tangent_end = TangentSource::None;
                    self.phase = Phase::Control;
                    self.apply_tangent_to_cp();
                    ToolAction::Continue
                } else {
                    let start_anchor = snap_to_node(pos, road_map, SNAP_RADIUS);
                    self.start_neighbors = Self::populate_neighbors(&start_anchor, road_map);
                    self.tangent_start = TangentSource::None;
                    self.start = Some(start_anchor);
                    self.phase = Phase::End;
                    ToolAction::Continue
                }
            }
            Phase::End => {
                let end_anchor = snap_to_node(pos, road_map, SNAP_RADIUS);
                self.end_neighbors = Self::populate_neighbors(&end_anchor, road_map);
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
            // Beim Gradwechsel CP2 und Tangenten zurücksetzen
            self.control_point2 = None;
            self.tangent_start = TangentSource::None;
            self.tangent_end = TangentSource::None;
            changed = true;
        }
        ui.add_space(4.0);

        // Tangenten-Auswahl (nur Cubic, wenn Start+End gesetzt)
        if self.degree == CurveDegree::Cubic {
            let show_tangent_ui = (self.phase == Phase::Control
                || (!self.last_created_ids.is_empty()
                    && self.last_start_anchor.is_some()
                    && self.last_end_anchor.is_some()))
                && (self.start.is_some() && self.end.is_some()
                    || self.last_start_anchor.is_some() && self.last_end_anchor.is_some());

            if show_tangent_ui {
                // Tangente Start
                if !self.start_neighbors.is_empty() {
                    let old_tangent = self.tangent_start;
                    let selected_text = match self.tangent_start {
                        TangentSource::None => "Manuell".to_string(),
                        TangentSource::Connection { neighbor_id, angle } => {
                            format!("→ Node #{} ({})", neighbor_id, angle_to_compass(angle))
                        }
                    };
                    ui.label("Tangente Start:");
                    egui::ComboBox::from_id_salt("tangent_start")
                        .selected_text(selected_text)
                        .show_ui(ui, |ui| {
                            ui.selectable_value(
                                &mut self.tangent_start,
                                TangentSource::None,
                                "Manuell",
                            );
                            for neighbor in &self.start_neighbors {
                                let label = format!(
                                    "→ Node #{} ({})",
                                    neighbor.neighbor_id,
                                    angle_to_compass(neighbor.angle)
                                );
                                ui.selectable_value(
                                    &mut self.tangent_start,
                                    TangentSource::Connection {
                                        neighbor_id: neighbor.neighbor_id,
                                        angle: neighbor.angle,
                                    },
                                    label,
                                );
                            }
                        });
                    if self.tangent_start != old_tangent {
                        self.apply_tangent_to_cp();
                        self.sync_derived();
                        if !self.last_created_ids.is_empty() {
                            self.recreate_needed = true;
                        }
                        changed = true;
                    }
                }

                // Tangente Ende
                if !self.end_neighbors.is_empty() {
                    let old_tangent = self.tangent_end;
                    let selected_text = match self.tangent_end {
                        TangentSource::None => "Manuell".to_string(),
                        TangentSource::Connection { neighbor_id, angle } => {
                            format!("→ Node #{} ({})", neighbor_id, angle_to_compass(angle))
                        }
                    };
                    ui.label("Tangente Ende:");
                    egui::ComboBox::from_id_salt("tangent_end")
                        .selected_text(selected_text)
                        .show_ui(ui, |ui| {
                            ui.selectable_value(
                                &mut self.tangent_end,
                                TangentSource::None,
                                "Manuell",
                            );
                            for neighbor in &self.end_neighbors {
                                let label = format!(
                                    "→ Node #{} ({})",
                                    neighbor.neighbor_id,
                                    angle_to_compass(neighbor.angle)
                                );
                                ui.selectable_value(
                                    &mut self.tangent_end,
                                    TangentSource::Connection {
                                        neighbor_id: neighbor.neighbor_id,
                                        angle: neighbor.angle,
                                    },
                                    label,
                                );
                            }
                        });
                    if self.tangent_end != old_tangent {
                        self.apply_tangent_to_cp();
                        self.sync_derived();
                        if !self.last_created_ids.is_empty() {
                            self.recreate_needed = true;
                        }
                        changed = true;
                    }
                }

                if !self.start_neighbors.is_empty() || !self.end_neighbors.is_empty() {
                    ui.add_space(4.0);
                }
            }
        }

        // Nachbearbeitungs-Modus
        let adjusting = !self.last_created_ids.is_empty()
            && self.last_start_anchor.is_some()
            && self.last_end_anchor.is_some()
            && self.last_control_point1.is_some();

        if adjusting {
            let (Some(start_anchor), Some(end_anchor), Some(cp1)) = (
                self.last_start_anchor,
                self.last_end_anchor,
                self.last_control_point1,
            ) else {
                return changed;
            };

            let start_pos = start_anchor.position();
            let end_pos = end_anchor.position();
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

        // Erneuter Klick ignoriert (Drag übernimmt)
        let action = tool.on_click(Vec2::new(5.0, 12.0), &road_map, false);
        assert_eq!(action, ToolAction::UpdatePreview);
        // CP1 bleibt beim ersten Wert
        assert_eq!(tool.control_point1, Some(Vec2::new(5.0, 8.0)));
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

        // CP2 per zweitem Klick (kein Ctrl nötig)
        let action = tool.on_click(Vec2::new(7.0, 8.0), &road_map, false);
        assert_eq!(action, ToolAction::UpdatePreview);
        assert!(tool.is_ready());
        assert_eq!(tool.control_point1, Some(Vec2::new(3.0, 8.0)));
        assert_eq!(tool.control_point2, Some(Vec2::new(7.0, 8.0)));
    }

    #[test]
    fn test_tool_cubic_drag_repositions() {
        let mut tool = CurveTool::new();
        tool.degree = CurveDegree::Cubic;
        let road_map = RoadMap::new(3);

        tool.on_click(Vec2::ZERO, &road_map, false);
        tool.on_click(Vec2::new(10.0, 0.0), &road_map, false);
        tool.on_click(Vec2::new(3.0, 8.0), &road_map, false); // CP1
        tool.on_click(Vec2::new(7.0, 8.0), &road_map, false); // CP2
        assert!(tool.is_ready());

        // Drag-Targets sind verfügbar
        let targets = tool.drag_targets();
        assert_eq!(targets.len(), 4); // Start, End, CP1, CP2

        // CP1 per Drag verschieben
        let grabbed = tool.on_drag_start(Vec2::new(3.0, 8.0), &road_map, 2.0);
        assert!(grabbed);
        tool.on_drag_update(Vec2::new(2.0, 6.0));
        assert_eq!(tool.control_point1, Some(Vec2::new(2.0, 6.0)));
        tool.on_drag_end(&road_map);

        // CP2 per Drag verschieben
        let grabbed = tool.on_drag_start(Vec2::new(7.0, 8.0), &road_map, 2.0);
        assert!(grabbed);
        tool.on_drag_update(Vec2::new(8.0, 6.0));
        assert_eq!(tool.control_point2, Some(Vec2::new(8.0, 6.0)));
        tool.on_drag_end(&road_map);
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
        tool.on_click(Vec2::new(7.0, 8.0), &road_map, false);

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
        tool.on_click(Vec2::new(7.0, 8.0), &road_map, false);
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

    // ── Drag-Tests ──

    #[test]
    fn test_drag_targets_empty_before_controls_complete() {
        let mut tool = CurveTool::new();
        tool.degree = CurveDegree::Quadratic;
        let road_map = RoadMap::new(3);

        assert!(tool.drag_targets().is_empty());
        tool.on_click(Vec2::ZERO, &road_map, false);
        assert!(tool.drag_targets().is_empty());
        tool.on_click(Vec2::new(10.0, 0.0), &road_map, false);
        assert!(tool.drag_targets().is_empty());
    }

    #[test]
    fn test_drag_targets_available_after_controls_complete() {
        let mut tool = CurveTool::new();
        tool.degree = CurveDegree::Quadratic;
        let road_map = RoadMap::new(3);

        tool.on_click(Vec2::ZERO, &road_map, false);
        tool.on_click(Vec2::new(10.0, 0.0), &road_map, false);
        tool.on_click(Vec2::new(5.0, 8.0), &road_map, false);

        let targets = tool.drag_targets();
        assert_eq!(targets.len(), 3); // Start, End, CP1
    }

    #[test]
    fn test_drag_start_returns_false_outside_radius() {
        let mut tool = CurveTool::new();
        let road_map = RoadMap::new(3);

        tool.on_click(Vec2::ZERO, &road_map, false);
        tool.on_click(Vec2::new(10.0, 0.0), &road_map, false);
        tool.on_click(Vec2::new(5.0, 8.0), &road_map, false);

        // Weit weg von allen Punkten
        let grabbed = tool.on_drag_start(Vec2::new(50.0, 50.0), &road_map, 2.0);
        assert!(!grabbed);
        assert!(tool.dragging.is_none());
    }

    #[test]
    fn test_drag_start_end_resnap() {
        let mut tool = CurveTool::new();
        let road_map = RoadMap::new(3);

        tool.on_click(Vec2::ZERO, &road_map, false);
        tool.on_click(Vec2::new(10.0, 0.0), &road_map, false);
        tool.on_click(Vec2::new(5.0, 8.0), &road_map, false);

        // Start draggen
        let grabbed = tool.on_drag_start(Vec2::new(0.0, 0.0), &road_map, 2.0);
        assert!(grabbed);
        tool.on_drag_update(Vec2::new(1.0, 1.0));
        // Während Drag: NewPosition
        match &tool.start {
            Some(ToolAnchor::NewPosition(pos)) => {
                assert!((pos.x - 1.0).abs() < 0.01);
            }
            _ => panic!("Start sollte NewPosition sein während Drag"),
        }
        tool.on_drag_end(&road_map);
        // Nach Drag: Re-Snap (kein Node in der Nähe → bleibt NewPosition)
        assert!(tool.dragging.is_none());
    }

    #[test]
    fn test_drag_quadratic_cp1() {
        let mut tool = CurveTool::new();
        tool.degree = CurveDegree::Quadratic;
        let road_map = RoadMap::new(3);

        tool.on_click(Vec2::ZERO, &road_map, false);
        tool.on_click(Vec2::new(10.0, 0.0), &road_map, false);
        tool.on_click(Vec2::new(5.0, 8.0), &road_map, false);

        // CP1 draggen
        let grabbed = tool.on_drag_start(Vec2::new(5.0, 8.0), &road_map, 2.0);
        assert!(grabbed);
        tool.on_drag_update(Vec2::new(3.0, 12.0));
        assert_eq!(tool.control_point1, Some(Vec2::new(3.0, 12.0)));
        tool.on_drag_end(&road_map);
        assert!(tool.dragging.is_none());
        assert_eq!(tool.control_point1, Some(Vec2::new(3.0, 12.0)));
    }
}
