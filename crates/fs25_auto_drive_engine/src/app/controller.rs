//! Application Controller fuer zentrale Event-Verarbeitung.

mod by_feature;

use super::ui_contract::DialogRequest;
use super::{AppCommand, AppIntent, AppState};

/// Orchestriert UI-Events und Use-Cases auf den AppState.
#[derive(Default)]
pub struct AppController;

impl AppController {
    /// Erstellt einen neuen Controller.
    pub fn new() -> Self {
        Self
    }

    /// Verarbeitet einen Intent ueber Intent->Command Mapping.
    ///
    /// Diese Methode ist das Engine-Ende der gemeinsamen Rust-Host-Dispatch-
    /// Seam: Aeussere Host-Adapter speisen hier nur bereits gemappte
    /// `AppIntent`s ein.
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

    /// Entnimmt alle aktuell ausstehenden host-nativen Dialog-Anforderungen.
    ///
    /// Diese Drain-Seam ist die kanonische Quelle fuer Datei-/Pfaddialoge
    /// ueber alle Hosts hinweg. Host-Adapter sollen diese Methode statt eines
    /// direkten Zugriffs auf `UiState::take_dialog_requests()` verwenden.
    pub fn take_dialog_requests(&self, state: &mut AppState) -> Vec<DialogRequest> {
        state.ui.take_dialog_requests()
    }
}
