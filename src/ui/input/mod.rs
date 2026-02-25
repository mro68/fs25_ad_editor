//! Viewport-Input-Handling: Maus-Events, Drag-Selektion, Scroll → AppIntent.
//!
//! Aufgeteilt in phasenbasierte Submodule:
//! - `clicks` — Klick-Events (Einfach-/Doppel-Klick, Tool-Routing)
//! - `drag_primary` — Drag-Start/-Ende (Selektion-Move, Kamera-Pan, Route-Tool-Drag)
//! - `pointer_delta` — Pan/Move-Deltas während aktiver Drags
//! - `zoom` — Scroll-Zoom auf Mausposition

mod clicks;
mod drag_primary;
mod pointer_delta;
mod zoom;

use super::context_menu;
use super::drag::{draw_drag_selection_overlay, DragSelection};
use super::keyboard;
use crate::app::{AppIntent, Camera2D, EditorTool, RoadMap};
use crate::shared::EditorOptions;
use std::collections::HashSet;

/// Modus des primären (Links-)Drags im Viewport.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub(crate) enum PrimaryDragMode {
    #[default]
    None,
    SelectionMove,
    CameraPan,
    /// Drag eines Route-Tool-Steuerpunkts (Anker/CP)
    RouteToolPointDrag,
}

/// Bündelt die gemeinsamen Parameter für Viewport-Event-Verarbeitung.
pub(crate) struct ViewportContext<'a> {
    pub ui: &'a egui::Ui,
    pub response: &'a egui::Response,
    pub viewport_size: [f32; 2],
    pub camera: &'a Camera2D,
    pub road_map: Option<&'a RoadMap>,
    pub selected_node_ids: &'a HashSet<u64>,
    pub active_tool: EditorTool,
    pub options: &'a EditorOptions,
    pub drag_targets: &'a [glam::Vec2],
}

/// Verwaltet den Input-Zustand für das Viewport (Drag, Selektion, Scroll)
#[derive(Default)]
pub struct InputState {
    pub(crate) primary_drag_mode: PrimaryDragMode,
    pub(crate) drag_selection: Option<DragSelection>,
}

impl InputState {
    /// Erstellt einen neuen, leeren Input-Zustand.
    pub fn new() -> Self {
        Self {
            primary_drag_mode: PrimaryDragMode::None,
            drag_selection: None,
        }
    }

    #[allow(clippy::too_many_arguments)]
    /// Sammelt Viewport-Events aus egui-Input und gibt AppIntents zurück.
    ///
    /// Diese Methode ist der zentrale UI→Intent-Einstieg für Maus-, Scroll-
    /// und Drag-Interaktionen im Viewport.
    ///
    /// `drag_targets` enthält die Weltpositionen verschiebbarer Punkte
    /// des aktiven Route-Tools (leer wenn kein Tool aktiv oder keine Targets).
    pub fn collect_viewport_events(
        &mut self,
        ui: &egui::Ui,
        response: &egui::Response,
        viewport_size: [f32; 2],
        camera: &Camera2D,
        road_map: Option<&RoadMap>,
        selected_node_ids: &HashSet<u64>,
        active_tool: EditorTool,
        route_tool_is_drawing: bool,
        options: &EditorOptions,
        drag_targets: &[glam::Vec2],
        distanzen_state: &mut crate::app::state::DistanzenState,
    ) -> Vec<AppIntent> {
        let ctx = ViewportContext {
            ui,
            response,
            viewport_size,
            camera,
            road_map,
            selected_node_ids,
            active_tool,
            options,
            drag_targets,
        };

        let mut events = Vec::new();

        events.push(AppIntent::ViewportResized {
            size: viewport_size,
        });

        // Keyboard-Shortcuts (ausgelagert in keyboard.rs)
        events.extend(keyboard::collect_keyboard_intents(
            ui,
            selected_node_ids,
            active_tool,
            route_tool_is_drawing,
            distanzen_state.active,
        ));

        let modifiers = ui.input(|i| i.modifiers);

        self.handle_drag_start(&ctx, modifiers, &mut events);
        self.handle_drag_update(&ctx);
        self.handle_drag_end(&ctx, &mut events);
        self.handle_clicks(&ctx, modifiers, &mut events);
        self.handle_pointer_delta(&ctx, &mut events);

        // Drag-Selektion Overlay (ausgelagert in drag.rs)
        draw_drag_selection_overlay(self.drag_selection.as_ref(), ui, response);

        // Neues Context-Menu-System: Alle 5 Varianten über einheitlichen Router
        let pointer_pos_world = response.hover_pos().map(|screen_pos| {
            let local = screen_pos - response.rect.min;
            camera.screen_to_world(
                glam::Vec2::new(local.x, local.y),
                glam::Vec2::new(viewport_size[0], viewport_size[1]),
            )
        });

        context_menu::show_viewport_context_menu(
            response,
            road_map,
            selected_node_ids,
            distanzen_state,
            pointer_pos_world,
            route_tool_is_drawing && active_tool == EditorTool::Route,
            &mut events,
        );

        self.handle_scroll_zoom(&ctx, &mut events);

        events
    }
}

/// Rechnet eine Bildschirmposition in Weltkoordinaten um.
pub(crate) fn screen_pos_to_world(
    pointer_pos: egui::Pos2,
    response: &egui::Response,
    viewport_size: [f32; 2],
    camera: &Camera2D,
) -> glam::Vec2 {
    let local = pointer_pos - response.rect.min;
    camera.screen_to_world(
        glam::Vec2::new(local.x, local.y),
        glam::Vec2::new(viewport_size[0], viewport_size[1]),
    )
}
