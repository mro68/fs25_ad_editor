//! Lifecycle-Methoden des StraightLineTool (RouteTool-Implementierung).

use super::super::common::linear_connections;
use super::super::{RouteTool, ToolAction, ToolPreview, ToolResult};
use super::geometry::{build_result, compute_line_positions};
use super::state::StraightLineTool;
use crate::app::group_registry::{GroupBase, GroupKind, GroupRecord};
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
            (Some(_), Some(_)) => "Bereit — Enter zum Ausfuehren, Escape zum Abbrechen",
        }
    }

    fn on_click(&mut self, pos: Vec2, road_map: &RoadMap, _ctrl: bool) -> ToolAction {
        let anchor = self.lifecycle.snap_at(pos, road_map);

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
                let snapped = self.lifecycle.snap_at(cursor_pos, road_map);
                snapped.position()
            }
        };

        let positions = compute_line_positions(start_pos, end_pos, self.seg.max_segment_length);
        let connections = linear_connections(positions.len());
        let styles = vec![(self.direction, self.priority); connections.len()];

        ToolPreview {
            nodes: positions,
            connections,
            connection_styles: styles,
            labels: vec![],
        }
    }

    fn render_config(&mut self, ui: &mut egui::Ui, distance_wheel_step_m: f32) -> bool {
        self.render_config_view(ui, distance_wheel_step_m)
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

    /// Setzt das Tool auf den Anfangszustand zurueck.
    ///
    /// Loescht nur Start/End. `lifecycle.last_created_ids` und `last_*_anchor`
    /// bleiben erhalten fuer Verkettung und Nachbearbeitung.
    fn reset(&mut self) {
        self.start = None;
        self.end = None;
    }

    fn is_ready(&self) -> bool {
        self.start.is_some() && self.end.is_some()
    }

    fn has_pending_input(&self) -> bool {
        self.start.is_some()
    }

    crate::impl_lifecycle_delegation!();

    fn current_end_anchor(&self) -> Option<super::super::ToolAnchor> {
        self.end.or(self.lifecycle.last_end_anchor)
    }

    fn save_anchors_for_recreate(&mut self, _road_map: &RoadMap) {
        // Anker nur ueberschreiben wenn aktuelle start/end gesetzt sind.
        // Beim Recreate sind start/end None — Anker bleiben erhalten.
        if self.start.is_some() {
            self.last_start_anchor = self.start;
        }
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

    fn make_group_record(&self, id: u64, node_ids: &[u64]) -> Option<GroupRecord> {
        let start = self.last_start_anchor?;
        let end = self.lifecycle.last_end_anchor?;
        Some(GroupRecord {
            id,
            node_ids: node_ids.to_vec(),
            start_anchor: start,
            end_anchor: end,
            original_positions: Vec::new(), // wird im Handler befüllt
            marker_node_ids: Vec::new(),
            locked: true,
            kind: GroupKind::Straight {
                base: GroupBase {
                    direction: self.direction,
                    priority: self.priority,
                    max_segment_length: self.seg.max_segment_length,
                },
            },
        })
    }

    fn load_for_edit(&mut self, record: &GroupRecord, kind: &GroupKind) {
        let GroupKind::Straight { base } = kind else {
            return;
        };
        self.start = Some(record.start_anchor);
        self.end = Some(record.end_anchor);
        self.direction = base.direction;
        self.priority = base.priority;
        self.seg.max_segment_length = base.max_segment_length;
        self.sync_derived();
    }
}
