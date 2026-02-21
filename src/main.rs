//! FS25 AutoDrive Editor (RADE).
//!
//! Rust-basierter Editor für AutoDrive-Kurse in Farming Simulator 25.
//! Hochperformant mit egui + wgpu für 100k+ Wegpunkte.

use eframe::egui;
use eframe::egui_wgpu;
use fs25_auto_drive_editor::{render, ui, AppController, AppIntent, AppState, EditorOptions};

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
}

impl EditorApp {
    fn new(render_state: &egui_wgpu::RenderState) -> Self {
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

        ui::render_status_bar(ctx, &self.state);
        events.extend(ui::render_menu(ctx, &self.state));
        events.extend(ui::render_toolbar(ctx, &self.state));
        events.extend(ui::render_properties_panel(ctx, &mut self.state));
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
        events.extend(ui::show_options_dialog(ctx, &mut self.state));

        egui::CentralPanel::default()
            .frame(egui::Frame::NONE)
            .show(ctx, |ui| {
                let (rect, response) =
                    ui.allocate_exact_size(ui.available_size(), egui::Sense::click_and_drag());

                let viewport_size = [rect.width(), rect.height()];

                events.extend(self.input.collect_viewport_events(
                    ui,
                    &response,
                    viewport_size,
                    &self.state.view.camera,
                    self.state.road_map.as_deref(),
                    &self.state.selection.selected_node_ids,
                    self.state.editor.active_tool,
                    &self.state.options,
                ));

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

                if self.state.road_map.is_none() {
                    ui.painter().text(
                        rect.center(),
                        egui::Align2::CENTER_CENTER,
                        "No file loaded. Use File → Open",
                        egui::FontId::proportional(20.0),
                        egui::Color32::WHITE,
                    );
                }
            });

        events
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
            renderer.set_background(&self.device, &self.queue, bg_map);
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
            || self.state.ui.show_marker_dialog
            || self.state.show_options_dialog
        {
            ctx.request_repaint();
        }
    }
}
