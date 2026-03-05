//! RouteTool-Implementierung fuer das ParkingTool.

use crate::app::segment_registry::{SegmentBase, SegmentKind, SegmentRecord};
use crate::app::tools::{ToolAction, ToolAnchor, ToolPreview, ToolResult};
use crate::core::{ConnectionDirection, ConnectionPriority, RoadMap};
use glam::Vec2;

use super::geometry;
use super::state::{ParkingPhase, ParkingTool};

impl crate::app::tools::RouteTool for ParkingTool {
    fn name(&self) -> &str {
        "Parkplatz"
    }

    fn icon(&self) -> &str {
        "\u{1f17f}" // 🅿
    }

    fn description(&self) -> &str {
        "Erzeugt ein Parkplatz-Layout mit Wendekreis"
    }

    fn status_text(&self) -> &str {
        match self.phase {
            ParkingPhase::Idle => "Klicken zum Platzieren — Alt+Mausrad zum Drehen",
            ParkingPhase::Configuring => "Layout konfigurieren — Bestätigen oder Abbrechen",
            ParkingPhase::Adjusting => "Klicken zum Fixieren — Alt+Mausrad zum Drehen",
        }
    }

    fn on_click(&mut self, pos: Vec2, _road_map: &RoadMap, _ctrl: bool) -> ToolAction {
        match self.phase {
            ParkingPhase::Idle => {
                self.origin = Some(pos);
                self.phase = ParkingPhase::Configuring;
                ToolAction::Continue
            }
            ParkingPhase::Configuring => {
                // Viewport-Klick waehrend Config → Repositionierung starten
                self.phase = ParkingPhase::Adjusting;
                ToolAction::Continue
            }
            ParkingPhase::Adjusting => {
                // Erneuter Klick → Position fixieren, zurueck zu Configuring
                self.origin = Some(pos);
                self.phase = ParkingPhase::Configuring;
                ToolAction::Continue
            }
        }
    }

    fn on_scroll_rotate(&mut self, delta: f32) {
        // Nur in Idle oder Adjusting rotierbar (in Configuring ist alles fixiert)
        if matches!(self.phase, ParkingPhase::Idle | ParkingPhase::Adjusting) {
            let step = std::f32::consts::PI / 36.0; // 5° pro Scroll-Schritt
            if delta > 0.0 {
                self.angle += step;
            } else {
                self.angle -= step;
            }
        }
    }

    fn preview(&self, cursor_pos: Vec2, _road_map: &RoadMap) -> ToolPreview {
        let (origin, angle) = match self.phase {
            ParkingPhase::Idle => (cursor_pos, self.angle),
            ParkingPhase::Configuring => {
                let origin = self.origin.unwrap_or(cursor_pos);
                (origin, self.angle)
            }
            ParkingPhase::Adjusting => (cursor_pos, self.angle),
        };

        let layout = geometry::generate_parking_layout(
            origin,
            angle,
            &self.config,
            self.direction,
            self.priority,
        );
        geometry::build_preview(&layout)
    }

    fn render_config(&mut self, ui: &mut egui::Ui, _distance_wheel_step_m: f32) -> bool {
        self.render_config_view(ui, _distance_wheel_step_m)
    }

    fn execute(&self, _road_map: &RoadMap) -> Option<ToolResult> {
        // Nur in Configuring-Phase ausfuehrbar (verhindert Execute mit veralteter Position)
        if self.phase != ParkingPhase::Configuring {
            return None;
        }
        let origin = self.origin?;
        let layout = geometry::generate_parking_layout(
            origin,
            self.angle,
            &self.config,
            self.direction,
            self.priority,
        );
        Some(geometry::build_parking_result(layout))
    }

    fn reset(&mut self) {
        self.phase = ParkingPhase::Idle;
        self.origin = None;
        self.angle = 0.0;
    }

    fn is_ready(&self) -> bool {
        self.phase == ParkingPhase::Configuring && self.origin.is_some()
    }

    fn has_pending_input(&self) -> bool {
        matches!(
            self.phase,
            ParkingPhase::Configuring | ParkingPhase::Adjusting
        )
    }

    // ── Lifecycle-Delegation (manuell, da kein SegmentConfig) ────

    fn set_direction(&mut self, dir: ConnectionDirection) {
        self.direction = dir;
    }

    fn set_priority(&mut self, prio: ConnectionPriority) {
        self.priority = prio;
    }

    fn set_snap_radius(&mut self, radius: f32) {
        self.lifecycle.snap_radius = radius;
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

    fn set_last_created(&mut self, ids: &[u64], _road_map: &RoadMap) {
        self.lifecycle.save_created_ids(ids);
    }

    /// Erstellt einen `SegmentRecord` fuer die Registry aus dem aktuellen Tool-Zustand.
    fn make_segment_record(&self, id: u64, node_ids: &[u64]) -> Option<SegmentRecord> {
        let origin = self.origin?;
        let angle = self.angle;
        Some(SegmentRecord {
            id,
            node_ids: node_ids.to_vec(),
            start_anchor: ToolAnchor::NewPosition(origin),
            end_anchor: ToolAnchor::NewPosition(origin),
            original_positions: Vec::new(), // wird im Handler befuellt
            marker_node_ids: Vec::new(),    // wird im Handler befuellt
            kind: SegmentKind::Parking {
                origin,
                angle,
                config: self.config.clone(),
                base: SegmentBase {
                    direction: self.direction,
                    priority: self.priority,
                    max_segment_length: 0.0,
                },
            },
        })
    }

    /// Laedt einen gespeicherten `SegmentRecord` zur nachtraeglichen Bearbeitung.
    fn load_for_edit(&mut self, _record: &SegmentRecord, kind: &SegmentKind) {
        let SegmentKind::Parking {
            origin,
            angle,
            config,
            base,
        } = kind
        else {
            return;
        };
        self.origin = Some(*origin);
        self.angle = *angle;
        self.phase = ParkingPhase::Configuring;
        self.config = config.clone();
        self.direction = base.direction;
        self.priority = base.priority;
    }
}
