//! Event-Sammlung fuer Panels, Dialoge und Viewport.

use eframe::egui;
use fs25_auto_drive_editor::app::{AppIntent, EditorTool};
use fs25_auto_drive_editor::ui;
use glam::Vec2;

use super::EditorApp;

impl EditorApp {
    /// Sammelt alle UI- und Viewport-Events des aktuellen Frames.
    pub(super) fn collect_ui_events(&mut self, ctx: &egui::Context) -> Vec<AppIntent> {
        let mut events = Vec::new();

        // Panels und Dialoge
        events.extend(self.collect_panel_events(ctx));
        events.extend(self.collect_dialog_events(ctx));
        let mut show_command_palette = self.state.ui.show_command_palette;
        events.extend(ui::command_palette::render_command_palette(
            ctx,
            &mut show_command_palette,
            &self.state,
        ));
        self.state.ui.show_command_palette = show_command_palette;

        // Zentraler Viewport (Rendering + Input + Overlays)
        egui::CentralPanel::default()
            .frame(egui::Frame::NONE)
            .show(ctx, |ui| {
                let (rect, response) =
                    ui.allocate_exact_size(ui.available_size(), egui::Sense::click_and_drag());
                let viewport_size = [rect.width(), rect.height()];
                let command_palette_open = self.state.ui.show_command_palette;

                events.extend(self.collect_viewport_events(
                    ui,
                    &response,
                    viewport_size,
                    command_palette_open,
                ));
                self.render_viewport(ui, rect, viewport_size);
                let overlay_intents = self.render_overlays(ui, rect, &response, viewport_size);
                events.extend(overlay_intents);
            });

        events
    }

    /// Sammelt Events aus Menue, Toolbar und Properties-Panel.
    fn collect_panel_events(&mut self, ctx: &egui::Context) -> Vec<AppIntent> {
        let mut events = Vec::new();

        ui::render_status_bar(ctx, &self.state);
        events.extend(ui::render_menu(ctx, &self.state));
        let (floating_events, should_close_floating_menu) =
            ui::render_floating_menu(ctx, &self.state);
        if should_close_floating_menu {
            self.state.ui.floating_menu = None;
        }
        events.extend(floating_events);
        events.extend(ui::render_route_defaults_panel(ctx, &self.state));

        // Rechte Sidebar: Marker + Eigenschaften untereinander, einklappbar
        // (muss vor CentralPanel aufgerufen werden)
        let road_map_for_properties = self.state.road_map.clone();
        let default_direction = self.state.editor.default_direction;
        let default_priority = self.state.editor.default_priority;
        let active_tool = self.state.editor.active_tool;
        let distance_wheel_step_m = numeric_distance_wheel_step(&self.state.options);
        let route_tool_panel = self.state.editor.route_tool_panel_state();
        egui::SidePanel::right("right_sidebar")
            .resizable(true)
            .default_width(200.0)
            .min_width(160.0)
            .show(ctx, |ui| {
                egui::ScrollArea::vertical().show(ui, |ui| {
                    egui::CollapsingHeader::new("Marker")
                        .default_open(true)
                        .show(ui, |ui| {
                            events.extend(ui::render_marker_content(
                                ui,
                                self.state.road_map.as_deref(),
                            ));
                        });

                    ui.separator();

                    egui::CollapsingHeader::new("Eigenschaften")
                        .default_open(true)
                        .show(ui, |ui| {
                            events.extend(ui::render_properties_content(
                                ui,
                                road_map_for_properties.as_deref(),
                                &self.state.selection.selected_node_ids,
                                default_direction,
                                default_priority,
                                distance_wheel_step_m,
                                Some(&self.state.group_registry),
                                Some(&self.state.tool_edit_store),
                                &mut self.state.ui.distanzen,
                            ));
                        });
                });
            });

        // Floating Edit-Panel (Streckenteilung / Route-Tool)
        let panel_pos = self
            .input
            .edit_panel_pos
            .map(|p| egui::Pos2::new(p[0], p[1]));
        let group_record = if let Some(es) = self.state.group_editing.as_ref() {
            self.state.group_registry.get(es.record_id)
        } else {
            None
        };
        events.extend(ui::render_edit_panel(
            ctx,
            self.state.road_map.as_deref(),
            &self.state.selection.selected_node_ids,
            &mut self.state.ui.distanzen,
            default_direction,
            default_priority,
            distance_wheel_step_m,
            active_tool,
            route_tool_panel,
            panel_pos,
            self.state.group_editing.as_ref(),
            group_record,
            Some(&self.state.tool_edit_store),
            &mut self.state.options,
        ));

        events
    }

    /// Sammelt Events aus allen offenen Dialogen.
    fn collect_dialog_events(&mut self, ctx: &egui::Context) -> Vec<AppIntent> {
        let mut events = Vec::new();

        events.extend(ui::handle_file_dialogs(&mut self.state.ui));
        events.extend(ui::show_heightmap_warning(
            ctx,
            self.state.ui.show_heightmap_warning,
        ));
        events.extend(ui::show_marker_dialog(
            ctx,
            &mut self.state.ui,
            self.state.road_map.as_deref(),
        ));
        events.extend(ui::show_dedup_dialog(ctx, &self.state.ui));
        events.extend(ui::show_confirm_dissolve_dialog(
            ctx,
            &mut self.state.ui.confirm_dissolve_group_id,
            self.state.options.language,
        ));
        events.extend(ui::show_zip_browser(ctx, &mut self.state.ui));
        events.extend(ui::show_overview_options_dialog(
            ctx,
            &mut self.state.ui.overview_options_dialog,
        ));
        events.extend(ui::show_post_load_dialog(ctx, &mut self.state.ui));
        events.extend(ui::show_save_overview_dialog(ctx, &mut self.state.ui));
        events.extend(ui::show_trace_all_fields_dialog(ctx, &mut self.state.ui));
        events.extend(ui::show_group_settings_popup(
            ctx,
            &mut self.state.ui.group_settings_popup,
            &mut self.state.options,
        ));
        events.extend(ui::show_options_dialog(
            ctx,
            self.state.show_options_dialog,
            &self.state.options,
        ));

        events
    }

    /// Sammelt Input-Events aus dem Viewport (Maus, Drag, Route-Tool-Kontextmenue).
    fn collect_viewport_events(
        &mut self,
        ui: &egui::Ui,
        response: &egui::Response,
        viewport_size: [f32; 2],
        command_palette_open: bool,
    ) -> Vec<AppIntent> {
        let mut events = Vec::new();

        // ── Paste-Vorschau hat Prioritaet: normale Klicks unterdruecken ──────
        if self.state.paste_preview_pos.is_some() {
            events.push(AppIntent::ViewportResized {
                size: viewport_size,
            });

            // Mauszeiger-Position → Vorschau aktualisieren
            if let Some(hover_screen) = response.hover_pos() {
                let local = hover_screen - response.rect.min;
                let vp = Vec2::new(viewport_size[0], viewport_size[1]);
                let world_pos = self
                    .state
                    .view
                    .camera
                    .screen_to_world(Vec2::new(local.x, local.y), vp);
                events.push(AppIntent::PastePreviewMoved { world_pos });
            }

            // Linksklick → Einfuegen bestaetigen
            if response.clicked() {
                events.push(AppIntent::PasteConfirmRequested);
            }

            // Esc → Vorschau abbrechen
            if ui.input(|i| i.key_pressed(egui::Key::Escape)) {
                events.push(AppIntent::PasteCancelled);
            }

            // Cursor als Fadenkreuz anzeigen
            ui.ctx().set_cursor_icon(egui::CursorIcon::Crosshair);

            return events;
        }
        // ─────────────────────────────────────────────────────────────────────

        let route_tool_view = self.state.editor.route_tool_viewport_data();
        let route_tool_is_drawing = route_tool_view.has_pending_input;
        let route_tool_segment_shortcuts_active = route_tool_view.segment_shortcuts_active;
        let default_direction = self.state.editor.default_direction;
        let default_priority = self.state.editor.default_priority;
        let farmland_available = self
            .state
            .farmland_polygons_arc()
            .is_some_and(|p| !p.is_empty());

        events.extend(self.input.collect_viewport_events(
            ui,
            response,
            viewport_size,
            &self.state.view.camera,
            self.state.road_map.as_deref(),
            &self.state.selection.selected_node_ids,
            self.state.editor.active_tool,
            route_tool_is_drawing,
            route_tool_segment_shortcuts_active,
            &self.state.options,
            command_palette_open,
            default_direction,
            default_priority,
            &route_tool_view.drag_targets,
            &mut self.state.ui.distanzen,
            route_tool_view.tangent_menu_data,
            !self.state.clipboard.nodes.is_empty(),
            farmland_available,
            self.state.group_editing.is_some(),
            Some(&self.state.group_registry),
            route_tool_view.needs_lasso_input,
        ));

        // Mauszeiger im Viewport je nach aktivem Werkzeug anpassen
        if response.hovered() {
            match self.state.editor.active_tool {
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

fn numeric_distance_wheel_step(options: &fs25_auto_drive_editor::shared::EditorOptions) -> f32 {
    options.mouse_wheel_distance_step_m.max(0.0)
}

#[cfg(test)]
mod tests {
    use super::numeric_distance_wheel_step;
    use fs25_auto_drive_editor::shared::{EditorOptions, ValueAdjustInputMode};

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
}
