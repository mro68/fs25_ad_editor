//! Egui-freie Panel-Bruecke fuer das FieldPathTool.

use super::state::{FieldPathMode, FieldPathPhase, FieldPathTool};
use crate::app::ui_contract::{
    FieldPathModeChoice, FieldPathPanelAction, FieldPathPanelPhase, FieldPathPanelState,
    FieldPathPreviewStatus, FieldPathSelectionSummary, RouteToolPanelEffect,
};

impl FieldPathTool {
    /// Liefert den egui-freien Panelzustand des FieldPathTools.
    pub(super) fn panel_state(&self) -> FieldPathPanelState {
        FieldPathPanelState {
            mode: mode_choice(self.mode),
            phase: panel_phase(self.phase),
            side1: self.selection_summary(true),
            side2: matches!(
                self.phase,
                FieldPathPhase::SelectingSide2 | FieldPathPhase::Preview
            )
            .then(|| self.selection_summary(false)),
            can_advance_to_side2: self.can_advance_to_side2(),
            can_compute: self.can_compute(),
            preview_status: match self.phase {
                FieldPathPhase::Preview if self.resampled_nodes.is_empty() => {
                    Some(FieldPathPreviewStatus::NoMiddleLine)
                }
                FieldPathPhase::Preview => Some(FieldPathPreviewStatus::Generated {
                    node_count: self.resampled_nodes.len(),
                }),
                _ => None,
            },
            node_spacing: self.config.node_spacing,
            simplify_tolerance: self.config.simplify_tolerance,
            connect_to_existing: self.config.connect_to_existing,
        }
    }

    /// Wendet eine semantische Panel-Aktion auf das FieldPathTool an.
    pub(super) fn apply_panel_action(
        &mut self,
        action: FieldPathPanelAction,
    ) -> RouteToolPanelEffect {
        let changed = match action {
            FieldPathPanelAction::SetMode(choice) => {
                let mode = mode_from_choice(choice);
                if self.mode == mode {
                    false
                } else {
                    self.mode = mode;
                    true
                }
            }
            FieldPathPanelAction::Start => {
                if self.phase == FieldPathPhase::Idle {
                    self.phase = FieldPathPhase::SelectingSide1;
                    true
                } else {
                    false
                }
            }
            FieldPathPanelAction::AdvanceToSide2 => {
                if self.phase == FieldPathPhase::SelectingSide1 && self.can_advance_to_side2() {
                    self.phase = FieldPathPhase::SelectingSide2;
                    true
                } else {
                    false
                }
            }
            FieldPathPanelAction::Compute => {
                if self.phase == FieldPathPhase::SelectingSide2 && self.can_compute() {
                    self.compute_centerline();
                    true
                } else {
                    false
                }
            }
            FieldPathPanelAction::BackToSide1 => {
                if self.phase == FieldPathPhase::SelectingSide2 {
                    self.phase = FieldPathPhase::SelectingSide1;
                    true
                } else {
                    false
                }
            }
            FieldPathPanelAction::BackToSide2 => {
                if self.phase == FieldPathPhase::Preview {
                    self.phase = FieldPathPhase::SelectingSide2;
                    self.centerline.clear();
                    self.resampled_nodes.clear();
                    true
                } else {
                    false
                }
            }
            FieldPathPanelAction::Reset => {
                let had_pending = !matches!(self.phase, FieldPathPhase::Idle)
                    || !self.side1_field_ids.is_empty()
                    || !self.side2_field_ids.is_empty()
                    || !self.side1_segments.is_empty()
                    || !self.side2_segments.is_empty()
                    || !self.centerline.is_empty()
                    || !self.resampled_nodes.is_empty();
                self.phase = FieldPathPhase::Idle;
                self.side1_field_ids.clear();
                self.side2_field_ids.clear();
                self.side1_segments.clear();
                self.side2_segments.clear();
                self.centerline.clear();
                self.resampled_nodes.clear();
                had_pending
            }
            FieldPathPanelAction::SetNodeSpacing(value) => {
                set_f32(&mut self.config.node_spacing, value.clamp(1.0, 50.0))
            }
            FieldPathPanelAction::SetSimplifyTolerance(value) => {
                set_f32(&mut self.config.simplify_tolerance, value.clamp(0.0, 20.0))
            }
            FieldPathPanelAction::SetConnectToExisting(value) => {
                set_bool(&mut self.config.connect_to_existing, value)
            }
        };

        RouteToolPanelEffect {
            changed,
            needs_recreate: false,
            next_action: None,
        }
    }

    fn selection_summary(&self, side1: bool) -> FieldPathSelectionSummary {
        let title = if side1 { "Seite 1" } else { "Seite 2" }.to_owned();

        match self.mode {
            FieldPathMode::Fields => {
                let ids = if side1 {
                    &self.side1_field_ids
                } else {
                    &self.side2_field_ids
                };
                if ids.is_empty() {
                    FieldPathSelectionSummary {
                        title,
                        text: "Keine Felder ausgewaehlt".to_owned(),
                        is_empty: true,
                    }
                } else {
                    let labels: Vec<String> = ids.iter().map(|id| format!("#{id}")).collect();
                    FieldPathSelectionSummary {
                        title,
                        text: format!("Felder: {}", labels.join(", ")),
                        is_empty: false,
                    }
                }
            }
            FieldPathMode::Boundaries => {
                let count = if side1 {
                    self.side1_segments.len()
                } else {
                    self.side2_segments.len()
                };
                FieldPathSelectionSummary {
                    title,
                    text: format!("Segmente: {count}"),
                    is_empty: count == 0,
                }
            }
        }
    }

    fn can_advance_to_side2(&self) -> bool {
        match self.mode {
            FieldPathMode::Fields => !self.side1_field_ids.is_empty(),
            FieldPathMode::Boundaries => !self.side1_segments.is_empty(),
        }
    }

    fn can_compute(&self) -> bool {
        match self.mode {
            FieldPathMode::Fields => !self.side2_field_ids.is_empty(),
            FieldPathMode::Boundaries => !self.side2_segments.is_empty(),
        }
    }
}

fn mode_choice(mode: FieldPathMode) -> FieldPathModeChoice {
    match mode {
        FieldPathMode::Fields => FieldPathModeChoice::Fields,
        FieldPathMode::Boundaries => FieldPathModeChoice::Boundaries,
    }
}

fn mode_from_choice(choice: FieldPathModeChoice) -> FieldPathMode {
    match choice {
        FieldPathModeChoice::Fields => FieldPathMode::Fields,
        FieldPathModeChoice::Boundaries => FieldPathMode::Boundaries,
    }
}

fn panel_phase(phase: FieldPathPhase) -> FieldPathPanelPhase {
    match phase {
        FieldPathPhase::Idle => FieldPathPanelPhase::Idle,
        FieldPathPhase::SelectingSide1 => FieldPathPanelPhase::SelectingSide1,
        FieldPathPhase::SelectingSide2 => FieldPathPanelPhase::SelectingSide2,
        FieldPathPhase::Preview => FieldPathPanelPhase::Preview,
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
