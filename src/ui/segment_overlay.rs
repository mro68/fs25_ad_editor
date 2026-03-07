//! Segment-Overlay: Zeichnet ein Lock-Icon ueber dem selektierten Segment-Node.
//!
//! Wenn genau ein Node selektiert ist und dieser zu einem gespeicherten Segment
//! gehoert, wird ein Schloss-Icon ueber dem Node angezeigt. Ein Klick auf das
//! Icon loest `SegmentOverlayEvent::LockToggled` aus.

use eframe::egui;
use glam::Vec2;
use indexmap::IndexSet;

use crate::app::{Camera2D, RoadMap, SegmentRegistry};

/// Event, der vom Segment-Overlay ausgeloest wird.
#[derive(Debug, Clone)]
pub enum SegmentOverlayEvent {
    /// Der Lock-Zustand des Segments soll umgeschaltet werden.
    LockToggled { segment_id: u64 },
}

/// Zeichnet ein Lock-Icon ueber dem selektierten Segment-Node.
///
/// Wenn genau ein Node selektiert ist und dieser zu einem gueltigen Segment
/// gehoert, wird ein Schloss-Icon (🔒 oder 🔓) 28px ueber dem Node gerendert.
/// Ein Klick auf das Icon loest `SegmentOverlayEvent::LockToggled` aus.
///
/// # Parameter
/// - `painter`: egui-Painter fuer den Viewport
/// - `rect`: Viewport-Rechteck in Screen-Koordinaten
/// - `camera`: Kamera fuer Welt→Screen-Transformation
/// - `viewport_size`: Viewport-Abmessungen (Pixel)
/// - `registry`: Segment-Registry
/// - `road_map`: RoadMap fuer Node-Positionen
/// - `selected_node_ids`: Aktuell selektierte Node-IDs
/// - `clicked_pos`: Screen-Position eines Klicks in diesem Frame (None = kein Klick)
pub fn render_segment_overlays(
    painter: &egui::Painter,
    rect: egui::Rect,
    camera: &Camera2D,
    viewport_size: Vec2,
    registry: &SegmentRegistry,
    road_map: &RoadMap,
    selected_node_ids: &IndexSet<u64>,
    clicked_pos: Option<egui::Pos2>,
) -> Vec<SegmentOverlayEvent> {
    let mut events = Vec::new();

    // Nur wenn genau 1 Node selektiert
    if selected_node_ids.len() != 1 {
        return events;
    }
    let selected_id = *selected_node_ids.iter().next().expect("len == 1");

    // Pruefen ob der selektierte Node zu einem gueltigen Segment gehoert
    let Some(record) = registry.find_first_by_node_id(selected_id) else {
        return events;
    };
    if !registry.is_segment_valid(record, road_map) {
        return events;
    }

    // Node-Position aus RoadMap holen
    let Some(node) = road_map.nodes.get(&selected_id) else {
        return events;
    };

    // Welt→Screen: 28px ueber dem Node
    let screen_local = camera.world_to_screen(node.position, viewport_size);
    let icon_pos = egui::pos2(
        rect.min.x + screen_local.x,
        rect.min.y + screen_local.y - 28.0,
    );

    // Icon-Text je nach Lock-Zustand
    let icon_text = if record.locked {
        "\u{1F512}" // 🔒
    } else {
        "\u{1F513}" // 🔓
    };

    // Hintergrund-Box fuer bessere Lesbarkeit
    let bg_rect = egui::Rect::from_center_size(icon_pos, egui::vec2(24.0, 24.0));
    painter.rect_filled(
        bg_rect,
        4.0,
        egui::Color32::from_rgba_unmultiplied(30, 30, 30, 200),
    );

    // Icon zeichnen
    painter.text(
        icon_pos,
        egui::Align2::CENTER_CENTER,
        icon_text,
        egui::FontId::proportional(16.0),
        egui::Color32::WHITE,
    );

    // Klick-Erkennung (Hit-Test mit kleinem Extra-Padding)
    if let Some(click) = clicked_pos {
        if bg_rect.expand(4.0).contains(click) {
            events.push(SegmentOverlayEvent::LockToggled {
                segment_id: record.id,
            });
        }
    }

    events
}
