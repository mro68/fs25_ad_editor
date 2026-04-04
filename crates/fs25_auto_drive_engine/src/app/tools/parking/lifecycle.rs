//! RouteTool-Implementierung fuer das ParkingTool.

use crate::app::tool_editing::{RouteToolEditPayload, ToolRouteBase};
use crate::app::tools::common::sync_tool_host;
use crate::app::tools::{
    RouteTool, RouteToolCore, RouteToolGroupEdit, RouteToolHostSync, RouteToolPanelBridge,
    RouteToolRotate, ToolAction, ToolHostContext, ToolPreview, ToolResult,
};
use crate::app::ui_contract::{RouteToolConfigState, RouteToolPanelAction, RouteToolPanelEffect};
use crate::core::RoadMap;
use glam::Vec2;

use super::geometry;
use super::state::{ParkingPhase, ParkingTool};

impl RouteToolPanelBridge for ParkingTool {
    fn status_text(&self) -> &str {
        match self.phase {
            ParkingPhase::Idle => "Klicken zum Platzieren — Alt+Mausrad zum Drehen",
            ParkingPhase::Configuring => "Layout konfigurieren — Bestaetigen oder Abbrechen",
            ParkingPhase::Adjusting => "Klicken zum Fixieren — Alt+Mausrad zum Drehen",
        }
    }

    fn panel_state(&self) -> RouteToolConfigState {
        RouteToolConfigState::Parking(self.panel_state())
    }

    fn apply_panel_action(&mut self, action: RouteToolPanelAction) -> RouteToolPanelEffect {
        let RouteToolPanelAction::Parking(action) = action else {
            return RouteToolPanelEffect::default();
        };

        self.apply_panel_action(action)
    }
}

impl RouteToolCore for ParkingTool {
    fn on_click(&mut self, pos: Vec2, _road_map: &RoadMap, _ctrl: bool) -> ToolAction {
        match self.phase {
            ParkingPhase::Idle => {
                self.origin = Some(pos);
                self.phase = ParkingPhase::Configuring;
                ToolAction::Continue
            }
            ParkingPhase::Configuring => {
                self.phase = ParkingPhase::Adjusting;
                ToolAction::Continue
            }
            ParkingPhase::Adjusting => {
                self.origin = Some(pos);
                self.phase = ParkingPhase::Configuring;
                ToolAction::Continue
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

    fn execute(&self, _road_map: &RoadMap) -> Option<ToolResult> {
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
}

impl RouteToolHostSync for ParkingTool {
    fn sync_host(&mut self, context: &ToolHostContext) {
        sync_tool_host(
            &mut self.direction,
            &mut self.priority,
            &mut self.lifecycle,
            context,
        );
    }
}

impl RouteToolRotate for ParkingTool {
    fn on_scroll_rotate(&mut self, delta: f32) {
        if matches!(self.phase, ParkingPhase::Idle | ParkingPhase::Adjusting) {
            let step = self.rotation_step_deg.to_radians();
            if delta > 0.0 {
                self.angle += step;
            } else {
                self.angle -= step;
            }
        }
    }
}

impl RouteTool for ParkingTool {
    fn as_rotate(&self) -> Option<&dyn RouteToolRotate> {
        Some(self)
    }

    fn as_rotate_mut(&mut self) -> Option<&mut dyn RouteToolRotate> {
        Some(self)
    }

    fn as_group_edit(&self) -> Option<&dyn RouteToolGroupEdit> {
        Some(self)
    }

    fn as_group_edit_mut(&mut self) -> Option<&mut dyn RouteToolGroupEdit> {
        Some(self)
    }
}

impl RouteToolGroupEdit for ParkingTool {
    fn build_edit_payload(&self) -> Option<RouteToolEditPayload> {
        let origin = self.origin?;
        let angle = self.angle;
        Some(RouteToolEditPayload::Parking {
            origin,
            angle,
            config: self.config.clone(),
            base: ToolRouteBase {
                direction: self.direction,
                priority: self.priority,
                max_segment_length: 0.0,
            },
        })
    }

    fn restore_edit_payload(&mut self, payload: &RouteToolEditPayload) {
        let RouteToolEditPayload::Parking {
            origin,
            angle,
            config,
            base,
        } = payload
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
