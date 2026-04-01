//! Lifecycle-Methoden des SplineTool (RouteTool-Implementierung).

use super::super::{
    common::{linear_connections, populate_neighbors, tangent_options},
    RouteTool, RouteToolId, ToolAction, ToolPreview, ToolResult,
};
use super::state::SplineTool;
use crate::app::group_registry::{GroupBase, GroupKind, GroupRecord};
use crate::app::tool_contract::TangentSource;
use crate::app::ui_contract::TangentMenuData;
use crate::core::RoadMap;
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
            1 => "Naechsten Punkt klicken (mind. 2 Punkte)",
            _ => "Weitere Punkte klicken — Enter bestaetigt, Escape abbrechen",
        }
    }

    fn on_click(&mut self, pos: Vec2, road_map: &RoadMap, _ctrl: bool) -> ToolAction {
        let (anchor, _neighbors) = self.lifecycle.snap_with_neighbors(pos, road_map);

        if self.anchors.is_empty() {
            // Verkettung: letzten Endpunkt als Start verwenden
            if let Some(last_end) = self.lifecycle.chaining_start_anchor() {
                self.lifecycle.prepare_for_chaining();
                self.last_anchors.clear();
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

        let snapped_cursor = self.lifecycle.snap_at(cursor_pos, road_map).position();

        let positions = if self.anchors.len() == 1 {
            // Nur Start + Cursor → gerade Linie (Preview)
            let start = self.anchors[0].position();
            vec![start, snapped_cursor]
        } else {
            // Spline durch alle Anker + Cursor
            self.compute_resampled(Some(snapped_cursor))
        };

        let connections = linear_connections(positions.len());
        let styles = vec![(self.direction, self.priority); connections.len()];

        // Kontrollpunkte (Anker) als zusaetzliche Preview-Nodes (fuer visuelle Markierung)
        let mut nodes = positions;
        for anchor in &self.anchors {
            nodes.push(anchor.position());
        }
        nodes.push(snapped_cursor);

        ToolPreview {
            nodes,
            connections,
            connection_styles: styles,
            labels: vec![],
        }
    }

    fn render_config(&mut self, ui: &mut egui::Ui, distance_wheel_step_m: f32) -> bool {
        self.render_config_view(ui, distance_wheel_step_m)
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

    /// Setzt das Tool auf den Anfangszustand zurueck.
    ///
    /// Loescht Anker und Tangenten-Auswahl. `last_anchors`, `start_neighbors`,
    /// `end_neighbors` und `lifecycle.last_*` bleiben erhalten fuer
    /// Nachbearbeitung und Verkettung.
    fn reset(&mut self) {
        self.anchors.clear();
        self.tangents.reset_tangents();
    }

    fn is_ready(&self) -> bool {
        self.anchors.len() >= 2
    }

    fn has_pending_input(&self) -> bool {
        !self.anchors.is_empty()
    }

    crate::impl_lifecycle_delegation!();

    fn current_end_anchor(&self) -> Option<super::super::ToolAnchor> {
        self.anchors
            .last()
            .copied()
            .or_else(|| self.last_anchors.last().copied())
            .or(self.lifecycle.last_end_anchor)
    }

    fn save_anchors_for_recreate(&mut self, road_map: &RoadMap) {
        // Nur bei Erst-Erstellung Anker uebernehmen; bei Recreate bleiben last_anchors erhalten
        if !self.anchors.is_empty() {
            self.last_anchors = self.anchors.clone();
        }
        // Nachbarn aus den richtigen Ankern befuellen (anchors oder last_anchors)
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
    }

    fn execute_from_anchors(&self, road_map: &RoadMap) -> Option<ToolResult> {
        // Aktuelle Tangenten verwenden (nicht last_tangent_*),
        // damit Aenderungen im Nachbearbeitungs-Modus wirksam werden
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

    fn tangent_menu_data(&self) -> Option<TangentMenuData> {
        let adjusting = !self.lifecycle.last_created_ids.is_empty() && self.last_anchors.len() >= 2;
        if !adjusting {
            return None;
        }

        let has_start = !self.tangents.start_neighbors.is_empty();
        let has_end = !self.tangents.end_neighbors.is_empty();
        if !has_start && !has_end {
            return None;
        }

        Some(TangentMenuData {
            start_options: tangent_options(&self.tangents.start_neighbors),
            end_options: tangent_options(&self.tangents.end_neighbors),
            current_start: self.tangents.tangent_start,
            current_end: self.tangents.tangent_end,
        })
    }

    fn apply_tangent_selection(&mut self, start: TangentSource, end: TangentSource) {
        self.tangents.tangent_start = start;
        self.tangents.tangent_end = end;
        self.sync_derived();
        if self.lifecycle.has_last_created() {
            self.lifecycle.recreate_needed = true;
        }
    }

    fn make_group_record(&self, id: u64, node_ids: &[u64]) -> Option<GroupRecord> {
        if self.last_anchors.len() < 2 {
            return None;
        }
        let start = *self.last_anchors.first()?;
        let end = *self.last_anchors.last()?;
        Some(GroupRecord {
            id,
            tool_id: Some(RouteToolId::Spline),
            node_ids: node_ids.to_vec(),
            start_anchor: start,
            end_anchor: end,
            original_positions: Vec::new(), // wird im Handler befüllt
            marker_node_ids: Vec::new(),
            locked: true,
            entry_node_id: None,
            exit_node_id: None,
            kind: GroupKind::Spline {
                anchors: self.last_anchors.clone(),
                tangent_start: self.tangents.last_tangent_start,
                tangent_end: self.tangents.last_tangent_end,
                base: GroupBase {
                    direction: self.direction,
                    priority: self.priority,
                    max_segment_length: self.seg.max_segment_length,
                },
            },
        })
    }

    fn load_for_edit(&mut self, _record: &GroupRecord, kind: &GroupKind) {
        let GroupKind::Spline {
            anchors,
            tangent_start,
            tangent_end,
            base,
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
        self.direction = base.direction;
        self.priority = base.priority;
        self.seg.max_segment_length = base.max_segment_length;
        self.sync_derived();
    }
}
