//! Egui-freie Panel-Bruecke fuer das ColorPathTool.

use super::state::{ColorPathPhase, ColorPathTool, ExistingConnectionMode};
use crate::app::ui_contract::{
    ColorPathPanelAction, ColorPathPanelPhase, ColorPathPanelState, ColorPathPreviewStats,
    ExistingConnectionModeChoice, RouteToolPanelEffect, RouteToolPanelFollowUp,
};

impl ColorPathTool {
    /// Liefert den egui-freien Panelzustand des ColorPathTools.
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

        // Wizard-Flags: Engine ist Quelle der Wahrheit, Host liest nur.
        let can_accept = self.phase.is_finalized()
            && preview_stats
                .as_ref()
                .is_some_and(|stats| stats.can_accept);
        let can_back = matches!(
            self.phase,
            ColorPathPhase::CenterlinePreview
                | ColorPathPhase::JunctionEdit
                | ColorPathPhase::Finalize
        );
        let can_next = match self.phase {
            ColorPathPhase::Sampling => !self.sampling.sampled_colors.is_empty(),
            ColorPathPhase::CenterlinePreview | ColorPathPhase::JunctionEdit => true,
            ColorPathPhase::Idle | ColorPathPhase::Finalize => false,
        };

        ColorPathPanelState {
            phase: panel_phase(self.phase),
            sample_count: self.sampling.sampled_colors.len(),
            avg_color: self.sampling.avg_color,
            palette_colors: self.matching.palette.clone(),
            can_compute: !self.sampling.sampled_colors.is_empty(),
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
    #[allow(deprecated)] // Legacy-Aktionen bleiben fuer Host-Kompat bis CP-11.
    pub(super) fn apply_panel_action(
        &mut self,
        action: ColorPathPanelAction,
    ) -> RouteToolPanelEffect {
        // Legacy-Aliasse: ComputePreview/BackToSampling leiten auf die Wizard-Aktionen um,
        // damit externe Hosts weiter kompilieren. Die neue Semantik ist „ein Phasenschritt
        // nach vorn/zurueck" und nicht mehr der Sprung ueber mehrere Stages (CP-05).
        let action = match action {
            ColorPathPanelAction::ComputePreview => ColorPathPanelAction::NextPhase,
            ColorPathPanelAction::BackToSampling => ColorPathPanelAction::PrevPhase,
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
            // Wizard-Vorwaerts-Transitions (CP-05).
            ColorPathPanelAction::NextPhase => self.advance_wizard_phase(),
            // Wizard-Rueckwaerts-Transitions (CP-05).
            ColorPathPanelAction::PrevPhase => self.retreat_wizard_phase(),
            // Finales Uebernehmen: Hook auf den bestehenden Apply-Pfad via Controller.
            ColorPathPanelAction::Accept => {
                if self.can_accept_now() {
                    follow_up = Some(RouteToolPanelFollowUp::ReadyToExecute);
                    // `changed` bleibt false: das Panel wird durch den nachgelagerten
                    // Apply-/Reset-Pfad des Controllers ohnehin neu bewertet.
                    false
                } else {
                    false
                }
            }
            // Legacy-Aliasse sind oben bereits umgemappt; diese Arme sind nie erreichbar.
            ColorPathPanelAction::ComputePreview | ColorPathPanelAction::BackToSampling => false,
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

    /// Liefert `true`, wenn `Accept` gerade einen echten Commit ausloesen darf.
    fn can_accept_now(&self) -> bool {
        self.phase.is_finalized()
            && self
                .preview_data
                .as_ref()
                .is_some_and(|preview| !preview.prepared_segments.is_empty())
    }

    /// Wizard-Vorwaerts: Idle ist Endstation, Sampling loest Stage E aus,
    /// CenterlinePreview wechselt in den Junction-Edit, JunctionEdit loest
    /// Stage F aus, Finalize ist Endstation.
    fn advance_wizard_phase(&mut self) -> bool {
        match self.phase {
            ColorPathPhase::Idle | ColorPathPhase::Finalize => false,
            ColorPathPhase::Sampling => {
                if self.sampling.sampled_colors.is_empty()
                    || self.sampling.lasso_start_world.is_none()
                {
                    return false;
                }
                if self.rebuild_preview_core_only() {
                    self.sync_editable_from_network();
                    self.phase = ColorPathPhase::CenterlinePreview;
                    true
                } else {
                    log::warn!(
                        "ColorPathTool: NextPhase abgebrochen — kein exportierbares Netz"
                    );
                    false
                }
            }
            ColorPathPhase::CenterlinePreview => {
                // F5: reines Phase-Toggle (keine Strukturaenderung) soll den
                // Stage-F-Cache nicht invalidieren → keinen Revisions-Bump.
                self.phase = ColorPathPhase::JunctionEdit;
                true
            }
            ColorPathPhase::JunctionEdit => {
                if self.rebuild_stage_f_only() {
                    self.bump_editable_revision();
                    self.phase = ColorPathPhase::Finalize;
                    true
                } else {
                    log::warn!(
                        "ColorPathTool: NextPhase abgebrochen — Stage F lieferte keine Segmente"
                    );
                    false
                }
            }
        }
    }

    /// Wizard-Rueckwaerts: Finalize verwirft Stage F, JunctionEdit kehrt zur
    /// reinen Centerline-Preview zurueck, CenterlinePreview geht zurueck ins
    /// Sampling (und verwirft dabei die Preview-Pipeline). Sampling/Idle sind
    /// Endstationen.
    fn retreat_wizard_phase(&mut self) -> bool {
        match self.phase {
            ColorPathPhase::Idle | ColorPathPhase::Sampling => false,
            ColorPathPhase::CenterlinePreview => {
                self.clear_preview_pipeline();
                self.editable = None;
                self.phase = ColorPathPhase::Sampling;
                true
            }
            ColorPathPhase::JunctionEdit => {
                // F5: reines Phase-Toggle ohne Strukturaenderung — kein Bump.
                self.phase = ColorPathPhase::CenterlinePreview;
                true
            }
            ColorPathPhase::Finalize => {
                // Nur Stage F verwerfen — Netz/Skeleton bleibt fuer den erneuten
                // Eintritt in `Finalize` erhalten.
                if let Some(preview) = self.preview_data.as_mut() {
                    preview.prepared_segments.clear();
                }
                self.cache.prepared_segments_key = None;
                self.bump_editable_revision();
                self.phase = ColorPathPhase::JunctionEdit;
                true
            }
        }
    }

    /// Fuehrt den Wizard von der aktuellen Phase bis `Finalize` durch.
    ///
    /// Interner Helfer fuer Benchmark- und Testharnische, der den echten
    /// UI-Fluss `Sampling → CenterlinePreview → JunctionEdit → Finalize`
    /// ueber wiederholte [`advance_wizard_phase`] nachstellt — ohne reale
    /// egui-Panels und ohne Junction-Drag. Gibt `true` zurueck, sobald
    /// `Finalize` erreicht ist; `false`, wenn eine Transition unterwegs
    /// scheitert (z. B. fehlende Farbsamples oder leeres Stage-F-Resultat).
    pub(super) fn run_wizard_to_finalize(&mut self) -> bool {
        while self.phase != ColorPathPhase::Finalize {
            if !self.advance_wizard_phase() {
                return false;
            }
        }
        true
    }
}

/// Bildet die engine-interne Wizard-Phase auf die DTO-Phase ab.
///
/// CP-04 erweitert `ColorPathPanelPhase` additiv um die Wizard-Varianten
/// `CenterlinePreview`, `JunctionEdit` und `Finalize`. Die Legacy-Variante
/// `Preview` bleibt fuer alte FFI-Hosts bestehen, wird aber nicht mehr
/// emittiert. Damit sehen Hosts die echte Wizard-Phase und koennen eigene
/// Fallbacks (z.B. alles ab `CenterlinePreview` als Legacy-„preview" werten)
/// waehrend der Migration abbilden.
fn panel_phase(phase: ColorPathPhase) -> ColorPathPanelPhase {
    match phase {
        ColorPathPhase::Idle => ColorPathPanelPhase::Idle,
        ColorPathPhase::Sampling => ColorPathPanelPhase::Sampling,
        ColorPathPhase::CenterlinePreview => ColorPathPanelPhase::CenterlinePreview,
        ColorPathPhase::JunctionEdit => ColorPathPanelPhase::JunctionEdit,
        ColorPathPhase::Finalize => ColorPathPanelPhase::Finalize,
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
