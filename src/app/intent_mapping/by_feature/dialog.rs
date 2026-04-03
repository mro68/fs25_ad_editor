//! Intent-Mapping fuer Dialoge, Optionen und Overlays.

use crate::app::{AppCommand, AppIntent, AppState};

/// Mappt Dialog-Intents auf Commands.
pub(super) fn map(_state: &AppState, intent: AppIntent) -> Vec<AppCommand> {
    match intent {
        AppIntent::ExitRequested => vec![AppCommand::RequestExit],
        AppIntent::OpenOptionsDialogRequested => vec![AppCommand::OpenOptionsDialog],
        AppIntent::CloseOptionsDialogRequested => vec![AppCommand::CloseOptionsDialog],
        AppIntent::OptionsChanged { options } => vec![AppCommand::ApplyOptions { options }],
        AppIntent::ResetOptionsRequested => vec![AppCommand::ResetOptions],
        AppIntent::CommandPaletteToggled => vec![AppCommand::ToggleCommandPalette],
        AppIntent::ToggleFloatingMenu { .. } => vec![],
        other => unreachable!("unerwarteter Dialog-Intent: {other:?}"),
    }
}
