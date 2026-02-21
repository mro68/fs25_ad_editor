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

    /// Sammelt Viewport-Events aus egui-Input und gibt AppIntents zurück.
    #[allow(clippy::too_many_arguments)]
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
    ) -> Vec<AppIntent> {
        let mut events = Vec::new();

        events.push(AppIntent::ViewportResized {
            size: viewport_size,
        });

        // Keyboard-Shortcuts (ausgelagert in keyboard.rs)
        events.extend(keyboard::collect_keyboard_intents(ui, selected_node_ids));

        let modifiers = ui.input(|i| i.modifiers);

        // ── Drag-Start ──────────────────────────────────────────────
        if response.drag_started_by(egui::PointerButton::Primary) {
            if modifiers.shift || modifiers.alt {
                // Shift = Rect-Selektion, Alt = Lasso-Selektion
                // Ctrl zusätzlich = additiv (zur bestehenden Selektion hinzufügen)
                if let Some(pointer_pos) = response.interact_pointer_pos() {
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
            } else {
                // Kein Shift/Alt: Selektion + Move-Drag oder Kamera-Pan
                let base_max_distance = camera
                    .pick_radius_world_scaled(viewport_size[1], options.selection_pick_radius_px);
                let move_max_distance = base_max_distance * options.selection_size_factor;

                // press_origin() liefert die exakte Klickposition (vor Drag-Schwelle),
                // interact_pointer_pos() hingegen die Position *nach* Drag-Erkennung
                // (offset um ~6px), was zu asymmetrischen Hitboxen führen kann.
                let press_pos = ui.input(|i| i.pointer.press_origin());
                let hit_info = press_pos.and_then(|pointer_pos| {
                    let world_pos =
                        screen_pos_to_world(pointer_pos, response, viewport_size, camera);
                    road_map
                        .and_then(|rm| rm.nearest_node(world_pos))
                        .filter(|hit| hit.distance <= move_max_distance)
                        .map(|hit| (hit.node_id, world_pos))
                });

                if let Some((hit_node_id, world_pos)) = hit_info {
                    let already_selected = selected_node_ids.contains(&hit_node_id);

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
        if let Some(selection) = self.drag_selection.as_mut() {
            if response.dragged_by(egui::PointerButton::Primary) {
                if let Some(pointer_pos) = response.interact_pointer_pos() {
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
        }

        // ── Drag-Ende ───────────────────────────────────────────────
        if response.drag_stopped_by(egui::PointerButton::Primary) {
            if let Some(selection) = self.drag_selection.take() {
                match selection.mode {
                    DragSelectionMode::Rect => {
                        if selection.points_screen.len() >= 2 {
                            let a = screen_pos_to_world(
                                selection.start_screen,
                                response,
                                viewport_size,
                                camera,
                            );
                            let b = screen_pos_to_world(
                                selection.points_screen[selection.points_screen.len() - 1],
                                response,
                                viewport_size,
                                camera,
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
                                    screen_pos_to_world(point, response, viewport_size, camera)
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
            }

            self.primary_drag_mode = PrimaryDragMode::None;
        }

        // ── Klick-Events ────────────────────────────────────────────
        if response.double_clicked_by(egui::PointerButton::Primary) {
            if let Some(pointer_pos) = response.interact_pointer_pos() {
                let world_pos = screen_pos_to_world(pointer_pos, response, viewport_size, camera);

                // Ctrl + Doppelklick = Segment additiv hinzufügen
                events.push(AppIntent::NodeSegmentBetweenIntersectionsRequested {
                    world_pos,
                    additive: modifiers.command,
                });
            }

            self.primary_drag_mode = PrimaryDragMode::None;
        } else if response.clicked_by(egui::PointerButton::Primary) {
            if let Some(pointer_pos) = response.interact_pointer_pos() {
                let world_pos = screen_pos_to_world(pointer_pos, response, viewport_size, camera);

                match active_tool {
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
                }
            }

            self.primary_drag_mode = PrimaryDragMode::None;
        }

        // ── Pointer-Delta (Pan / Move) ──────────────────────────────
        let pointer_delta = ui.input(|i| i.pointer.delta());
        if pointer_delta != egui::Vec2::ZERO {
            let wpp = camera.world_per_pixel(viewport_size[1]);

            if self.drag_selection.is_some() {
                // Während Drag-Selektion keine Pan/Move-Events senden.
            } else if response.dragged_by(egui::PointerButton::Primary) {
                match self.primary_drag_mode {
                    PrimaryDragMode::SelectionMove if !selected_node_ids.is_empty() => {
                        events.push(AppIntent::MoveSelectedNodesRequested {
                            delta_world: glam::Vec2::new(
                                pointer_delta.x * wpp,
                                pointer_delta.y * wpp,
                            ),
                        });
                    }
                    PrimaryDragMode::CameraPan | PrimaryDragMode::None => {
                        events.push(AppIntent::CameraPan {
                            delta: glam::Vec2::new(-pointer_delta.x * wpp, -pointer_delta.y * wpp),
                        });
                    }
                    PrimaryDragMode::SelectionMove => {}
                }
            } else if response.dragged_by(egui::PointerButton::Middle)
                || response.dragged_by(egui::PointerButton::Secondary)
            {
                events.push(AppIntent::CameraPan {
                    delta: glam::Vec2::new(-pointer_delta.x * wpp, -pointer_delta.y * wpp),
                });
            }
        }

        // ── Drag-Selektion Overlay (ausgelagert in drag.rs) ─────────
        draw_drag_selection_overlay(self.drag_selection.as_ref(), ui, response);

        // ── Kontextmenü (ausgelagert in context_menu.rs) ────────────
        context_menu::show_connection_context_menu(
            response,
            road_map,
            selected_node_ids,
            &mut events,
        );

        // ── Marker-Kontextmenü (für Single-Node-Selection) ──────────
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

        // ── Scroll-Zoom (auf Mausposition) ──────────────────────────
        let scroll = ui.input(|i| i.smooth_scroll_delta.y);
        if scroll != 0.0 {
            let step = options.camera_scroll_zoom_step;
            let factor = if scroll > 0.0 { step } else { 1.0 / step };
            let focus_world = response
                .hover_pos()
                .map(|pos| screen_pos_to_world(pos, response, viewport_size, camera));
            events.push(AppIntent::CameraZoom {
                factor,
                focus_world,
            });
        }

        events
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
