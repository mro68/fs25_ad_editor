//! Viewport-Input-Handling: Maus-Events, Drag-Selektion, Scroll → AppIntent.
//!
//! Aufgeteilt in phasenbasierte Submodule:
//! - `clicks` — Klick-Events (Einfach-/Doppel-Klick, Tool-Routing)
//! - `drag_primary` — Drag-Start/-Ende (Selektion-Move, Kamera-Pan, Route-Tool-Drag)
//! - `pointer_delta` — Pan/Move-Deltas waehrend aktiver Drags
//! - `zoom` — Scroll-Zoom auf Mausposition

mod clicks;
mod drag_primary;
mod pointer_delta;
mod zoom;

use super::context_menu;
use super::drag::{draw_drag_selection_overlay, DragSelection};
use super::keyboard;
use crate::app::{
    AppIntent, Camera2D, ConnectionDirection, ConnectionPriority, EditorTool, GroupRegistry,
    RoadMap,
};
use crate::shared::EditorOptions;
use fs25_auto_drive_host_bridge::{
    HostInputModifiers, HostPointerButton, HostTangentMenuSnapshot, HostTapKind,
    HostViewportInputBatch, HostViewportInputEvent,
};
use indexmap::IndexSet;

/// Modus des primaeren (Links-)Drags im Viewport.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub(crate) enum PrimaryDragMode {
    #[default]
    None,
    /// Drag eines Route-Tool-Steuerpunkts (Anker/CP)
    RouteToolPointDrag,
}

/// Ergebnis eines Viewport-Input-Sammeldurchlaufs.
#[derive(Debug, Default)]
pub struct ViewportInputEvents {
    /// Lokale Intents fuer nicht-bridge-faehige Gesten.
    pub intents: Vec<AppIntent>,
    /// Optionaler Batch bridge-faehiger Viewport-Input-Events.
    pub host_input_batch: Option<HostViewportInputBatch>,
}

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

/// Immutable Snapshot des Kontextmenue-States beim Rechtsklick.
///
/// Dieser Snapshot gefriert den kompletten Zustand ein, der zum Zeitpunkt
/// des Rechtsklicks galt — damit Menueinhalt stabil bleibt, solange das
/// Popup offen ist. Zustandsaenderungen (Escape, Deselection etc.) beeinflussen
/// nicht das bereits offene Menue.
#[derive(Debug, Clone)]
struct ContextMenuSnapshot {
    /// Eingefrorene Menu-Variante
    variant: context_menu::MenuVariant,
    /// Eingefrorene Selection-Menge (geklonter Arc = O(1))
    selection: indexmap::IndexSet<u64>,
    /// Bildschirmposition des Rechtsklicks (fuer Panel-Positionierung)
    screen_pos: Option<egui::Pos2>,
}

/// Verwaltet den Input-Zustand fuer das Viewport (Drag, Selektion, Scroll)
#[derive(Default)]
pub struct InputState {
    pub(crate) primary_drag_mode: PrimaryDragMode,
    /// Gibt an, ob der aktuelle Primaer-Drag ueber die Bridge-Seam laeuft.
    pub(crate) primary_drag_via_bridge: bool,
    pub(crate) drag_selection: Option<DragSelection>,
    /// Snapshot des Menue-Zustands, gueltig solange das Popup offen ist.
    /// Wird beim Rechtsklick gesetzt und erst geleert, wenn egui das Popup schliesst.
    context_menu_snapshot: Option<ContextMenuSnapshot>,
    /// Bildschirmposition des letzten CM-Klicks fuer Edit-Panel-Positionierung.
    pub edit_panel_pos: Option<[f32; 2]>,
    /// Zeigt an, ob gerade eine Gruppen-Rotation per Alt+Mausrad laeuft.
    /// Wird benutzt um Begin/End-Lifecycle Intents korrekt zu emittieren.
    pub(crate) rotation_active: bool,
}

impl InputState {
    /// Erstellt einen neuen, leeren Input-Zustand.
    pub fn new() -> Self {
        Self {
            primary_drag_mode: PrimaryDragMode::None,
            primary_drag_via_bridge: false,
            drag_selection: None,
            context_menu_snapshot: None,
            edit_panel_pos: None,
            rotation_active: false,
        }
    }

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

pub(crate) fn host_modifiers(modifiers: egui::Modifiers) -> HostInputModifiers {
    HostInputModifiers {
        shift: modifiers.shift,
        alt: modifiers.alt,
        command: modifiers.command || modifiers.ctrl,
    }
}

pub(crate) fn to_viewport_screen_pos(
    pointer_pos: egui::Pos2,
    response: &egui::Response,
) -> [f32; 2] {
    let local = pointer_pos - response.rect.min;
    [local.x, local.y]
}

pub(crate) fn host_pointer_button(button: egui::PointerButton) -> Option<HostPointerButton> {
    match button {
        egui::PointerButton::Primary => Some(HostPointerButton::Primary),
        egui::PointerButton::Middle => Some(HostPointerButton::Middle),
        egui::PointerButton::Secondary => Some(HostPointerButton::Secondary),
        _ => None,
    }
}

pub(crate) fn host_tap_kind(is_double: bool) -> HostTapKind {
    if is_double {
        HostTapKind::Double
    } else {
        HostTapKind::Single
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
