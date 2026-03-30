//! Haupt-App und Event-Loop-Integration fuer den Editor.

mod event_collection;
mod helpers;
mod overlays;

use eframe::egui;
use eframe::egui_wgpu;
use fs25_auto_drive_editor::{render, ui, AppController, AppIntent, AppState, EditorOptions};

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
    fn process_events(&mut self, ctx: &egui::Context, events: &[AppIntent]) {
        for event in events {
            if let AppIntent::ToggleFloatingMenu { kind } = event {
                self.toggle_floating_menu(ctx, *kind);
                continue;
            }

            if let Err(e) = self
                .controller
                .handle_intent(&mut self.state, event.clone())
            {
                log::error!("Event handling failed: {:#}", e);
            }
        }
    }
}
