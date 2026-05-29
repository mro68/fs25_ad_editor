//! Panel-Bruecke und Status-Texte fuer das Arc-only-Verrundungs-Tool.

use super::geometry::ArcValidation;
use super::state::{clamp_arc_max_angle_deg, RoundingTool};
use crate::app::tool_editing::RouteToolEditPayload;
use crate::app::tools::RouteToolPanelBridge;
use crate::app::ui_contract::{
    RoundingPanelAction, RoundingPanelState, RouteToolConfigState, RouteToolPanelAction,
    RouteToolPanelEffect, ROUNDING_ARC_RADIUS_LIMITS,
};

impl RoundingTool {
    fn arc_status_text(&self) -> &str {
        if self.lifecycle.restored_for_edit {
            return "Nachbearbeitung — Radius oder Max-Winkel anpassen und Enter zum Neuaufbau druecken.";
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

    fn preview_node_count(&self) -> Option<usize> {
        self.arc.plan.as_ref().map(|plan| plan.arc_positions.len())
    }

    fn update_stored_payload_after_arc_change(&mut self) {
        if let Some(RouteToolEditPayload::RoundingArc {
            radius_m,
            max_angle_deg,
            ..
        }) = self.lifecycle.edit_payload.as_mut()
        {
            *radius_m = self.arc.radius_m;
            *max_angle_deg = self.arc.max_angle_deg;
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
        self.arc_status_text()
    }

    fn panel_state(&self) -> RouteToolConfigState {
        RouteToolConfigState::Rounding(RoundingPanelState {
            arc_radius_m: self.arc.radius_m,
            max_angle_deg: self.arc.max_angle_deg,
            selected_node_count: self.arc.selected_node_ids.len(),
            preview_node_count: self.preview_node_count(),
            is_adjusting: self.is_adjusting(),
        })
    }

    fn apply_panel_action(&mut self, action: RouteToolPanelAction) -> RouteToolPanelEffect {
        let RouteToolPanelAction::Rounding(action) = action else {
            return RouteToolPanelEffect::default();
        };

        let changed = match action {
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
            RoundingPanelAction::SetMaxAngleDeg(value) => {
                let next = clamp_arc_max_angle_deg(value);
                if (self.arc.max_angle_deg - next).abs() < f32::EPSILON {
                    false
                } else {
                    self.arc.max_angle_deg = next;
                    self.refresh_arc_state();
                    self.update_stored_payload_after_arc_change();
                    true
                }
            }
        };

        self.finish_panel_change(changed)
    }
}
