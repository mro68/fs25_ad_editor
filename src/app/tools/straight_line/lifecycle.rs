//! Lifecycle-Methoden des StraightLineTool (RouteTool-Implementierung).

use super::super::common::linear_connections;
use super::super::{snap_to_node, RouteTool, ToolAction, ToolPreview, ToolResult};
use super::geometry::{build_result, compute_line_positions};
use super::state::StraightLineTool;
use crate::app::segment_registry::{SegmentKind, SegmentRecord};
use crate::core::RoadMap;
use glam::Vec2;

impl RouteTool for StraightLineTool {
    fn name(&self) -> &str {
        "Gerade Strecke"
    }

    fn icon(&self) -> &str {
        "━"
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
            if let Some(last_end) = self.lifecycle.chaining_start_anchor() {
                self.lifecycle.prepare_for_chaining();
                self.last_start_anchor = None;
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
        let connections = linear_connections(positions.len());

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

    fn has_pending_input(&self) -> bool {
        self.start.is_some()
    }

    crate::impl_lifecycle_delegation!();

    fn set_last_created(&mut self, ids: &[u64], _road_map: &RoadMap) {
        // Anker nur überschreiben wenn aktuelle start/end gesetzt sind.
        // Beim Recreate sind start/end None — Anker bleiben erhalten.
        if self.start.is_some() {
            self.last_start_anchor = self.start;
        }
        if self.end.is_some() {
            self.lifecycle.last_end_anchor = self.end;
        }
        self.lifecycle.save_created_ids(ids);
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

    fn make_segment_record(&self, id: u64, node_ids: &[u64]) -> Option<SegmentRecord> {
        let start = self.last_start_anchor?;
        let end = self.lifecycle.last_end_anchor?;
        Some(SegmentRecord {
            id,
            node_ids: node_ids.to_vec(),
            start_anchor: start,
            end_anchor: end,
            kind: SegmentKind::Straight {
                direction: self.direction,
                priority: self.priority,
                max_segment_length: self.seg.max_segment_length,
            },
        })
    }

    fn load_for_edit(&mut self, record: &SegmentRecord, kind: &SegmentKind) {
        let SegmentKind::Straight {
            direction,
            priority,
            max_segment_length,
        } = kind
        else {
            return;
        };
        self.start = Some(record.start_anchor);
        self.end = Some(record.end_anchor);
        self.direction = *direction;
        self.priority = *priority;
        self.seg.max_segment_length = *max_segment_length;
        self.sync_derived();
    }
}
