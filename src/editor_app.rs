//! Haupt-App und Event-Loop-Integration fuer den Editor.

use eframe::egui;
use eframe::egui_wgpu;
use fs25_auto_drive_editor::{
    render, ui, AppController, AppIntent, AppState, EditorOptions, EditorTool, ValueAdjustInputMode,
};

/// Haupt-Anwendungsstruktur.
pub(crate) struct EditorApp {
    state: AppState,
    controller: AppController,
    renderer: std::sync::Arc<std::sync::Mutex<render::Renderer>>,
    device: eframe::wgpu::Device,
    queue: eframe::wgpu::Queue,
    input: ui::InputState,
    /// Gecachte Cursor-Weltposition fuer Tool-Preview
    /// (bleibt erhalten wenn Maus den Viewport verlaesst).
    last_cursor_world: Option<glam::Vec2>,
}

impl EditorApp {
    /// Erstellt die Editor-App mit geladenen Optionen und initialisiertem Renderer.
    pub(crate) fn new(render_state: &egui_wgpu::RenderState) -> Self {
        // Optionen aus TOML laden (oder Standardwerte)
        let config_path = EditorOptions::config_path();
        let editor_options = EditorOptions::load_from_file(&config_path);

        let mut state = AppState::new();
        state.options = editor_options;

        Self {
            state,
            controller: AppController::new(),
            renderer: std::sync::Arc::new(std::sync::Mutex::new(render::Renderer::new(
                render_state,
            ))),
            device: render_state.device.clone(),
            queue: render_state.queue.clone(),
            input: ui::InputState::new(),
            last_cursor_world: None,
        }
    }
}

impl eframe::App for EditorApp {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        if self.state.should_exit {
            ctx.send_viewport_cmd(egui::ViewportCommand::Close);
            return;
        }

        let events = self.collect_ui_events(ctx);

        let has_meaningful_events = events
            .iter()
            .any(|e| !matches!(e, AppIntent::ViewportResized { .. }));

        self.process_events(&events);

        self.sync_background_upload();

        self.maybe_request_repaint(ctx, has_meaningful_events);
    }
}

impl EditorApp {
    /// Sammelt alle UI- und Viewport-Events des aktuellen Frames.
    fn collect_ui_events(&mut self, ctx: &egui::Context) -> Vec<AppIntent> {
        let mut events = Vec::new();

        // Panels und Dialoge
        events.extend(self.collect_panel_events(ctx));
        events.extend(self.collect_dialog_events(ctx));

        // Zentraler Viewport (Rendering + Input + Overlays)
        egui::CentralPanel::default()
            .frame(egui::Frame::NONE)
            .show(ctx, |ui| {
                let (rect, response) =
                    ui.allocate_exact_size(ui.available_size(), egui::Sense::click_and_drag());
                let viewport_size = [rect.width(), rect.height()];

                events.extend(self.collect_viewport_events(ui, &response, viewport_size));
                self.render_viewport(ui, rect, viewport_size);
                self.render_overlays(ui, rect, &response, viewport_size);
            });

        events
    }

    /// Sammelt Events aus Menue, Toolbar und Properties-Panel.
    fn collect_panel_events(&mut self, ctx: &egui::Context) -> Vec<AppIntent> {
        let mut events = Vec::new();

        ui::render_status_bar(ctx, &self.state);
        events.extend(ui::render_menu(ctx, &self.state));
        events.extend(ui::render_toolbar(ctx, &self.state));
        events.extend(ui::render_route_defaults_panel(
            ctx,
            self.state.editor.default_direction,
            self.state.editor.default_priority,
        ));

        let road_map_for_properties = self.state.road_map.clone();
        let default_direction = self.state.editor.default_direction;
        let default_priority = self.state.editor.default_priority;
        let active_tool = self.state.editor.active_tool;
        let distance_wheel_step_m = match self.state.options.value_adjust_input_mode {
            ValueAdjustInputMode::DragHorizontal => 0.0,
            ValueAdjustInputMode::MouseWheel => self.state.options.mouse_wheel_distance_step_m,
        };
        let route_tool_manager = if active_tool == EditorTool::Route {
            Some(&mut self.state.editor.tool_manager)
        } else {
            None
        };
        events.extend(ui::render_properties_panel(
            ctx,
            road_map_for_properties.as_deref(),
            &self.state.selection.selected_node_ids,
            default_direction,
            default_priority,
            distance_wheel_step_m,
            active_tool,
            route_tool_manager,
            Some(&self.state.segment_registry),
            &mut self.state.ui.distanzen,
        ));

        // Floating Edit-Panel (Streckenteilung / Route-Tool)
        let panel_pos = self
            .input
            .edit_panel_pos
            .map(|p| egui::Pos2::new(p[0], p[1]));
        let edit_tool_manager = if active_tool == EditorTool::Route {
            Some(&mut self.state.editor.tool_manager)
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
            edit_tool_manager,
            panel_pos,
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
        events.extend(ui::show_zip_browser(ctx, &mut self.state.ui));
        events.extend(ui::show_overview_options_dialog(
            ctx,
            &mut self.state.ui.overview_options_dialog,
        ));
        events.extend(ui::show_post_load_dialog(ctx, &mut self.state.ui));
        events.extend(ui::show_save_overview_dialog(ctx, &mut self.state.ui));
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
                let vp = glam::Vec2::new(viewport_size[0], viewport_size[1]);
                let world_pos = self
                    .state
                    .view
                    .camera
                    .screen_to_world(glam::Vec2::new(local.x, local.y), vp);
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

        let drag_targets = self
            .state
            .editor
            .tool_manager
            .active_tool()
            .map(|t| t.drag_targets())
            .unwrap_or_default();

        let route_tool_is_drawing = self
            .state
            .editor
            .tool_manager
            .active_tool()
            .map(|t| t.has_pending_input())
            .unwrap_or(false);
        let default_direction = self.state.editor.default_direction;
        let default_priority = self.state.editor.default_priority;

        // Tangenten-Daten vom aktiven Route-Tool abfragen (nur Daten, kein UI)
        let tangent_data = if self.state.editor.active_tool == EditorTool::Route {
            self.state
                .editor
                .tool_manager
                .active_tool()
                .and_then(|t| t.tangent_menu_data())
        } else {
            None
        };

        events.extend(
            self.input.collect_viewport_events(
                ui,
                response,
                viewport_size,
                &self.state.view.camera,
                self.state.road_map.as_deref(),
                &self.state.selection.selected_node_ids,
                self.state.editor.active_tool,
                route_tool_is_drawing,
                &self.state.options,
                default_direction,
                default_priority,
                &drag_targets,
                &mut self.state.ui.distanzen,
                tangent_data,
                !self.state.clipboard.nodes.is_empty(),
                self.state
                    .farmland_polygons
                    .as_ref()
                    .is_some_and(|p| !p.is_empty()),
                Some(&self.state.segment_registry),
            ),
        );

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

    /// Zeichnet die wgpu-Render-Szene in den Viewport.
    fn render_viewport(&mut self, ui: &egui::Ui, rect: egui::Rect, viewport_size: [f32; 2]) {
        let render_data = render::WgpuRenderData {
            scene: self
                .controller
                .build_render_scene(&self.state, viewport_size),
        };

        let callback = egui_wgpu::Callback::new_paint_callback(
            rect,
            render::WgpuRenderCallback {
                renderer: self.renderer.clone(),
                render_data,
                device: self.device.clone(),
                queue: self.queue.clone(),
            },
        );

        ui.painter().add(callback);
    }

    /// Zeichnet Tool-Preview und Distanzen-Overlay ueber den Viewport.
    fn render_overlays(
        &mut self,
        ui: &egui::Ui,
        rect: egui::Rect,
        response: &egui::Response,
        viewport_size: [f32; 2],
    ) {
        // ── Tool-Preview-Overlay ─────────────
        if self.state.editor.active_tool == EditorTool::Route {
            let vp = glam::Vec2::new(viewport_size[0], viewport_size[1]);

            if let Some(hover_pos) = response.hover_pos() {
                let local = hover_pos - rect.min;
                self.last_cursor_world = Some(
                    self.state
                        .view
                        .camera
                        .screen_to_world(glam::Vec2::new(local.x, local.y), vp),
                );
            }

            if let Some(cursor_world) = self.last_cursor_world {
                if let Some(rm) = self.state.road_map.as_deref() {
                    let painter = ui.painter_at(rect);
                    let ctx = ui::tool_preview::ToolPreviewContext {
                        painter: &painter,
                        rect,
                        camera: &self.state.view.camera,
                        viewport_size: vp,
                        tool_manager: &self.state.editor.tool_manager,
                        road_map: rm,
                        cursor_world,
                        options: &self.state.options,
                    };

                    ui::render_tool_preview(&ctx);
                }
            }
        }

        // ── Paste-Vorschau-Overlay ──────────────
        if let Some(paste_pos) = self.state.paste_preview_pos {
            let vp = glam::Vec2::new(viewport_size[0], viewport_size[1]);
            ui::paint_clipboard_preview(
                &ui.painter_at(rect),
                rect,
                &self.state.view.camera,
                vp,
                &self.state.clipboard,
                paste_pos,
                self.state.options.copy_preview_opacity,
            );
        }

        // ── Distanzen-Vorschau-Overlay ──────────
        if self.state.ui.distanzen.active && !self.state.ui.distanzen.preview_positions.is_empty() {
            let vp = glam::Vec2::new(viewport_size[0], viewport_size[1]);
            ui::paint_preview_polyline(
                &ui.painter_at(rect),
                rect,
                &self.state.view.camera,
                vp,
                &self.state.ui.distanzen.preview_positions,
            );
        }

        // ── Segment-Overlay ──────────────────
        if let Some(rm) = self.state.road_map.as_deref() {
            if !self.state.segment_registry.is_empty() {
                let vp = glam::Vec2::new(viewport_size[0], viewport_size[1]);
                // Klick nur weiterreichen wenn der Response einen Klick registriert hat
                let clicked_pos = if response.clicked() {
                    ui.ctx().input(|i| i.pointer.interact_pos())
                } else {
                    None
                };
                let ctrl_held = ui.ctx().input(|i| i.modifiers.ctrl);
                let painter = ui.painter_at(rect);
                let overlay_events = ui::render_segment_overlays(
                    &painter,
                    rect,
                    &self.state.view.camera,
                    vp,
                    &self.state.segment_registry,
                    rm,
                    self.state.selection.selected_node_ids.as_ref(),
                    clicked_pos,
                    ctrl_held,
                    self.state.options.segment_lock_icon_size_px,
                );
                for ev in overlay_events {
                    match ev {
                        ui::SegmentOverlayEvent::LockToggled { segment_id } => {
                            self.controller
                                .handle_intent(
                                    &mut self.state,
                                    AppIntent::ToggleSegmentLockRequested { segment_id },
                                )
                                .ok();
                        }
                        ui::SegmentOverlayEvent::Dissolved { segment_id } => {
                            self.controller
                                .handle_intent(
                                    &mut self.state,
                                    AppIntent::DissolveSegmentRequested { segment_id },
                                )
                                .ok();
                        }
                    }
                }
            }
        }

        if self.state.road_map.is_none() {
            ui.painter().text(
                rect.center(),
                egui::Align2::CENTER_CENTER,
                "No file loaded. Use File → Open",
                egui::FontId::proportional(20.0),
                egui::Color32::WHITE,
            );
        }
    }

    fn process_events(&mut self, events: &[AppIntent]) {
        for event in events {
            if let Err(e) = self
                .controller
                .handle_intent(&mut self.state, event.clone())
            {
                log::error!("Event handling failed: {:#}", e);
            }
        }
    }

    fn sync_background_upload(&mut self) {
        if !self.state.view.background_dirty {
            return;
        }
        self.state.view.background_dirty = false;

        let Ok(mut renderer) = self.renderer.lock() else {
            log::error!("Renderer-Lock fehlgeschlagen (Mutex vergiftet)");
            return;
        };
        if let Some(bg_map) = self.state.view.background_map.as_deref() {
            renderer.set_background(
                &self.device,
                &self.queue,
                bg_map,
                self.state.view.background_scale,
            );
            log::info!("Background-Map in Renderer hochgeladen");
        } else {
            renderer.clear_background();
            log::info!("Background-Map aus Renderer entfernt");
        }
    }

    fn maybe_request_repaint(&self, ctx: &egui::Context, has_meaningful_events: bool) {
        if has_meaningful_events
            || ctx.input(|i| i.pointer.is_moving())
            || self.state.ui.show_heightmap_warning
            || self.state.ui.marker_dialog.visible
            || self.state.show_options_dialog
        {
            ctx.request_repaint();
        }
    }
}
