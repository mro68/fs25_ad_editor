//! Lifecycle-Methoden des SplineTool (RouteTool-Implementierung).

use super::super::{
    common::populate_neighbors, snap_to_node, RouteTool, ToolAction, ToolAnchor, ToolPreview,
    ToolResult,
};
use super::state::SplineTool;
use crate::app::segment_registry::{SegmentKind, SegmentRecord};
use crate::core::{ConnectionDirection, ConnectionPriority, RoadMap};
use glam::Vec2;

impl RouteTool for SplineTool {
    fn name(&self) -> &str {
        "Spline"
    }

    fn icon(&self) -> &str {
        "〰"
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
        let anchor = snap_to_node(pos, road_map, self.lifecycle.snap_radius);

        if self.anchors.is_empty() {
            // Verkettung: letzten Endpunkt als Start verwenden
            if let Some(last_end) = self.lifecycle.chaining_start_anchor() {
                self.lifecycle.last_created_ids.clear();
                self.last_anchors.clear();
                self.lifecycle.last_end_anchor = None;
                self.lifecycle.recreate_needed = false;
                self.tangents.reset_tangents();
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

        let snapped_cursor =
            snap_to_node(cursor_pos, road_map, self.lifecycle.snap_radius).position();

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
        self.render_config_view(ui)
    }

    fn execute(&self, road_map: &RoadMap) -> Option<ToolResult> {
        Self::build_result_from_anchors(
            &self.anchors,
            self.seg.max_segment_length,
            self.direction,
            self.priority,
            self.tangents.tangent_start,
            self.tangents.tangent_end,
            road_map,
        )
    }

    fn reset(&mut self) {
        self.anchors.clear();
        self.tangents.reset_tangents();
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
        self.lifecycle.snap_radius = radius;
    }

    fn set_last_created(&mut self, ids: Vec<u64>, road_map: &RoadMap) {
        // Nur bei Erst-Erstellung Anker übernehmen; bei Recreate bleiben last_anchors erhalten
        if !self.anchors.is_empty() {
            self.last_anchors = self.anchors.clone();
            if let Some(last) = self.anchors.last() {
                self.lifecycle.last_end_anchor = Some(*last);
            }
        }
        // Nachbarn aus den richtigen Ankern befüllen (anchors oder last_anchors)
        let source = if !self.anchors.is_empty() {
            &self.anchors
        } else {
            &self.last_anchors
        };
        if let Some(first) = source.first() {
            self.tangents.start_neighbors = populate_neighbors(first, road_map);
        }
        if let Some(last) = source.last() {
            self.tangents.end_neighbors = populate_neighbors(last, road_map);
        }
        self.tangents.save_for_recreate();
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
        // Aktuelle Tangenten verwenden (nicht last_tangent_*),
        // damit Änderungen im Nachbearbeitungs-Modus wirksam werden
        Self::build_result_from_anchors(
            &self.last_anchors,
            self.seg.max_segment_length,
            self.direction,
            self.priority,
            self.tangents.tangent_start,
            self.tangents.tangent_end,
            road_map,
        )
    }

    fn make_segment_record(&self, id: u64, node_ids: Vec<u64>) -> Option<SegmentRecord> {
        if self.last_anchors.len() < 2 {
            return None;
        }
        let start = *self.last_anchors.first()?;
        let end = *self.last_anchors.last()?;
        Some(SegmentRecord {
            id,
            node_ids,
            start_anchor: start,
            end_anchor: end,
            kind: SegmentKind::Spline {
                anchors: self.last_anchors.clone(),
                tangent_start: self.tangents.last_tangent_start,
                tangent_end: self.tangents.last_tangent_end,
                direction: self.direction,
                priority: self.priority,
                max_segment_length: self.seg.max_segment_length,
            },
        })
    }

    fn load_for_edit(&mut self, _record: &SegmentRecord, kind: &SegmentKind) {
        let SegmentKind::Spline {
            anchors,
            tangent_start,
            tangent_end,
            direction,
            priority,
            max_segment_length,
        } = kind
        else {
            return;
        };
        self.anchors = anchors.clone();
        self.last_anchors = anchors.clone();
        self.tangents.tangent_start = *tangent_start;
        self.tangents.tangent_end = *tangent_end;
        self.tangents.last_tangent_start = *tangent_start;
        self.tangents.last_tangent_end = *tangent_end;
        self.direction = *direction;
        self.priority = *priority;
        self.seg.max_segment_length = *max_segment_length;
        self.sync_derived();
    }
}
