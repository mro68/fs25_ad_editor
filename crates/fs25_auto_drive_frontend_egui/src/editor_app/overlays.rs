//! Overlay-Rendering fuer Gruppen, Boundaries, Vorschau und Distanzen.

use crate::app::{AppIntent, Camera2D};
use crate::ui;
use eframe::egui;
use fs25_auto_drive_host_bridge::HostActiveTool;
use glam::Vec2;

use super::EditorApp;

impl EditorApp {
    /// Zeichnet Tool-Preview und Distanzen-Overlay ueber den Viewport.
    /// Gibt gesammelte Overlay-Events als `AppIntent`-Vec zurueck.
    /// `chrome_snapshot` wird vom Aufrufer uebergeben, damit kein doppelter Build pro Frame entsteht.
    pub(super) fn render_overlays(
        &mut self,
        ui: &egui::Ui,
        rect: egui::Rect,
        response: &egui::Response,
        viewport_size: [f32; 2],
        chrome_snapshot: &fs25_auto_drive_host_bridge::HostChromeSnapshot,
    ) -> Vec<AppIntent> {
        let mut overlay_events: Vec<AppIntent> = Vec::new();
        let vp = Vec2::new(viewport_size[0], viewport_size[1]);
        let session_snapshot = self.session.snapshot_owned();
        let camera = Camera2D {
            position: Vec2::new(
                session_snapshot.viewport.camera_position[0],
                session_snapshot.viewport.camera_position[1],
            ),
            zoom: session_snapshot.viewport.zoom,
        };

        if chrome_snapshot.active_tool == HostActiveTool::Route
            && let Some(hover_pos) = response.hover_pos()
        {
            let local = hover_pos - rect.min;
            let cursor_world = camera.screen_to_world(Vec2::new(local.x, local.y), vp);
            self.last_cursor_world = Some(cursor_world);
        }

        let overlay_snapshot = self
            .session
            .build_viewport_overlay_snapshot(self.last_cursor_world);

        // ── Tool-Preview-Overlay ─────────────
        if let Some(preview) = overlay_snapshot.route_tool_preview.as_ref() {
            let painter = ui.painter_at(rect);
            let ctx = ui::tool_preview::ToolPreviewContext {
                painter: &painter,
                rect,
                camera: &camera,
                viewport_size: vp,
                preview,
                options: &chrome_snapshot.options,
            };

            ui::render_tool_preview(&ctx);
        }

        // ── Paste-Vorschau-Overlay ──────────────
        if let Some(preview) = overlay_snapshot.clipboard_preview.as_ref() {
            ui::paint_clipboard_snapshot_preview(&ui.painter_at(rect), rect, &camera, vp, preview);
        }

        // ── Distanzen-Vorschau-Overlay ──────────
        if let Some(distance_preview) = overlay_snapshot.distance_preview.as_ref() {
            ui::paint_preview_polyline(
                &ui.painter_at(rect),
                rect,
                &camera,
                vp,
                &distance_preview.points,
            );
        }

        // ── Segment-Overlay ──────────────────
        if !overlay_snapshot.group_locks.is_empty() {
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
                &camera,
                vp,
                &overlay_snapshot.group_locks,
                clicked_pos,
                ctrl_held,
                chrome_snapshot.options.segment_lock_icon_size_px,
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

        // ── Gruppen-Boundary-Overlay ──────────────────
        if !overlay_snapshot.group_boundaries.is_empty() {
            // Icons lazy initialisieren (benoetigen egui::Context)
            if self.group_boundary_icons.is_none() {
                self.group_boundary_icons = Some(ui::GroupBoundaryIcons::load(ui.ctx()));
            }
            if let Some(icons) = &self.group_boundary_icons {
                let painter = ui.painter_at(rect);
                ui::render_group_boundary_overlays(
                    &painter,
                    rect,
                    &camera,
                    vp,
                    &overlay_snapshot.group_boundaries,
                    icons,
                    chrome_snapshot.options.segment_lock_icon_size_px,
                );
            }
        }

        if overlay_snapshot.show_no_file_hint {
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
