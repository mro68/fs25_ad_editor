//! Dispatch-Gruppe der `HostBridgeSession`: `apply_action`/`apply_intent` sowie
//! Komfort-Actions (Toolwechsel, Undo/Redo, Dialog-Drain). Reine interne
//! Aufteilung — die oeffentliche Session-Surface bleibt unveraendert.

use anyhow::Result;
use fs25_auto_drive_engine::app::AppIntent;

use super::HostBridgeSession;
use crate::dto::{HostActiveTool, HostDialogRequest, HostDialogResult, HostSessionAction};

impl HostBridgeSession {
    /// Wendet eine explizite Host-Aktion auf die Session an.
    ///
    /// Die Methode delegiert auf die gemeinsame Rust-Host-Dispatch-Seam in
    /// `crate::dispatch::apply_host_action(...)` und markiert den Snapshot-
    /// Cache nur nach erfolgreich verarbeiteten Aktionen als dirty.
    pub fn apply_action(&mut self, action: HostSessionAction) -> Result<()> {
        if let HostSessionAction::QueryNodeDetails { node_id } = action {
            self.set_inspected_node_id(Some(node_id));
            return Ok(());
        }

        self.reconcile_host_local_dialog_state_for_action(&action);

        let handled = crate::dispatch::apply_host_action_with_viewport_input_state(
            &mut self.controller,
            &mut self.state,
            &mut self.viewport_input_state,
            action,
        )?;
        if handled {
            self.snapshot_dirty = true;
            self.drain_engine_requests();
            self.sync_chrome_from_engine();
        }
        Ok(())
    }

    /// Verarbeitet einen `AppIntent` direkt ueber den App-Controller.
    ///
    /// Diese Methode bleibt als Uebergangs-Seam fuer Hosts, die bereits auf
    /// Session-Ownership umgestellt sind, aber noch nicht alle Schreibpfade auf
    /// `HostSessionAction` umgehangen haben. Vor der Verarbeitung werden fuer
    /// intentsensitive Host-Dialoge die lokalen Draft-Werte in den Engine-State
    /// zurueckgespiegelt.
    pub fn apply_intent(&mut self, intent: AppIntent) -> Result<()> {
        self.reconcile_host_local_dialog_state_for_intent(&intent);
        self.controller.handle_intent(&mut self.state, intent)?;
        self.snapshot_dirty = true;
        self.drain_engine_requests();
        self.sync_chrome_from_engine();
        Ok(())
    }

    /// Schaltet die Command-Palette um.
    pub fn toggle_command_palette(&mut self) -> Result<()> {
        self.apply_action(HostSessionAction::ToggleCommandPalette)
    }

    /// Setzt das aktive Editor-Tool.
    pub fn set_editor_tool(&mut self, tool: HostActiveTool) -> Result<()> {
        self.apply_action(HostSessionAction::SetEditorTool { tool })
    }

    /// Oeffnet oder schliesst den Optionen-Dialog explizit.
    pub fn set_options_dialog_visible(&mut self, visible: bool) -> Result<()> {
        let action = if visible {
            HostSessionAction::OpenOptionsDialog
        } else {
            HostSessionAction::CloseOptionsDialog
        };
        self.apply_action(action)
    }

    /// Fuehrt einen Undo-Schritt aus.
    pub fn undo(&mut self) -> Result<()> {
        self.apply_action(HostSessionAction::Undo)
    }

    /// Fuehrt einen Redo-Schritt aus.
    pub fn redo(&mut self) -> Result<()> {
        self.apply_action(HostSessionAction::Redo)
    }

    /// Entnimmt alle aktuell ausstehenden Dialog-Anforderungen als Bridge-DTOs.
    ///
    /// Dies ist die kanonische oeffentliche Dialog-Drain-Seam der Bridge fuer
    /// Hosts ohne direkten Zugriff auf `AppController` und `AppState`.
    /// Chrome-Sichtbarkeits-Requests werden hier NICHT zurueckgegeben — sie
    /// werden durch `drain_engine_requests()` direkt in `chrome_state` verarbeitet.
    pub fn take_dialog_requests(&mut self) -> Vec<HostDialogRequest> {
        let requests = std::mem::take(&mut self.pending_dialog_requests);
        if !requests.is_empty() {
            self.snapshot_dirty = true;
        }
        requests
    }

    /// Reicht ein host-seitiges Dialog-Ergebnis an die Engine weiter.
    ///
    /// Dies ist das semantische Gegenstueck zur Dialog-Drain-Seam
    /// `take_dialog_requests()`.
    pub fn submit_dialog_result(&mut self, result: HostDialogResult) -> Result<()> {
        self.apply_action(HostSessionAction::SubmitDialogResult { result })
    }
}
