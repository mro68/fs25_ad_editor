//! Segment-Overlay: Zeichnet Rahmen und Lock-Icons als egui-Overlay.
//!
//! Fuer jedes gueltiges Segment in der Registry wird eine Bounding-Box
//! gezeichnet sowie an den 4 Seitenmittelpunkten je ein Lock-Icon.

use eframe::egui;
use glam::Vec2;

use crate::app::SegmentRegistry;
use crate::core::{Camera2D, RoadMap};

/// Event, der vom Segment-Overlay ausgeloest wird.
#[derive(Debug, Clone)]
pub enum SegmentOverlayEvent {
    /// Der Lock-Zustand des Segments soll umgeschaltet werden.
    LockToggled { segment_id: u64 },
}

/// Zeichnet Segment-Rahmen und Lock-Icons als egui-Overlay.
///
/// Iteriert ueber alle gueltigen Segmente in der Registry und zeichnet:
/// 1. Eine halbtransparente Fuellung (nur wenn locked)
/// 2. Einen Rahmen um die Bounding-Box
/// 3. Lock-Icons an N/O/S/W-Seitenmittelpunkten
///
/// Gibt Events zurueck, wenn ein Lock-Icon in diesem Frame angeklickt wurde.
///
/// # Parameter
/// - `painter`: egui-Painter fuer den Viewport
/// - `rect`: Viewport-Rechteck in Screen-Koordinaten
/// - `camera`: Kamera fuer Welt→Screen-Transformation
/// - `viewport_size`: Viewport-Abmessungen (Pixel)
/// - `registry`: Segment-Registry
/// - `road_map`: RoadMap fuer Node-Positionen
/// - `clicked_pos`: Screen-Position eines Klicks in diesem Frame (None = kein Klick)
pub fn render_segment_overlays(
    painter: &egui::Painter,
    rect: egui::Rect,
    camera: &Camera2D,
    viewport_size: Vec2,
    registry: &SegmentRegistry,
    road_map: &RoadMap,
    clicked_pos: Option<egui::Pos2>,
) -> Vec<SegmentOverlayEvent> {
    let mut events = Vec::new();

    for record in registry.records() {
        if !registry.is_segment_valid(record, road_map) {
            continue;
        }
        let Some((world_min, world_max)) = registry.segment_bounding_box(record.id, road_map)
        else {
            continue;
        };

        // Welt-AABB → Screen-AABB
        let s_a = camera.world_to_screen(world_min, viewport_size);
        let s_b = camera.world_to_screen(world_max, viewport_size);
        let screen_min = egui::pos2(rect.min.x + s_a.x.min(s_b.x), rect.min.y + s_a.y.min(s_b.y));
        let screen_max = egui::pos2(rect.min.x + s_a.x.max(s_b.x), rect.min.y + s_a.y.max(s_b.y));
        let screen_rect = egui::Rect::from_min_max(screen_min, screen_max);

        // Fuellung nur wenn locked (15% Schwarz = 38 von 255)
        if record.locked {
            painter.rect_filled(
                screen_rect,
                0.0,
                egui::Color32::from_rgba_unmultiplied(0, 0, 0, 38),
            );
        }

        // Rahmen
        let stroke_color = if record.locked {
            egui::Color32::from_rgba_unmultiplied(255, 200, 0, 200)
        } else {
            egui::Color32::from_rgba_unmultiplied(160, 160, 160, 140)
        };
        painter.rect_stroke(
            screen_rect,
            0.0,
            egui::Stroke::new(1.0, stroke_color),
            egui::StrokeKind::Inside,
        );

        // Lock-Icons an den 4 Seitenmittelpunkten
        let cx = (screen_min.x + screen_max.x) * 0.5;
        let cy = (screen_min.y + screen_max.y) * 0.5;
        let icon_positions = [
            egui::pos2(cx, screen_min.y), // N (oben)
            egui::pos2(screen_max.x, cy), // O (rechts)
            egui::pos2(cx, screen_max.y), // S (unten)
            egui::pos2(screen_min.x, cy), // W (links)
        ];

        let icon_text = if record.locked {
            "\u{1F512}"
        } else {
            "\u{1F513}"
        };
        let font_id = egui::FontId::proportional(14.0);
        let hit_half = 12.0_f32;

        for &icon_pos in &icon_positions {
            // Hintergrund-Box fuer bessere Lesbarkeit
            let bg_rect = egui::Rect::from_center_size(icon_pos, egui::vec2(20.0, 20.0));
            painter.rect_filled(
                bg_rect,
                3.0,
                egui::Color32::from_rgba_unmultiplied(30, 30, 30, 190),
            );

            // Icon-Text
            painter.text(
                icon_pos,
                egui::Align2::CENTER_CENTER,
                icon_text,
                font_id.clone(),
                egui::Color32::WHITE,
            );

            // Klick-Erkennung (Hit-Test auf 24x24 Bereich)
            if let Some(click) = clicked_pos {
                let hit_rect = egui::Rect::from_center_size(
                    icon_pos,
                    egui::vec2(hit_half * 2.0, hit_half * 2.0),
                );
                if hit_rect.contains(click) {
                    events.push(SegmentOverlayEvent::LockToggled {
                        segment_id: record.id,
                    });
                }
            }
        }
    }

    events
}
