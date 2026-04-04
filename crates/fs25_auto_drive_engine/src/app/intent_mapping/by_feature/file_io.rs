//! Intent-Mapping fuer Datei- und XML-I/O.

use crate::app::{AppCommand, AppIntent, AppState};

/// Mappt Datei-I/O-Intents auf Commands.
pub(super) fn map(_state: &AppState, intent: AppIntent) -> Vec<AppCommand> {
    match intent {
        AppIntent::OpenFileRequested => vec![AppCommand::RequestOpenFileDialog],
        AppIntent::SaveRequested => vec![AppCommand::SaveFile { path: None }],
        AppIntent::SaveAsRequested => vec![AppCommand::RequestSaveFileDialog],
        AppIntent::HeightmapSelectionRequested => vec![
            AppCommand::DismissHeightmapWarning,
            AppCommand::RequestHeightmapDialog,
        ],
        AppIntent::HeightmapCleared => vec![AppCommand::ClearHeightmap],
        AppIntent::HeightmapWarningConfirmed => vec![
            AppCommand::ConfirmAndSaveFile,
            AppCommand::DismissHeightmapWarning,
        ],
        AppIntent::HeightmapWarningCancelled => vec![AppCommand::DismissHeightmapWarning],
        AppIntent::FileSelected { path } => vec![AppCommand::LoadFile { path }],
        AppIntent::SaveFilePathSelected { path } => vec![AppCommand::SaveFile { path: Some(path) }],
        AppIntent::HeightmapSelected { path } => vec![AppCommand::SetHeightmap { path }],
        AppIntent::DeduplicateConfirmed => vec![AppCommand::DeduplicateNodes],
        AppIntent::DeduplicateCancelled => vec![AppCommand::DismissDeduplicateDialog],
        other => unreachable!("unerwarteter FileIo-Intent: {other:?}"),
    }
}
