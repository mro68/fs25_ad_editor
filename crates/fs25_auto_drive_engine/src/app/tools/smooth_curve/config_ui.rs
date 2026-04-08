//! Egui-freie Panel-Bruecke fuer das Geglaettete-Kurve-Tool.

use super::super::common::SegmentConfig;
use super::state::SmoothCurveTool;
use crate::app::ui_contract::{
    RouteToolPanelEffect, SegmentConfigPanelAction, SegmentLengthKind, SmoothCurvePanelAction,
    SmoothCurvePanelState, SmoothCurveSteererState, SMOOTH_CURVE_MAX_ANGLE_LIMITS,
    SMOOTH_CURVE_MIN_DISTANCE_LIMITS,
};

impl SmoothCurveTool {
    /// Wendet nur die Distanz-Aktion auf den Segmentzustand an.
    fn apply_segment_distance_only(
        seg: &mut SegmentConfig,
        value: f32,
        adjusting: bool,
        ready: bool,
        length: f32,
    ) -> RouteToolPanelEffect {
        let result = seg.apply_panel_action(
            SegmentConfigPanelAction::SetMaxSegmentLength(value),
            adjusting,
            ready,
            length,
            false,
        );
        RouteToolPanelEffect {
            changed: result.changed,
            needs_recreate: result.recreate,
            next_action: None,
        }
    }
    /// Liefert den egui-freien Panelzustand des Geglaettete-Kurve-Tools.
    pub(super) fn panel_state(&self) -> SmoothCurvePanelState {
        let adjusting = !self.lifecycle.last_created_ids.is_empty()
            && self.last_start_anchor.is_some()
            && self.lifecycle.last_end_anchor.is_some();

        let length = if adjusting {
            let start = self.last_start_anchor.unwrap().position();
            let end = self.lifecycle.last_end_anchor.unwrap().position();
            start.distance(end)
        } else {
            self.total_distance()
        };

        SmoothCurvePanelState {
            max_angle_deg: self.max_angle_deg,
            segment: self.seg.panel_state(
                adjusting,
                self.start.is_some() && self.end.is_some(),
                length,
                SegmentLengthKind::SmoothRoute,
                false,
            ),
            min_distance: self.min_distance,
            approach_steerer: self
                .approach_steerer
                .map(|position| SmoothCurveSteererState {
                    position,
                    is_manual: self.approach_manual,
                }),
            departure_steerer: self
                .departure_steerer
                .map(|position| SmoothCurveSteererState {
                    position,
                    is_manual: self.departure_manual,
                }),
            control_nodes: self.control_nodes.clone(),
            preview_node_count: (!self.preview_positions.is_empty())
                .then_some(self.preview_positions.len()),
        }
    }

    /// Wendet eine semantische Panel-Aktion auf das Geglaettete-Kurve-Tool an.
    pub(super) fn apply_panel_action(
        &mut self,
        action: SmoothCurvePanelAction,
    ) -> RouteToolPanelEffect {
        match action {
            SmoothCurvePanelAction::SetMaxAngleDeg(value) => self.apply_angle_action(value),
            SmoothCurvePanelAction::SetMaxSegmentLength(value) => self.apply_segment_action(value),
            SmoothCurvePanelAction::SetMinDistance(value) => self.apply_min_distance_action(value),
            SmoothCurvePanelAction::ResetApproachSteerer => self.reset_approach_steerer(),
            SmoothCurvePanelAction::ResetDepartureSteerer => self.reset_departure_steerer(),
            SmoothCurvePanelAction::RemoveControlNode { index } => self.remove_control_node(index),
        }
    }

    fn apply_angle_action(&mut self, value: f32) -> RouteToolPanelEffect {
        let clamped = SMOOTH_CURVE_MAX_ANGLE_LIMITS.clamp(value);
        if (self.max_angle_deg - clamped).abs() < f32::EPSILON {
            return RouteToolPanelEffect::default();
        }
        self.max_angle_deg = clamped;
        self.update_preview();
        let needs_recreate = !self.lifecycle.last_created_ids.is_empty();
        if needs_recreate {
            self.lifecycle.recreate_needed = true;
        }
        RouteToolPanelEffect {
            changed: true,
            needs_recreate,
            next_action: None,
        }
    }

    fn apply_segment_action(&mut self, value: f32) -> RouteToolPanelEffect {
        let adjusting = !self.lifecycle.last_created_ids.is_empty()
            && self.last_start_anchor.is_some()
            && self.lifecycle.last_end_anchor.is_some();
        let length = if adjusting {
            let start = self.last_start_anchor.unwrap().position();
            let end = self.lifecycle.last_end_anchor.unwrap().position();
            start.distance(end)
        } else {
            self.total_distance()
        };
        let effect = Self::apply_segment_distance_only(
            &mut self.seg,
            value,
            adjusting,
            self.start.is_some() && self.end.is_some(),
            length,
        );
        if !effect.changed {
            return effect;
        }
        self.update_preview();
        if effect.needs_recreate {
            self.lifecycle.recreate_needed = true;
        }
        effect
    }

    fn apply_min_distance_action(&mut self, value: f32) -> RouteToolPanelEffect {
        let clamped = SMOOTH_CURVE_MIN_DISTANCE_LIMITS.clamp(value);
        if (self.min_distance - clamped).abs() < f32::EPSILON {
            return RouteToolPanelEffect::default();
        }
        self.min_distance = clamped;
        self.update_preview();
        let needs_recreate = !self.lifecycle.last_created_ids.is_empty();
        if needs_recreate {
            self.lifecycle.recreate_needed = true;
        }
        RouteToolPanelEffect {
            changed: true,
            needs_recreate,
            next_action: None,
        }
    }

    fn reset_approach_steerer(&mut self) -> RouteToolPanelEffect {
        if !self.approach_manual {
            return RouteToolPanelEffect::default();
        }
        self.approach_manual = false;
        self.update_preview();
        let needs_recreate = !self.lifecycle.last_created_ids.is_empty();
        if needs_recreate {
            self.lifecycle.recreate_needed = true;
        }
        RouteToolPanelEffect {
            changed: true,
            needs_recreate,
            next_action: None,
        }
    }

    fn reset_departure_steerer(&mut self) -> RouteToolPanelEffect {
        if !self.departure_manual {
            return RouteToolPanelEffect::default();
        }
        self.departure_manual = false;
        self.update_preview();
        let needs_recreate = !self.lifecycle.last_created_ids.is_empty();
        if needs_recreate {
            self.lifecycle.recreate_needed = true;
        }
        RouteToolPanelEffect {
            changed: true,
            needs_recreate,
            next_action: None,
        }
    }

    fn remove_control_node(&mut self, index: usize) -> RouteToolPanelEffect {
        if index >= self.control_nodes.len() {
            return RouteToolPanelEffect::default();
        }
        self.control_nodes.remove(index);
        self.sync_derived();
        self.update_preview();
        let needs_recreate = !self.lifecycle.last_created_ids.is_empty();
        if needs_recreate {
            self.lifecycle.recreate_needed = true;
        }
        RouteToolPanelEffect {
            changed: true,
            needs_recreate,
            next_action: None,
        }
    }
}
