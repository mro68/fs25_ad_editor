//! Panel-Bruecke und Status-Texte fuer den Arc-Pfad des Verrundungs-Tools.

use super::geometry::ArcValidation;
use super::state::{RoundingMode, RoundingTool};
use crate::app::tools::RouteToolPanelBridge;
use crate::app::ui_contract::{
    RoundingPanelAction, RoundingPanelState, RouteToolConfigState, RouteToolPanelAction,
    RouteToolPanelEffect, ROUNDING_ARC_RADIUS_LIMITS, ROUNDING_SAMPLE_SPACING_LIMITS,
};

impl RoundingTool {
    fn arc_status_text(&self) -> &str {
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
                "Radius passt nicht in die erste Strecke auf mindestens einer Seite."
            }
            ArcValidation::Ready => "Bereit — Enter verrundet den Corner mit festem Kreisbogen.",
        }
    }

    fn quadratic_status_text(&self) -> &str {
        "3-Punkt-Modus ist vorbereitet und folgt im naechsten Commit."
    }

    fn preview_node_count(&self) -> Option<usize> {
        match self.mode {
            RoundingMode::ArcOnePoint => {
                self.arc.plan.as_ref().map(|plan| plan.arc_positions.len())
            }
            RoundingMode::QuadraticThreePoint => None,
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
            mode_locked: false,
            arc_radius_m: self.arc.radius_m,
            arc_sample_spacing_m: self.arc.sample_spacing_m,
            quadratic_sample_spacing_m: self.quadratic_sample_spacing_m,
            selected_node_count: self.selected_node_count,
            chain_node_count: 0,
            preview_node_count: self.preview_node_count(),
            is_adjusting: false,
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
                    true
                }
            }
            RoundingPanelAction::SetQuadraticSampleSpacing(value) => {
                let next = ROUNDING_SAMPLE_SPACING_LIMITS.clamp(value);
                if (self.quadratic_sample_spacing_m - next).abs() < f32::EPSILON {
                    false
                } else {
                    self.quadratic_sample_spacing_m = next;
                    true
                }
            }
        };

        RouteToolPanelEffect {
            changed,
            needs_recreate: false,
            next_action: None,
        }
    }
}
