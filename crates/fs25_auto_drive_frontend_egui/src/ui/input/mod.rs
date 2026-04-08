//! Viewport-Input-Handling: Maus-Events, Drag-Selektion, Scroll → AppIntent.
//!
//! Aufgeteilt in phasenbasierte Submodule:
//! - `clicks` — Klick-Events (Einfach-/Doppel-Klick, Tool-Routing)
//! - `drag_primary` — Drag-Start/-Ende (Selektion-Move, Kamera-Pan, Route-Tool-Drag)
//! - `pointer_delta` — Pan/Move-Deltas waehrend aktiver Drags
//! - `zoom` — Scroll-Zoom auf Mausposition
//!
//! Interne Hilfsmodule:
//! - `state` — `InputState`, `PrimaryDragMode`, `ContextMenuSnapshot`
//! - `viewport_collect` — `ViewportContext`, `collect_viewport_events()`
//! - `helpers` — Konvertierungshilfsfunktionen

mod clicks;
mod drag_primary;
mod pointer_delta;
mod zoom;

mod helpers;
mod state;
mod viewport_collect;

use super::context_menu;
use super::drag::{draw_drag_selection_overlay, DragSelection};
use super::keyboard;
use crate::app::AppIntent;
use fs25_auto_drive_host_bridge::HostViewportInputBatch;

pub use state::InputState;
pub(crate) use state::PrimaryDragMode;
pub(crate) use viewport_collect::ViewportContext;
pub(crate) use helpers::{
    host_modifiers, host_pointer_button, host_tap_kind, screen_pos_to_world,
    to_viewport_screen_pos,
};

/// Ergebnis eines Viewport-Input-Sammeldurchlaufs.
#[derive(Debug, Default)]
pub struct ViewportInputEvents {
    /// Lokale Intents fuer nicht-bridge-faehige Gesten.
    pub intents: Vec<AppIntent>,
    /// Optionaler Batch bridge-faehiger Viewport-Input-Events.
    pub host_input_batch: Option<HostViewportInputBatch>,
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::app::state::DistanzenState;
    use fs25_auto_drive_host_bridge::{HostPointerButton, HostViewportInputEvent};
    use indexmap::IndexSet;

    const VIEWPORT_SIZE: [f32; 2] = [800.0, 600.0];
    const DRAG_START_POS: egui::Pos2 = egui::pos2(120.0, 120.0);
    const DRAG_MOVE_1: egui::Pos2 = egui::pos2(220.0, 180.0);
    const DRAG_MOVE_2: egui::Pos2 = egui::pos2(300.0, 240.0);
    const DRAG_MOVE_3: egui::Pos2 = egui::pos2(360.0, 300.0);

    struct FrameOutcome {
        intents: Vec<AppIntent>,
        host_events: Vec<HostViewportInputEvent>,
    }

    fn frame_input(events: Vec<egui::Event>, modifiers: egui::Modifiers) -> egui::RawInput {
        egui::RawInput {
            screen_rect: Some(egui::Rect::from_min_size(
                egui::Pos2::ZERO,
                egui::vec2(VIEWPORT_SIZE[0], VIEWPORT_SIZE[1]),
            )),
            modifiers,
            events,
            ..Default::default()
        }
    }

    fn pointer_move_input(pos: egui::Pos2, modifiers: egui::Modifiers) -> egui::RawInput {
        frame_input(vec![egui::Event::PointerMoved(pos)], modifiers)
    }

    fn pointer_button_input(
        pos: egui::Pos2,
        pressed: bool,
        modifiers: egui::Modifiers,
    ) -> egui::RawInput {
        frame_input(
            vec![egui::Event::PointerButton {
                pos,
                button: egui::PointerButton::Primary,
                pressed,
                modifiers,
            }],
            modifiers,
        )
    }

    fn collect_frame(
        ctx: &egui::Context,
        input_state: &mut InputState,
        raw_input: egui::RawInput,
        active_tool: EditorTool,
        route_tool_is_drawing: bool,
        tool_needs_lasso: bool,
    ) -> FrameOutcome {
        let options = EditorOptions::default();
        let selected_node_ids = IndexSet::new();
        let mut distanzen_state = DistanzenState::default();
        let mut outcome = None;

        let _ = ctx.run_ui(raw_input, |ui| {
            let (_, response) = ui.allocate_exact_size(
                egui::vec2(VIEWPORT_SIZE[0], VIEWPORT_SIZE[1]),
                egui::Sense::click_and_drag(),
            );
            let viewport_events = input_state.collect_viewport_events(
                ui,
                &response,
                VIEWPORT_SIZE,
                &Camera2D::default(),
                None,
                &selected_node_ids,
                active_tool,
                route_tool_is_drawing,
                false,
                &options,
                false,
                ConnectionDirection::default(),
                ConnectionPriority::default(),
                &[],
                &mut distanzen_state,
                None,
                false,
                false,
                false,
                None,
                tool_needs_lasso,
            );
            outcome = Some(FrameOutcome {
                intents: viewport_events.intents,
                host_events: viewport_events
                    .host_input_batch
                    .map(|batch| batch.events)
                    .unwrap_or_default(),
            });
        });

        outcome.expect("Viewport-Frame sollte ein Ergebnis liefern")
    }

    fn has_primary_drag_start(events: &[HostViewportInputEvent]) -> bool {
        events.iter().any(|event| {
            matches!(
                event,
                HostViewportInputEvent::DragStart {
                    button: HostPointerButton::Primary,
                    ..
                }
            )
        })
    }

    fn has_primary_drag_update(events: &[HostViewportInputEvent]) -> bool {
        events.iter().any(|event| {
            matches!(
                event,
                HostViewportInputEvent::DragUpdate {
                    button: HostPointerButton::Primary,
                    ..
                }
            )
        })
    }

    fn has_primary_drag_end(events: &[HostViewportInputEvent]) -> bool {
        events.iter().any(|event| {
            matches!(
                event,
                HostViewportInputEvent::DragEnd {
                    button: HostPointerButton::Primary,
                    ..
                }
            )
        })
    }

    fn has_primary_drag_lifecycle_event(events: &[HostViewportInputEvent]) -> bool {
        has_primary_drag_start(events)
            || has_primary_drag_update(events)
            || has_primary_drag_end(events)
    }

    /// Prüft, dass Shift-Drag weiterhin als Rechteck-Selektion über den Bridge-Drag-Lifecycle läuft.
    #[test]
    fn test_rect_drag_smoke_uses_bridge_lifecycle() {
        let ctx = egui::Context::default();
        let mut input_state = InputState::default();
        let modifiers = egui::Modifiers {
            shift: true,
            ..Default::default()
        };

        collect_frame(
            &ctx,
            &mut input_state,
            pointer_move_input(DRAG_START_POS, modifiers),
            EditorTool::Select,
            false,
            false,
        );
        collect_frame(
            &ctx,
            &mut input_state,
            pointer_button_input(DRAG_START_POS, true, modifiers),
            EditorTool::Select,
            false,
            false,
        );

        let start_frame = collect_frame(
            &ctx,
            &mut input_state,
            pointer_move_input(DRAG_MOVE_1, modifiers),
            EditorTool::Select,
            false,
            false,
        );
        assert_eq!(
            input_state
                .drag_selection
                .as_ref()
                .map(|selection| selection.mode),
            Some(super::super::drag::DragSelectionMode::Rect)
        );
        assert!(has_primary_drag_start(&start_frame.host_events));
        assert!(input_state.primary_drag_via_bridge);

        let update_frame = collect_frame(
            &ctx,
            &mut input_state,
            pointer_move_input(DRAG_MOVE_2, modifiers),
            EditorTool::Select,
            false,
            false,
        );
        assert!(has_primary_drag_update(&update_frame.host_events));

        let end_frame = collect_frame(
            &ctx,
            &mut input_state,
            pointer_button_input(DRAG_MOVE_2, false, modifiers),
            EditorTool::Select,
            false,
            false,
        );
        assert!(end_frame.intents.is_empty());
        assert!(has_primary_drag_end(&end_frame.host_events));
        assert!(input_state.drag_selection.is_none());
        assert!(!input_state.primary_drag_via_bridge);
    }

    /// Prüft, dass normales Alt-Drag weiterhin als Node-Lasso über den Bridge-Drag-Lifecycle läuft.
    #[test]
    fn test_lasso_drag_smoke_uses_bridge_lifecycle() {
        let ctx = egui::Context::default();
        let mut input_state = InputState::default();
        let modifiers = egui::Modifiers {
            alt: true,
            ..Default::default()
        };

        collect_frame(
            &ctx,
            &mut input_state,
            pointer_move_input(DRAG_START_POS, modifiers),
            EditorTool::Select,
            false,
            false,
        );
        collect_frame(
            &ctx,
            &mut input_state,
            pointer_button_input(DRAG_START_POS, true, modifiers),
            EditorTool::Select,
            false,
            false,
        );

        let start_frame = collect_frame(
            &ctx,
            &mut input_state,
            pointer_move_input(DRAG_MOVE_1, modifiers),
            EditorTool::Select,
            false,
            false,
        );
        assert_eq!(
            input_state
                .drag_selection
                .as_ref()
                .map(|selection| selection.mode),
            Some(super::super::drag::DragSelectionMode::Lasso)
        );
        assert!(has_primary_drag_start(&start_frame.host_events));
        assert!(input_state.primary_drag_via_bridge);

        let update_frame = collect_frame(
            &ctx,
            &mut input_state,
            pointer_move_input(DRAG_MOVE_2, modifiers),
            EditorTool::Select,
            false,
            false,
        );
        assert!(has_primary_drag_update(&update_frame.host_events));

        let end_frame = collect_frame(
            &ctx,
            &mut input_state,
            pointer_button_input(DRAG_MOVE_2, false, modifiers),
            EditorTool::Select,
            false,
            false,
        );
        assert!(end_frame.intents.is_empty());
        assert!(has_primary_drag_end(&end_frame.host_events));
        assert!(input_state.drag_selection.is_none());
        assert!(!input_state.primary_drag_via_bridge);
    }

    /// Prüft, dass Tool-Lasso lokal abgeschlossen wird und keinen Bridge-Drag-Lifecycle emittiert.
    #[test]
    fn test_tool_lasso_drag_smoke_emits_completed_intent_without_bridge_drag() {
        let ctx = egui::Context::default();
        let mut input_state = InputState::default();
        let modifiers = egui::Modifiers {
            alt: true,
            ..Default::default()
        };

        collect_frame(
            &ctx,
            &mut input_state,
            pointer_move_input(DRAG_START_POS, modifiers),
            EditorTool::Route,
            true,
            true,
        );
        collect_frame(
            &ctx,
            &mut input_state,
            pointer_button_input(DRAG_START_POS, true, modifiers),
            EditorTool::Route,
            true,
            true,
        );

        let start_frame = collect_frame(
            &ctx,
            &mut input_state,
            pointer_move_input(DRAG_MOVE_1, modifiers),
            EditorTool::Route,
            true,
            true,
        );
        assert_eq!(
            input_state
                .drag_selection
                .as_ref()
                .map(|selection| selection.mode),
            Some(super::super::drag::DragSelectionMode::ToolLasso)
        );
        assert!(!has_primary_drag_lifecycle_event(&start_frame.host_events));
        assert!(!input_state.primary_drag_via_bridge);

        let update_frame_1 = collect_frame(
            &ctx,
            &mut input_state,
            pointer_move_input(DRAG_MOVE_2, modifiers),
            EditorTool::Route,
            true,
            true,
        );
        assert!(!has_primary_drag_lifecycle_event(
            &update_frame_1.host_events
        ));

        let update_frame_2 = collect_frame(
            &ctx,
            &mut input_state,
            pointer_move_input(DRAG_MOVE_3, modifiers),
            EditorTool::Route,
            true,
            true,
        );
        assert!(!has_primary_drag_lifecycle_event(
            &update_frame_2.host_events
        ));

        let end_frame = collect_frame(
            &ctx,
            &mut input_state,
            pointer_button_input(DRAG_MOVE_3, false, modifiers),
            EditorTool::Route,
            true,
            true,
        );
        assert!(!has_primary_drag_lifecycle_event(&end_frame.host_events));

        let polygon = end_frame.intents.iter().find_map(|intent| match intent {
            AppIntent::RouteToolLassoCompleted { polygon } => Some(polygon),
            _ => None,
        });
        let polygon = polygon.expect("Tool-Lasso sollte ein Abschluss-Intent emittieren");
        assert_eq!(polygon.len(), 3);
        assert!(polygon.windows(2).all(|window| {
            let [left, right] = window else {
                return true;
            };
            left.x < right.x && left.y < right.y
        }));
        assert!(input_state.drag_selection.is_none());
        assert!(!input_state.primary_drag_via_bridge);
    }
}
