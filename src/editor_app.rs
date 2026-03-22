//! Haupt-App und Event-Loop-Integration fuer den Editor.

use eframe::egui;
use eframe::egui_wgpu;
use fs25_auto_drive_editor::app::group_registry::{
    TOOL_INDEX_BYPASS, TOOL_INDEX_CURVE_CUBIC, TOOL_INDEX_CURVE_QUAD, TOOL_INDEX_PARKING,
    TOOL_INDEX_ROUTE_OFFSET, TOOL_INDEX_SMOOTH_CURVE, TOOL_INDEX_SPLINE, TOOL_INDEX_STRAIGHT,
};
use fs25_auto_drive_editor::app::state::{FloatingMenuKind, FloatingMenuState};
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
    /// Gecachte egui-Textur-Handles fuer Gruppen-Boundary-Icons (lazy initialisiert).
    group_boundary_icons: Option<ui::GroupBoundaryIcons>,
}

impl EditorApp {
    /// Erstellt die Editor-App mit geladenen Optionen und initialisiertem Renderer.
    pub(crate) fn new(render_state: &egui_wgpu::RenderState) -> Self {
        // Optionen aus TOML laden (oder Standardwerte)
        let config_path = EditorOptions::config_path();
        let editor_options = EditorOptions::load_from_file(&config_path);

        let mut state = AppState::new();
        state.set_options(editor_options);

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
            group_boundary_icons: None,
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

        self.process_events(ctx, &events);

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
        events.extend(ui::command_palette::render_command_palette(
            ctx,
            &mut self.state.ui.show_command_palette,
            Some(&self.state.editor.tool_manager),
            self.state.options.language,
        ));

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
        let distance_wheel_step_m = match self.state.options.value_adjust_input_mode {
            ValueAdjustInputMode::DragHorizontal => 0.0,
            ValueAdjustInputMode::MouseWheel => self.state.options.mouse_wheel_distance_step_m,
        };
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
        let edit_tool_manager = if active_tool == EditorTool::Route {
            Some(&mut self.state.editor.tool_manager)
        } else {
            None
        };
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
            edit_tool_manager,
            panel_pos,
            self.state.group_editing.as_ref(),
            group_record,
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
                command_palette_open,
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
                self.state.group_editing.is_some(),
                Some(&self.state.group_registry),
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
    /// Gibt gesammelte Overlay-Events als `AppIntent`-Vec zurueck.
    fn render_overlays(
        &mut self,
        ui: &egui::Ui,
        rect: egui::Rect,
        response: &egui::Response,
        viewport_size: [f32; 2],
    ) -> Vec<AppIntent> {
        let mut overlay_events: Vec<AppIntent> = Vec::new();
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
            if !self.state.group_registry.is_empty() {
                let vp = glam::Vec2::new(viewport_size[0], viewport_size[1]);
                // Klick nur weiterreichen wenn der Response einen Klick registriert hat
                let clicked_pos = if response.clicked() {
                    ui.ctx().input(|i| i.pointer.interact_pos())
                } else {
                    None
                };
                let ctrl_held = ui.ctx().input(|i| i.modifiers.ctrl);
                let painter = ui.painter_at(rect);
                let group_overlay_events = ui::render_group_overlays(
                    &painter,
                    rect,
                    &self.state.view.camera,
                    vp,
                    &self.state.group_registry,
                    rm,
                    self.state.selection.selected_node_ids.as_ref(),
                    clicked_pos,
                    ctrl_held,
                    self.state.options.segment_lock_icon_size_px,
                );
                for ev in group_overlay_events {
                    match ev {
                        ui::GroupOverlayEvent::LockToggled { segment_id } => {
                            overlay_events.push(AppIntent::ToggleGroupLockRequested { segment_id });
                        }
                        ui::GroupOverlayEvent::Dissolved { segment_id } => {
                            overlay_events.push(AppIntent::DissolveGroupRequested { segment_id });
                        }
                    }
                }
            }
        }

        // ── Gruppen-Boundary-Overlay ──────────────────
        if let Some(rm) = self.state.road_map.as_deref() {
            if !self.state.group_registry.is_empty() {
                // Cache aufwaermen (O(1) wenn bereits gecacht, sonst O(|Records| * |connections|))
                self.state.group_registry.warm_boundary_cache(rm);

                // Icons lazy initialisieren (benoetigen egui::Context)
                if self.group_boundary_icons.is_none() {
                    self.group_boundary_icons = Some(ui::GroupBoundaryIcons::load(ui.ctx()));
                }
                if let Some(icons) = &self.group_boundary_icons {
                    let vp = glam::Vec2::new(viewport_size[0], viewport_size[1]);
                    let painter = ui.painter_at(rect);
                    ui::render_group_boundary_overlays(
                        &painter,
                        rect,
                        &self.state.view.camera,
                        vp,
                        &self.state.group_registry,
                        rm,
                        self.state.selection.selected_node_ids.as_ref(),
                        icons,
                        self.state.options.segment_lock_icon_size_px,
                        self.state.options.show_all_group_boundaries,
                    );
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

        overlay_events
    }

    fn process_events(&mut self, ctx: &egui::Context, events: &[AppIntent]) {
        for event in events {
            if let AppIntent::ToggleFloatingMenu { kind } = event {
                self.toggle_floating_menu(ctx, *kind);
                continue;
            }

            if let AppIntent::SelectRouteToolRequested { index } = event {
                self.update_last_route_tool_index(*index);
            }

            if let Err(e) = self
                .controller
                .handle_intent(&mut self.state, event.clone())
            {
                log::error!("Event handling failed: {:#}", e);
            }
        }
    }

    fn update_last_route_tool_index(&mut self, index: usize) {
        match index {
            TOOL_INDEX_STRAIGHT
            | TOOL_INDEX_CURVE_QUAD
            | TOOL_INDEX_CURVE_CUBIC
            | TOOL_INDEX_SPLINE => {
                self.state.editor.last_basic_command_index = index;
            }
            TOOL_INDEX_SMOOTH_CURVE => {
                self.state.editor.last_basic_command_index = index;
                self.state.editor.last_smooth_curve_index = index;
            }
            TOOL_INDEX_BYPASS | TOOL_INDEX_PARKING | TOOL_INDEX_ROUTE_OFFSET => {
                self.state.editor.last_section_tool_index = index;
            }
            _ => {}
        }
    }

    fn toggle_floating_menu(&mut self, ctx: &egui::Context, kind: FloatingMenuKind) {
        if let Some(existing) = self.state.ui.floating_menu {
            if existing.kind == kind {
                self.state.ui.floating_menu = None;
            } else {
                let pos = ctx.input(|i| i.pointer.hover_pos().or(i.pointer.latest_pos()));
                self.state.ui.floating_menu = pos.map(|p| FloatingMenuState { kind, pos: p });
            }
        } else {
            let pos = ctx.input(|i| i.pointer.hover_pos().or(i.pointer.latest_pos()));
            self.state.ui.floating_menu = pos.map(|p| FloatingMenuState { kind, pos: p });
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
            || self.state.ui.show_command_palette
            || self.state.ui.floating_menu.is_some()
            || self.state.ui.show_heightmap_warning
            || self.state.ui.marker_dialog.visible
            || self.state.show_options_dialog
        {
            ctx.request_repaint();
        }
    }
}
