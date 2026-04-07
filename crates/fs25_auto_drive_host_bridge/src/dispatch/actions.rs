//! Apply-Funktionen fuer Host-Aktionen ueber AppController und AppState.

use anyhow::{anyhow, Result};
use fs25_auto_drive_engine::app::{AppController, AppIntent, AppState};

use crate::dto::{HostSessionAction, HostViewportInputBatch};

use super::mappings::map_host_action_to_intent;
use super::viewport_input::{apply_viewport_input_batch, HostViewportInputState};

fn apply_stateless_host_action(
    controller: &mut AppController,
    state: &mut AppState,
    action: HostSessionAction,
) -> Result<bool> {
    let Some(intent) = map_host_action_to_intent(action) else {
        return Ok(false);
    };

    controller.handle_intent(state, intent)?;
    Ok(true)
}

/// Wendet eine Host-Action inklusive stateful Viewport-Input auf Controller und State an.
pub fn apply_host_action_with_viewport_input_state(
    controller: &mut AppController,
    state: &mut AppState,
    input_state: &mut HostViewportInputState,
    action: HostSessionAction,
) -> Result<bool> {
    match action {
        HostSessionAction::SubmitViewportInput { batch } => {
            apply_viewport_input_batch(controller, state, input_state, batch)
        }
        other => apply_stateless_host_action(controller, state, other),
    }
}

/// Wendet die gemeinsame Rust-Host-Dispatch-Seam auf Controller und State an.
///
/// Rueckgabe:
/// - `Ok(true)`: Es wurde ein Intent erzeugt und erfolgreich verarbeitet.
/// - `Ok(false)`: Die Action war semantisch ein No-Op ohne Intent.
pub fn apply_host_action(
    controller: &mut AppController,
    state: &mut AppState,
    action: HostSessionAction,
) -> Result<bool> {
    match action {
        HostSessionAction::SubmitViewportInput { .. } => Err(anyhow!(
            "SubmitViewportInput requires HostViewportInputState; use apply_host_action_with_viewport_input_state(...) or HostBridgeSession"
        )),
        other => apply_stateless_host_action(controller, state, other),
    }
}

/// Wendet einen stabil gemappten Engine-Intent ueber die Host-Bridge-Seam an.
///
/// Rueckgabe:
/// - `Ok(true)`: Der Intent wurde auf eine Host-Action gemappt und verarbeitet.
/// - `Ok(false)`: Der Intent gehoert nicht zur stabilen Host-Action-Surface.
pub fn apply_mapped_intent(
    controller: &mut AppController,
    state: &mut AppState,
    intent: &AppIntent,
) -> Result<bool> {
    use super::mappings::map_intent_to_host_action;

    let Some(action) = map_intent_to_host_action(intent) else {
        return Ok(false);
    };

    apply_host_action(controller, state, action)
}

// Stellt sicher, dass HostViewportInputBatch im Modul sichtbar bleibt (Lint-Unterdrückung).
const _: fn() = || {
    let _ = std::mem::size_of::<HostViewportInputBatch>();
};
