//! Viewport-Input-Handling: Maus-Events, Drag-Selektion, Scroll → AppIntent.

use super::context_menu;
use super::drag::{draw_drag_selection_overlay, DragSelection, DragSelectionMode};
use super::keyboard;
use crate::app::{AppIntent, Camera2D, EditorTool, RoadMap};
use crate::shared::EditorOptions;
use std::collections::HashSet;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
enum PrimaryDragMode {
    #[default]
    None,
    SelectionMove,
    CameraPan,
    /// Drag eines Route-Tool-Steuerpunkts (Anker/CP)
    RouteToolPointDrag,
}

/// Bündelt die gemeinsamen Parameter für Viewport-Event-Verarbeitung.
struct ViewportContext<'a> {
    ui: &'a egui::Ui,
    response: &'a egui::Response,
    viewport_size: [f32; 2],
    camera: &'a Camera2D,
    road_map: Option<&'a RoadMap>,
    selected_node_ids: &'a HashSet<u64>,
    active_tool: EditorTool,
    options: &'a EditorOptions,
    drag_targets: &'a [glam::Vec2],
}

/// Verwaltet den Input-Zustand für das Viewport (Drag, Selektion, Scroll)
#[derive(Default)]
pub struct InputState {
    primary_drag_mode: PrimaryDragMode,
    drag_selection: Option<DragSelection>,
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
        options: &EditorOptions,
        drag_targets: &[glam::Vec2],
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
        ));

        let modifiers = ui.input(|i| i.modifiers);

        self.handle_drag_start(&ctx, modifiers, &mut events);
        self.handle_drag_update(&ctx);
        self.handle_drag_end(&ctx, &mut events);
        self.handle_clicks(&ctx, modifiers, &mut events);
        self.handle_pointer_delta(&ctx, &mut events);

        // Drag-Selektion Overlay (ausgelagert in drag.rs)
        draw_drag_selection_overlay(self.drag_selection.as_ref(), ui, response);

        // Kontextmenüs (ausgelagert in context_menu.rs)
        context_menu::show_connection_context_menu(
            response,
            road_map,
            selected_node_ids,
            &mut events,
        );
        if selected_node_ids.len() == 1 {
            if let Some(&node_id) = selected_node_ids.iter().next() {
                context_menu::show_node_marker_context_menu(
                    response,
                    road_map,
                    node_id,
                    &mut events,
                );
            }
        }

        self.handle_scroll_zoom(&ctx, &mut events);

        events
    }

    // ── Drag-Start ──────────────────────────────────────────────

    fn handle_drag_start(
        &mut self,
        ctx: &ViewportContext,
        modifiers: egui::Modifiers,
        events: &mut Vec<AppIntent>,
    ) {
        if !ctx.response.drag_started_by(egui::PointerButton::Primary) {
            return;
        }

        if modifiers.shift || modifiers.alt {
            // Shift = Rect-Selektion, Alt = Lasso-Selektion
            // Ctrl zusätzlich = additiv (zur bestehenden Selektion hinzufügen)
            if let Some(pointer_pos) = ctx.response.interact_pointer_pos() {
                let mode = if modifiers.alt {
                    DragSelectionMode::Lasso
                } else {
                    DragSelectionMode::Rect
                };

                self.drag_selection = Some(DragSelection {
                    mode,
                    additive: modifiers.command,
                    start_screen: pointer_pos,
                    points_screen: vec![pointer_pos],
                });
                self.primary_drag_mode = PrimaryDragMode::None;
            }
            return;
        }

        // Kein Shift/Alt: Selektion + Move-Drag oder Kamera-Pan
        let base_max_distance = ctx
            .camera
            .pick_radius_world_scaled(ctx.viewport_size[1], ctx.options.selection_pick_radius_px);
        let move_max_distance = base_max_distance * ctx.options.selection_size_factor;

        // Route-Tool Drag-Target Hit-Test (hat Vorrang vor Node-Move)
        let press_pos = ctx.ui.input(|i| i.pointer.press_origin());
        let route_drag_hit = if ctx.active_tool == EditorTool::Route && !ctx.drag_targets.is_empty()
        {
            press_pos.and_then(|pointer_pos| {
                let world_pos =
                    screen_pos_to_world(pointer_pos, ctx.response, ctx.viewport_size, ctx.camera);
                let hit = ctx
                    .drag_targets
                    .iter()
                    .any(|t| t.distance(world_pos) <= base_max_distance);
                if hit {
                    Some(world_pos)
                } else {
                    None
                }
            })
        } else {
            None
        };

        if let Some(world_pos) = route_drag_hit {
            // Steuerpunkt-Drag im Route-Tool starten
            events.push(AppIntent::RouteToolDragStarted { world_pos });
            self.primary_drag_mode = PrimaryDragMode::RouteToolPointDrag;
        } else {
            // press_origin() liefert die exakte Klickposition (vor Drag-Schwelle),
            // interact_pointer_pos() hingegen die Position *nach* Drag-Erkennung
            // (offset um ~6px), was zu asymmetrischen Hitboxen führen kann.
            let hit_info = press_pos.and_then(|pointer_pos| {
                let world_pos =
                    screen_pos_to_world(pointer_pos, ctx.response, ctx.viewport_size, ctx.camera);
                ctx.road_map
                    .and_then(|rm| rm.nearest_node(world_pos))
                    .filter(|hit| hit.distance <= move_max_distance)
                    .map(|hit| (hit.node_id, world_pos))
            });

            if let Some((hit_node_id, world_pos)) = hit_info {
                let already_selected = ctx.selected_node_ids.contains(&hit_node_id);

                if !already_selected {
                    // Node noch nicht selektiert → sofort selektieren (Mouse-Down)
                    let extend_path = modifiers.shift;
                    let additive = modifiers.command || extend_path;
                    events.push(AppIntent::NodePickRequested {
                        world_pos,
                        additive,
                        extend_path,
                    });
                }

                // Move-Operation starten
                events.push(AppIntent::BeginMoveSelectedNodesRequested);
                self.primary_drag_mode = PrimaryDragMode::SelectionMove;
            } else {
                self.primary_drag_mode = PrimaryDragMode::CameraPan;
            }
        }
    }

    // ── Drag-Update ─────────────────────────────────────────────

    fn handle_drag_update(&mut self, ctx: &ViewportContext) {
        let Some(selection) = self.drag_selection.as_mut() else {
            return;
        };
        if !ctx.response.dragged_by(egui::PointerButton::Primary) {
            return;
        }
        if let Some(pointer_pos) = ctx.response.interact_pointer_pos() {
            match selection.mode {
                DragSelectionMode::Rect => {
                    if selection.points_screen.len() == 1 {
                        selection.points_screen.push(pointer_pos);
                    } else {
                        selection.points_screen[1] = pointer_pos;
                    }
                }
                DragSelectionMode::Lasso => {
                    selection.push_lasso_point(pointer_pos);
                }
            }
        }
    }

    // ── Drag-Ende ───────────────────────────────────────────────

    fn handle_drag_end(&mut self, ctx: &ViewportContext, events: &mut Vec<AppIntent>) {
        if !ctx.response.drag_stopped_by(egui::PointerButton::Primary) {
            return;
        }

        if let Some(selection) = self.drag_selection.take() {
            match selection.mode {
                DragSelectionMode::Rect => {
                    if selection.points_screen.len() >= 2 {
                        let a = screen_pos_to_world(
                            selection.start_screen,
                            ctx.response,
                            ctx.viewport_size,
                            ctx.camera,
                        );
                        let b = screen_pos_to_world(
                            selection.points_screen[selection.points_screen.len() - 1],
                            ctx.response,
                            ctx.viewport_size,
                            ctx.camera,
                        );

                        events.push(AppIntent::SelectNodesInRectRequested {
                            min: a,
                            max: b,
                            additive: selection.additive,
                        });
                    }
                }
                DragSelectionMode::Lasso => {
                    if selection.points_screen.len() >= 3 {
                        let polygon = selection
                            .points_screen
                            .into_iter()
                            .map(|point| {
                                screen_pos_to_world(
                                    point,
                                    ctx.response,
                                    ctx.viewport_size,
                                    ctx.camera,
                                )
                            })
                            .collect::<Vec<_>>();

                        events.push(AppIntent::SelectNodesInLassoRequested {
                            polygon,
                            additive: selection.additive,
                        });
                    }
                }
            }
        }

        // Wenn der Drag als SelectionMove lief, sende End-Move-Intent
        if self.primary_drag_mode == PrimaryDragMode::SelectionMove {
            events.push(AppIntent::EndMoveSelectedNodesRequested);
        } else if self.primary_drag_mode == PrimaryDragMode::RouteToolPointDrag {
            events.push(AppIntent::RouteToolDragEnded);
        }

        self.primary_drag_mode = PrimaryDragMode::None;
    }

    // ── Klick-Events ────────────────────────────────────────────

    fn handle_clicks(
        &mut self,
        ctx: &ViewportContext,
        modifiers: egui::Modifiers,
        events: &mut Vec<AppIntent>,
    ) {
        if ctx.response.double_clicked_by(egui::PointerButton::Primary) {
            if let Some(pointer_pos) = ctx.response.interact_pointer_pos() {
                let world_pos =
                    screen_pos_to_world(pointer_pos, ctx.response, ctx.viewport_size, ctx.camera);

                // Ctrl + Doppelklick = Segment additiv hinzufügen
                events.push(AppIntent::NodeSegmentBetweenIntersectionsRequested {
                    world_pos,
                    additive: modifiers.command,
                });
            }

            self.primary_drag_mode = PrimaryDragMode::None;
        } else if ctx.response.clicked_by(egui::PointerButton::Primary) {
            if let Some(pointer_pos) = ctx.response.interact_pointer_pos() {
                let world_pos =
                    screen_pos_to_world(pointer_pos, ctx.response, ctx.viewport_size, ctx.camera);

                match ctx.active_tool {
                    EditorTool::Connect => {
                        // Connect-Tool: Node-Pick für Verbindung
                        events.push(AppIntent::ConnectToolNodeClicked { world_pos });
                    }
                    EditorTool::AddNode => {
                        // AddNode-Tool: Neuen Node an Klickposition
                        events.push(AppIntent::AddNodeRequested { world_pos });
                    }
                    EditorTool::Select => {
                        // Standard-Selektion (Fallback bei Klick ohne Drag)
                        let extend_path = modifiers.shift;
                        let additive = modifiers.command || extend_path;

                        events.push(AppIntent::NodePickRequested {
                            world_pos,
                            additive,
                            extend_path,
                        });

                        // Klick ins Leere → Deselektieren
                        // (wird vom Controller erledigt wenn kein Node in Reichweite)
                    }
                    EditorTool::Route => {
                        // Route-Tool: Klick an Viewport-Position
                        events.push(AppIntent::RouteToolClicked {
                            world_pos,
                            ctrl: modifiers.command,
                        });
                    }
                }
            }

            self.primary_drag_mode = PrimaryDragMode::None;
        }
    }

    // ── Pointer-Delta (Pan / Move) ──────────────────────────────

    fn handle_pointer_delta(&mut self, ctx: &ViewportContext, events: &mut Vec<AppIntent>) {
        let pointer_delta = ctx.ui.input(|i| i.pointer.delta());
        if pointer_delta == egui::Vec2::ZERO {
            return;
        }

        let wpp = ctx.camera.world_per_pixel(ctx.viewport_size[1]);

        if self.drag_selection.is_some() {
            // Während Drag-Selektion keine Pan/Move-Events senden.
        } else if ctx.response.dragged_by(egui::PointerButton::Primary) {
            match self.primary_drag_mode {
                PrimaryDragMode::SelectionMove if !ctx.selected_node_ids.is_empty() => {
                    events.push(AppIntent::MoveSelectedNodesRequested {
                        delta_world: glam::Vec2::new(pointer_delta.x * wpp, pointer_delta.y * wpp),
                    });
                }
                PrimaryDragMode::RouteToolPointDrag => {
                    // Aktuelle Mausposition in Welt-Koordinaten
                    if let Some(pointer_pos) = ctx.response.interact_pointer_pos() {
                        let world_pos = screen_pos_to_world(
                            pointer_pos,
                            ctx.response,
                            ctx.viewport_size,
                            ctx.camera,
                        );
                        events.push(AppIntent::RouteToolDragUpdated { world_pos });
                    }
                }
                PrimaryDragMode::CameraPan | PrimaryDragMode::None => {
                    events.push(AppIntent::CameraPan {
                        delta: glam::Vec2::new(-pointer_delta.x * wpp, -pointer_delta.y * wpp),
                    });
                }
                PrimaryDragMode::SelectionMove => {}
            }
        } else if ctx.response.dragged_by(egui::PointerButton::Middle)
            || ctx.response.dragged_by(egui::PointerButton::Secondary)
        {
            events.push(AppIntent::CameraPan {
                delta: glam::Vec2::new(-pointer_delta.x * wpp, -pointer_delta.y * wpp),
            });
        }
    }

    // ── Scroll-Zoom (auf Mausposition) ──────────────────────────

    fn handle_scroll_zoom(&self, ctx: &ViewportContext, events: &mut Vec<AppIntent>) {
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

fn screen_pos_to_world(
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
