//! Lifecycle-Methoden des StraightLineTool (RouteTool-Implementierung).

use super::super::{snap_to_node, RouteTool, ToolAction, ToolAnchor, ToolPreview, ToolResult};
use super::geometry::{build_result, compute_line_positions};
use super::state::StraightLineTool;
use crate::core::{ConnectionDirection, ConnectionPriority, RoadMap};
use glam::Vec2;

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
        let anchor = snap_to_node(pos, road_map, self.lifecycle.snap_radius);

        if self.start.is_none() {
            // Verkettung: letzten Endpunkt als Start verwenden
            if let Some(last_end) = self.lifecycle.last_end_anchor {
                self.lifecycle.last_created_ids.clear();
                self.last_start_anchor = None;
                self.lifecycle.last_end_anchor = None;
                self.lifecycle.recreate_needed = false;
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
                let snapped = snap_to_node(cursor_pos, road_map, self.lifecycle.snap_radius);
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
        self.render_config_view(ui)
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
        // lifecycle.last_created_ids, last_*_anchor bleiben erhalten für Nachbearbeitung/Verkettung
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
        self.lifecycle.snap_radius = radius;
    }

    fn set_last_created(&mut self, ids: Vec<u64>, _road_map: &RoadMap) {
        // Anker nur überschreiben wenn aktuelle start/end gesetzt sind.
        // Beim Recreate sind start/end None — Anker bleiben erhalten.
        if self.start.is_some() {
            self.last_start_anchor = self.start;
        }
        if self.end.is_some() {
            self.lifecycle.last_end_anchor = self.end;
        }
        self.lifecycle.last_created_ids = ids;
        self.lifecycle.recreate_needed = false;
    }

    fn last_created_ids(&self) -> &[u64] {
        &self.lifecycle.last_created_ids
    }

    fn last_end_anchor(&self) -> Option<ToolAnchor> {
        self.lifecycle.last_end_anchor
    }

    fn needs_recreate(&self) -> bool {
        self.lifecycle.recreate_needed
    }

    fn clear_recreate_flag(&mut self) {
        self.lifecycle.recreate_needed = false;
    }

    fn execute_from_anchors(&self, road_map: &RoadMap) -> Option<ToolResult> {
        let start = self.last_start_anchor?;
        let end = self.lifecycle.last_end_anchor?;
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
