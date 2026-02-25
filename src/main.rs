//! FS25 AutoDrive Editor (RADE).
//!
//! Rust-basierter Editor für AutoDrive-Kurse in Farming Simulator 25.
//! Hochperformant mit egui + wgpu für 100k+ Wegpunkte.

use eframe::egui;
use eframe::egui_wgpu;
use fs25_auto_drive_editor::{
    render, ui, AppController, AppIntent, AppState, EditorOptions, EditorTool,
};

fn main() -> Result<(), eframe::Error> {
    AppRunner::run()
}

struct AppRunner;

impl AppRunner {
    fn run() -> Result<(), eframe::Error> {
        // Logger initialisieren
        env_logger::Builder::from_default_env()
            .filter_level(log::LevelFilter::Info)
            .init();

        log::info!(
            "FS25 AutoDrive Editor v{} startet...",
            env!("CARGO_PKG_VERSION")
        );

        let options = eframe::NativeOptions {
            viewport: egui::ViewportBuilder::default()
                .with_inner_size([1280.0, 720.0])
                .with_title("FS25 AutoDrive Editor"),
            renderer: eframe::Renderer::Wgpu,
            multisampling: 4,
            ..Default::default()
        };

        eframe::run_native(
            "FS25 AutoDrive Editor",
            options,
            Box::new(|cc| {
                // SVG/Bild-Loader für egui installieren (benötigt für Toolbar-Icons)
                egui_extras::install_image_loaders(&cc.egui_ctx);

                let render_state = cc.wgpu_render_state.as_ref().ok_or_else(|| {
                    anyhow::anyhow!(
                        "wgpu nicht verfügbar: Renderer konnte nicht initialisiert werden"
                    )
                })?;
                Ok(Box::new(EditorApp::new(render_state)))
            }),
        )
    }
}

/// Haupt-Anwendungsstruktur
struct EditorApp {
    state: AppState,
    controller: AppController,
    renderer: std::sync::Arc<std::sync::Mutex<render::Renderer>>,
    device: eframe::wgpu::Device,
    queue: eframe::wgpu::Queue,
    input: ui::InputState,
    /// Gecachte Cursor-Weltposition für Tool-Preview (bleibt erhalten wenn Maus den Viewport verlässt).
    last_cursor_world: Option<glam::Vec2>,
}

impl EditorApp {
    fn new(render_state: &egui_wgpu::RenderState) -> Self {
        // Optionen aus TOML laden (oder Standardwerte)
        let config_path = EditorOptions::config_path();
        let editor_options = EditorOptions::load_from_file(&config_path);

        let mut state = AppState::new();
        state.view.background_opacity = editor_options.background_opacity_default;
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

        self.process_events(events);

        self.sync_background_upload();

        self.maybe_request_repaint(ctx, has_meaningful_events);
    }
}

impl EditorApp {
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

    /// Sammelt Events aus Menü, Toolbar und Properties-Panel.
    fn collect_panel_events(&mut self, ctx: &egui::Context) -> Vec<AppIntent> {
        let mut events = Vec::new();

        ui::render_status_bar(ctx, &self.state);
        events.extend(ui::render_menu(ctx, &self.state));
        events.extend(ui::render_toolbar(ctx, &self.state));

        let road_map_for_properties = self.state.road_map.clone();
        let default_direction = self.state.editor.default_direction;
        let default_priority = self.state.editor.default_priority;
        let active_tool = self.state.editor.active_tool;
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
            active_tool,
            route_tool_manager,
            Some(&self.state.segment_registry),
            &self.state.options,
            &mut self.state.ui.distanzen,
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
        events.extend(ui::show_options_dialog(
            ctx,
            self.state.show_options_dialog,
            &self.state.options,
        ));

        events
    }

    /// Sammelt Input-Events aus dem Viewport (Maus, Drag, Route-Tool-Kontextmenü).
    fn collect_viewport_events(
        &mut self,
        ui: &egui::Ui,
        response: &egui::Response,
        viewport_size: [f32; 2],
    ) -> Vec<AppIntent> {
        let mut events = Vec::new();

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

        events.extend(self.input.collect_viewport_events(
            ui,
            response,
            viewport_size,
            &self.state.view.camera,
            self.state.road_map.as_deref(),
            &self.state.selection.selected_node_ids,
            self.state.editor.active_tool,
            route_tool_is_drawing,
            &self.state.options,
            &drag_targets,
            self.state.ui.distanzen.active,
        ));

        // Tool-Kontextmenü (z.B. Tangenten-Auswahl für Cubic-Kurve)
        if self.state.editor.active_tool == EditorTool::Route {
            if let Some(tool) = self.state.editor.tool_manager.active_tool_mut() {
                if tool.render_context_menu(response) && tool.needs_recreate() {
                    events.push(AppIntent::RouteToolConfigChanged);
                }
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

    /// Zeichnet Tool-Preview und Distanzen-Overlay über den Viewport.
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
                    ui::render_tool_preview(
                        &ui.painter_at(rect),
                        rect,
                        &self.state.view.camera,
                        vp,
                        &self.state.editor.tool_manager,
                        rm,
                        cursor_world,
                    );
                }
            }
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

    fn process_events(&mut self, events: Vec<AppIntent>) {
        for event in events {
            if let Err(e) = self.controller.handle_intent(&mut self.state, event) {
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
