//! Egui-freie Panel-Bruecke fuer das ParkingTool.

use super::state::{ParkingPhase, ParkingTool, RampSide};
use crate::app::ui_contract::{
    ParkingPanelAction, ParkingPanelState, ParkingRampSideChoice, RouteToolPanelEffect,
    PARKING_BAY_LENGTH_LIMITS, PARKING_ENTRY_EXIT_T_LIMITS, PARKING_MAX_NODE_DISTANCE_LIMITS,
    PARKING_NUM_ROWS_LIMITS, PARKING_RAMP_LENGTH_LIMITS, PARKING_ROTATION_STEP_LIMITS,
    PARKING_ROW_SPACING_LIMITS,
};

impl ParkingTool {
    /// Liefert den egui-freien Panelzustand des ParkingTools.
    pub(super) fn panel_state(&self) -> ParkingPanelState {
        ParkingPanelState {
            num_rows: self.config.num_rows,
            row_spacing: self.config.row_spacing,
            bay_length: self.config.bay_length,
            max_node_distance: self.config.max_node_distance,
            entry_t: self.config.entry_t,
            exit_t: self.config.exit_t,
            ramp_length: self.config.ramp_length,
            entry_side: match self.config.entry_side {
                RampSide::Left => ParkingRampSideChoice::Left,
                RampSide::Right => ParkingRampSideChoice::Right,
            },
            exit_side: match self.config.exit_side {
                RampSide::Left => ParkingRampSideChoice::Left,
                RampSide::Right => ParkingRampSideChoice::Right,
            },
            marker_group: self.config.marker_group.clone(),
            rotation_step_deg: self.rotation_step_deg,
            angle_deg: self.origin.map(|_| self.angle.to_degrees()),
            hint_text: self.origin.map(|_| match self.phase {
                ParkingPhase::Idle => "Alt+Mausrad zum Drehen".to_owned(),
                ParkingPhase::Configuring => {
                    "Position fixiert — Viewport-Klick zum Verschieben".to_owned()
                }
                ParkingPhase::Adjusting => {
                    "Klicken zum Fixieren — Alt+Mausrad zum Drehen".to_owned()
                }
            }),
        }
    }

    /// Wendet eine semantische Panel-Aktion auf das ParkingTool an.
    pub(super) fn apply_panel_action(
        &mut self,
        action: ParkingPanelAction,
    ) -> RouteToolPanelEffect {
        let changed = match action {
            ParkingPanelAction::SetNumRows(value) => set_usize(
                &mut self.config.num_rows,
                PARKING_NUM_ROWS_LIMITS.clamp(value),
            ),
            ParkingPanelAction::SetRowSpacing(value) => set_f32(
                &mut self.config.row_spacing,
                PARKING_ROW_SPACING_LIMITS.clamp(value),
            ),
            ParkingPanelAction::SetBayLength(value) => set_f32(
                &mut self.config.bay_length,
                PARKING_BAY_LENGTH_LIMITS.clamp(value),
            ),
            ParkingPanelAction::SetMaxNodeDistance(value) => set_f32(
                &mut self.config.max_node_distance,
                PARKING_MAX_NODE_DISTANCE_LIMITS.clamp(value),
            ),
            ParkingPanelAction::SetEntryT(value) => set_f32(
                &mut self.config.entry_t,
                PARKING_ENTRY_EXIT_T_LIMITS.clamp(value),
            ),
            ParkingPanelAction::SetExitT(value) => set_f32(
                &mut self.config.exit_t,
                PARKING_ENTRY_EXIT_T_LIMITS.clamp(value),
            ),
            ParkingPanelAction::SetRampLength(value) => set_f32(
                &mut self.config.ramp_length,
                PARKING_RAMP_LENGTH_LIMITS.clamp(value),
            ),
            ParkingPanelAction::SetEntrySide(value) => set_side(&mut self.config.entry_side, value),
            ParkingPanelAction::SetExitSide(value) => set_side(&mut self.config.exit_side, value),
            ParkingPanelAction::SetMarkerGroup(value) => {
                set_string(&mut self.config.marker_group, value)
            }
            ParkingPanelAction::SetRotationStepDeg(value) => set_f32(
                &mut self.rotation_step_deg,
                PARKING_ROTATION_STEP_LIMITS.clamp(value),
            ),
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

fn set_usize(target: &mut usize, value: usize) -> bool {
    if *target == value {
        false
    } else {
        *target = value;
        true
    }
}

fn set_string(target: &mut String, value: String) -> bool {
    if *target == value {
        false
    } else {
        *target = value;
        true
    }
}

fn set_side(target: &mut RampSide, value: ParkingRampSideChoice) -> bool {
    let mapped = match value {
        ParkingRampSideChoice::Left => RampSide::Left,
        ParkingRampSideChoice::Right => RampSide::Right,
    };
    if *target == mapped {
        false
    } else {
        *target = mapped;
        true
    }
}
