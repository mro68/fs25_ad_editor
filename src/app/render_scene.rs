//! Builder für Render-Szenen aus dem AppState.

use crate::app::use_cases::background_map::calculate_background_opacity_for_zoom;
use crate::app::AppState;
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

    // Berechne Hintergrund-Opacity basierend auf Zoom und Optionen
    let background_opacity = calculate_background_opacity_for_zoom(
        state.options.background_opacity_normal,
        state.options.background_opacity_min_zoom,
        state.view.camera.zoom,
        state.options.camera_zoom_min,
        state.options.background_fade_start_zoom,
    );

    RenderScene {
        road_map: state.road_map.clone(),
        camera: state.view.camera.clone(),
        viewport_size,
        render_quality: state.view.render_quality,
        selected_node_ids: state.selection.selected_node_ids.clone(),
        connect_source_node: state.editor.connect_source_node,
        background_map: state.view.background_map.clone(),
        background_opacity,
        background_visible: state.view.background_visible,
        options: state.options.clone(),
        hidden_node_ids,
    }
}

#[cfg(test)]
mod tests {
    use super::build;
    use crate::app::AppState;

    #[test]
    fn build_applies_zoom_based_opacity() {
        let mut state = AppState::new();
        // Standard: opacity_normal=1.0, opacity_min_zoom=0.2, fade_start_zoom=3.0, zoom_min=0.1

        // Zoom über fade_start → volle Opacity
        state.view.camera.zoom = 5.0;
        let scene = build(&state, [1280.0, 720.0]);
        assert!((scene.background_opacity - 1.0).abs() < 0.01);

        // Zoom genau am Minimum → min_zoom-Opacity
        state.view.camera.zoom = 0.1;
        let scene = build(&state, [1280.0, 720.0]);
        assert!((scene.background_opacity - 0.2).abs() < 0.01);

        // Zoom genau bei fade_start → volle Opacity
        state.view.camera.zoom = 3.0;
        let scene = build(&state, [1280.0, 720.0]);
        assert!((scene.background_opacity - 1.0).abs() < 0.01);

        // Zoom mitten im Fade-Bereich (z.B. 1.55 zwischen 0.1 und 3.0)
        state.view.camera.zoom = 1.55;
        let scene = build(&state, [1280.0, 720.0]);
        assert!(scene.background_opacity > 0.2 && scene.background_opacity < 1.0);
    }
}
