//! Intent-Mapping fuer Kamera, Viewport und Background-/Overview-Features.

use crate::app::{AppCommand, AppIntent, AppState};

/// Mappt View-Intents auf Commands.
pub(super) fn map(state: &AppState, intent: AppIntent) -> Vec<AppCommand> {
    match intent {
        AppIntent::ResetCameraRequested => vec![AppCommand::ResetCamera],
        AppIntent::ZoomInRequested => vec![AppCommand::ZoomIn],
        AppIntent::ZoomOutRequested => vec![AppCommand::ZoomOut],
        AppIntent::ViewportResized { size } => vec![AppCommand::SetViewportSize { size }],
        AppIntent::CameraPan { delta } => vec![AppCommand::PanCamera { delta }],
        AppIntent::CameraZoom {
            factor,
            focus_world,
        } => vec![AppCommand::ZoomCamera {
            factor,
            focus_world,
        }],
        AppIntent::CenterOnNodeRequested { node_id } => vec![AppCommand::CenterOnNode { node_id }],
        AppIntent::RenderQualityChanged { quality } => {
            vec![AppCommand::SetRenderQuality { quality }]
        }
        AppIntent::BackgroundMapSelectionRequested => vec![AppCommand::RequestBackgroundMapDialog],
        AppIntent::BackgroundMapSelected { path, crop_size } => {
            vec![AppCommand::LoadBackgroundMap { path, crop_size }]
        }
        AppIntent::ToggleBackgroundVisibility => vec![AppCommand::ToggleBackgroundVisibility],
        AppIntent::ScaleBackground { factor } => vec![AppCommand::ScaleBackground { factor }],
        AppIntent::ZipBackgroundBrowseRequested { path } => {
            vec![AppCommand::BrowseZipBackground { path }]
        }
        AppIntent::ZipBackgroundFileSelected {
            zip_path,
            entry_name,
        } => vec![AppCommand::LoadBackgroundFromZip {
            zip_path,
            entry_name,
            crop_size: None,
        }],
        AppIntent::ZipBrowserCancelled => vec![AppCommand::CloseZipBrowser],
        AppIntent::GenerateOverviewRequested => vec![AppCommand::OpenOverviewSourceDialog],
        AppIntent::OverviewZipBrowseRequested => vec![AppCommand::RequestOverviewDialog],
        AppIntent::GenerateOverviewFromZip { path } => vec![
            AppCommand::DismissPostLoadDialog,
            AppCommand::OpenOverviewOptionsDialog { path },
        ],
        AppIntent::OverviewOptionsConfirmed => vec![AppCommand::GenerateOverviewWithOptions],
        AppIntent::OverviewOptionsCancelled => vec![AppCommand::CloseOverviewOptionsDialog],
        AppIntent::PostLoadDialogDismissed => vec![AppCommand::DismissPostLoadDialog],
        AppIntent::SaveBackgroundAsOverviewConfirmed => {
            let path = state.ui.save_overview_dialog.target_path.clone();
            vec![
                AppCommand::SaveBackgroundAsOverview { path },
                AppCommand::DismissSaveOverviewDialog,
            ]
        }
        AppIntent::SaveBackgroundAsOverviewDismissed => vec![AppCommand::DismissSaveOverviewDialog],
        AppIntent::ZoomToFitRequested => vec![AppCommand::ZoomToFit],
        AppIntent::ZoomToSelectionBoundsRequested => vec![AppCommand::ZoomToSelectionBounds],
        other => unreachable!("unerwarteter View-Intent: {other:?}"),
    }
}
