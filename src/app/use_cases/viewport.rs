//! Use-Case-Funktionen für Viewport-Zustand.

use crate::app::AppState;
use crate::shared::RenderQuality;

/// Aktualisiert die gespeicherte Viewport-Größe.
pub fn resize(state: &mut AppState, size: [f32; 2]) {
    state.view.viewport_size = size;
}

/// Aktualisiert die Render-Qualitätsstufe.
pub fn set_render_quality(state: &mut AppState, quality: RenderQuality) {
    state.view.render_quality = quality;
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn resize_updates_viewport_size() {
        let mut state = AppState::new();

        resize(&mut state, [1920.0, 1080.0]);

        assert_eq!(state.view.viewport_size, [1920.0, 1080.0]);
    }

    #[test]
    fn set_render_quality_updates_quality() {
        let mut state = AppState::new();
        assert_eq!(state.view.render_quality, RenderQuality::High);

        set_render_quality(&mut state, RenderQuality::Low);

        assert_eq!(state.view.render_quality, RenderQuality::Low);
    }
}
