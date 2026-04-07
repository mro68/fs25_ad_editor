//! Event-Sammlung fuer Panels, Dialoge und Viewport.

use crate::app::ui_contract::{panel_action_to_intent, HostUiSnapshot};
use crate::app::{AppIntent, EditorTool};
use crate::shared::EditorOptions;
use crate::ui;
use eframe::egui;
use fs25_auto_drive_host_bridge::{
    map_host_action_to_intent, map_intent_to_host_action, HostChromeSnapshot, HostDialogResult,
    HostRouteToolViewportSnapshot, HostSessionAction,
};
use glam::Vec2;

use super::{CollectedEvent, EditorApp};

fn map_dialog_results_to_intents(dialog_results: Vec<HostDialogResult>) -> Vec<AppIntent> {
    dialog_results
        .into_iter()
        .filter_map(|result| {
            map_host_action_to_intent(HostSessionAction::SubmitDialogResult { result })
        })
        .collect()
}

fn map_intent_to_collected_event(intent: AppIntent) -> CollectedEvent {
    if let Some(action) = map_intent_to_host_action(&intent) {
        CollectedEvent::HostAction(action)
    } else {
        CollectedEvent::Intent(intent)
    }
}

impl EditorApp {
    /// Sammelt alle UI- und Viewport-Events des aktuellen Frames.
    pub(super) fn collect_ui_events(&mut self, ctx: &egui::Context) -> Vec<CollectedEvent> {
        let mut events = Vec::new();
        let host_ui_snapshot = self.session.build_host_ui_snapshot();
        let host_chrome_snapshot = self.session.build_host_chrome_snapshot();
        let mut top_ui = ui::common::create_top_level_ui(ctx, "editor_app_top_level_panels");

        // Panels und Dialoge
        events.extend(self.collect_panel_events(
            ctx,
            &host_ui_snapshot,
            &host_chrome_snapshot,
            &mut top_ui,
        ));
        events.extend(
            self.collect_dialog_events(ctx, &host_ui_snapshot)
                .into_iter()
                .map(map_intent_to_collected_event),
        );
        let mut show_command_palette = host_ui_snapshot
            .command_palette_state()
            .is_some_and(|state| state.visible);
        events.extend(
            ui::command_palette::render_command_palette(
                ctx,
                &mut show_command_palette,
                self.session.app_state(),
                &host_chrome_snapshot,
            )
            .into_iter()
            .map(map_intent_to_collected_event),
        );
        if show_command_palette != host_chrome_snapshot.show_command_palette {
            events.push(CollectedEvent::HostAction(
                HostSessionAction::ToggleCommandPalette,
            ));
        }

        // Zentraler Viewport (Rendering + Input + Overlays)
        egui::CentralPanel::default()
            .frame(egui::Frame::NONE)
            .show_inside(&mut top_ui, |ui| {
                let (rect, response) =
                    ui.allocate_exact_size(ui.available_size(), egui::Sense::click_and_drag());
                let viewport_size = [rect.width(), rect.height()];
                let command_palette_open = host_chrome_snapshot.show_command_palette;

                events.extend(self.collect_viewport_events(
                    ui,
                    &response,
                    viewport_size,
                    command_palette_open,
                ));
                self.render_viewport(ui, rect, viewport_size);
                let overlay_intents = self.render_overlays(ui, rect, &response, viewport_size);
                events.extend(
                    overlay_intents
                        .into_iter()
                        .map(map_intent_to_collected_event),
                );
            });

        events
    }

    /// Sammelt Events aus Menue, Toolbar und Properties-Panel.
    fn collect_panel_events(
        &mut self,
        ctx: &egui::Context,
        host_ui_snapshot: &HostUiSnapshot,
        host_chrome_snapshot: &HostChromeSnapshot,
        top_ui: &mut egui::Ui,
    ) -> Vec<CollectedEvent> {
        let mut events = Vec::new();
        let should_close_floating_menu = {
            let state = self.session.app_state();

            ui::status::render_status_bar_inside(top_ui, state, host_chrome_snapshot);
            events.extend(
                ui::menu::render_menu_inside(top_ui, state, host_chrome_snapshot)
                    .into_iter()
                    .map(map_intent_to_collected_event),
            );
            let (floating_events, should_close) = ui::render_floating_menu(
                ctx,
                self.session.chrome_state().floating_menu,
                state,
                host_chrome_snapshot,
            );
            events.extend(
                floating_events
                    .into_iter()
                    .map(map_intent_to_collected_event),
            );
            events.extend(
                ui::defaults_panel::render_route_defaults_panel_inside(
                    top_ui,
                    state,
                    host_chrome_snapshot,
                )
                .into_iter()
                .map(map_intent_to_collected_event),
            );
            should_close
        };
        if should_close_floating_menu {
            self.session.clear_floating_menu();
        }

        let route_tool_panel = host_ui_snapshot.route_tool_panel_state().cloned();
        let panel_state = self.session.panel_properties_state_mut();
        let distance_wheel_step_m = numeric_distance_wheel_step(panel_state.options);

        // Rechte Sidebar: Marker + Eigenschaften untereinander, einklappbar
        // (muss vor CentralPanel aufgerufen werden)
        egui::Panel::right("right_sidebar")
            .resizable(true)
            .default_size(200.0)
            .min_size(160.0)
            .show_inside(top_ui, |ui| {
                egui::ScrollArea::vertical().show(ui, |ui| {
                    egui::CollapsingHeader::new("Marker")
                        .default_open(true)
                        .show(ui, |ui| {
                            events.extend(
                                ui::render_marker_content(ui, panel_state.road_map)
                                    .into_iter()
                                    .map(map_intent_to_collected_event),
                            );
                        });

                    ui.separator();

                    egui::CollapsingHeader::new("Eigenschaften")
                        .default_open(true)
                        .show(ui, |ui| {
                            events.extend(
                                ui::render_properties_content(
                                    ui,
                                    panel_state.road_map,
                                    panel_state.selected_node_ids,
                                    panel_state.default_direction,
                                    panel_state.default_priority,
                                    distance_wheel_step_m,
                                    Some(panel_state.group_registry),
                                    Some(panel_state.tool_edit_store),
                                    panel_state.distanzen,
                                )
                                .into_iter()
                                .map(map_intent_to_collected_event),
                            );
                        });
                });
            });

        // Floating Edit-Panel (Streckenteilung / Route-Tool)
        let panel_pos = self
            .input
            .edit_panel_pos
            .map(|p| egui::Pos2::new(p[0], p[1]));
        let group_record = panel_state
            .group_editing
            .and_then(|edit_state| panel_state.group_registry.get(edit_state.record_id));
        events.extend(
            ui::render_edit_panel(
                ctx,
                panel_state.road_map,
                panel_state.selected_node_ids,
                panel_state.distanzen,
                panel_state.default_direction,
                panel_state.default_priority,
                distance_wheel_step_m,
                panel_state.active_tool,
                route_tool_panel,
                panel_pos,
                panel_state.group_editing,
                group_record,
                Some(panel_state.tool_edit_store),
                panel_state.options,
            )
            .into_iter()
            .map(map_intent_to_collected_event),
        );

        events
    }

    /// Sammelt Events aus allen offenen Dialogen.
    fn collect_dialog_events(
        &mut self,
        ctx: &egui::Context,
        host_ui_snapshot: &HostUiSnapshot,
    ) -> Vec<AppIntent> {
        let mut events = Vec::new();

        let dialog_results = ui::handle_file_dialogs(self.session.take_dialog_requests());
        events.extend(map_dialog_results_to_intents(dialog_results));
        let dialog_state = self.session.dialog_ui_state_mut();
        events.extend(ui::show_heightmap_warning(
            ctx,
            dialog_state.ui.show_heightmap_warning,
        ));
        events.extend(ui::show_marker_dialog(
            ctx,
            dialog_state.ui,
            dialog_state.road_map,
        ));
        events.extend(ui::show_dedup_dialog(ctx, dialog_state.ui));
        events.extend(ui::show_confirm_dissolve_dialog(
            ctx,
            &mut dialog_state.ui.confirm_dissolve_group_id,
            dialog_state.options.language,
        ));
        events.extend(ui::show_zip_browser(ctx, dialog_state.ui));
        events.extend(ui::show_overview_options_dialog(
            ctx,
            &mut dialog_state.ui.overview_options_dialog,
        ));
        events.extend(ui::show_post_load_dialog(ctx, dialog_state.ui));
        events.extend(ui::show_save_overview_dialog(ctx, dialog_state.ui));
        events.extend(ui::show_trace_all_fields_dialog(ctx, dialog_state.ui));
        events.extend(ui::show_group_settings_popup(
            ctx,
            &mut dialog_state.ui.group_settings_popup,
            dialog_state.options,
        ));
        if let Some(options_panel_state) = host_ui_snapshot.options_panel_state() {
            let panel_actions = ui::show_options_dialog(
                ctx,
                options_panel_state.visible,
                &options_panel_state.options,
            );
            events.extend(panel_actions.into_iter().map(panel_action_to_intent));
        }

        events
    }

    /// Sammelt Input-Events aus dem Viewport (Maus, Drag, Route-Tool-Kontextmenue).
    fn collect_viewport_events(
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

fn numeric_distance_wheel_step(options: &EditorOptions) -> f32 {
    options.mouse_wheel_distance_step_m.max(0.0)
}

#[cfg(test)]
mod tests {
    use fs25_auto_drive_host_bridge::{HostDialogRequestKind, HostDialogResult};

    use super::{map_dialog_results_to_intents, numeric_distance_wheel_step};
    use crate::app::AppIntent;
    use crate::shared::{EditorOptions, ValueAdjustInputMode};

    fn options_with(mode: ValueAdjustInputMode, step: f32) -> EditorOptions {
        EditorOptions {
            value_adjust_input_mode: mode,
            mouse_wheel_distance_step_m: step,
            ..EditorOptions::default()
        }
    }

    #[test]
    fn numeric_distance_wheel_step_is_independent_from_input_mode() {
        let drag_options = options_with(ValueAdjustInputMode::DragHorizontal, 0.25);
        let wheel_options = options_with(ValueAdjustInputMode::MouseWheel, 0.25);

        assert_eq!(numeric_distance_wheel_step(&drag_options), 0.25);
        assert_eq!(numeric_distance_wheel_step(&wheel_options), 0.25);
    }

    #[test]
    fn numeric_distance_wheel_step_clamps_negative_values() {
        let options = options_with(ValueAdjustInputMode::DragHorizontal, -0.25);
        assert_eq!(numeric_distance_wheel_step(&options), 0.0);
    }

    #[test]
    fn map_dialog_results_to_intents_routes_save_file_and_curseplay_export_results() {
        let intents = map_dialog_results_to_intents(vec![
            HostDialogResult::PathSelected {
                kind: HostDialogRequestKind::SaveFile,
                path: "/tmp/savegame.xml".to_string(),
            },
            HostDialogResult::PathSelected {
                kind: HostDialogRequestKind::CurseplayExport,
                path: "/tmp/customField.xml".to_string(),
            },
        ]);

        assert_eq!(intents.len(), 2);
        assert!(matches!(
            &intents[0],
            AppIntent::SaveFilePathSelected { path } if path == "/tmp/savegame.xml"
        ));
        assert!(matches!(
            &intents[1],
            AppIntent::CurseplayExportPathSelected { path } if path == "/tmp/customField.xml"
        ));
    }

    #[test]
    fn map_dialog_results_to_intents_routes_background_zip_selection_to_zip_browse() {
        let intents = map_dialog_results_to_intents(vec![HostDialogResult::PathSelected {
            kind: HostDialogRequestKind::BackgroundMap,
            path: "/tmp/background_map.ZIP".to_string(),
        }]);

        assert_eq!(intents.len(), 1);
        assert!(matches!(
            &intents[0],
            AppIntent::ZipBackgroundBrowseRequested { path } if path == "/tmp/background_map.ZIP"
        ));
    }

    #[test]
    fn map_dialog_results_to_intents_drops_cancelled_results() {
        let intents = map_dialog_results_to_intents(vec![HostDialogResult::Cancelled {
            kind: HostDialogRequestKind::SaveFile,
        }]);

        assert!(intents.is_empty());
    }
}
