//! Egui-freie Panel-Bruecke fuer das Ausweichstrecken-Tool.

use super::geometry::compute_bypass_positions;
use super::state::BypassTool;
use crate::app::ui_contract::{
    BypassPanelAction, BypassPanelState, RouteToolPanelEffect, BYPASS_BASE_SPACING_LIMITS,
    BYPASS_OFFSET_LIMITS,
};

impl BypassTool {
    /// Liefert den egui-freien Panelzustand des Ausweichstrecken-Tools.
    pub(super) fn panel_state(&self) -> BypassPanelState {
        let preview = self
            .has_chain()
            .then(|| {
                compute_bypass_positions(&self.chain_positions, self.offset, self.base_spacing)
            })
            .flatten();

        BypassPanelState {
            has_chain: self.has_chain(),
            offset: self.offset,
            base_spacing: self.base_spacing,
            new_node_count: preview.as_ref().map(|(positions, _)| positions.len()),
            chain_node_count: self.chain_positions.len(),
            transition_length_m: preview
                .and_then(|(_, d_blend)| (d_blend > 0.0).then_some(d_blend)),
        }
    }

    /// Wendet eine semantische Panel-Aktion auf das Ausweichstrecken-Tool an.
    pub(super) fn apply_panel_action(&mut self, action: BypassPanelAction) -> RouteToolPanelEffect {
        let changed = match action {
            BypassPanelAction::SetOffset(value) => {
                set_f32(&mut self.offset, BYPASS_OFFSET_LIMITS.clamp(value))
            }
            BypassPanelAction::SetBaseSpacing(value) => set_f32(
                &mut self.base_spacing,
                BYPASS_BASE_SPACING_LIMITS.clamp(value),
            ),
        };
        if changed {
            self.cached_positions = None;
            self.cached_connections = None;
        }
        RouteToolPanelEffect {
            changed,
            needs_recreate: false,
            next_action: None,
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
