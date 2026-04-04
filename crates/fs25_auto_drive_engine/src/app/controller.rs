//! Application Controller fuer zentrale Event-Verarbeitung.

mod by_feature;

use super::render_assets;
use super::render_scene;
use super::ui_contract::{
    CommandPalettePanelState, HostUiSnapshot, OptionsPanelState, PanelState,
};
use super::{AppCommand, AppIntent, AppState};
use crate::shared::{RenderAssetsSnapshot, RenderScene};

/// Orchestriert UI-Events und Use-Cases auf den AppState.
#[derive(Default)]
pub struct AppController;

impl AppController {
    /// Erstellt einen neuen Controller.
    pub fn new() -> Self {
        Self
    }

    /// Verarbeitet einen Intent ueber Intent->Command Mapping.
    pub fn handle_intent(&mut self, state: &mut AppState, intent: AppIntent) -> anyhow::Result<()> {
        let commands = self.map_intent_to_commands(state, intent);
        for command in commands {
            self.handle_command(state, command)?;
        }

        Ok(())
    }

    fn map_intent_to_commands(&self, state: &AppState, intent: AppIntent) -> Vec<AppCommand> {
        super::intent_mapping::map_intent_to_commands(state, intent)
    }

    /// Fuehrt mutierende Commands auf dem AppState aus.
    /// Dispatcht an Feature-Handler in `handlers/`.
    pub fn handle_command(
        &mut self,
        state: &mut AppState,
        command: AppCommand,
    ) -> anyhow::Result<()> {
        state.command_log.record(&command);
        by_feature::handle(state, command)
    }

    /// Baut die Render-Szene aus dem aktuellen AppState.
    pub fn build_render_scene(&self, state: &AppState, viewport_size: [f32; 2]) -> RenderScene {
        render_scene::build(state, viewport_size)
    }

    /// Baut den host-neutralen Render-Asset-Snapshot aus dem aktuellen AppState.
    pub fn build_render_assets(&self, state: &AppState) -> RenderAssetsSnapshot {
        render_assets::build(state)
    }

    /// Baut den host-neutralen UI-Snapshot fuer Dialoge und Tool-Fenster.
    pub fn build_host_ui_snapshot(&self, state: &AppState) -> HostUiSnapshot {
        let mut panels = Vec::new();

        panels.push(PanelState::CommandPalette(CommandPalettePanelState {
            visible: state.ui.show_command_palette,
        }));

        panels.push(PanelState::Options(OptionsPanelState {
            visible: state.ui.show_options_dialog,
            options: state.options.clone(),
        }));

        if let Some(route_tool_panel) = state.editor.route_tool_panel_state() {
            panels.push(PanelState::RouteTool(route_tool_panel));
        }

        HostUiSnapshot {
            panels,
            dialog_requests: state.ui.dialog_requests.clone(),
        }
    }
}
