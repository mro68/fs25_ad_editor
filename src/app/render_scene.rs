//! Builder für Render-Szenen aus dem AppState.

use crate::app::AppState;
use crate::shared::RenderScene;

/// Baut eine RenderScene aus dem aktuellen AppState.
pub fn build(state: &AppState, viewport_size: [f32; 2]) -> RenderScene {
    RenderScene {
        road_map: state.road_map.clone(),
        camera: state.view.camera.clone(),
        viewport_size,
        render_quality: state.view.render_quality,
        selected_node_ids: state.selection.selected_node_ids.iter().copied().collect(),
        connect_source_node: state.editor.connect_source_node,
        background_map: state.view.background_map.clone(),
        background_opacity: state.view.background_opacity,
        background_visible: state.view.background_visible,
        options: state.options.clone(),
    }
}
