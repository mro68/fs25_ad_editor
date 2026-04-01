//! Overlay-Rendering fuer Gruppen, Boundaries, Vorschau und Distanzen.

use eframe::egui;
use fs25_auto_drive_editor::{ui, AppIntent, EditorTool};
use glam::Vec2;

use super::EditorApp;

impl EditorApp {
    /// Zeichnet Tool-Preview und Distanzen-Overlay ueber den Viewport.
    /// Gibt gesammelte Overlay-Events als `AppIntent`-Vec zurueck.
    pub(super) fn render_overlays(
        &mut self,
        ui: &egui::Ui,
        rect: egui::Rect,
        response: &egui::Response,
        viewport_size: [f32; 2],
    ) -> Vec<AppIntent> {
        let mut overlay_events: Vec<AppIntent> = Vec::new();

        // ── Tool-Preview-Overlay ─────────────
        if self.state.editor.active_tool == EditorTool::Route {
            let vp = Vec2::new(viewport_size[0], viewport_size[1]);

            if let Some(hover_pos) = response.hover_pos() {
                let local = hover_pos - rect.min;
                self.last_cursor_world = Some(
                    self.state
                        .view
                        .camera
                        .screen_to_world(Vec2::new(local.x, local.y), vp),
                );
            }

            if let Some(cursor_world) = self.last_cursor_world {
                if let Some(rm) = self.state.road_map.as_deref() {
                    if let Some(preview) = self.state.editor.route_tool_preview(cursor_world, rm) {
                        let painter = ui.painter_at(rect);
                        let ctx = ui::tool_preview::ToolPreviewContext {
                            painter: &painter,
                            rect,
                            camera: &self.state.view.camera,
                            viewport_size: vp,
                            preview: &preview,
                            options: &self.state.options,
                        };

                        ui::render_tool_preview(&ctx);
                    }
                }
            }
        }

        // ── Paste-Vorschau-Overlay ──────────────
        if let Some(paste_pos) = self.state.paste_preview_pos {
            let vp = Vec2::new(viewport_size[0], viewport_size[1]);
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
            let vp = Vec2::new(viewport_size[0], viewport_size[1]);
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
                let vp = Vec2::new(viewport_size[0], viewport_size[1]);
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
                    let vp = Vec2::new(viewport_size[0], viewport_size[1]);
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
}
