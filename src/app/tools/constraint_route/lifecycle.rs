//! Lifecycle-Methoden des ConstraintRouteTool (RouteTool-Implementierung).

use super::super::common::linear_connections;
use super::super::{snap_to_node, RouteTool, ToolAction, ToolPreview, ToolResult};
use super::geometry::build_result;
use super::state::{ConstraintRouteTool, Phase};
use crate::app::segment_registry::{SegmentKind, SegmentRecord};
use crate::core::RoadMap;
use glam::Vec2;

impl RouteTool for ConstraintRouteTool {
    fn name(&self) -> &str {
        "Constraint Route"
    }

    fn icon(&self) -> &str {
        "⊿"
    }

    fn description(&self) -> &str {
        "Erzeugt eine winkelgeglättete Route mit automatischen Tangenten-Übergängen"
    }

    fn status_text(&self) -> &str {
        match self.phase {
            Phase::Start => "Startpunkt klicken",
            Phase::End => "Endpunkt klicken",
            Phase::ControlNodes => {
                "Kontrollpunkte klicken (Enter bestätigt, Rechtsklick entfernt letzten)"
            }
        }
    }

    fn on_click(&mut self, pos: Vec2, road_map: &RoadMap, _ctrl: bool) -> ToolAction {
        let anchor = snap_to_node(pos, road_map, self.lifecycle.snap_radius);

        match self.phase {
            Phase::Start => {
                // Verkettung: letzten Endpunkt als Start verwenden
                if let Some(last_end) = self.lifecycle.chaining_start_anchor() {
                    self.lifecycle.prepare_for_chaining();
                    self.last_start_anchor = None;
                    self.last_end_anchor = None;
                    self.last_control_nodes.clear();
                    self.start = Some(last_end);
                    self.start_neighbor_dirs =
                        ConstraintRouteTool::collect_neighbor_dirs(&last_end, road_map);
                    self.end = Some(anchor);
                    self.end_neighbor_dirs =
                        ConstraintRouteTool::collect_neighbor_dirs(&anchor, road_map);
                    self.phase = Phase::ControlNodes;
                    self.sync_derived();
                    self.update_preview();
                    ToolAction::UpdatePreview
                } else {
                    self.start = Some(anchor);
                    self.start_neighbor_dirs =
                        ConstraintRouteTool::collect_neighbor_dirs(&anchor, road_map);
                    self.phase = Phase::End;
                    ToolAction::Continue
                }
            }
            Phase::End => {
                self.end = Some(anchor);
                self.end_neighbor_dirs =
                    ConstraintRouteTool::collect_neighbor_dirs(&anchor, road_map);
                self.phase = Phase::ControlNodes;
                self.sync_derived();
                self.update_preview();
                ToolAction::UpdatePreview
            }
            Phase::ControlNodes => {
                // Neuen Kontrollpunkt hinzufügen
                self.control_nodes.push(pos);
                self.sync_derived();
                self.update_preview();
                ToolAction::UpdatePreview
            }
        }
    }

    fn preview(&self, cursor_pos: Vec2, road_map: &RoadMap) -> ToolPreview {
        match self.phase {
            Phase::Start => ToolPreview::default(),
            Phase::End => {
                // Linie vom Start zur aktuellen Mausposition
                let start_pos = match &self.start {
                    Some(a) => a.position(),
                    None => return ToolPreview::default(),
                };
                let snapped = snap_to_node(cursor_pos, road_map, self.lifecycle.snap_radius);
                let end_pos = snapped.position();
                let nodes = vec![start_pos, end_pos];
                let connections = linear_connections(nodes.len());
                ToolPreview { nodes, connections }
            }
            Phase::ControlNodes => {
                // Gecachte Solver-Ausgabe verwenden
                if self.preview_positions.is_empty() {
                    return ToolPreview::default();
                }
                let connections = linear_connections(self.preview_positions.len());
                ToolPreview {
                    nodes: self.preview_positions.clone(),
                    connections,
                }
            }
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
            &self.control_nodes,
            self.seg.max_segment_length,
            self.max_angle_deg,
            &self.start_neighbor_dirs,
            &self.end_neighbor_dirs,
            self.direction,
            self.priority,
            road_map,
        )
    }

    /// Setzt das Tool auf den Anfangszustand zurück.
    ///
    /// Löscht Start/End/Kontrollpunkte. `lifecycle.last_created_ids` und `last_*_anchor`
    /// bleiben erhalten für Verkettung und Nachbearbeitung.
    fn reset(&mut self) {
        self.start = None;
        self.end = None;
        self.control_nodes.clear();
        self.phase = Phase::Start;
        self.dragging = None;
        self.preview_positions.clear();
        self.start_neighbor_dirs.clear();
        self.end_neighbor_dirs.clear();
    }

    fn is_ready(&self) -> bool {
        self.start.is_some() && self.end.is_some()
    }

    fn has_pending_input(&self) -> bool {
        self.phase != Phase::Start
    }

    crate::impl_lifecycle_delegation!();

    fn set_last_created(&mut self, ids: &[u64], _road_map: &RoadMap) {
        if self.start.is_some() {
            self.last_start_anchor = self.start;
        }
        if self.end.is_some() {
            self.last_end_anchor = self.end;
        }
        if !self.control_nodes.is_empty() {
            self.last_control_nodes = self.control_nodes.clone();
        }
        self.lifecycle.save_created_ids(ids);
    }

    fn execute_from_anchors(&self, road_map: &RoadMap) -> Option<ToolResult> {
        let start = self.last_start_anchor?;
        let end = self.last_end_anchor.or(self.lifecycle.last_end_anchor)?;
        build_result(
            start,
            end,
            &self.last_control_nodes,
            self.seg.max_segment_length,
            self.max_angle_deg,
            &self.start_neighbor_dirs,
            &self.end_neighbor_dirs,
            self.direction,
            self.priority,
            road_map,
        )
    }

    fn make_segment_record(&self, id: u64, node_ids: &[u64]) -> Option<SegmentRecord> {
        let start = self.last_start_anchor?;
        let end = self.last_end_anchor.or(self.lifecycle.last_end_anchor)?;
        Some(SegmentRecord {
            id,
            node_ids: node_ids.to_vec(),
            start_anchor: start,
            end_anchor: end,
            kind: SegmentKind::ConstraintRoute {
                control_nodes: self.last_control_nodes.clone(),
                max_angle_deg: self.max_angle_deg,
                direction: self.direction,
                priority: self.priority,
                max_segment_length: self.seg.max_segment_length,
            },
        })
    }

    fn load_for_edit(&mut self, record: &SegmentRecord, kind: &SegmentKind) {
        let SegmentKind::ConstraintRoute {
            control_nodes,
            max_angle_deg,
            direction,
            priority,
            max_segment_length,
        } = kind
        else {
            return;
        };
        self.start = Some(record.start_anchor);
        self.end = Some(record.end_anchor);
        self.control_nodes = control_nodes.clone();
        self.max_angle_deg = *max_angle_deg;
        self.direction = *direction;
        self.priority = *priority;
        self.seg.max_segment_length = *max_segment_length;
        self.phase = Phase::ControlNodes;
        self.sync_derived();
        self.update_preview();
    }

    fn drag_targets(&self) -> Vec<Vec2> {
        self.get_drag_targets()
    }

    fn on_drag_start(&mut self, pos: Vec2, road_map: &RoadMap, pick_radius: f32) -> bool {
        self.handle_drag_start(pos, road_map, pick_radius)
    }

    fn on_drag_update(&mut self, pos: Vec2) {
        self.handle_drag_update(pos);
    }

    fn on_drag_end(&mut self, road_map: &RoadMap) {
        self.handle_drag_end(road_map);
    }
}
