//! Builder fuer explizite Render-Assets aus dem AppState.

use crate::app::AppState;
use crate::shared::{
    RenderAssetSnapshot, RenderAssetsSnapshot, RenderBackgroundAssetSnapshot,
    RenderBackgroundWorldBounds,
};

/// Baut den host-neutralen Render-Asset-Snapshot aus dem aktuellen AppState.
///
/// Der Snapshot enthaelt langlebige Asset-Daten (derzeit Background) inklusive
/// monotoner Revisionen. Hosts koennen damit Uploads lokal synchronisieren,
/// ohne hostspezifischen Zustand in `AppState` zurueckzuschreiben.
pub fn build(state: &AppState) -> RenderAssetsSnapshot {
    let mut assets = Vec::with_capacity(1);

    if let Some(background) = state.view.background_map.as_deref() {
        let world_bounds = background.world_bounds();
        assets.push(RenderAssetSnapshot::background(RenderBackgroundAssetSnapshot {
            image: background.image_arc(),
            world_bounds: RenderBackgroundWorldBounds::new(
                world_bounds.min_x,
                world_bounds.max_x,
                world_bounds.min_z,
                world_bounds.max_z,
            ),
            scale: state.view.background_scale,
            asset_revision: state.view.background_asset_revision,
            transform_revision: state.view.background_transform_revision,
        }));
    }

    RenderAssetsSnapshot::new(
        state.view.background_asset_revision,
        state.view.background_transform_revision,
        assets,
    )
}

#[cfg(test)]
mod tests {
    use super::build;
    use crate::app::AppState;
    use crate::core::BackgroundMap;

    #[test]
    fn build_render_assets_without_background_is_empty() {
        let state = AppState::new();

        let snapshot = build(&state);

        assert_eq!(snapshot.background_asset_revision(), 0);
        assert_eq!(snapshot.background_transform_revision(), 0);
        assert!(snapshot.assets().is_empty());
        assert!(snapshot.background().is_none());
    }

    #[test]
    fn build_render_assets_includes_background_snapshot_and_revisions() {
        let mut state = AppState::new();
        let image = image::DynamicImage::new_rgba8(8, 8);
        let background = BackgroundMap::from_image(image, "test-image", None)
            .expect("Test-Background muss gebaut werden koennen");

        state.view.background_map = Some(std::sync::Arc::new(background));
        state.view.background_scale = 1.5;
        state.view.background_asset_revision = 2;
        state.view.background_transform_revision = 3;

        let snapshot = build(&state);
        let background = snapshot
            .background()
            .expect("Background-Snapshot erwartet");

        assert_eq!(snapshot.background_asset_revision(), 2);
        assert_eq!(snapshot.background_transform_revision(), 3);
        assert_eq!(background.asset_revision, 2);
        assert_eq!(background.transform_revision, 3);
        assert_eq!(background.scale, 1.5);
        assert_eq!(background.image.width(), 8);
        assert_eq!(background.image.height(), 8);
    }
}
