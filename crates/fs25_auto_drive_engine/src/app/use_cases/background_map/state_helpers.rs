//! Interne Hilfsfunktionen fuer State-Mutation der Background-Map.

use crate::app::AppState;
use crate::core::BackgroundMap;
use std::sync::Arc;

pub(super) fn apply_background_map(state: &mut AppState, bg_map: BackgroundMap) {
    apply_background_map_with_scale(state, bg_map, 1.0);
}

pub(super) fn apply_background_map_with_scale(
    state: &mut AppState,
    bg_map: BackgroundMap,
    scale: f32,
) {
    let image_arc = bg_map.image_arc();
    state.view.background_map = Some(Arc::new(bg_map));
    state.view.background_scale = scale;
    state.view.mark_background_asset_changed();
    state.background_image = Some(image_arc);
}

pub(super) fn clear_background_assets(state: &mut AppState) {
    let had_background = state.view.background_map.is_some() || state.background_image.is_some();
    state.view.background_map = None;
    if had_background {
        state.view.mark_background_asset_changed();
    }
    state.background_image = None;
}

pub(super) fn persist_overview_defaults(state: &mut AppState) {
    if let Err(error) = super::super::options::save_editor_options(&state.options) {
        let message = format!(
            "Uebersichtskarten-Voreinstellungen konnten nicht gespeichert werden: {}",
            error
        );
        log::warn!("{}", message);
        state.ui.status_message = Some(message);
    }
}
