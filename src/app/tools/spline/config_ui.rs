//! Egui-freie Panel-Bruecke fuer das Catmull-Rom-Spline-Tool.

use super::super::common::tangent_options;
use super::super::RouteTool;
use super::SplineTool;
use crate::app::tool_contract::TangentSource;
use crate::app::ui_contract::{
    RouteToolPanelEffect, SegmentConfigPanelAction, SplinePanelAction, SplinePanelState,
    TangentOptionData, TangentSelectionState,
};

impl SplineTool {
    /// Liefert den egui-freien Panelzustand des Catmull-Rom-Spline-Tools.
    pub(super) fn panel_state(&self) -> SplinePanelState {
        let adjusting = !self.lifecycle.last_created_ids.is_empty() && self.last_anchors.len() >= 2;
        let length = if adjusting {
            Self::spline_length_from_anchors(
                &self.last_anchors,
                self.tangents.tangent_start,
                self.tangents.tangent_end,
            )
        } else {
            self.spline_length()
        };

        SplinePanelState {
            control_point_count: (!adjusting && self.is_ready()).then_some(self.anchors.len()),
            start_tangent: adjusting
                .then(|| {
                    tangent_selection_state(
                        "Tangente Start:",
                        self.tangents.tangent_start,
                        &self.tangents.start_neighbors,
                    )
                })
                .filter(|state| !state.options.is_empty()),
            end_tangent: adjusting
                .then(|| {
                    tangent_selection_state(
                        "Tangente Ende:",
                        self.tangents.tangent_end,
                        &self.tangents.end_neighbors,
                    )
                })
                .filter(|state| !state.options.is_empty()),
            segment: self.seg.panel_state(
                adjusting,
                self.is_ready(),
                length,
                "Spline-Laenge",
                true,
            ),
        }
    }

    /// Wendet eine semantische Panel-Aktion auf das Catmull-Rom-Spline-Tool an.
    pub(super) fn apply_panel_action(&mut self, action: SplinePanelAction) -> RouteToolPanelEffect {
        match action {
            SplinePanelAction::SetTangentStart(source) => self.apply_tangent_action(true, source),
            SplinePanelAction::SetTangentEnd(source) => self.apply_tangent_action(false, source),
            SplinePanelAction::Segment(segment_action) => self.apply_segment_action(segment_action),
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

    fn apply_segment_action(&mut self, action: SegmentConfigPanelAction) -> RouteToolPanelEffect {
        let adjusting = !self.lifecycle.last_created_ids.is_empty() && self.last_anchors.len() >= 2;
        let length = if adjusting {
            Self::spline_length_from_anchors(
                &self.last_anchors,
                self.tangents.tangent_start,
                self.tangents.tangent_end,
            )
        } else {
            self.spline_length()
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

fn tangent_selection_state(
    label: &str,
    current: TangentSource,
    neighbors: &[crate::core::ConnectedNeighbor],
) -> TangentSelectionState {
    TangentSelectionState {
        label: label.to_owned(),
        none_label: "Standard".to_owned(),
        current,
        options: tangent_options(neighbors)
            .into_iter()
            .filter(|option| option.source != TangentSource::None)
            .collect::<Vec<TangentOptionData>>(),
        enabled: true,
    }
}
