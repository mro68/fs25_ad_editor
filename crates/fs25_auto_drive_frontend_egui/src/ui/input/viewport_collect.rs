//! ViewportContext und collect_viewport_events()-Implementierung.

use super::context_menu;
use super::draw_drag_selection_overlay;
use super::keyboard;
use super::state::ContextMenuSnapshot;
use super::{InputState, ViewportInputEvents};
use crate::app::{
    AppIntent, Camera2D, ConnectionDirection, ConnectionPriority, EditorTool, GroupRegistry,
    RoadMap,
};
use crate::shared::EditorOptions;
use fs25_auto_drive_host_bridge::{
    HostNodeDetails, HostTangentMenuSnapshot, HostViewportInputBatch, HostViewportInputEvent,
};
use indexmap::IndexSet;

/// Buendelt die gemeinsamen Parameter fuer Viewport-Event-Verarbeitung.
pub(crate) struct ViewportContext<'a> {
    pub ui: &'a egui::Ui,
    pub response: &'a egui::Response,
    pub viewport_size: [f32; 2],
    pub camera: &'a Camera2D,
    pub selected_node_ids: &'a IndexSet<u64>,
    pub active_tool: EditorTool,
    pub options: &'a EditorOptions,
    pub drag_targets: &'a [[f32; 2]],
    /// Gibt an, ob das aktive Route-Tool Alt+Drag als Lasso-Eingabe benoetigt.
    pub tool_needs_lasso: bool,
}

impl InputState {
    #[allow(clippy::too_many_arguments)]
    /// Sammelt Viewport-Events aus egui-Input und gibt AppIntents zurueck.
    ///
    /// Diese Methode ist der zentrale UI→Intent-Einstieg fuer Maus-, Scroll-
    /// und Drag-Interaktionen im Viewport.
    ///
    /// `drag_targets` enthaelt die Weltpositionen verschiebbarer Punkte
    /// des aktiven Route-Tools (leer wenn kein Tool aktiv oder keine Targets).
    ///
    /// `tangent_data` enthaelt optionale Tangenten-Menuedaten vom aktiven Route-Tool
    /// (nur bei kubischer Kurve in Control-Phase mit Nachbarn).
    ///
    /// `focused_node_details` enthaelt optional vorab geladene Bridge-Details fuer
    /// das Info-Submenu eines fokussierten Kontextmenue-Nodes.
    ///
    /// `route_tool_segment_shortcuts_active` schaltet die Pfeiltasten von Kamera-Pan
    /// auf Segment-Shortcuts um, sobald das aktive Tool diese Capability aktuell anbietet.
    ///
    /// `clipboard_has_data` zeigt an, ob die Zwischenablage Nodes enthaelt (fuer Paste-Precondition).
    pub fn collect_viewport_events(
        &mut self,
        ui: &egui::Ui,
        response: &egui::Response,
        viewport_size: [f32; 2],
        camera: &Camera2D,
        road_map: Option<&RoadMap>,
        selected_node_ids: &IndexSet<u64>,
        active_tool: EditorTool,
        route_tool_is_drawing: bool,
        route_tool_segment_shortcuts_active: bool,
        options: &EditorOptions,
        command_palette_open: bool,
        default_direction: ConnectionDirection,
        default_priority: ConnectionPriority,
        drag_targets: &[[f32; 2]],
        distanzen_state: &mut crate::app::state::DistanzenState,
        tangent_data: Option<HostTangentMenuSnapshot>,
        focused_node_details: Option<&HostNodeDetails>,
        clipboard_has_data: bool,
        farmland_polygons_loaded: bool,
        group_editing_active: bool,
        group_registry: Option<&GroupRegistry>,
        tool_needs_lasso: bool,
    ) -> ViewportInputEvents {
        let ctx = ViewportContext {
            ui,
            response,
            viewport_size,
            camera,
            selected_node_ids,
            active_tool,
            options,
            drag_targets,
            tool_needs_lasso,
        };

        let mut local_intents = Vec::new();
        let mut host_events = Vec::new();

        host_events.push(HostViewportInputEvent::Resize {
            size_px: viewport_size,
        });

        // Keyboard-Shortcuts (ausgelagert in keyboard.rs)
        local_intents.extend(keyboard::collect_keyboard_intents(
            ui,
            selected_node_ids,
            keyboard::KeyboardContext::new(
                active_tool,
                route_tool_is_drawing,
                route_tool_segment_shortcuts_active,
                distanzen_state.active,
                clipboard_has_data,
                command_palette_open,
            ),
        ));

        let modifiers = ui.input(|i| i.modifiers);

        self.handle_drag_start(&ctx, modifiers, &mut local_intents, &mut host_events);
        self.handle_drag_update(&ctx);
        self.handle_drag_end(&ctx, &mut local_intents, &mut host_events);
        self.handle_clicks(&ctx, modifiers, &mut local_intents, &mut host_events);
        self.handle_pointer_delta(&ctx, &mut local_intents, &mut host_events);

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
        // Guard: Kein Kontextmenue waehrend Rect/Lasso-Drag
        if response.secondary_clicked() && self.drag_selection.is_none() {
            // Node unter Mausposition finden (fuer NodeFocused-Menue)
            let clicked_node_id = pointer_pos_world.and_then(|pos| {
                road_map.and_then(|rm| {
                    context_menu::find_nearest_node_at(pos, rm, options.snap_radius())
                })
            });

            // Route-Tool-Prioritaet pruefen
            let route_tool_active_for_menu =
                route_tool_is_drawing && active_tool == EditorTool::Route;

            let variant = context_menu::determine_menu_variant(
                selected_node_ids,
                clicked_node_id,
                route_tool_active_for_menu,
                tangent_data.clone(),
            );

            self.context_menu_snapshot = Some(ContextMenuSnapshot {
                variant,
                selection: selected_node_ids.clone(),
                screen_pos: response.hover_pos(),
            });
        }

        // Eingefrorener Snapshot verwenden, falls vorhanden, sonst live berechnen.
        let (variant, menu_selection) = if let Some(snapshot) = &self.context_menu_snapshot {
            (snapshot.variant.clone(), &snapshot.selection)
        } else {
            let v = context_menu::determine_menu_variant(
                selected_node_ids,
                None, // Kein fokussierter Node ausserhalb von RMT-Snapshot
                route_tool_is_drawing && active_tool == EditorTool::Route,
                tangent_data,
            );
            (v, selected_node_ids)
        };

        let events_before = local_intents.len();
        let menu_is_open = context_menu::render_context_menu(
            response,
            road_map,
            menu_selection,
            distanzen_state.active,
            clipboard_has_data,
            farmland_polygons_loaded,
            group_editing_active,
            options,
            default_direction,
            default_priority,
            &variant,
            group_registry,
            focused_node_details,
            &mut local_intents,
        );

        // CM hat neuen edit-mode-Intent emittiert → Panel-Position speichern
        if local_intents.len() > events_before {
            let has_edit_intent = local_intents[events_before..].iter().any(|e| {
                matches!(
                    e,
                    AppIntent::StreckenteilungAktivieren
                        | AppIntent::RouteToolWithAnchorsRequested { .. }
                )
            });
            if has_edit_intent && let Some(snapshot) = &self.context_menu_snapshot {
                self.edit_panel_pos = snapshot.screen_pos.map(|p| [p.x, p.y]);
            }
        }

        // Cache leeren sobald das Popup geschlossen ist.
        if !menu_is_open {
            self.context_menu_snapshot = None;
        }

        self.handle_scroll_zoom(&ctx, &mut local_intents, &mut host_events);

        ViewportInputEvents {
            intents: local_intents,
            host_input_batch: if host_events.is_empty() {
                None
            } else {
                Some(HostViewportInputBatch {
                    events: host_events,
                })
            },
        }
    }
}
