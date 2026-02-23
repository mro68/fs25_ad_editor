//! Tool-Preview-Overlay: Zeichnet die Vorschau-Geometrie des aktiven Route-Tools.

use eframe::egui;
use glam::Vec2;

use crate::app::tools::{ToolManager, ToolPreview};
use crate::app::{Camera2D, RoadMap};

/// Zeichnet das Tool-Preview-Overlay in den Viewport.
///
/// Liest die Preview-Daten vom aktiven Route-Tool und rendert
/// Verbindungen (Linien) und Nodes (Kreise/Rauten) halbtransparent.
pub fn render_tool_preview(
    painter: &egui::Painter,
    rect: egui::Rect,
    camera: &Camera2D,
    viewport_size: Vec2,
    tool_manager: &ToolManager,
    road_map: &RoadMap,
    cursor_world: Vec2,
) {
    let Some(tool) = tool_manager.active_tool() else {
        return;
    };

    let preview = tool.preview(cursor_world, road_map);
    if preview.nodes.is_empty() {
        return;
    }

    paint_preview(painter, rect, camera, viewport_size, &preview);
}

/// Zeichnet eine `ToolPreview`-Geometrie (Verbindungen + Nodes).
fn paint_preview(
    painter: &egui::Painter,
    rect: egui::Rect,
    camera: &Camera2D,
    viewport_size: Vec2,
    preview: &ToolPreview,
) {
    let preview_color = egui::Color32::from_rgba_unmultiplied(0, 200, 255, 180);
    let cp_color = egui::Color32::from_rgba_unmultiplied(255, 160, 0, 220);
    let mut has_connection = vec![false; preview.nodes.len()];

    for &(a, b) in &preview.connections {
        if let Some(flag) = has_connection.get_mut(a) {
            *flag = true;
        }
        if let Some(flag) = has_connection.get_mut(b) {
            *flag = true;
        }
    }

    // Verbindungen zeichnen
    for &(a, b) in &preview.connections {
        if let (Some(&pa), Some(&pb)) = (preview.nodes.get(a), preview.nodes.get(b)) {
            let sa = camera.world_to_screen(pa, viewport_size);
            let sb = camera.world_to_screen(pb, viewport_size);
            painter.line_segment(
                [
                    egui::pos2(rect.min.x + sa.x, rect.min.y + sa.y),
                    egui::pos2(rect.min.x + sb.x, rect.min.y + sb.y),
                ],
                egui::Stroke::new(2.0, preview_color),
            );
        }
    }

    // Nodes zeichnen
    for (i, &pos) in preview.nodes.iter().enumerate() {
        let sp = camera.world_to_screen(pos, viewport_size);
        let screen_pos = egui::pos2(rect.min.x + sp.x, rect.min.y + sp.y);

        // Steuerpunkte (ohne Verbindung) als Raute, Rest als Kreis
        let is_control = !has_connection[i];
        if is_control {
            paint_diamond(painter, screen_pos, 5.0, cp_color);
        } else {
            painter.circle_filled(screen_pos, 3.5, preview_color);
        }
    }
}

/// Zeichnet eine Raute (Steuerpunkt-Marker).
fn paint_diamond(painter: &egui::Painter, center: egui::Pos2, size: f32, color: egui::Color32) {
    let stroke = egui::Stroke::new(2.0, color);
    let top = egui::pos2(center.x, center.y - size);
    let right = egui::pos2(center.x + size, center.y);
    let bottom = egui::pos2(center.x, center.y + size);
    let left = egui::pos2(center.x - size, center.y);

    painter.line_segment([top, right], stroke);
    painter.line_segment([right, bottom], stroke);
    painter.line_segment([bottom, left], stroke);
    painter.line_segment([left, top], stroke);
}
