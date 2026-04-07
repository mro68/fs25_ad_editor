//! Egui-freie Panel-Bruecke fuer das Gerade-Strecke-Tool.

use super::state::StraightLineTool;
use crate::app::ui_contract::{
    RouteToolPanelEffect, SegmentConfigPanelAction, SegmentLengthKind, StraightPanelAction,
    StraightPanelState,
};

impl StraightLineTool {
    /// Liefert den egui-freien Panelzustand des Gerade-Strecke-Tools.
    pub(super) fn panel_state(&self) -> StraightPanelState {
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

        StraightPanelState {
            segment: self.seg.panel_state(
                adjusting,
                self.start.is_some() && self.end.is_some(),
                length,
                SegmentLengthKind::StraightLine,
                true,
            ),
        }
    }

    /// Wendet eine semantische Panel-Aktion auf das Gerade-Strecke-Tool an.
    pub(super) fn apply_panel_action(
        &mut self,
        action: StraightPanelAction,
    ) -> RouteToolPanelEffect {
        match action {
            StraightPanelAction::Segment(segment_action) => {
                self.apply_segment_action(segment_action)
            }
        }
    }

    fn apply_segment_action(&mut self, action: SegmentConfigPanelAction) -> RouteToolPanelEffect {
        let adjusting = !self.lifecycle.last_created_ids.is_empty()
            && self.last_start_anchor.is_some()
            && self.lifecycle.last_end_anchor.is_some();
        let length = if adjusting {
            let start = self
                .last_start_anchor
                .expect("Start-Anker muss im Adjusting-Modus vorhanden sein")
                .position();
            let end = self
                .lifecycle
                .last_end_anchor
                .expect("End-Anker muss im Adjusting-Modus vorhanden sein")
                .position();
            start.distance(end)
        } else {
            self.total_distance()
        };
        let result = self.seg.apply_panel_action(
            action,
            adjusting,
            self.start.is_some() && self.end.is_some(),
            length,
            true,
        );
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
