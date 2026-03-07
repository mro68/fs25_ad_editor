
//! Segment-Overlay: Zeichnet Lock-Icons ueber selektierten Segment-Nodes.
//!
//! Fuer jeden selektierten Node wird geprueft, ob er zu einem gespeicherten Segment
//! gehoert. Pro Segment wird maximal ein Schloss-Icon (ueber dem ersten selektierten
//! Node dieses Segments) angezeigt. Ein Klick auf das Icon loest
//! `SegmentOverlayEvent::LockToggled` aus.

use eframe::egui;
use glam::Vec2;
use indexmap::IndexSet;

use crate::app::{Camera2D, RoadMap, SegmentRegistry};

/// Event, der vom Segment-Overlay ausgeloest wird.
#[derive(Debug, Clone)]
pub enum SegmentOverlayEvent {
    /// Der Lock-Zustand des Segments soll umgeschaltet werden.
    LockToggled { segment_id: u64 },
    /// Das Segment soll aufgeloest werden (nur Segment-Record entfernen).
    Dissolved { segment_id: u64 },
}

/// Zeichnet Lock-Icons ueber selektierten Segment-Nodes.
///
/// Fuer jeden selektierten Node wird geprueft, ob er zu einem gueltigen Segment
/// gehoert. Pro Segment wird ein Schloss-Icon (🔒 oder 🔓) 28px ueber dem ersten
/// selektierten Node dieses Segments gerendert. Bei Multi-Selection ueber mehrere
/// Segmente werden mehrere Icons gezeichnet.
/// Ein Klick auf ein Icon loest `SegmentOverlayEvent::LockToggled` aus.
/// `Ctrl` + Klick loest `SegmentOverlayEvent::Dissolved` aus.
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
/// - `ctrl_held`: true, wenn im Klick-Frame die Ctrl-Taste gedrueckt war
#[allow(clippy::too_many_arguments)]
pub fn render_segment_overlays(
    painter: &egui::Painter,
    rect: egui::Rect,
    camera: &Camera2D,
    viewport_size: Vec2,
    registry: &SegmentRegistry,
    road_map: &RoadMap,
    selected_node_ids: &IndexSet<u64>,
    clicked_pos: Option<egui::Pos2>,
    ctrl_held: bool,
) -> Vec<SegmentOverlayEvent> {
    let mut events = Vec::new();

    // Bei leerer Selektion gibt es nichts zu zeichnen
    if selected_node_ids.is_empty() {
        return events;
    }

    // Pro Segment nur ein Icon zeichnen (Deduplizierung ueber Segment-ID)
    let mut seen_segment_ids = std::collections::HashSet::new();

    for &selected_id in selected_node_ids.iter() {
        // Pruefen ob dieser Node zu einem Segment gehoert
        let Some(record) = registry.find_first_by_node_id(selected_id) else {
            continue;
        };

        // Jedes Segment hoechstens einmal (Icon ueber erstem selektierten Node)
        if !seen_segment_ids.insert(record.id) {
            continue;
        }

        if !registry.is_segment_valid(record, road_map) {
            continue;
        }

        // Node-Position aus RoadMap holen
        let Some(node) = road_map.nodes.get(&selected_id) else {
            continue;
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
                if ctrl_held {
                    events.push(SegmentOverlayEvent::Dissolved {
                        segment_id: record.id,
                    });
                } else {
                    events.push(SegmentOverlayEvent::LockToggled {
                        segment_id: record.id,
                    });
                }
            }
        }
    }

    events
}
