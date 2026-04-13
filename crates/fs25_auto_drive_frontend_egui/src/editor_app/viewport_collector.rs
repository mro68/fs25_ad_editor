//! Event-Sammlung fuer den Viewport (Maus, Drag, Kontextmenue).

use crate::app::{AppIntent, Camera2D, EditorTool, RoadMap};
use crate::ui::context_menu::{self, MenuVariant};
use eframe::egui;
use fs25_auto_drive_host_bridge::{HostRouteToolViewportSnapshot, HostSessionAction};
use glam::Vec2;

use super::{map_intent_to_collected_event, CollectedEvent, EditorApp};

fn snapshot_focused_node_id(input: &crate::ui::InputState) -> Option<u64> {
    input
        .context_menu_snapshot
        .as_ref()
        .and_then(|snapshot| match &snapshot.variant {
            MenuVariant::NodeFocused { focused_node_id } => Some(*focused_node_id),
            _ => None,
        })
}

fn live_secondary_click_focused_node_id(
    input: &crate::ui::InputState,
    response: &egui::Response,
    viewport_size: [f32; 2],
    camera: &Camera2D,
    road_map: Option<&RoadMap>,
    snap_radius: f32,
) -> Option<u64> {
    if !response.secondary_clicked() || input.drag_selection.is_some() {
        return None;
    }

    let hover_pos = response.hover_pos()?;
    let local = hover_pos - response.rect.min;
    let world_pos = camera.screen_to_world(
        Vec2::new(local.x, local.y),
        Vec2::new(viewport_size[0], viewport_size[1]),
    );

    road_map.and_then(|map| context_menu::find_nearest_node_at(world_pos, map, snap_radius))
}

fn focused_context_menu_node_id(
    input: &crate::ui::InputState,
    response: &egui::Response,
    viewport_size: [f32; 2],
    camera: &Camera2D,
    road_map: Option<&RoadMap>,
    snap_radius: f32,
) -> Option<u64> {
    snapshot_focused_node_id(input).or_else(|| {
        live_secondary_click_focused_node_id(
            input,
            response,
            viewport_size,
            camera,
            road_map,
            snap_radius,
        )
    })
}

impl EditorApp {
    /// Sammelt Input-Events aus dem Viewport (Maus, Drag, Route-Tool-Kontextmenue).
    pub(super) fn collect_viewport_events(
        &mut self,
        ui: &egui::Ui,
        response: &egui::Response,
        viewport_size: [f32; 2],
        command_palette_open: bool,
    ) -> Vec<CollectedEvent> {
        let mut events = Vec::new();
        let HostRouteToolViewportSnapshot {
            drag_targets,
            has_pending_input: route_tool_is_drawing,
            segment_shortcuts_active: route_tool_segment_shortcuts_active,
            tangent_menu_data,
            needs_lasso_input,
        } = self.session.build_route_tool_viewport_snapshot();
        let focused_node_id = {
            let app_state = self.session.app_state();
            focused_context_menu_node_id(
                &self.input,
                response,
                viewport_size,
                &app_state.view.camera,
                app_state.road_map.as_deref(),
                app_state.options.snap_radius(),
            )
        };
        let focused_node_details =
            focused_node_id.and_then(|node_id| self.session.node_details(node_id));
        let viewport_state = self.session.viewport_input_context_mut();

        // ── Paste-Vorschau hat Prioritaet: normale Klicks unterdruecken ──────
        if viewport_state.paste_preview_active {
            events.push(CollectedEvent::HostAction(
                HostSessionAction::SubmitViewportInput {
                    batch: fs25_auto_drive_host_bridge::HostViewportInputBatch {
                        events: vec![
                            fs25_auto_drive_host_bridge::HostViewportInputEvent::Resize {
                                size_px: viewport_size,
                            },
                        ],
                    },
                },
            ));

            // Mauszeiger-Position → Vorschau aktualisieren
            if let Some(hover_screen) = response.hover_pos() {
                let local = hover_screen - response.rect.min;
                let vp = Vec2::new(viewport_size[0], viewport_size[1]);
                let world_pos = viewport_state
                    .camera
                    .screen_to_world(Vec2::new(local.x, local.y), vp);
                events.push(map_intent_to_collected_event(
                    AppIntent::PastePreviewMoved { world_pos },
                ));
            }

            // Linksklick → Einfuegen bestaetigen
            if response.clicked() {
                events.push(map_intent_to_collected_event(
                    AppIntent::PasteConfirmRequested,
                ));
            }

            // Esc → Vorschau abbrechen
            if ui.input(|i| i.key_pressed(egui::Key::Escape)) {
                events.push(map_intent_to_collected_event(AppIntent::PasteCancelled));
            }

            // Cursor als Fadenkreuz anzeigen
            ui.ctx().set_cursor_icon(egui::CursorIcon::Crosshair);

            return events;
        }
        // ─────────────────────────────────────────────────────────────────────

        let viewport_events = self.input.collect_viewport_events(
            ui,
            response,
            viewport_size,
            viewport_state.camera,
            viewport_state.road_map,
            viewport_state.selected_node_ids,
            viewport_state.active_tool,
            route_tool_is_drawing,
            route_tool_segment_shortcuts_active,
            viewport_state.options,
            command_palette_open,
            viewport_state.default_direction,
            viewport_state.default_priority,
            &drag_targets,
            viewport_state.distanzen,
            tangent_menu_data,
            focused_node_details.as_ref(),
            viewport_state.clipboard_has_nodes,
            viewport_state.farmland_available,
            viewport_state.group_editing_active,
            Some(viewport_state.group_registry),
            needs_lasso_input,
        );
        events.extend(
            viewport_events
                .intents
                .into_iter()
                .map(map_intent_to_collected_event),
        );
        if let Some(host_batch) = viewport_events.host_input_batch {
            events.push(CollectedEvent::HostAction(
                HostSessionAction::SubmitViewportInput { batch: host_batch },
            ));
        }

        // Mauszeiger im Viewport je nach aktivem Werkzeug anpassen
        if response.hovered() {
            match viewport_state.active_tool {
                EditorTool::AddNode => {
                    ui.ctx().set_cursor_icon(egui::CursorIcon::Crosshair);
                }
                EditorTool::Connect => {
                    ui.ctx().set_cursor_icon(egui::CursorIcon::ResizeNwSe);
                }
                _ => {}
            }
        }

        events
    }
}
