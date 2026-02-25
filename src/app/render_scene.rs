//! Builder für Render-Szenen aus dem AppState.

use crate::app::AppState;
use crate::app::use_cases::background_map::calculate_adjusted_opacity_for_zoom;
use crate::shared::RenderScene;
use std::collections::HashSet;
use std::sync::Arc;

/// Baut eine RenderScene aus dem aktuellen AppState.
pub fn build(state: &AppState, viewport_size: [f32; 2]) -> RenderScene {
    // Wenn Distanzen-Vorschau aktiv + hide_original → selektierte Nodes ausblenden
    let hidden_node_ids = if state.ui.distanzen.should_hide_original() {
        state.selection.selected_node_ids.clone()
    } else {
        Arc::new(HashSet::new())
    };

    // Berechne die Opacity mit Zoom-basierter Anpassung
    let adjusted_opacity = calculate_adjusted_opacity_for_zoom(
        state.view.background_opacity,
        state.view.camera.zoom,
        state.options.camera_zoom_min,
        state.options.background_opacity_at_min_zoom,
    );

    RenderScene {
        road_map: state.road_map.clone(),
        camera: state.view.camera.clone(),
        viewport_size,
        render_quality: state.view.render_quality,
        selected_node_ids: state.selection.selected_node_ids.clone(),
        connect_source_node: state.editor.connect_source_node,
        background_map: state.view.background_map.clone(),
        background_opacity: adjusted_opacity,
        background_visible: state.view.background_visible,
        options: state.options.clone(),
        hidden_node_ids,
    }
}
