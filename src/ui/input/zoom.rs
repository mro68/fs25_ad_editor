//! Scroll-Zoom auf Mausposition.

use super::{screen_pos_to_world, InputState, ViewportContext};
use crate::app::AppIntent;

impl InputState {
    /// Verarbeitet Scroll-Zoom auf die aktuelle Mausposition.
    pub(crate) fn handle_scroll_zoom(&self, ctx: &ViewportContext, events: &mut Vec<AppIntent>) {
        let scroll = ctx.ui.input(|i| i.smooth_scroll_delta.y);
        if scroll == 0.0 {
            return;
        }

        let step = ctx.options.camera_scroll_zoom_step;
        let factor = if scroll > 0.0 { step } else { 1.0 / step };
        let focus_world = ctx
            .response
            .hover_pos()
            .map(|pos| screen_pos_to_world(pos, ctx.response, ctx.viewport_size, ctx.camera));
        events.push(AppIntent::CameraZoom {
            factor,
            focus_world,
        });
    }
}
