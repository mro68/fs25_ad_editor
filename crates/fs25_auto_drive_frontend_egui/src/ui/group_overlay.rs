//! Gruppen-Overlay: Zeichnet Lock-Icons ueber selektierten Gruppen-Nodes.
//!
//! Pro Segment wird maximal ein Schloss-Icon ueber der uebergebenen Weltposition
//! angezeigt. Ein Klick auf das Icon loest
//! `GroupOverlayEvent::LockToggled` aus.

use eframe::egui;
use glam::Vec2;

use crate::app::ui_contract::GroupLockOverlaySnapshot;
use crate::app::Camera2D;

/// Event, der vom Segment-Overlay ausgeloest wird.
#[derive(Debug, Clone)]
pub enum GroupOverlayEvent {
    /// Der Lock-Zustand des Segments soll umgeschaltet werden.
    LockToggled { segment_id: u64 },
    /// Das Segment soll aufgeloest werden (nur Segment-Record entfernen).
    Dissolved { segment_id: u64 },
}

/// Zeichnet Lock-Icons ueber selektierten Segment-Nodes.
///
/// Der UI-Layer rendert ausschliesslich aus den host-neutralen Overlay-Daten.
/// Ein Klick auf ein Icon loest
/// `GroupOverlayEvent::LockToggled` aus. `Ctrl` + Klick loest
/// `GroupOverlayEvent::Dissolved` aus.
///
/// # Parameter
/// - `painter`: egui-Painter fuer den Viewport
/// - `rect`: Viewport-Rechteck in Screen-Koordinaten
/// - `camera`: Kamera fuer Welt→Screen-Transformation
/// - `viewport_size`: Viewport-Abmessungen (Pixel)
/// - `overlays`: Vorberechnete Segment-Lock-Overlays aus dem App-Layer
/// - `clicked_pos`: Screen-Position eines Klicks in diesem Frame (None = kein Klick)
/// - `ctrl_held`: true, wenn im Klick-Frame die Ctrl-Taste gedrueckt war
/// - `icon_size_px`: Schriftgroesse des Lock-Icons in Pixeln
#[allow(clippy::too_many_arguments)]
pub fn render_group_overlays(
    painter: &egui::Painter,
    rect: egui::Rect,
    camera: &Camera2D,
    viewport_size: Vec2,
    overlays: &[GroupLockOverlaySnapshot],
    clicked_pos: Option<egui::Pos2>,
    ctrl_held: bool,
    icon_size_px: f32,
) -> Vec<GroupOverlayEvent> {
    let mut events = Vec::new();

    if overlays.is_empty() {
        return events;
    }

    let icon_size = icon_size_px.max(4.0);
    let icon_offset = icon_size + 12.0;
    let bg_size = egui::vec2(icon_size + 8.0, icon_size + 8.0);

    for overlay in overlays {
        let screen_local = camera.world_to_screen(overlay.world_pos, viewport_size);
        let icon_pos = egui::pos2(
            rect.min.x + screen_local.x,
            rect.min.y + screen_local.y - icon_offset,
        );

        let icon_text = if overlay.locked {
            "\u{1F512}"
        } else {
            "\u{1F513}"
        };

        let bg_rect = egui::Rect::from_center_size(icon_pos, bg_size);
        painter.rect_filled(
            bg_rect,
            4.0,
            egui::Color32::from_rgba_unmultiplied(30, 30, 30, 200),
        );

        painter.text(
            icon_pos,
            egui::Align2::CENTER_CENTER,
            icon_text,
            egui::FontId::proportional(icon_size),
            egui::Color32::WHITE,
        );

        if let Some(click) = clicked_pos {
            if bg_rect.expand(4.0).contains(click) {
                let ev = if ctrl_held {
                    GroupOverlayEvent::Dissolved {
                        segment_id: overlay.segment_id,
                    }
                } else {
                    GroupOverlayEvent::LockToggled {
                        segment_id: overlay.segment_id,
                    }
                };
                events.push(ev);
            }
        }
    }

    events
}
