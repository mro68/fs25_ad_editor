//! Egui-freie Panel-Bruecke fuer das ColorPathTool.

use super::state::{ColorPathPhase, ColorPathTool, ExistingConnectionMode};
use crate::app::ui_contract::{
    ColorPathPanelAction, ColorPathPanelPhase, ColorPathPanelState, ColorPathPreviewStats,
    ExistingConnectionModeChoice, RouteToolPanelEffect,
};

impl ColorPathTool {
    /// Liefert den egui-freien Panelzustand des ColorPathTools.
    pub(super) fn panel_state(&self) -> ColorPathPanelState {
        let preview_stats = if self.phase == ColorPathPhase::Preview {
            let (junction_count, open_end_count, segment_count) = self.preview_stats();
            Some(ColorPathPreviewStats {
                junction_count,
                open_end_count,
                segment_count,
                node_count: self.preview_node_count(),
                can_accept: self
                    .preview_data
                    .as_ref()
                    .is_some_and(|preview| !preview.prepared_segments.is_empty()),
            })
        } else {
            None
        };

        ColorPathPanelState {
            phase: panel_phase(self.phase),
            sample_count: self.sampling.sampled_colors.len(),
            avg_color: self.sampling.avg_color,
            palette_colors: self.matching.palette.clone(),
            can_compute: !self.sampling.sampled_colors.is_empty(),
            preview_stats,
            exact_color_match: self.config.exact_color_match,
            color_tolerance: self.config.color_tolerance,
            node_spacing: self.config.node_spacing,
            simplify_tolerance: self.config.simplify_tolerance,
            junction_radius: self.config.junction_radius,
            noise_filter: self.config.noise_filter,
            existing_connection_mode: mode_choice(self.config.existing_connection_mode),
        }
    }

    /// Wendet eine semantische Panel-Aktion auf das ColorPathTool an.
    pub(super) fn apply_panel_action(
        &mut self,
        action: ColorPathPanelAction,
    ) -> RouteToolPanelEffect {
        let changed = match action {
            ColorPathPanelAction::StartSampling => {
                if self.phase == ColorPathPhase::Idle {
                    self.phase = ColorPathPhase::Sampling;
                    true
                } else {
                    false
                }
            }
            ColorPathPanelAction::ComputePreview => {
                if self.phase == ColorPathPhase::Sampling
                    && !self.sampling.sampled_colors.is_empty()
                {
                    self.compute_pipeline();
                    true
                } else {
                    false
                }
            }
            ColorPathPanelAction::BackToSampling => {
                if self.phase == ColorPathPhase::Preview {
                    self.phase = ColorPathPhase::Sampling;
                    self.clear_preview_pipeline();
                    true
                } else {
                    false
                }
            }
            ColorPathPanelAction::Reset => {
                let had_pending = self.phase != ColorPathPhase::Idle
                    || !self.sampling.lasso_regions.is_empty()
                    || !self.sampling.sampled_colors.is_empty()
                    || self.preview_data.is_some()
                    || self.sampling_preview.is_some();
                self.phase = ColorPathPhase::Idle;
                self.sampling = super::state::SamplingInput::default();
                self.matching = super::state::MatchingSpec::default();
                self.sampling_preview = None;
                self.preview_data = None;
                self.cache = super::state::ColorPathCacheState::default();
                had_pending
            }
            ColorPathPanelAction::SetExactColorMatch(value) => {
                if set_bool(&mut self.config.exact_color_match, value) {
                    self.on_matching_config_changed();
                    true
                } else {
                    false
                }
            }
            ColorPathPanelAction::SetColorTolerance(value) => {
                if set_f32(&mut self.config.color_tolerance, value.clamp(1.0, 80.0)) {
                    self.on_matching_config_changed();
                    true
                } else {
                    false
                }
            }
            ColorPathPanelAction::SetNodeSpacing(value) => {
                if set_f32(&mut self.config.node_spacing, value.clamp(1.0, 50.0)) {
                    self.on_preview_geometry_config_changed();
                    true
                } else {
                    false
                }
            }
            ColorPathPanelAction::SetSimplifyTolerance(value) => {
                if set_f32(&mut self.config.simplify_tolerance, value.clamp(0.0, 20.0)) {
                    self.on_preview_geometry_config_changed();
                    true
                } else {
                    false
                }
            }
            ColorPathPanelAction::SetJunctionRadius(value) => {
                if set_f32(&mut self.config.junction_radius, value.clamp(0.0, 100.0)) {
                    self.on_preview_geometry_config_changed();
                    true
                } else {
                    false
                }
            }
            ColorPathPanelAction::SetNoiseFilter(value) => {
                if set_bool(&mut self.config.noise_filter, value) {
                    self.on_preview_core_config_changed();
                    true
                } else {
                    false
                }
            }
            ColorPathPanelAction::SetExistingConnectionMode(choice) => {
                set_existing_connection_mode(&mut self.config.existing_connection_mode, choice)
            }
        };

        RouteToolPanelEffect {
            changed,
            needs_recreate: false,
            next_action: None,
        }
    }
}

fn panel_phase(phase: ColorPathPhase) -> ColorPathPanelPhase {
    match phase {
        ColorPathPhase::Idle => ColorPathPanelPhase::Idle,
        ColorPathPhase::Sampling => ColorPathPanelPhase::Sampling,
        ColorPathPhase::Preview => ColorPathPanelPhase::Preview,
    }
}

fn mode_choice(mode: ExistingConnectionMode) -> ExistingConnectionModeChoice {
    match mode {
        ExistingConnectionMode::Never => ExistingConnectionModeChoice::Never,
        ExistingConnectionMode::OpenEnds => ExistingConnectionModeChoice::OpenEnds,
        ExistingConnectionMode::OpenEndsAndJunctions => {
            ExistingConnectionModeChoice::OpenEndsAndJunctions
        }
    }
}

fn mode_from_choice(choice: ExistingConnectionModeChoice) -> ExistingConnectionMode {
    match choice {
        ExistingConnectionModeChoice::Never => ExistingConnectionMode::Never,
        ExistingConnectionModeChoice::OpenEnds => ExistingConnectionMode::OpenEnds,
        ExistingConnectionModeChoice::OpenEndsAndJunctions => {
            ExistingConnectionMode::OpenEndsAndJunctions
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

fn set_existing_connection_mode(
    target: &mut ExistingConnectionMode,
    choice: ExistingConnectionModeChoice,
) -> bool {
    let value = mode_from_choice(choice);
    if *target == value {
        false
    } else {
        *target = value;
        true
    }
}
