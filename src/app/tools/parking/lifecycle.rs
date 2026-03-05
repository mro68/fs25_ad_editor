//! RouteTool-Implementierung fuer das ParkingTool.

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
            ParkingPhase::Idle => "Ursprung klicken (Mitte der Buchten-Enden)",
            ParkingPhase::Placed => "Maus bewegen zum Drehen — Klick zum Bestaetigen",
        }
    }

    fn on_click(&mut self, pos: Vec2, _road_map: &RoadMap, _ctrl: bool) -> ToolAction {
        match self.phase {
            ParkingPhase::Idle => {
                self.origin = Some(pos);
                self.angle = 0.0;
                self.phase = ParkingPhase::Placed;
                ToolAction::Continue
            }
            ParkingPhase::Placed => {
                // Winkel aus letzter Mausposition einfrieren
                if let Some(origin) = self.origin {
                    let delta = pos - origin;
                    if delta.length() > 0.5 {
                        self.frozen_angle = Some(delta.y.atan2(delta.x));
                    } else {
                        self.frozen_angle = Some(self.angle);
                    }
                }
                ToolAction::ReadyToExecute
            }
        }
    }

    fn preview(&self, cursor_pos: Vec2, _road_map: &RoadMap) -> ToolPreview {
        let (origin, angle) = match self.phase {
            ParkingPhase::Idle => (cursor_pos, 0.0),
            ParkingPhase::Placed => {
                let origin = self.origin.unwrap_or(cursor_pos);
                let delta = cursor_pos - origin;
                let a = if delta.length() > 0.5 {
                    delta.y.atan2(delta.x)
                } else {
                    self.angle
                };
                (origin, a)
            }
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
        let origin = self.origin?;
        let angle = self.frozen_angle.unwrap_or(self.angle);
        let layout = geometry::generate_parking_layout(
            origin,
            angle,
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
        self.frozen_angle = None;
    }

    fn is_ready(&self) -> bool {
        self.phase == ParkingPhase::Placed && self.frozen_angle.is_some()
    }

    fn has_pending_input(&self) -> bool {
        self.phase == ParkingPhase::Placed
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
}
