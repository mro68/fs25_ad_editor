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
use crate::app::tools::common::TangentMenuData;
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

/// Immutable Snapshot des Kontextmenü-States beim Rechtsklick.
///
/// Dieser Snapshot gefriert den kompletten Zustand ein, der zum Zeitpunkt
/// des Rechtsklicks galt — damit Menüinhalt stabil bleibt, solange das
/// Popup offen ist. Zustandsänderungen (Escape, Deselection etc.) beeinflussen
/// nicht das bereits offene Menü.
#[derive(Debug, Clone)]
struct ContextMenuSnapshot {
    /// Eingefrorene Menu-Variante
    variant: context_menu::MenuVariant,
    /// Eingefrorene Selection-Menge
    selection: HashSet<u64>,
    /// Pointer-Position in Weltkoordinaten beim Rechtsklick
    /// (für zukünftige kontextabhängige Menu-Rendering-Features)
    #[allow(dead_code)]
    pointer_pos_world: Option<glam::Vec2>,
}

/// Verwaltet den Input-Zustand für das Viewport (Drag, Selektion, Scroll)
#[derive(Default)]
pub struct InputState {
    pub(crate) primary_drag_mode: PrimaryDragMode,
    pub(crate) drag_selection: Option<DragSelection>,
    /// Snapshot des Menü-Zustands, gültig solange das Popup offen ist.
    /// Wird beim Rechtsklick gesetzt und erst geleert, wenn egui das Popup schließt.
    context_menu_snapshot: Option<ContextMenuSnapshot>,
}

/// Berechnet den vorhergesagten Selektions-State nach einem RMT-Klick.
///
/// Reine Berechnung im UI-Layer (keine State-Mutation), damit der Snapshot
/// sofort mit dem korrekten Menü erstellt werden kann. Die tatsächliche
/// State-Mutation erfolgt asynchron über `ContextMenuPick` → Controller.
fn predict_selection_after_rmt(
    current: &HashSet<u64>,
    clicked_node: Option<u64>,
    toggle: bool,
) -> (HashSet<u64>, Option<u64>) {
    let Some(nid) = clicked_node else {
        // Klick ins Leere: Selektion beibehalten, kein Fokus
        return (current.clone(), None);
    };

    if toggle {
        // Ctrl+RMT: Toggle
        if current.contains(&nid) {
            let mut predicted = current.clone();
            predicted.remove(&nid);
            (predicted, None)
        } else {
            let mut predicted = current.clone();
            predicted.insert(nid);
            (predicted, Some(nid))
        }
    } else if current.is_empty() {
        // Leere Selektion → exklusiv selektieren
        let mut predicted = HashSet::new();
        predicted.insert(nid);
        (predicted, Some(nid))
    } else {
        // Bestehende Selektion → Node hinzufügen
        let mut predicted = current.clone();
        predicted.insert(nid);
        (predicted, Some(nid))
    }
}

impl InputState {
    /// Erstellt einen neuen, leeren Input-Zustand.
    pub fn new() -> Self {
        Self {
            primary_drag_mode: PrimaryDragMode::None,
            drag_selection: None,
            context_menu_snapshot: None,
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
    ///
    /// `tangent_data` enthält optionale Tangenten-Menüdaten vom aktiven Route-Tool
    /// (nur bei kubischer Kurve in Control-Phase mit Nachbarn).
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
        tangent_data: Option<TangentMenuData>,
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

        // ── Einheitliches Context-Menu-System ───────────────────────────
        // Genau EIN `response.context_menu()`-Aufruf pro Frame.

        let pointer_pos_world = response.hover_pos().map(|screen_pos| {
            let local = screen_pos - response.rect.min;
            camera.screen_to_world(
                glam::Vec2::new(local.x, local.y),
                glam::Vec2::new(viewport_size[0], viewport_size[1]),
            )
        });

        // Beim Rechtsklick: Snapshot erstellen und einfrieren
        // Guard: Kein Kontextmenü während Rect/Lasso-Drag
        if response.secondary_clicked() && self.drag_selection.is_none() {
            // Node unter Mausposition finden
            let clicked_node_id = pointer_pos_world.and_then(|pos| {
                road_map.and_then(|rm| {
                    context_menu::find_nearest_node_at(pos, rm, options.snap_radius())
                })
            });

            let toggle = modifiers.command; // Ctrl (Linux/Win) / Cmd (macOS)

            // Optimistischen Selection-State berechnen (was nach Intent-Verarbeitung gelten wird)
            let (predicted_selection, predicted_focus) =
                predict_selection_after_rmt(selected_node_ids, clicked_node_id, toggle);

            // Intent emittieren — State wird vom Controller nachgezogen
            if let Some(world_pos) = pointer_pos_world {
                events.push(AppIntent::ContextMenuPick {
                    node_id: clicked_node_id,
                    world_pos,
                    toggle,
                });
            }

            // Route-Tool-Priorität prüfen
            let route_tool_active_for_menu =
                route_tool_is_drawing && active_tool == EditorTool::Route;

            let variant = context_menu::determine_menu_variant(
                &predicted_selection,
                predicted_focus,
                route_tool_active_for_menu,
                tangent_data.clone(),
            );

            self.context_menu_snapshot = Some(ContextMenuSnapshot {
                variant,
                selection: predicted_selection,
                pointer_pos_world,
            });
        }

        // Eingefrorener Snapshot verwenden, falls vorhanden, sonst live berechnen.
        let (variant, menu_selection) = if let Some(snapshot) = &self.context_menu_snapshot {
            (snapshot.variant.clone(), &snapshot.selection)
        } else {
            let v = context_menu::determine_menu_variant(
                selected_node_ids,
                None, // Kein fokussierter Node außerhalb von RMT-Snapshot
                route_tool_is_drawing && active_tool == EditorTool::Route,
                tangent_data,
            );
            (v, selected_node_ids)
        };

        let menu_is_open = context_menu::render_context_menu(
            response,
            road_map,
            menu_selection,
            distanzen_state,
            &variant,
            &mut events,
        );

        // Cache leeren sobald das Popup geschlossen ist.
        if !menu_is_open {
            self.context_menu_snapshot = None;
        }

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
