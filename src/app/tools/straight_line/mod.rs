//! Gerade-Strecke-Tool: Zeichnet eine Linie zwischen zwei Punkten
//! und füllt automatisch Zwischen-Nodes ein.

use super::{
    common::{self, SegmentConfig},
    snap_to_node, RouteTool, ToolAction, ToolAnchor, ToolPreview, ToolResult,
};
use crate::core::{ConnectionDirection, ConnectionPriority, RoadMap};
use crate::shared::SNAP_RADIUS;
use glam::Vec2;

/// Gerade-Strecke-Tool
pub struct StraightLineTool {
    start: Option<ToolAnchor>,
    end: Option<ToolAnchor>,
    /// Segment-Konfiguration (Abstand / Node-Anzahl)
    pub(crate) seg: SegmentConfig,
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
    /// Snap-Radius in Welteinheiten (aus EditorOptions)
    snap_radius: f32,
}

impl StraightLineTool {
    /// Erstellt ein neues Gerade-Strecke-Tool mit Standardwerten.
    pub fn new() -> Self {
        Self {
            start: None,
            end: None,
            seg: SegmentConfig::new(6.0),
            direction: ConnectionDirection::Dual,
            priority: ConnectionPriority::Regular,
            last_created_ids: Vec::new(),
            last_start_anchor: None,
            last_end_anchor: None,
            recreate_needed: false,
            snap_radius: SNAP_RADIUS,
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
        self.seg.sync_from_length(self.total_distance());
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
        let anchor = snap_to_node(pos, road_map, self.snap_radius);

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
                let snapped = snap_to_node(cursor_pos, road_map, self.snap_radius);
                snapped.position()
            }
        };

        let positions = compute_line_positions(start_pos, end_pos, self.seg.max_segment_length);
        let connections: Vec<(usize, usize)> = (0..positions.len().saturating_sub(1))
            .map(|i| (i, i + 1))
            .collect();

        ToolPreview {
            nodes: positions,
            connections,
        }
    }

    fn render_config(&mut self, ui: &mut egui::Ui) -> bool {
        // Nachbearbeitungs-Modus: letzte Strecke anpassen
        let adjusting = !self.last_created_ids.is_empty()
            && self.last_start_anchor.is_some()
            && self.last_end_anchor.is_some();

        if adjusting {
            let Some(start_anchor) = self.last_start_anchor else {
                return false;
            };
            let Some(end_anchor) = self.last_end_anchor else {
                return false;
            };
            let distance = start_anchor.position().distance(end_anchor.position());
            let (changed, recreate) = self.seg.render_adjusting(ui, distance, "Streckenlänge");
            if recreate {
                self.recreate_needed = true;
            }
            changed
        } else if self.is_ready() {
            let distance = self.total_distance();
            self.seg.render_live(ui, distance, "Streckenlänge")
        } else {
            self.seg.render_default(ui)
        }
    }

    fn execute(&self, road_map: &RoadMap) -> Option<ToolResult> {
        let start = *self.start.as_ref()?;
        let end = *self.end.as_ref()?;
        build_result(
            start,
            end,
            self.seg.max_segment_length,
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

    fn set_snap_radius(&mut self, radius: f32) {
        self.snap_radius = radius;
    }

    fn set_last_created(&mut self, ids: Vec<u64>, _road_map: &RoadMap) {
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
            self.seg.max_segment_length,
            self.direction,
            self.priority,
            road_map,
        )
    }
}

/// Gemeinsame Logik für `execute()` und `execute_from_anchors()`:
/// Berechnet Positionen und delegiert Node-/Verbindungs-Aufbau an `assemble_tool_result`.
fn build_result(
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

#[cfg(test)]
mod tests;
