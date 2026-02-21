//! Gerade-Strecke-Tool: Zeichnet eine Linie zwischen zwei Punkten
//! und füllt automatisch Zwischen-Nodes ein.

use super::{RouteTool, ToolAction, ToolAnchor, ToolPreview, ToolResult};
use crate::core::{ConnectionDirection, ConnectionPriority, NodeFlag, RoadMap};
use glam::Vec2;

/// Snap-Distanz: Klick innerhalb dieses Radius rastet auf existierenden Node ein.
const SNAP_RADIUS: f32 = 3.0;

/// Welcher Wert wurde zuletzt vom User geändert?
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum LastEdited {
    /// User hat Segment-Länge angepasst → Node-Anzahl wird berechnet
    Distance,
    /// User hat Node-Anzahl angepasst → Segment-Länge wird berechnet
    NodeCount,
}

/// Gerade-Strecke-Tool
pub struct StraightLineTool {
    start: Option<ToolAnchor>,
    end: Option<ToolAnchor>,
    /// Maximaler Abstand zwischen Zwischen-Nodes
    pub max_segment_length: f32,
    /// Gewünschte Anzahl Nodes (inkl. Start+End)
    pub node_count: usize,
    /// Welcher Parameter zuletzt vom User geändert wurde
    last_edited: LastEdited,
    /// Richtung für die erzeugten Verbindungen (aus Editor-Defaults)
    pub direction: ConnectionDirection,
    /// Priorität für die erzeugten Verbindungen (aus Editor-Defaults)
    pub priority: ConnectionPriority,
    /// IDs der zuletzt erstellten Nodes (für Nachbearbeitung)
    last_created_ids: Vec<u64>,
    /// Start-Anker der letzten Erstellung (für Neuberechnung)
    last_start_anchor: Option<ToolAnchor>,
    /// End-Anker der letzten Erstellung (für Verkettung)
    last_end_anchor: Option<ToolAnchor>,
    /// Signalisiert, dass Config geändert wurde und Neuberechnung nötig ist
    recreate_needed: bool,
}

impl StraightLineTool {
    /// Erstellt ein neues Gerade-Strecke-Tool mit Standardwerten.
    pub fn new() -> Self {
        Self {
            start: None,
            end: None,
            max_segment_length: 6.0,
            node_count: 2,
            last_edited: LastEdited::Distance,
            direction: ConnectionDirection::Dual,
            priority: ConnectionPriority::Regular,
            last_created_ids: Vec::new(),
            last_start_anchor: None,
            last_end_anchor: None,
            recreate_needed: false,
        }
    }

    /// Berechnet die Gesamtlänge der Strecke (0.0 wenn nicht bereit).
    fn total_distance(&self) -> f32 {
        match (&self.start, &self.end) {
            (Some(s), Some(e)) => s.position().distance(e.position()),
            _ => 0.0,
        }
    }

    /// Synchronisiert den jeweils abhängigen Wert.
    fn sync_derived(&mut self) {
        let distance = self.total_distance();
        if distance < f32::EPSILON {
            return;
        }
        match self.last_edited {
            LastEdited::Distance => {
                // Node-Anzahl aus Distanz ableiten
                let segments = (distance / self.max_segment_length).ceil().max(1.0) as usize;
                self.node_count = segments + 1;
            }
            LastEdited::NodeCount => {
                // Segment-Länge aus Node-Anzahl ableiten
                let segments = (self.node_count.max(2) - 1) as f32;
                self.max_segment_length = distance / segments;
            }
        }
    }
}

impl Default for StraightLineTool {
    fn default() -> Self {
        Self::new()
    }
}

/// Berechnet die Zwischen-Positionen einer geraden Strecke.
fn compute_line_positions(start: Vec2, end: Vec2, max_segment_length: f32) -> Vec<Vec2> {
    let distance = start.distance(end);
    if distance < f32::EPSILON {
        return vec![start];
    }
    let segment_count = (distance / max_segment_length).ceil().max(1.0) as usize;
    (0..=segment_count)
        .map(|i| start.lerp(end, i as f32 / segment_count as f32))
        .collect()
}

/// Versucht, auf einen existierenden Node zu snappen.
fn snap_to_node(pos: Vec2, road_map: &RoadMap) -> ToolAnchor {
    if let Some(hit) = road_map.nearest_node(pos) {
        if hit.distance <= SNAP_RADIUS {
            // Position aus der RoadMap holen
            if let Some(node) = road_map.nodes.get(&hit.node_id) {
                return ToolAnchor::ExistingNode(hit.node_id, node.position);
            }
        }
    }
    ToolAnchor::NewPosition(pos)
}

impl RouteTool for StraightLineTool {
    fn name(&self) -> &str {
        "📏 Gerade Strecke"
    }

    fn description(&self) -> &str {
        "Zeichnet eine gerade Linie zwischen zwei Punkten mit Zwischen-Nodes"
    }

    fn status_text(&self) -> &str {
        match (&self.start, &self.end) {
            (None, _) => "Startpunkt klicken",
            (Some(_), None) => "Endpunkt klicken",
            (Some(_), Some(_)) => "Bereit — Enter zum Ausführen, Escape zum Abbrechen",
        }
    }

    fn on_click(&mut self, pos: Vec2, road_map: &RoadMap, _ctrl: bool) -> ToolAction {
        let anchor = snap_to_node(pos, road_map);

        if self.start.is_none() {
            // Verkettung: letzten Endpunkt als Start verwenden
            if let Some(last_end) = self.last_end_anchor {
                self.last_created_ids.clear();
                self.last_start_anchor = None;
                self.last_end_anchor = None;
                self.recreate_needed = false;
                self.start = Some(last_end);
                self.end = Some(anchor);
                self.sync_derived();
                ToolAction::ReadyToExecute
            } else {
                self.start = Some(anchor);
                ToolAction::Continue
            }
        } else {
            self.end = Some(anchor);
            self.sync_derived();
            ToolAction::ReadyToExecute
        }
    }

    fn preview(&self, cursor_pos: Vec2, road_map: &RoadMap) -> ToolPreview {
        let start_pos = match &self.start {
            Some(anchor) => anchor.position(),
            None => return ToolPreview::default(),
        };

        let end_pos = match &self.end {
            Some(anchor) => anchor.position(),
            None => {
                // Preview zur aktuellen Mausposition
                let snapped = snap_to_node(cursor_pos, road_map);
                snapped.position()
            }
        };

        let positions = compute_line_positions(start_pos, end_pos, self.max_segment_length);
        let connections: Vec<(usize, usize)> = (0..positions.len().saturating_sub(1))
            .map(|i| (i, i + 1))
            .collect();

        ToolPreview {
            nodes: positions,
            connections,
        }
    }

    fn render_config(&mut self, ui: &mut egui::Ui) -> bool {
        let mut changed = false;

        // Nachbearbeitungs-Modus: letzte Strecke anpassen
        let adjusting = !self.last_created_ids.is_empty()
            && self.last_start_anchor.is_some()
            && self.last_end_anchor.is_some();

        if adjusting {
            let Some(start_anchor) = self.last_start_anchor else {
                return changed;
            };
            let Some(end_anchor) = self.last_end_anchor else {
                return changed;
            };
            let start_pos = start_anchor.position();
            let end_pos = end_anchor.position();
            let distance = start_pos.distance(end_pos);

            ui.label(format!("Streckenlänge: {:.1} m", distance));
            ui.add_space(4.0);

            // Segment-Länge
            ui.label("Min. Abstand:");
            let max_seg = distance.max(1.0);
            if ui
                .add(egui::Slider::new(&mut self.max_segment_length, 1.0..=max_seg).suffix(" m"))
                .changed()
            {
                self.last_edited = LastEdited::Distance;
                // Node-Anzahl ableiten
                let segments = (distance / self.max_segment_length).ceil().max(1.0) as usize;
                self.node_count = segments + 1;
                self.recreate_needed = true;
                changed = true;
            }

            ui.add_space(4.0);

            // Node-Anzahl
            ui.label("Anzahl Nodes:");
            let max_nodes = (distance / 1.0).ceil().max(2.0) as usize;
            if ui
                .add(egui::Slider::new(&mut self.node_count, 2..=max_nodes))
                .changed()
            {
                self.last_edited = LastEdited::NodeCount;
                // Segment-Länge ableiten
                let segments = (self.node_count.max(2) - 1) as f32;
                self.max_segment_length = distance / segments;
                self.recreate_needed = true;
                changed = true;
            }
        } else if self.is_ready() {
            let distance = self.total_distance();
            ui.label(format!("Streckenlänge: {:.1} m", distance));
            ui.add_space(4.0);

            // Segment-Länge
            ui.label("Min. Abstand:");
            let old_seg = self.max_segment_length;
            let max_seg = distance.max(1.0);
            if ui
                .add(egui::Slider::new(&mut self.max_segment_length, 1.0..=max_seg).suffix(" m"))
                .changed()
            {
                self.last_edited = LastEdited::Distance;
                self.sync_derived();
                changed = true;
            }
            // Wenn Slider nicht angefasst wurde, sicherstellen dass der Wert geclampt ist
            if self.max_segment_length != old_seg && self.last_edited == LastEdited::Distance {
                self.sync_derived();
            }

            ui.add_space(4.0);

            // Node-Anzahl
            ui.label("Anzahl Nodes:");
            let max_nodes = (distance / 1.0).ceil().max(2.0) as usize;
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
        let start = *self.start.as_ref()?;
        let end = *self.end.as_ref()?;
        build_result(
            start,
            end,
            self.max_segment_length,
            self.direction,
            self.priority,
            road_map,
        )
    }

    fn reset(&mut self) {
        self.start = None;
        self.end = None;
        // last_created_ids, last_*_anchor bleiben erhalten für Nachbearbeitung/Verkettung
    }

    fn is_ready(&self) -> bool {
        self.start.is_some() && self.end.is_some()
    }

    fn set_direction(&mut self, dir: ConnectionDirection) {
        self.direction = dir;
    }

    fn set_priority(&mut self, prio: ConnectionPriority) {
        self.priority = prio;
    }

    fn set_last_created(&mut self, ids: Vec<u64>) {
        // Anker nur überschreiben wenn aktuelle start/end gesetzt sind.
        // Beim Recreate sind start/end None — Anker bleiben erhalten.
        if self.start.is_some() {
            self.last_start_anchor = self.start;
        }
        if self.end.is_some() {
            self.last_end_anchor = self.end;
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
        let start = self.last_start_anchor?;
        let end = self.last_end_anchor?;
        build_result(
            start,
            end,
            self.max_segment_length,
            self.direction,
            self.priority,
            road_map,
        )
    }
}

/// Gemeinsame Logik für `execute()` und `execute_from_anchors()`:
/// Berechnet Positionen, erstellt Nodes und Verbindungen.
fn build_result(
    start: ToolAnchor,
    end: ToolAnchor,
    max_segment_length: f32,
    direction: ConnectionDirection,
    priority: ConnectionPriority,
    road_map: &RoadMap,
) -> Option<ToolResult> {
    let start_pos = start.position();
    let end_pos = end.position();
    let positions = compute_line_positions(start_pos, end_pos, max_segment_length);

    let mut new_nodes: Vec<(Vec2, NodeFlag)> = Vec::new();
    let mut internal_connections: Vec<(usize, usize, ConnectionDirection, ConnectionPriority)> =
        Vec::new();
    let mut external_connections: Vec<(usize, u64, ConnectionDirection, ConnectionPriority)> =
        Vec::new();

    // Mapping: Position-Index → new_nodes-Index (oder None wenn existierender Node)
    let mut pos_to_new_idx: Vec<Option<usize>> = Vec::with_capacity(positions.len());

    for (i, &pos) in positions.iter().enumerate() {
        let is_start = i == 0;
        let is_end = i == positions.len() - 1;

        let existing_id = if is_start {
            match start {
                ToolAnchor::ExistingNode(id, _) => Some(id),
                _ => None,
            }
        } else if is_end {
            match end {
                ToolAnchor::ExistingNode(id, _) => Some(id),
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
            match start {
                ToolAnchor::ExistingNode(id, _) => Some(id),
                _ => None,
            }
        } else {
            None
        };

        let b_existing = if is_end_b {
            match end {
                ToolAnchor::ExistingNode(id, _) => Some(id),
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
mod tests;
