//! Spline-Tool: Interpolierender Catmull-Rom-Spline durch alle geklickten Punkte.
//!
//! **Ablauf:** Punkte per Klick setzen → Vorschau wird fortlaufend aktualisiert
//! (Cursor = nächster Punkt) → Enter bestätigt den Kurs.
//!
//! Einstellungsmöglichkeiten: Max. Segment-Länge / Node-Anzahl, Richtung, Priorität.
//! Mindestens 2 Punkte für eine gerade Strecke, ab 3 Punkten entsteht eine Kurve.

use super::{RouteTool, ToolAction, ToolAnchor, ToolPreview, ToolResult};
use crate::core::{ConnectedNeighbor, ConnectionDirection, ConnectionPriority, NodeFlag, RoadMap};
use crate::shared::SNAP_RADIUS;
use glam::Vec2;

/// Welcher Wert wurde zuletzt vom User geändert?
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum LastEdited {
    Distance,
    NodeCount,
}

/// Quelle einer Tangente am Start- oder Endpunkt des Splines.
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum SplineTangentSource {
    /// Kein Tangentenvorschlag — Phantom-Punkt wird gespiegelt (Standard)
    None,
    /// Tangente aus bestehender Verbindung
    Connection { neighbor_id: u64, angle: f32 },
}

// ── Catmull-Rom-Geometrie ────────────────────────────────────────

/// Berechnet einen Punkt auf einem Catmull-Rom-Segment (t ∈ [0, 1]).
///
/// p0, p1, p2, p3: vier aufeinanderfolgende Kontrollpunkte.
/// Die Kurve verläuft von p1 nach p2.
fn catmull_rom_point(p0: Vec2, p1: Vec2, p2: Vec2, p3: Vec2, t: f32) -> Vec2 {
    let t2 = t * t;
    let t3 = t2 * t;
    0.5 * ((2.0 * p1)
        + (-p0 + p2) * t
        + (2.0 * p0 - 5.0 * p1 + 4.0 * p2 - p3) * t2
        + (-p0 + 3.0 * p1 - 3.0 * p2 + p3) * t3)
}

/// Berechnet eine dichte Punktliste entlang einer Catmull-Rom-Spline durch `points`.
///
/// Für Rand-Segmente werden Phantom-Punkte gespiegelt, damit die Kurve
/// natürlich durch den ersten und letzten Punkt läuft.
/// Wenn `start_phantom`/`end_phantom` gesetzt, werden die Phantom-Punkte überschrieben.
///
/// `samples_per_segment`: Anzahl der Zwischenpunkte pro Segment (ohne Endpunkt).
fn catmull_rom_chain_with_tangents(
    points: &[Vec2],
    samples_per_segment: usize,
    start_phantom: Option<Vec2>,
    end_phantom: Option<Vec2>,
) -> Vec<Vec2> {
    if points.len() < 2 {
        return points.to_vec();
    }
    if points.len() == 2 {
        // Gerade Linie — kein Spline nötig
        let mut result = Vec::with_capacity(samples_per_segment + 1);
        for i in 0..=samples_per_segment {
            let t = i as f32 / samples_per_segment as f32;
            result.push(points[0].lerp(points[1], t));
        }
        return result;
    }

    let n = points.len();
    let mut result = Vec::with_capacity((n - 1) * samples_per_segment + 1);

    for seg in 0..(n - 1) {
        // Phantom-Punkte an den Rändern (ggf. durch Tangente überschrieben)
        let p0 = if seg == 0 {
            start_phantom.unwrap_or_else(|| 2.0 * points[0] - points[1])
        } else {
            points[seg - 1]
        };
        let p1 = points[seg];
        let p2 = points[seg + 1];
        let p3 = if seg + 2 < n {
            points[seg + 2]
        } else {
            end_phantom.unwrap_or_else(|| 2.0 * points[n - 1] - points[n - 2])
        };

        let steps = if seg == n - 2 {
            samples_per_segment + 1 // letztes Segment: Endpunkt einschließen
        } else {
            samples_per_segment
        };

        for i in 0..steps {
            let t = i as f32 / samples_per_segment as f32;
            result.push(catmull_rom_point(p0, p1, p2, p3, t));
        }
    }

    result
}

/// Kompatibilität: Standard-Catmull-Rom-Chain ohne Tangent-Override.
#[cfg(test)]
fn catmull_rom_chain(points: &[Vec2], samples_per_segment: usize) -> Vec<Vec2> {
    catmull_rom_chain_with_tangents(points, samples_per_segment, None, None)
}

/// Approximierte Länge einer Polyline.
fn polyline_length(points: &[Vec2]) -> f32 {
    points.windows(2).map(|w| w[0].distance(w[1])).sum()
}

/// Verteilt Punkte gleichmäßig (Arc-Length) entlang einer Polyline.
fn resample_by_distance(polyline: &[Vec2], max_segment_length: f32) -> Vec<Vec2> {
    if polyline.len() < 2 {
        return polyline.to_vec();
    }

    let total = polyline_length(polyline);
    if total < f32::EPSILON {
        return vec![polyline[0]];
    }

    let segment_count = (total / max_segment_length).ceil().max(1.0) as usize;
    let spacing = total / segment_count as f32;

    let mut result = Vec::with_capacity(segment_count + 1);
    result.push(polyline[0]);

    let mut poly_idx = 0;
    let mut remainder = 0.0f32; // Rest-Distanz im aktuellen Polyline-Segment

    for _ in 1..segment_count {
        let mut needed = spacing;

        loop {
            if poly_idx + 1 >= polyline.len() {
                break;
            }
            let seg_len = polyline[poly_idx].distance(polyline[poly_idx + 1]);
            let available = seg_len - remainder;

            if available >= needed {
                remainder += needed;
                let t = remainder / seg_len;
                result.push(polyline[poly_idx].lerp(polyline[poly_idx + 1], t));
                break;
            } else {
                needed -= available;
                remainder = 0.0;
                poly_idx += 1;
            }
        }
    }

    // Endpunkt immer exakt übernehmen
    result.push(*polyline.last().unwrap());
    result
}

// ── Spline-Tool ──────────────────────────────────────────────────

/// Spline-Tool: Interpolierender Catmull-Rom-Spline durch geklickte Punkte.
pub struct SplineTool {
    /// Alle bestätigten Kontrollpunkte (geklickt)
    anchors: Vec<ToolAnchor>,
    /// Maximaler Abstand zwischen Zwischen-Nodes (Standard: 2m)
    pub max_segment_length: f32,
    /// Gewünschte Anzahl Nodes (inkl. Start+End)
    pub node_count: usize,
    last_edited: LastEdited,
    pub direction: ConnectionDirection,
    pub priority: ConnectionPriority,
    /// IDs der zuletzt erstellten Nodes (für Nachbearbeitung)
    last_created_ids: Vec<u64>,
    /// Anker der letzten Erstellung (für Nachbearbeitung)
    last_anchors: Vec<ToolAnchor>,
    /// End-Anker der letzten Erstellung (für Verkettung)
    last_end_anchor: Option<ToolAnchor>,
    /// Signalisiert, dass Config geändert wurde und Neuberechnung nötig ist
    recreate_needed: bool,
    /// Gewählte Tangente am Startpunkt
    tangent_start: SplineTangentSource,
    /// Gewählte Tangente am Endpunkt
    tangent_end: SplineTangentSource,
    /// Verfügbare Nachbarn am Startpunkt (Cache)
    start_neighbors: Vec<ConnectedNeighbor>,
    /// Verfügbare Nachbarn am Endpunkt (Cache)
    end_neighbors: Vec<ConnectedNeighbor>,
    /// Tangente Start der letzten Erstellung (für Recreation)
    last_tangent_start: SplineTangentSource,
    /// Tangente Ende der letzten Erstellung (für Recreation)
    last_tangent_end: SplineTangentSource,
    /// Snap-Radius in Welteinheiten (aus EditorOptions)
    snap_radius: f32,
}

impl SplineTool {
    /// Erstellt ein neues Spline-Tool mit Standardwerten.
    pub fn new() -> Self {
        Self {
            anchors: Vec::new(),
            max_segment_length: 2.0,
            node_count: 2,
            last_edited: LastEdited::Distance,
            direction: ConnectionDirection::Dual,
            priority: ConnectionPriority::Regular,
            last_created_ids: Vec::new(),
            last_anchors: Vec::new(),
            last_end_anchor: None,
            recreate_needed: false,
            tangent_start: SplineTangentSource::None,
            tangent_end: SplineTangentSource::None,
            start_neighbors: Vec::new(),
            end_neighbors: Vec::new(),
            last_tangent_start: SplineTangentSource::None,
            last_tangent_end: SplineTangentSource::None,
            snap_radius: SNAP_RADIUS,
        }
    }

    /// Sammelt die Positionen aller Anker.
    fn anchor_positions(&self) -> Vec<Vec2> {
        self.anchors.iter().map(|a| a.position()).collect()
    }

    /// Berechnet einen Phantom-Punkt aus einer Tangente.
    ///
    /// Der Phantom-Punkt liegt in der Verlängerung der Tangente,
    /// im Abstand des ersten/letzten Segments (wie bei der Spiegelung).
    fn phantom_from_tangent(anchor_pos: Vec2, tangent_angle: f32, neighbor_pos: Vec2) -> Vec2 {
        // Phantom-Punkt in Richtung weg vom Nachbar, im gleichen Abstand
        let dist = anchor_pos.distance(neighbor_pos).max(1.0);
        // Tangent-Angle zeigt zum Nachbar → Phantom in Gegenrichtung
        let dir = Vec2::from_angle(tangent_angle + std::f32::consts::PI);
        anchor_pos + dir * dist
    }

    /// Berechnet die Phantom-Punkte für Start und Ende basierend auf Tangenten.
    fn compute_phantoms(
        points: &[Vec2],
        tangent_start: SplineTangentSource,
        tangent_end: SplineTangentSource,
    ) -> (Option<Vec2>, Option<Vec2>) {
        let start_phantom = if let SplineTangentSource::Connection { angle, .. } = tangent_start {
            if points.len() >= 2 {
                Some(Self::phantom_from_tangent(points[0], angle, points[1]))
            } else {
                None
            }
        } else {
            None
        };

        let end_phantom = if let SplineTangentSource::Connection { angle, .. } = tangent_end {
            if points.len() >= 2 {
                let n = points.len();
                Some(Self::phantom_from_tangent(
                    points[n - 1],
                    angle,
                    points[n - 2],
                ))
            } else {
                None
            }
        } else {
            None
        };

        (start_phantom, end_phantom)
    }

    /// Befüllt die Nachbar-Liste für einen Snap-Node.
    fn populate_neighbors(anchor: &ToolAnchor, road_map: &RoadMap) -> Vec<ConnectedNeighbor> {
        match anchor {
            ToolAnchor::ExistingNode(id, _) => road_map.connected_neighbors(*id),
            ToolAnchor::NewPosition(_) => Vec::new(),
        }
    }

    /// Berechnet die dichte Spline-Polyline aus den Ankern (+ optionaler Cursor-Position).
    fn compute_dense_polyline(&self, extra_cursor: Option<Vec2>) -> Vec<Vec2> {
        let mut pts = self.anchor_positions();
        if let Some(c) = extra_cursor {
            pts.push(c);
        }
        if pts.len() < 2 {
            return pts;
        }
        let (start_phantom, end_phantom) =
            Self::compute_phantoms(&pts, self.tangent_start, self.tangent_end);
        catmull_rom_chain_with_tangents(&pts, 32, start_phantom, end_phantom)
    }

    /// Berechnet die verteilt gesampelten Positionen (für Nodes).
    fn compute_resampled(&self, extra_cursor: Option<Vec2>) -> Vec<Vec2> {
        let dense = self.compute_dense_polyline(extra_cursor);
        resample_by_distance(&dense, self.max_segment_length)
    }

    /// Spline-Länge über aktuelle Anker.
    fn spline_length(&self) -> f32 {
        let dense = self.compute_dense_polyline(None);
        polyline_length(&dense)
    }

    /// Synchronisiert den jeweils abhängigen Wert.
    fn sync_derived(&mut self) {
        let length = self.spline_length();
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

    /// Spline-Länge aus gegebenen Ankern (mit Tangenten).
    fn spline_length_from_anchors(
        anchors: &[ToolAnchor],
        tangent_start: SplineTangentSource,
        tangent_end: SplineTangentSource,
    ) -> f32 {
        let pts: Vec<Vec2> = anchors.iter().map(|a| a.position()).collect();
        if pts.len() < 2 {
            return 0.0;
        }
        let (start_phantom, end_phantom) = Self::compute_phantoms(&pts, tangent_start, tangent_end);
        let dense = catmull_rom_chain_with_tangents(&pts, 32, start_phantom, end_phantom);
        polyline_length(&dense)
    }

    /// Baut ToolResult aus gegebenen Ankern.
    fn build_result_from_anchors(
        anchors: &[ToolAnchor],
        max_segment_length: f32,
        direction: ConnectionDirection,
        priority: ConnectionPriority,
        tangent_start: SplineTangentSource,
        tangent_end: SplineTangentSource,
        road_map: &RoadMap,
    ) -> Option<ToolResult> {
        if anchors.len() < 2 {
            return None;
        }

        let pts: Vec<Vec2> = anchors.iter().map(|a| a.position()).collect();
        let (start_phantom, end_phantom) = Self::compute_phantoms(&pts, tangent_start, tangent_end);
        let dense = catmull_rom_chain_with_tangents(&pts, 32, start_phantom, end_phantom);
        let positions = resample_by_distance(&dense, max_segment_length);

        let first_anchor = anchors.first()?;
        let last_anchor = anchors.last()?;

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
                match first_anchor {
                    ToolAnchor::ExistingNode(id, _) => Some(*id),
                    _ => None,
                }
            } else if is_end {
                match last_anchor {
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

        // Verbindungen aufbauen
        for i in 0..positions.len().saturating_sub(1) {
            let a_new_idx = pos_to_new_idx[i];
            let b_new_idx = pos_to_new_idx[i + 1];

            let is_start_a = i == 0;
            let is_end_b = i + 1 == positions.len() - 1;

            let a_existing = if is_start_a {
                match first_anchor {
                    ToolAnchor::ExistingNode(id, _) => Some(*id),
                    _ => None,
                }
            } else {
                None
            };

            let b_existing = if is_end_b {
                match last_anchor {
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
}

impl Default for SplineTool {
    fn default() -> Self {
        Self::new()
    }
}

/// Versucht, auf einen existierenden Node zu snappen.
fn snap_to_node(pos: Vec2, road_map: &RoadMap, snap_radius: f32) -> ToolAnchor {
    if let Some(hit) = road_map.nearest_node(pos) {
        if hit.distance <= snap_radius {
            if let Some(node) = road_map.nodes.get(&hit.node_id) {
                return ToolAnchor::ExistingNode(hit.node_id, node.position);
            }
        }
    }
    ToolAnchor::NewPosition(pos)
}

/// Wandelt einen Winkel (Radiant) in eine Kompass-Richtung um.
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

impl RouteTool for SplineTool {
    fn name(&self) -> &str {
        "〰️ Spline"
    }

    fn description(&self) -> &str {
        "Zeichnet einen Catmull-Rom-Spline durch alle geklickten Punkte"
    }

    fn status_text(&self) -> &str {
        match self.anchors.len() {
            0 => "Startpunkt klicken",
            1 => "Nächsten Punkt klicken (mind. 2 Punkte)",
            _ => "Weitere Punkte klicken — Enter bestätigt, Escape abbrechen",
        }
    }

    fn on_click(&mut self, pos: Vec2, road_map: &RoadMap, _ctrl: bool) -> ToolAction {
        let anchor = snap_to_node(pos, road_map, self.snap_radius);

        if self.anchors.is_empty() {
            // Verkettung: letzten Endpunkt als Start verwenden
            if let Some(last_end) = self.last_end_anchor {
                self.last_created_ids.clear();
                self.last_anchors.clear();
                self.last_end_anchor = None;
                self.recreate_needed = false;
                self.tangent_start = SplineTangentSource::None;
                self.tangent_end = SplineTangentSource::None;
                self.anchors.push(last_end);
                self.anchors.push(anchor);
                self.sync_derived();
                return ToolAction::UpdatePreview;
            }
        }

        self.anchors.push(anchor);

        if self.anchors.len() >= 2 {
            self.sync_derived();
            ToolAction::UpdatePreview
        } else {
            ToolAction::Continue
        }
    }

    fn preview(&self, cursor_pos: Vec2, road_map: &RoadMap) -> ToolPreview {
        if self.anchors.is_empty() {
            return ToolPreview::default();
        }

        // Cursor als nächster Punkt (gesnappt)
        let snapped_cursor = snap_to_node(cursor_pos, road_map, self.snap_radius).position();

        let positions = if self.anchors.len() == 1 {
            // Nur Start + Cursor → gerade Linie (Preview)
            let start = self.anchors[0].position();
            vec![start, snapped_cursor]
        } else {
            // Spline durch alle Anker + Cursor
            self.compute_resampled(Some(snapped_cursor))
        };

        let connections: Vec<(usize, usize)> = (0..positions.len().saturating_sub(1))
            .map(|i| (i, i + 1))
            .collect();

        // Kontrollpunkte (Anker) als zusätzliche Preview-Nodes (für visuelle Markierung)
        let mut nodes = positions;
        for anchor in &self.anchors {
            nodes.push(anchor.position());
        }
        nodes.push(snapped_cursor);

        ToolPreview { nodes, connections }
    }

    fn render_config(&mut self, ui: &mut egui::Ui) -> bool {
        let mut changed = false;

        // Tangenten-Auswahl nur im Nachbearbeitungs-Modus —
        // Start/Ende stehen erst nach Enter fest
        let adjusting = !self.last_created_ids.is_empty() && self.last_anchors.len() >= 2;

        if adjusting {
            // Tangente Start
            if !self.start_neighbors.is_empty() {
                let old_tangent = self.tangent_start;
                let selected_text = match self.tangent_start {
                    SplineTangentSource::None => "Standard".to_string(),
                    SplineTangentSource::Connection { neighbor_id, angle } => {
                        format!("→ Node #{} ({})", neighbor_id, angle_to_compass(angle))
                    }
                };
                ui.label("Tangente Start:");
                egui::ComboBox::from_id_salt("spline_tangent_start")
                    .selected_text(selected_text)
                    .show_ui(ui, |ui| {
                        ui.selectable_value(
                            &mut self.tangent_start,
                            SplineTangentSource::None,
                            "Standard",
                        );
                        for neighbor in &self.start_neighbors {
                            let label = format!(
                                "→ Node #{} ({})",
                                neighbor.neighbor_id,
                                angle_to_compass(neighbor.angle)
                            );
                            ui.selectable_value(
                                &mut self.tangent_start,
                                SplineTangentSource::Connection {
                                    neighbor_id: neighbor.neighbor_id,
                                    angle: neighbor.angle,
                                },
                                label,
                            );
                        }
                    });
                if self.tangent_start != old_tangent {
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
                    SplineTangentSource::None => "Standard".to_string(),
                    SplineTangentSource::Connection { neighbor_id, angle } => {
                        format!("→ Node #{} ({})", neighbor_id, angle_to_compass(angle))
                    }
                };
                ui.label("Tangente Ende:");
                egui::ComboBox::from_id_salt("spline_tangent_end")
                    .selected_text(selected_text)
                    .show_ui(ui, |ui| {
                        ui.selectable_value(
                            &mut self.tangent_end,
                            SplineTangentSource::None,
                            "Standard",
                        );
                        for neighbor in &self.end_neighbors {
                            let label = format!(
                                "→ Node #{} ({})",
                                neighbor.neighbor_id,
                                angle_to_compass(neighbor.angle)
                            );
                            ui.selectable_value(
                                &mut self.tangent_end,
                                SplineTangentSource::Connection {
                                    neighbor_id: neighbor.neighbor_id,
                                    angle: neighbor.angle,
                                },
                                label,
                            );
                        }
                    });
                if self.tangent_end != old_tangent {
                    if !self.last_created_ids.is_empty() {
                        self.recreate_needed = true;
                    }
                    changed = true;
                }
            }

            if !self.start_neighbors.is_empty() || !self.end_neighbors.is_empty() {
                ui.add_space(4.0);
            }

            // Slider für Min. Abstand und Node-Anzahl im Nachbearbeitungs-Modus
            let length = Self::spline_length_from_anchors(
                &self.last_anchors,
                self.tangent_start,
                self.tangent_end,
            );

            ui.label(format!("Spline-Länge: {:.1} m", length));
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
            let length = self.spline_length();
            ui.label(format!("Spline-Länge: {:.1} m", length));
            ui.label(format!("Kontrollpunkte: {}", self.anchors.len()));
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
        Self::build_result_from_anchors(
            &self.anchors,
            self.max_segment_length,
            self.direction,
            self.priority,
            self.tangent_start,
            self.tangent_end,
            road_map,
        )
    }

    fn reset(&mut self) {
        self.anchors.clear();
        self.tangent_start = SplineTangentSource::None;
        self.tangent_end = SplineTangentSource::None;
        // start_neighbors / end_neighbors bleiben erhalten —
        // werden in set_last_created befüllt und im Nachbearbeitungs-Modus benötigt
    }

    fn is_ready(&self) -> bool {
        self.anchors.len() >= 2
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

    fn set_last_created(&mut self, ids: Vec<u64>, road_map: &RoadMap) {
        // Nur bei Erst-Erstellung Anker übernehmen; bei Recreate bleiben last_anchors erhalten
        if !self.anchors.is_empty() {
            self.last_anchors = self.anchors.clone();
            if let Some(last) = self.anchors.last() {
                self.last_end_anchor = Some(*last);
            }
        }
        // Nachbarn aus den richtigen Ankern befüllen (anchors oder last_anchors)
        let source = if !self.anchors.is_empty() {
            &self.anchors
        } else {
            &self.last_anchors
        };
        if let Some(first) = source.first() {
            self.start_neighbors = Self::populate_neighbors(first, road_map);
        }
        if let Some(last) = source.last() {
            self.end_neighbors = Self::populate_neighbors(last, road_map);
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
        // Aktuelle Tangenten verwenden (nicht last_tangent_*),
        // damit Änderungen im Nachbearbeitungs-Modus wirksam werden
        Self::build_result_from_anchors(
            &self.last_anchors,
            self.max_segment_length,
            self.direction,
            self.priority,
            self.tangent_start,
            self.tangent_end,
            road_map,
        )
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    // ── Catmull-Rom-Geometrie ──

    #[test]
    fn test_catmull_rom_two_points_straight_line() {
        let points = vec![Vec2::ZERO, Vec2::new(10.0, 0.0)];
        let result = catmull_rom_chain(&points, 10);
        assert_eq!(result.len(), 11);
        assert!((result[0] - Vec2::ZERO).length() < 0.001);
        assert!((result[10] - Vec2::new(10.0, 0.0)).length() < 0.001);
    }

    #[test]
    fn test_catmull_rom_passes_through_control_points() {
        let points = vec![
            Vec2::new(0.0, 0.0),
            Vec2::new(5.0, 10.0),
            Vec2::new(10.0, 0.0),
        ];
        let result = catmull_rom_chain(&points, 20);

        // Startpunkt
        assert!((result[0] - points[0]).length() < 0.01);
        // Mittelpunkt (bei t=0 des zweiten Segments = Index 20)
        assert!((result[20] - points[1]).length() < 0.01);
        // Endpunkt
        assert!(
            (result.last().unwrap().distance(points[2])) < 0.01,
            "Endpunkt: {:?} vs {:?}",
            result.last(),
            points[2]
        );
    }

    #[test]
    fn test_catmull_rom_four_points() {
        let points = vec![
            Vec2::new(0.0, 0.0),
            Vec2::new(3.0, 5.0),
            Vec2::new(7.0, 5.0),
            Vec2::new(10.0, 0.0),
        ];
        let result = catmull_rom_chain(&points, 10);

        // Muss mindestens (4-1)*10 + 1 = 31 Punkte haben
        assert_eq!(result.len(), 31);
        // Start- und Endpunkte
        assert!((result[0] - points[0]).length() < 0.01);
        assert!(result.last().unwrap().distance(points[3]) < 0.01);
        // Durchlaufen durch Zwischenpunkte
        assert!((result[10] - points[1]).length() < 0.01);
        assert!((result[20] - points[2]).length() < 0.01);
    }

    #[test]
    fn test_resample_preserves_endpoints() {
        let polyline = vec![
            Vec2::new(0.0, 0.0),
            Vec2::new(5.0, 0.0),
            Vec2::new(10.0, 0.0),
        ];
        let resampled = resample_by_distance(&polyline, 3.0);

        assert!((resampled[0] - Vec2::ZERO).length() < 0.01);
        assert!((resampled.last().unwrap().distance(Vec2::new(10.0, 0.0))) < 0.01);
    }

    #[test]
    fn test_resample_spacing() {
        let polyline = vec![
            Vec2::new(0.0, 0.0),
            Vec2::new(5.0, 0.0),
            Vec2::new(10.0, 0.0),
        ];
        let resampled = resample_by_distance(&polyline, 2.0);

        // 10m / 2m = 5 Segmente → 6 Punkte
        assert_eq!(resampled.len(), 6);
        for i in 0..resampled.len() - 1 {
            let dist = resampled[i].distance(resampled[i + 1]);
            assert!(
                (dist - 2.0).abs() < 0.1,
                "Segment {} hat Abstand {:.3}m",
                i,
                dist
            );
        }
    }

    // ── Tool-Flow ──

    #[test]
    fn test_spline_tool_click_flow() {
        let mut tool = SplineTool::new();
        let road_map = RoadMap::new(3);

        assert!(!tool.is_ready());
        assert_eq!(tool.status_text(), "Startpunkt klicken");

        // Erster Klick
        let action = tool.on_click(Vec2::ZERO, &road_map, false);
        assert_eq!(action, ToolAction::Continue);
        assert!(!tool.is_ready());

        // Zweiter Klick → bereit
        let action = tool.on_click(Vec2::new(10.0, 0.0), &road_map, false);
        assert_eq!(action, ToolAction::UpdatePreview);
        assert!(tool.is_ready());

        // Dritter Klick → immer noch bereit, Spline wird aktualisiert
        let action = tool.on_click(Vec2::new(5.0, 8.0), &road_map, false);
        assert_eq!(action, ToolAction::UpdatePreview);
        assert!(tool.is_ready());
        assert_eq!(tool.anchors.len(), 3);
    }

    #[test]
    fn test_spline_tool_execute() {
        let mut tool = SplineTool::new();
        tool.max_segment_length = 2.0;
        let road_map = RoadMap::new(3);

        tool.on_click(Vec2::ZERO, &road_map, false);
        tool.on_click(Vec2::new(10.0, 0.0), &road_map, false);
        tool.on_click(Vec2::new(5.0, 8.0), &road_map, false);

        let result = tool.execute(&road_map).expect("Ergebnis erwartet");
        assert!(result.new_nodes.len() >= 3);
        assert_eq!(
            result.internal_connections.len(),
            result.new_nodes.len() - 1,
        );
    }

    #[test]
    fn test_spline_tool_reset() {
        let mut tool = SplineTool::new();
        let road_map = RoadMap::new(3);

        tool.on_click(Vec2::ZERO, &road_map, false);
        tool.on_click(Vec2::new(10.0, 0.0), &road_map, false);
        assert!(tool.is_ready());

        tool.reset();
        assert!(!tool.is_ready());
        assert!(tool.anchors.is_empty());
    }

    #[test]
    fn test_spline_tool_chaining() {
        let mut tool = SplineTool::new();
        let road_map = RoadMap::new(3);

        // Erste Strecke
        tool.on_click(Vec2::ZERO, &road_map, false);
        tool.on_click(Vec2::new(10.0, 0.0), &road_map, false);
        tool.on_click(Vec2::new(5.0, 8.0), &road_map, false);
        tool.set_last_created(vec![100, 101, 102, 103], &road_map);
        tool.reset();

        // Verkettung: nächster Klick übernimmt letzten Endpunkt
        let action = tool.on_click(Vec2::new(20.0, 0.0), &road_map, false);
        assert_eq!(action, ToolAction::UpdatePreview);
        assert_eq!(tool.anchors.len(), 2);
    }

    #[test]
    fn test_spline_tool_preview_with_cursor() {
        let mut tool = SplineTool::new();
        let road_map = RoadMap::new(3);

        tool.on_click(Vec2::ZERO, &road_map, false);
        tool.on_click(Vec2::new(10.0, 0.0), &road_map, false);

        let preview = tool.preview(Vec2::new(5.0, 8.0), &road_map);
        // Vorschau sollte Nodes und Connections enthalten
        assert!(!preview.nodes.is_empty());
        assert!(!preview.connections.is_empty());
    }

    #[test]
    fn test_spline_execute_from_anchors() {
        let mut tool = SplineTool::new();
        tool.max_segment_length = 2.0;
        let road_map = RoadMap::new(3);

        tool.on_click(Vec2::ZERO, &road_map, false);
        tool.on_click(Vec2::new(10.0, 0.0), &road_map, false);
        tool.on_click(Vec2::new(5.0, 8.0), &road_map, false);
        let original = tool.execute(&road_map).unwrap();
        tool.set_last_created(vec![1, 2, 3, 4, 5], &road_map);
        tool.reset();

        // Nachbearbeitung mit anderer Segment-Länge
        tool.max_segment_length = 5.0;
        let result = tool
            .execute_from_anchors(&road_map)
            .expect("Ergebnis erwartet");
        assert!(result.new_nodes.len() < original.new_nodes.len());
    }
}
