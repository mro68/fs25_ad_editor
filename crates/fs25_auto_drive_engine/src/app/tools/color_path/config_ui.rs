//! Egui-freie Panel-Bruecke fuer das ColorPathTool.

use super::state::{ColorPathPhase, ColorPathTool, ExistingConnectionMode};
use crate::app::ui_contract::{
    ColorPathPanelAction, ColorPathPanelPhase, ColorPathPanelState, ColorPathPreviewStats,
    ExistingConnectionModeChoice, RouteToolPanelEffect, RouteToolPanelFollowUp,
};

impl ColorPathTool {
    /// Liefert den egui-freien Panelzustand des ColorPathTools.
    ///
    /// Single-Step-Modell (CP-06): Die Engine ist alleinige Quelle der
    /// Compute-/Accept-Flags. `can_compute` ist nur in `Sampling` mit
    /// vorhandenen Farbsamples wahr; `can_accept` haengt direkt an
    /// [`ColorPathTool::can_execute`]. Die Legacy-Flags `can_next`/`can_back`
    /// liefert die Engine nicht mehr — sie sind konstant `false`.
    #[allow(deprecated)] // Legacy-Flags can_next/can_back bleiben fuer DTO-Kompat additiv (CP-11 entfernt sie).
    pub(super) fn panel_state(&self) -> ColorPathPanelState {
        let preview_stats = if self.phase.is_editing() {
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

        // Single-Step-Wizard: kanonische Flags sind `can_compute` (Sampling)
        // und `can_accept` (Editing + ausfuehrbar). Legacy-Flags konstant
        // `false`, damit Hosts auf den Reset/Compute/Accept-Drilldown migrieren.
        let can_compute = matches!(self.phase, ColorPathPhase::Sampling)
            && !self.sampling.sampled_colors.is_empty();
        let can_accept = self.can_execute();
        let can_back = false;
        let can_next = false;

        ColorPathPanelState {
            phase: panel_phase(self.phase),
            sample_count: self.sampling.sampled_colors.len(),
            avg_color: self.sampling.avg_color,
            palette_colors: self.matching.palette.clone(),
            can_compute,
            can_next,
            can_back,
            can_accept,
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
    ///
    /// Single-Step-Modell (CP-06): die kanonischen Aktionen sind
    /// `StartSampling` / `Compute` / `Accept` / `Reset`. Die Legacy-Aktionen
    /// werden gemaess folgender Mapping-Tabelle uebersetzt:
    ///
    /// - `ComputePreview` → wie `Compute`.
    /// - `NextPhase` in `Sampling` → wie `Compute`; in `Editing` ist es ein
    ///   No-Op (`changed = false`, kein Effekt — CP-06 hat den Wizard auf
    ///   einen Schritt verdichtet).
    /// - `PrevPhase` / `BackToSampling` → wie `Reset`. Begruendung: nach
    ///   CP-06 existiert kein Zwischenzustand mehr, in den die Pipeline
    ///   teilweise zurueckgenommen werden koennte; ein Rueckweg aus `Editing`
    ///   verwirft Editable + Preview ohnehin vollstaendig und ist damit
    ///   semantisch ein vollstaendiger Reset.
    #[allow(deprecated)] // Legacy-Aktionen bleiben fuer Host-Kompat bis CP-11.
    pub(super) fn apply_panel_action(
        &mut self,
        action: ColorPathPanelAction,
    ) -> RouteToolPanelEffect {
        // Schritt 1: Legacy-Aktionen auf die kanonischen Aktionen abbilden.
        let action = match action {
            ColorPathPanelAction::ComputePreview => ColorPathPanelAction::Compute,
            ColorPathPanelAction::NextPhase => match self.phase {
                ColorPathPhase::Sampling => ColorPathPanelAction::Compute,
                // No-Op in Editing/Idle: kein Phasenwechsel mehr moeglich.
                ColorPathPhase::Idle | ColorPathPhase::Editing => {
                    return RouteToolPanelEffect::default();
                }
            },
            ColorPathPanelAction::PrevPhase | ColorPathPanelAction::BackToSampling => {
                ColorPathPanelAction::Reset
            }
            other => other,
        };

        let mut follow_up: Option<RouteToolPanelFollowUp> = None;
        let changed = match action {
            ColorPathPanelAction::StartSampling => {
                if self.phase == ColorPathPhase::Idle {
                    self.phase = ColorPathPhase::Sampling;
                    true
                } else {
                    false
                }
            }
            // Kanonische Single-Step-Aktion: aus `Sampling` mit Samples direkt
            // nach `Editing` rechnen. In allen anderen Phasen No-Op.
            ColorPathPanelAction::Compute => {
                if matches!(self.phase, ColorPathPhase::Sampling)
                    && !self.sampling.sampled_colors.is_empty()
                    && self.sampling.lasso_start_world.is_some()
                {
                    let phase_before = self.phase;
                    self.compute_to_editing();
                    let advanced = self.phase != phase_before;
                    if advanced {
                        // Compute hat das Netz neu aufgebaut; der Host soll
                        // die Vorschau neu auswerten.
                        follow_up = Some(RouteToolPanelFollowUp::UpdatePreview);
                    }
                    advanced
                } else {
                    false
                }
            }
            // Finales Uebernehmen: Hook auf den bestehenden Apply-Pfad via Controller.
            ColorPathPanelAction::Accept => {
                if self.can_execute() {
                    follow_up = Some(RouteToolPanelFollowUp::ReadyToExecute);
                    // `changed` bleibt false: der nachgelagerte Apply-/Reset-
                    // Pfad des Controllers bewertet das Panel ohnehin neu.
                    false
                } else {
                    false
                }
            }
            // Legacy-Aliasse sind oben bereits umgemappt; diese Arme sind nie erreichbar.
            ColorPathPanelAction::ComputePreview
            | ColorPathPanelAction::BackToSampling
            | ColorPathPanelAction::NextPhase
            | ColorPathPanelAction::PrevPhase => false,
            ColorPathPanelAction::Reset => {
                let had_pending = self.phase != ColorPathPhase::Idle
                    || !self.sampling.lasso_regions.is_empty()
                    || !self.sampling.sampled_colors.is_empty()
                    || self.preview_data.is_some()
                    || self.sampling_preview.is_some();
                self.reset_all();
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
            next_action: follow_up,
        }
    }

    /// Fuehrt das Tool von der aktuellen Phase bis `Editing` durch.
    ///
    /// Interner Helfer fuer Benchmark- und Testharnische. Gibt `true` zurueck,
    /// sobald `Editing` mit fertiger Stage F erreicht ist (Single-Step,
    /// CP-08 final umbenannt von `run_wizard_to_finalize`).
    pub(super) fn run_to_editing(&mut self) -> bool {
        if self.phase != ColorPathPhase::Editing {
            if !matches!(self.phase, ColorPathPhase::Sampling)
                || self.sampling.sampled_colors.is_empty()
                || self.sampling.lasso_start_world.is_none()
            {
                return false;
            }
            self.compute_to_editing();
            if self.phase != ColorPathPhase::Editing {
                return false;
            }
        }
        self.can_execute()
    }
}

/// Bildet die engine-interne Wizard-Phase auf die DTO-Phase ab.
///
/// Single-Step (CP-06): die Engine emittiert nur noch die kanonischen
/// DTO-Varianten `Idle` / `Sampling` / `Editing`. Die Legacy-DTO-Varianten
/// `Preview` / `CenterlinePreview` / `JunctionEdit` / `Finalize` werden vom
/// DTO-Layer fuer eingehende Strings weiterhin tolerant auf `Editing`
/// gefaltet (additiv), aber von der Engine ausgehend nicht mehr gesetzt.
fn panel_phase(phase: ColorPathPhase) -> ColorPathPanelPhase {
    match phase {
        ColorPathPhase::Idle => ColorPathPanelPhase::Idle,
        ColorPathPhase::Sampling => ColorPathPanelPhase::Sampling,
        ColorPathPhase::Editing => ColorPathPanelPhase::Editing,
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
