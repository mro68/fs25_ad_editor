//! Egui-freie Panel-Bruecke fuer das Strecken-Versatz-Tool.

use super::state::RouteOffsetTool;
use crate::app::ui_contract::{
    RouteOffsetPanelAction, RouteOffsetPanelState, RouteToolPanelEffect,
    ROUTE_OFFSET_BASE_SPACING_LIMITS, ROUTE_OFFSET_DISTANCE_LIMITS,
};

impl RouteOffsetTool {
    /// Liefert den egui-freien Panelzustand des Strecken-Versatz-Tools.
    pub(super) fn panel_state(&self) -> RouteOffsetPanelState {
        RouteOffsetPanelState {
            has_chain: self.has_chain(),
            empty_message: (!self.has_chain())
                .then_some("Kette selektieren und Route-Tool neu aktivieren.".to_owned()),
            left_enabled: self.config.left_enabled,
            left_distance: self.config.left_distance,
            right_enabled: self.config.right_enabled,
            right_distance: self.config.right_distance,
            base_spacing: self.config.base_spacing,
            keep_original: self.config.keep_original,
            chain_node_count: self.chain_positions.len(),
        }
    }

    /// Wendet eine semantische Panel-Aktion auf das Strecken-Versatz-Tool an.
    pub(super) fn apply_panel_action(
        &mut self,
        action: RouteOffsetPanelAction,
    ) -> RouteToolPanelEffect {
        let changed = match action {
            RouteOffsetPanelAction::SetLeftEnabled(value) => {
                set_bool(&mut self.config.left_enabled, value)
            }
            RouteOffsetPanelAction::SetLeftDistance(value) => set_f32(
                &mut self.config.left_distance,
                ROUTE_OFFSET_DISTANCE_LIMITS.clamp(value),
            ),
            RouteOffsetPanelAction::SetRightEnabled(value) => {
                set_bool(&mut self.config.right_enabled, value)
            }
            RouteOffsetPanelAction::SetRightDistance(value) => set_f32(
                &mut self.config.right_distance,
                ROUTE_OFFSET_DISTANCE_LIMITS.clamp(value),
            ),
            RouteOffsetPanelAction::SetBaseSpacing(value) => set_f32(
                &mut self.config.base_spacing,
                ROUTE_OFFSET_BASE_SPACING_LIMITS.clamp(value),
            ),
            RouteOffsetPanelAction::SetKeepOriginal(value) => {
                set_bool(&mut self.config.keep_original, value)
            }
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
