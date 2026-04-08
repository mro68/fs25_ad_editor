//! Egui-freie Panel-Bruecke und Tangenten-Helfer fuer das Bézier-Kurven-Tool.

use super::super::common::tangent_options;
use super::super::RouteToolCore;
use super::geometry::{approx_length, cubic_bezier, quadratic_bezier};
use super::state::{CurveDegree, CurveTool, Phase};
use crate::app::tool_contract::TangentSource;
use crate::app::ui_contract::{
    CurveDegreeChoice, CurvePanelAction, CurvePanelState, CurveTangentsPanelState,
    RouteToolPanelEffect, SegmentConfigPanelAction, SegmentLengthKind, TangentHelpHint,
    TangentMenuData, TangentNoneReason, TangentOptionData, TangentSelectionState,
};

impl CurveTool {
    /// Liefert den egui-freien Panelzustand des Bézier-Kurven-Tools.
    pub(super) fn panel_state(&self) -> CurvePanelState {
        let adjusting = !self.lifecycle.last_created_ids.is_empty()
            && self.last_start_anchor.is_some()
            && self.lifecycle.last_end_anchor.is_some()
            && self.last_control_point1.is_some();
        let tangents = if self.degree == CurveDegree::Cubic {
            let in_control = self.phase == Phase::Control;
            let can_edit_tangents = in_control || adjusting;
            let start_none_reason = if self.tangents.start_neighbors.is_empty() {
                TangentNoneReason::NoConnection
            } else {
                TangentNoneReason::NoTangent
            };
            let end_none_reason = if self.tangents.end_neighbors.is_empty() {
                TangentNoneReason::NoConnection
            } else {
                TangentNoneReason::NoTangent
            };

            Some(CurveTangentsPanelState {
                help_hint: (!can_edit_tangents).then_some(TangentHelpHint::SetStartEnd),
                start: TangentSelectionState {
                    none_reason: start_none_reason,
                    current: self.tangents.tangent_start,
                    options: neighbor_options(&self.tangents.start_neighbors),
                    enabled: can_edit_tangents,
                },
                end: TangentSelectionState {
                    none_reason: end_none_reason,
                    current: self.tangents.tangent_end,
                    options: neighbor_options(&self.tangents.end_neighbors),
                    enabled: can_edit_tangents,
                },
            })
        } else {
            None
        };

        let length = if adjusting {
            let start_pos = self.last_start_anchor.unwrap().position();
            let end_pos = self.lifecycle.last_end_anchor.unwrap().position();
            let cp1 = self.last_control_point1.unwrap();
            let cp2 = self.last_control_point2;
            match self.degree {
                CurveDegree::Quadratic => {
                    approx_length(|t| quadratic_bezier(start_pos, cp1, end_pos, t), 64)
                }
                CurveDegree::Cubic => {
                    let cp2v = cp2.unwrap_or(cp1);
                    approx_length(|t| cubic_bezier(start_pos, cp1, cp2v, end_pos, t), 64)
                }
            }
        } else {
            self.curve_length()
        };

        CurvePanelState {
            degree: match self.degree {
                CurveDegree::Quadratic => CurveDegreeChoice::Quadratic,
                CurveDegree::Cubic => CurveDegreeChoice::Cubic,
            },
            tangents,
            segment: self.seg.panel_state(
                adjusting,
                self.is_ready(),
                length,
                SegmentLengthKind::Curve,
                true,
            ),
        }
    }

    /// Wendet eine semantische Panel-Aktion auf das Bézier-Kurven-Tool an.
    pub(super) fn apply_panel_action(&mut self, action: CurvePanelAction) -> RouteToolPanelEffect {
        match action {
            CurvePanelAction::SetDegree(choice) => self.apply_degree_action(choice),
            CurvePanelAction::SetTangentStart(source) => self.apply_tangent_action(true, source),
            CurvePanelAction::SetTangentEnd(source) => self.apply_tangent_action(false, source),
            CurvePanelAction::Segment(segment_action) => self.apply_segment_action(segment_action),
        }
    }

    /// Liefert Tangenten-Menuedaten fuer das zentrale Kontextmenue (nur Daten, kein UI).
    ///
    /// Nur aktiv fuer kubische Kurven in `Phase::Control` oder im Adjusting-Modus,
    /// wenn Nachbarn an Start- oder Endpunkt vorhanden sind.
    pub(super) fn build_tangent_menu_data(&self) -> Option<TangentMenuData> {
        if self.degree != CurveDegree::Cubic {
            return None;
        }

        let in_control = self.phase == Phase::Control;
        let adjusting = self.lifecycle.has_last_created()
            && self.last_start_anchor.is_some()
            && self.lifecycle.last_end_anchor.is_some();
        if !in_control && !adjusting {
            return None;
        }

        let has_start = !self.tangents.start_neighbors.is_empty();
        let has_end = !self.tangents.end_neighbors.is_empty();
        if !has_start && !has_end {
            return None;
        }

        Some(TangentMenuData {
            start_options: tangent_options(&self.tangents.start_neighbors),
            end_options: tangent_options(&self.tangents.end_neighbors),
            current_start: self.tangents.tangent_start,
            current_end: self.tangents.tangent_end,
        })
    }

    /// Wendet die vom User gewaehlten Tangenten aus dem Kontextmenue an.
    ///
    /// Aktualisiert Kontrollpunkte, derived state und setzt ggf. das Recreate-Flag.
    pub(super) fn apply_tangent_from_menu(&mut self, start: TangentSource, end: TangentSource) {
        self.tangents.tangent_start = start;
        self.tangents.tangent_end = end;
        self.apply_tangent_to_cp();
        self.sync_derived();
        self.init_apex();
        if self.lifecycle.has_last_created() {
            self.lifecycle.recreate_needed = true;
        }
    }

    fn apply_degree_action(&mut self, choice: CurveDegreeChoice) -> RouteToolPanelEffect {
        let degree = match choice {
            CurveDegreeChoice::Quadratic => CurveDegree::Quadratic,
            CurveDegreeChoice::Cubic => CurveDegree::Cubic,
        };
        if self.degree == degree {
            return RouteToolPanelEffect::default();
        }

        self.degree = degree;
        self.control_point2 = None;
        self.tangents.reset_tangents();
        RouteToolPanelEffect {
            changed: true,
            needs_recreate: false,
            next_action: None,
        }
    }

    fn apply_tangent_action(
        &mut self,
        is_start: bool,
        source: TangentSource,
    ) -> RouteToolPanelEffect {
        let current = if is_start {
            self.tangents.tangent_start
        } else {
            self.tangents.tangent_end
        };
        if current == source {
            return RouteToolPanelEffect::default();
        }

        if is_start {
            self.tangents.tangent_start = source;
        } else {
            self.tangents.tangent_end = source;
        }
        self.apply_tangent_to_cp();
        self.sync_derived();
        self.init_apex();
        let needs_recreate = self.lifecycle.has_last_created();
        if needs_recreate {
            self.lifecycle.recreate_needed = true;
        }
        RouteToolPanelEffect {
            changed: true,
            needs_recreate,
            next_action: None,
        }
    }

    fn apply_segment_action(&mut self, action: SegmentConfigPanelAction) -> RouteToolPanelEffect {
        let adjusting = !self.lifecycle.last_created_ids.is_empty()
            && self.last_start_anchor.is_some()
            && self.lifecycle.last_end_anchor.is_some()
            && self.last_control_point1.is_some();
        let length = if adjusting {
            let start_pos = self.last_start_anchor.unwrap().position();
            let end_pos = self.lifecycle.last_end_anchor.unwrap().position();
            let cp1 = self.last_control_point1.unwrap();
            let cp2 = self.last_control_point2;
            match self.degree {
                CurveDegree::Quadratic => {
                    approx_length(|t| quadratic_bezier(start_pos, cp1, end_pos, t), 64)
                }
                CurveDegree::Cubic => {
                    let cp2v = cp2.unwrap_or(cp1);
                    approx_length(|t| cubic_bezier(start_pos, cp1, cp2v, end_pos, t), 64)
                }
            }
        } else {
            self.curve_length()
        };
        let result = self
            .seg
            .apply_panel_action(action, adjusting, self.is_ready(), length, true);
        if result.recreate {
            self.lifecycle.recreate_needed = true;
        }
        RouteToolPanelEffect {
            changed: result.changed,
            needs_recreate: result.recreate,
            next_action: None,
        }
    }
}

fn neighbor_options(neighbors: &[crate::core::ConnectedNeighbor]) -> Vec<TangentOptionData> {
    tangent_options(neighbors)
        .into_iter()
        .filter(|option| option.source != TangentSource::None)
        .collect()
}
