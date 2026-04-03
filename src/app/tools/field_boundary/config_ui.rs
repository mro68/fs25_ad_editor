//! Egui-freie Panel-Bruecke fuer das FieldBoundaryTool.

use super::state::{FieldBoundaryPhase, FieldBoundaryTool};
use crate::app::ui_contract::{
    FieldBoundaryPanelAction, FieldBoundaryPanelState, RouteToolPanelEffect,
};
use crate::core::{ConnectionDirection, ConnectionPriority};

impl FieldBoundaryTool {
    /// Liefert den egui-freien Panelzustand des FieldBoundaryTools.
    pub(super) fn panel_state(&self) -> FieldBoundaryPanelState {
        FieldBoundaryPanelState {
            selected_field_id: self.selected_polygon.as_ref().map(|polygon| polygon.id),
            empty_selection_text: self
                .selected_polygon
                .is_none()
                .then_some("Kein Feld ausgewaehlt — in ein Feld klicken".to_owned()),
            node_spacing: self.node_spacing,
            offset: self.offset,
            straighten_tolerance: self.straighten_tolerance,
            corner_detection_enabled: self.corner_detection_enabled,
            corner_angle_threshold_deg: self.corner_angle_threshold_deg,
            corner_rounding_enabled: self.corner_rounding_enabled,
            corner_rounding_radius: self.corner_rounding_radius,
            corner_rounding_max_angle_deg: self.corner_rounding_max_angle_deg,
            direction: self.direction,
            priority: self.priority,
            hint_text: (self.phase == FieldBoundaryPhase::Configuring)
                .then_some("Erneuter Klick im Viewport → anderes Feld auswählen".to_owned()),
        }
    }

    /// Wendet eine semantische Panel-Aktion auf das FieldBoundaryTool an.
    pub(super) fn apply_panel_action(
        &mut self,
        action: FieldBoundaryPanelAction,
    ) -> RouteToolPanelEffect {
        let changed = match action {
            FieldBoundaryPanelAction::SetNodeSpacing(value) => {
                set_f32(&mut self.node_spacing, value.clamp(1.0, 50.0))
            }
            FieldBoundaryPanelAction::SetOffset(value) => {
                set_f32(&mut self.offset, value.clamp(-20.0, 20.0))
            }
            FieldBoundaryPanelAction::SetStraightenTolerance(value) => {
                set_f32(&mut self.straighten_tolerance, value.clamp(0.0, 10.0))
            }
            FieldBoundaryPanelAction::SetCornerDetectionEnabled(value) => {
                set_bool(&mut self.corner_detection_enabled, value)
            }
            FieldBoundaryPanelAction::SetCornerAngleThresholdDeg(value) => set_f32(
                &mut self.corner_angle_threshold_deg,
                value.clamp(10.0, 170.0),
            ),
            FieldBoundaryPanelAction::SetCornerRoundingEnabled(value) => {
                set_bool(&mut self.corner_rounding_enabled, value)
            }
            FieldBoundaryPanelAction::SetCornerRoundingRadius(value) => {
                set_f32(&mut self.corner_rounding_radius, value.clamp(1.0, 50.0))
            }
            FieldBoundaryPanelAction::SetCornerRoundingMaxAngleDeg(value) => set_f32(
                &mut self.corner_rounding_max_angle_deg,
                value.clamp(1.0, 45.0),
            ),
            FieldBoundaryPanelAction::SetDirection(value) => {
                set_direction(&mut self.direction, value)
            }
            FieldBoundaryPanelAction::SetPriority(value) => set_priority(&mut self.priority, value),
        };

        RouteToolPanelEffect {
            changed,
            needs_recreate: false,
            next_action: None,
        }
    }
}

fn set_f32(target: &mut f32, value: f32) -> bool {
    if (*target - value).abs() < f32::EPSILON {
        false
    } else {
        *target = value;
        true
    }
}

fn set_bool(target: &mut bool, value: bool) -> bool {
    if *target == value {
        false
    } else {
        *target = value;
        true
    }
}

fn set_direction(target: &mut ConnectionDirection, value: ConnectionDirection) -> bool {
    if *target == value {
        false
    } else {
        *target = value;
        true
    }
}

fn set_priority(target: &mut ConnectionPriority, value: ConnectionPriority) -> bool {
    if *target == value {
        false
    } else {
        *target = value;
        true
    }
}
