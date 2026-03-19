//! Lifecycle-Methoden des SmoothCurveTool (RouteTool-Implementierung).

use super::super::common::linear_connections;
use super::super::{RouteTool, ToolAction, ToolPreview, ToolResult};
use super::geometry::{build_result, BuildResultParams};
use super::state::{Phase, SmoothCurveTool};
use crate::app::segment_registry::{SegmentBase, SegmentKind, SegmentRecord};
use crate::core::RoadMap;
use glam::Vec2;

impl RouteTool for SmoothCurveTool {
    fn name(&self) -> &str {
        "Geglättete Kurve"
    }

    fn icon(&self) -> &str {
        "⊿"
    }

    fn description(&self) -> &str {
        "Erzeugt eine winkelgeglaettete Route mit automatischen Tangenten-Uebergaengen"
    }

    fn status_text(&self) -> &str {
        match self.phase {
            Phase::Start => "Startpunkt klicken",
            Phase::End => "Endpunkt klicken",
            Phase::ControlNodes => {
                "Kontrollpunkte klicken (Enter bestaetigt, Rechtsklick entfernt letzten)"
            }
        }
    }

    fn on_click(&mut self, pos: Vec2, road_map: &RoadMap, _ctrl: bool) -> ToolAction {
        let (anchor, _neighbors) = self.lifecycle.snap_with_neighbors(pos, road_map);

        match self.phase {
            Phase::Start => {
                // Verkettung: letzten Endpunkt als Start verwenden
                if let Some(last_end) = self.lifecycle.chaining_start_anchor() {
                    self.lifecycle.prepare_for_chaining();
                    self.last_start_anchor = None;
                    self.last_control_nodes.clear();
                    self.start = Some(last_end);
                    self.start_neighbor_dirs =
                        SmoothCurveTool::collect_neighbor_dirs(&last_end, road_map);
                    self.end = Some(anchor);
                    self.end_neighbor_dirs =
                        SmoothCurveTool::collect_neighbor_dirs(&anchor, road_map);
                    self.phase = Phase::ControlNodes;
                    self.sync_derived();
                    self.update_preview();
                    ToolAction::UpdatePreview
                } else {
                    self.start = Some(anchor);
                    self.start_neighbor_dirs =
                        SmoothCurveTool::collect_neighbor_dirs(&anchor, road_map);
                    self.phase = Phase::End;
                    ToolAction::Continue
                }
            }
            Phase::End => {
                self.end = Some(anchor);
                self.end_neighbor_dirs = SmoothCurveTool::collect_neighbor_dirs(&anchor, road_map);
                self.phase = Phase::ControlNodes;
                self.sync_derived();
                self.update_preview();
                ToolAction::UpdatePreview
            }
            Phase::ControlNodes => {
                // Neuen Kontrollpunkt hinzufuegen
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
                let snapped = self.lifecycle.snap_at(cursor_pos, road_map);
                let end_pos = snapped.position();
                let nodes = vec![start_pos, end_pos];
                let connections = linear_connections(nodes.len());
                let styles = vec![(self.direction, self.priority); connections.len()];
                ToolPreview {
                    nodes,
                    connections,
                    connection_styles: styles,
                    labels: vec![],
                }
            }
            Phase::ControlNodes => {
                // Gecachte Solver-Ausgabe verwenden
                if self.preview_positions.is_empty() {
                    return ToolPreview::default();
                }
                // Cow::Borrowed vermeidet Clone bei unveraendertem Cache-Hit
                let connections: Vec<(usize, usize)> = if self.preview_connections.is_empty() {
                    linear_connections(self.preview_positions.len())
                } else {
                    self.preview_connections.clone()
                };
                let styles = vec![(self.direction, self.priority); connections.len()];
                // Positionen als Basis; Steuerpunkte als unverbundene Rauten-Nodes
                let mut nodes = self.preview_positions.clone();

                // Steuerpunkte als unverbundene Nodes hinzufuegen (werden als Rauten gerendert)
                if let Some(ap) = self.approach_steerer {
                    nodes.push(ap);
                }
                if let Some(dp) = self.departure_steerer {
                    nodes.push(dp);
                }
                // Kontrollpunkte als unverbundene Nodes hinzufuegen
                for &cp in &self.control_nodes {
                    nodes.push(cp);
                }

                ToolPreview {
                    nodes,
                    connections,
                    connection_styles: styles,
                    labels: vec![],
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

        // Kontrollpunkte inkl. manuell verschobener Steuerpunkte zusammenbauen
        let mut solver_control = Vec::new();
        if self.approach_manual {
            if let Some(ap) = self.approach_steerer {
                solver_control.push(ap);
            }
        }
        solver_control.extend_from_slice(&self.control_nodes);
        if self.departure_manual {
            if let Some(dp) = self.departure_steerer {
                solver_control.push(dp);
            }
        }

        build_result(
            &BuildResultParams {
                start,
                end,
                control_nodes: &solver_control,
                max_segment_length: self.seg.max_segment_length,
                max_angle_deg: self.max_angle_deg,
                start_neighbor_dirs: if self.approach_manual {
                    &[]
                } else {
                    &self.start_neighbor_dirs
                },
                end_neighbor_dirs: if self.departure_manual {
                    &[]
                } else {
                    &self.end_neighbor_dirs
                },
                min_distance: self.min_distance,
                direction: self.direction,
                priority: self.priority,
            },
            road_map,
        )
    }

    /// Setzt das Tool auf den Anfangszustand zurueck.
    ///
    /// Loescht Start/End/Kontrollpunkte. `lifecycle.last_created_ids` und `last_*_anchor`
    /// bleiben erhalten fuer Verkettung und Nachbearbeitung.
    fn reset(&mut self) {
        self.start = None;
        self.end = None;
        self.control_nodes.clear();
        self.phase = Phase::Start;
        self.dragging = None;
        self.preview_positions.clear();
        self.preview_connections.clear();
        self.start_neighbor_dirs.clear();
        self.end_neighbor_dirs.clear();
        self.approach_steerer = None;
        self.departure_steerer = None;
        self.approach_manual = false;
        self.departure_manual = false;
    }

    fn is_ready(&self) -> bool {
        self.start.is_some() && self.end.is_some()
    }

    fn has_pending_input(&self) -> bool {
        self.phase != Phase::Start
    }

    crate::impl_lifecycle_delegation!();

    fn current_end_anchor(&self) -> Option<super::super::ToolAnchor> {
        self.end.or(self.lifecycle.last_end_anchor)
    }

    fn save_anchors_for_recreate(&mut self, _road_map: &RoadMap) {
        if self.start.is_some() {
            self.last_start_anchor = self.start;
        }
        if !self.control_nodes.is_empty() {
            self.last_control_nodes = self.control_nodes.clone();
        }
    }

    fn execute_from_anchors(&self, road_map: &RoadMap) -> Option<ToolResult> {
        let start = self.last_start_anchor?;
        let end = self.lifecycle.last_end_anchor?;
        build_result(
            &BuildResultParams {
                start,
                end,
                control_nodes: &self.last_control_nodes,
                max_segment_length: self.seg.max_segment_length,
                max_angle_deg: self.max_angle_deg,
                start_neighbor_dirs: &self.start_neighbor_dirs,
                end_neighbor_dirs: &self.end_neighbor_dirs,
                min_distance: self.min_distance,
                direction: self.direction,
                priority: self.priority,
            },
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
            original_positions: Vec::new(), // wird im Handler befüllt
            marker_node_ids: Vec::new(),
            locked: true,
            kind: SegmentKind::SmoothCurve {
                control_nodes: self.last_control_nodes.clone(),
                max_angle_deg: self.max_angle_deg,
                min_distance: self.min_distance,
                base: SegmentBase {
                    direction: self.direction,
                    priority: self.priority,
                    max_segment_length: self.seg.max_segment_length,
                },
            },
        })
    }

    fn load_for_edit(&mut self, record: &SegmentRecord, kind: &SegmentKind) {
        let SegmentKind::SmoothCurve {
            control_nodes,
            max_angle_deg,
            min_distance,
            base,
        } = kind
        else {
            return;
        };
        self.start = Some(record.start_anchor);
        self.end = Some(record.end_anchor);
        self.control_nodes = control_nodes.clone();
        self.max_angle_deg = *max_angle_deg;
        self.direction = base.direction;
        self.priority = base.priority;
        self.seg.max_segment_length = base.max_segment_length;
        self.min_distance = *min_distance;
        self.phase = Phase::ControlNodes;
        self.sync_derived();
        self.update_preview();
    }

    fn drag_targets(&self) -> Vec<Vec2> {
        super::drag::drag_targets(self)
    }

    fn on_drag_start(&mut self, pos: Vec2, road_map: &RoadMap, pick_radius: f32) -> bool {
        super::drag::on_drag_start(self, pos, road_map, pick_radius)
    }

    fn on_drag_update(&mut self, pos: Vec2) {
        super::drag::on_drag_update(self, pos);
    }

    fn on_drag_end(&mut self, road_map: &RoadMap) {
        super::drag::on_drag_end(self, road_map);
    }
}
