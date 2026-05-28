//! Panel-Bruecke und Status-Texte fuer das Verrundungs-Tool.

use super::geometry::{ArcValidation, QuadraticValidation};
use super::state::{RoundingMode, RoundingTool};
use crate::app::tool_editing::RouteToolEditPayload;
use crate::app::tools::RouteToolPanelBridge;
use crate::app::ui_contract::{
    RoundingPanelAction, RoundingPanelState, RouteToolConfigState, RouteToolPanelAction,
    RouteToolPanelEffect, ROUNDING_ARC_RADIUS_LIMITS, ROUNDING_SAMPLE_SPACING_LIMITS,
};

impl RoundingTool {
    fn arc_status_text(&self) -> &str {
        if self.lifecycle.restored_for_edit {
            return "Nachbearbeitung — Radius oder Abtastung anpassen und Enter zum Neuaufbau druecken.";
        }
        if !self.lifecycle.last_created_ids.is_empty() {
            return "Zuletzt erzeugter Arc bleibt fuer Panel-Recreate verknuepft.";
        }

        match self.arc.validation {
            ArcValidation::NeedSingleSelection => "Arc-Modus erwartet genau 1 selektierten Node.",
            ArcValidation::MissingCornerPosition => {
                "Corner-Position konnte fuer die aktuelle Selektion nicht geladen werden."
            }
            ArcValidation::NeedTwoRouteSides => {
                "Selektierter Node braucht genau 2 eindeutige Anschlussseiten."
            }
            ArcValidation::AmbiguousJunction => {
                "Junction mit mehr als 2 Anschlussseiten ist in CP-02 ungueltig."
            }
            ArcValidation::NoThroughPath => {
                "Zwischen den beiden Seiten existiert keine gerichtete Durchfahrt ueber den Corner."
            }
            ArcValidation::DegenerateStretch => {
                "Mindestens eine Corner-Strecke ist zu kurz oder degeneriert."
            }
            ArcValidation::UnsupportedCornerAngle => {
                "Corner-Winkel laesst keinen stabilen Kreisbogen mit festem Radius zu."
            }
            ArcValidation::RadiusTooLarge => {
                "Radius passt nicht in mindestens eine Anschlussstrecke des Corner-Pfads."
            }
            ArcValidation::Ready => "Bereit — Enter verrundet den Corner mit festem Kreisbogen.",
        }
    }

    fn quadratic_status_text(&self) -> &str {
        if self.lifecycle.restored_for_edit {
            return "Nachbearbeitung — Abtastung anpassen und Enter zum Neuaufbau druecken.";
        }
        if !self.lifecycle.last_created_ids.is_empty() {
            return "Zuletzt erzeugte Quadratic-Verrundung bleibt fuer Panel-Recreate verknuepft.";
        }

        match self.quadratic.validation {
            QuadraticValidation::NeedOrderedThreeNodeChain => {
                "3-Punkt-Modus erwartet genau 3 selektierte Nodes als geordnete Kette."
            }
            QuadraticValidation::MissingChainNodeContext => {
                "Selektionskontext der 3er-Kette konnte nicht geladen werden."
            }
            QuadraticValidation::MissingOuterStartStretch => {
                "P1 braucht genau eine Aussenstrecke ausserhalb der Selektion."
            }
            QuadraticValidation::AmbiguousOuterStartStretch => {
                "P1 hat mehrere Aussenstrecken; CP-03 raet keinen Ast."
            }
            QuadraticValidation::MissingOuterEndStretch => {
                "P3 braucht genau eine Aussenstrecke ausserhalb der Selektion."
            }
            QuadraticValidation::AmbiguousOuterEndStretch => {
                "P3 hat mehrere Aussenstrecken; CP-03 raet keinen Ast."
            }
            QuadraticValidation::ControlHasExternalConnections => {
                "P2 darf in CP-03 keine zusaetzlichen Aussenverbindungen haben."
            }
            QuadraticValidation::BrokenSelectedChain => {
                "Die 3er-Selektion braucht intern eindeutige Anchor-Pfade als P1 -> P2 -> P3."
            }
            QuadraticValidation::DegenerateOuterStretch => {
                "Mindestens eine Aussenstrecke der 3er-Kette ist zu kurz oder degeneriert."
            }
            QuadraticValidation::TangentsMissFixedControl => {
                "Die Aussenstrecken muessen sich mit passender Richtung im festen Steuerpunkt P2 schneiden."
            }
            QuadraticValidation::NoThroughPath => {
                "Ueber P1 -> P2 -> P3 existiert keine gerichtete Durchfahrt fuer die Verrundung."
            }
            QuadraticValidation::Ready => {
                "Bereit — Enter ersetzt die mittlere Node durch eine quadratische Verrundung."
            }
        }
    }

    fn preview_node_count(&self) -> Option<usize> {
        match self.mode {
            RoundingMode::ArcOnePoint => {
                self.arc.plan.as_ref().map(|plan| plan.arc_positions.len())
            }
            RoundingMode::QuadraticThreePoint => self
                .quadratic
                .plan
                .as_ref()
                .map(|plan| plan.curve_positions.len()),
        }
    }

    fn update_stored_payload_after_arc_change(&mut self) {
        if let Some(RouteToolEditPayload::RoundingArc {
            radius_m,
            sample_spacing_m,
            ..
        }) = self.lifecycle.edit_payload.as_mut()
        {
            *radius_m = self.arc.radius_m;
            *sample_spacing_m = self.arc.sample_spacing_m;
        }
    }

    fn update_stored_payload_after_quadratic_change(&mut self) {
        if let Some(RouteToolEditPayload::RoundingQuadratic {
            sample_spacing_m, ..
        }) = self.lifecycle.edit_payload.as_mut()
        {
            *sample_spacing_m = self.quadratic.sample_spacing_m;
        }
    }

    fn finish_panel_change(&mut self, changed: bool) -> RouteToolPanelEffect {
        if changed && !self.lifecycle.last_created_ids.is_empty() {
            self.lifecycle.recreate_needed = true;
        }

        RouteToolPanelEffect {
            changed,
            needs_recreate: changed && !self.lifecycle.last_created_ids.is_empty(),
            next_action: None,
        }
    }
}

impl RouteToolPanelBridge for RoundingTool {
    fn status_text(&self) -> &str {
        match self.mode {
            RoundingMode::ArcOnePoint => self.arc_status_text(),
            RoundingMode::QuadraticThreePoint => self.quadratic_status_text(),
        }
    }

    fn panel_state(&self) -> RouteToolConfigState {
        RouteToolConfigState::Rounding(RoundingPanelState {
            mode: self.panel_mode(),
            mode_locked: self.mode_locked(),
            arc_radius_m: self.arc.radius_m,
            arc_sample_spacing_m: self.arc.sample_spacing_m,
            quadratic_sample_spacing_m: self.quadratic.sample_spacing_m,
            selected_node_count: match self.mode {
                RoundingMode::ArcOnePoint => self.arc.selected_node_ids.len(),
                RoundingMode::QuadraticThreePoint => self.quadratic.selected_node_ids.len(),
            },
            chain_node_count: self.quadratic.chain_node_ids.len(),
            preview_node_count: self.preview_node_count(),
            is_adjusting: self.is_adjusting(),
        })
    }

    fn apply_panel_action(&mut self, action: RouteToolPanelAction) -> RouteToolPanelEffect {
        let RouteToolPanelAction::Rounding(action) = action else {
            return RouteToolPanelEffect::default();
        };

        let changed = match action {
            RoundingPanelAction::SetMode(mode) => self.set_panel_mode(mode),
            RoundingPanelAction::SetArcRadius(value) => {
                let next = ROUNDING_ARC_RADIUS_LIMITS.clamp(value);
                if (self.arc.radius_m - next).abs() < f32::EPSILON {
                    false
                } else {
                    self.arc.radius_m = next;
                    self.refresh_arc_state();
                    self.update_stored_payload_after_arc_change();
                    true
                }
            }
            RoundingPanelAction::SetArcSampleSpacing(value) => {
                let next = ROUNDING_SAMPLE_SPACING_LIMITS.clamp(value);
                if (self.arc.sample_spacing_m - next).abs() < f32::EPSILON {
                    false
                } else {
                    self.arc.sample_spacing_m = next;
                    self.refresh_arc_state();
                    self.update_stored_payload_after_arc_change();
                    true
                }
            }
            RoundingPanelAction::SetQuadraticSampleSpacing(value) => {
                let next = ROUNDING_SAMPLE_SPACING_LIMITS.clamp(value);
                if (self.quadratic.sample_spacing_m - next).abs() < f32::EPSILON {
                    false
                } else {
                    self.quadratic.sample_spacing_m = next;
                    self.refresh_quadratic_state();
                    self.update_stored_payload_after_quadratic_change();
                    true
                }
            }
        };

        self.finish_panel_change(changed)
    }
}
