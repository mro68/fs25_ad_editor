//! Haupt-App und Event-Loop-Integration fuer den Editor.

mod event_collection;
mod helpers;
mod overlays;

use crate::app::{use_cases, AppController, AppIntent, AppState};
use crate::{render, ui};
use eframe::egui;
use eframe::egui_wgpu;

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
        let editor_options = use_cases::options::load_editor_options();

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

        self.process_events(ctx, events);

        self.sync_background_upload();

        self.maybe_request_repaint(ctx, has_meaningful_events);
    }
}

impl EditorApp {
    fn process_events(&mut self, ctx: &egui::Context, events: Vec<AppIntent>) {
        for event in events {
            match event {
                AppIntent::ToggleFloatingMenu { kind } => {
                    self.toggle_floating_menu(ctx, kind);
                }
                intent => {
                    if let Err(e) = self.controller.handle_intent(&mut self.state, intent) {
                        self.state.ui.status_message =
                            Some(format!("Aktion fehlgeschlagen: {}", e));
                        log::error!("Event handling failed: {:#}", e);
                    }
                }
            }
        }
    }
}
