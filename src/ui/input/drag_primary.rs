//! Drag-Start/-Ende: Selektion-Move, Kamera-Pan, Route-Tool-Drag, Rect/Lasso-Selektion.

use super::super::drag::DragSelectionMode;
use super::{screen_pos_to_world, DragSelection, InputState, PrimaryDragMode, ViewportContext};
use crate::app::{AppIntent, EditorTool};

impl InputState {
    /// Erkennt Drag-Beginn und bestimmt den Drag-Modus (Pan, Move, Selektion, Route-Tool).
    pub(crate) fn handle_drag_start(
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
                    let extend_path = modifiers.shift;
                    let additive = modifiers.command || extend_path;
                    events.push(AppIntent::NodePickRequested {
                        world_pos,
                        additive,
                        extend_path,
                    });
                }

                events.push(AppIntent::BeginMoveSelectedNodesRequested);
                self.primary_drag_mode = PrimaryDragMode::SelectionMove;
            } else {
                self.primary_drag_mode = PrimaryDragMode::CameraPan;
            }
        }
    }

    /// Aktualisiert die Drag-Selektion (Rect/Lasso) während des Ziehens.
    pub(crate) fn handle_drag_update(&mut self, ctx: &ViewportContext) {
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

    /// Beendet einen Drag und emittiert die resultierenden Intents (Selektion, Move-Ende, etc.).
    pub(crate) fn handle_drag_end(&mut self, ctx: &ViewportContext, events: &mut Vec<AppIntent>) {
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

        if self.primary_drag_mode == PrimaryDragMode::SelectionMove {
            events.push(AppIntent::EndMoveSelectedNodesRequested);
        } else if self.primary_drag_mode == PrimaryDragMode::RouteToolPointDrag {
            events.push(AppIntent::RouteToolDragEnded);
        }

        self.primary_drag_mode = PrimaryDragMode::None;
    }
}
