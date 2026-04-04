//! Tool-Preview-Overlay: Zeichnet die Vorschau-Geometrie des aktiven Route-Tools.

use eframe::egui;
use glam::Vec2;

use crate::app::state::Clipboard;
use crate::app::tools::ToolPreview;
use crate::app::{Camera2D, ConnectionDirection, ConnectionPriority};
use crate::shared::EditorOptions;

/// Kontext-Bündel für `render_tool_preview`.
///
/// Kapselt mehrere Parameter (Painter, Viewport, Camera, RoadMap, etc.),
/// damit die Funktion unter dem Clippy-Limit für Argumentanzahl bleibt.
pub struct ToolPreviewContext<'a> {
    pub painter: &'a egui::Painter,
    pub rect: egui::Rect,
    pub camera: &'a Camera2D,
    pub viewport_size: Vec2,
    pub preview: &'a ToolPreview,
    pub options: &'a EditorOptions,
}

/// Zeichnet das Tool-Preview-Overlay in den Viewport.
///
/// Rendert bereits app-seitig vorbereitete Preview-Daten als halbtransparentes Overlay.
pub fn render_tool_preview(ctx: &ToolPreviewContext<'_>) {
    if ctx.preview.nodes.is_empty() {
        return;
    }

    paint_preview(
        ctx.painter,
        ctx.rect,
        ctx.camera,
        ctx.viewport_size,
        ctx.preview,
        ctx.options,
    );
}

/// Zeichnet eine `ToolPreview`-Geometrie (Verbindungen + Nodes).
pub fn paint_preview(
    painter: &egui::Painter,
    rect: egui::Rect,
    camera: &Camera2D,
    viewport_size: Vec2,
    preview: &ToolPreview,
    options: &EditorOptions,
) {
    let world_per_pixel = camera.world_per_pixel(viewport_size.y);
    if world_per_pixel <= 0.0 {
        return;
    }

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

    let thickness_main_px = (options.connection_thickness_world / world_per_pixel).max(1.0);
    let thickness_sub_px = (options.connection_thickness_subprio_world / world_per_pixel).max(1.0);
    let arrow_len_px = (options.arrow_length_world / world_per_pixel).max(0.5);
    let arrow_width_px = (options.arrow_width_world / world_per_pixel).max(0.5);

    // Verbindungen zeichnen
    for (idx, &(a, b)) in preview.connections.iter().enumerate() {
        let style = preview
            .connection_styles
            .get(idx)
            .copied()
            .unwrap_or((ConnectionDirection::Regular, ConnectionPriority::Regular));
        let color = preview_connection_color(style.0, style.1, options);
        let thickness_px = match style.1 {
            ConnectionPriority::Regular => thickness_main_px,
            ConnectionPriority::SubPriority => thickness_sub_px,
        };

        if let (Some(&pa), Some(&pb)) = (preview.nodes.get(a), preview.nodes.get(b)) {
            let sa = camera.world_to_screen(pa, viewport_size);
            let sb = camera.world_to_screen(pb, viewport_size);
            let from = egui::pos2(rect.min.x + sa.x, rect.min.y + sa.y);
            let to = egui::pos2(rect.min.x + sb.x, rect.min.y + sb.y);
            painter.line_segment([from, to], egui::Stroke::new(thickness_px, color));

            if style.0 != ConnectionDirection::Dual {
                paint_arrow(painter, from, to, color, arrow_len_px, arrow_width_px);
            }
        }
    }

    // Nodes zeichnen
    for (i, &pos) in preview.nodes.iter().enumerate() {
        let sp = camera.world_to_screen(pos, viewport_size);
        let screen_pos = egui::pos2(rect.min.x + sp.x, rect.min.y + sp.y);

        let is_control = !has_connection[i];
        if is_control {
            paint_diamond(painter, screen_pos, 5.0, cp_color);
        } else {
            painter.circle_filled(screen_pos, 3.5, preview_node_color(options));
        }
    }

    // Node-Labels zeichnen (z.B. fuer Parking-Tool Vorschau)
    let label_color = egui::Color32::from_rgba_unmultiplied(220, 220, 220, 180);
    for &(node_idx, ref text) in &preview.labels {
        if let Some(&pos) = preview.nodes.get(node_idx) {
            let sp = camera.world_to_screen(pos, viewport_size);
            let screen_pos = egui::pos2(rect.min.x + sp.x + 8.0, rect.min.y + sp.y - 8.0);
            painter.text(
                screen_pos,
                egui::Align2::LEFT_BOTTOM,
                text,
                egui::FontId::proportional(11.0),
                label_color,
            );
        }
    }
}

/// Zeichnet eine einfache Polyline-Vorschau ohne temporaere `ToolPreview`-Allokationen.
///
/// Verbindungen werden implizit als aufeinanderfolgende Punkte (`i -> i+1`) gezeichnet.
pub fn paint_preview_polyline(
    painter: &egui::Painter,
    rect: egui::Rect,
    camera: &Camera2D,
    viewport_size: Vec2,
    positions: &[Vec2],
) {
    if positions.is_empty() {
        return;
    }

    let preview_color = egui::Color32::from_rgba_unmultiplied(0, 200, 255, 180);

    if positions.len() >= 2 {
        for window in positions.windows(2) {
            let pa = window[0];
            let pb = window[1];
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

    for &pos in positions {
        let sp = camera.world_to_screen(pos, viewport_size);
        let screen_pos = egui::pos2(rect.min.x + sp.x, rect.min.y + sp.y);
        painter.circle_filled(screen_pos, 3.5, preview_color);
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

fn paint_arrow(
    painter: &egui::Painter,
    from: egui::Pos2,
    to: egui::Pos2,
    color: egui::Color32,
    length_px: f32,
    width_px: f32,
) {
    let dir = to - from;
    let len = dir.length();
    if len <= f32::EPSILON {
        return;
    }
    let dir_norm = dir / len;
    let center = from + dir_norm * (len * 0.5);
    let tip = center + dir_norm * (length_px * 2.0 / 3.0);
    let base = center - dir_norm * (length_px / 3.0);
    let perp = egui::Vec2::new(-dir_norm.y, dir_norm.x);
    let left = base + perp * (width_px * 0.5);
    let right = base - perp * (width_px * 0.5);

    painter.add(egui::epaint::Shape::convex_polygon(
        vec![tip, left, right],
        color,
        egui::Stroke::NONE,
    ));
}

fn preview_node_color(options: &EditorOptions) -> egui::Color32 {
    color32_from_rgba(options.connection_color_regular)
}

fn preview_connection_color(
    direction: ConnectionDirection,
    priority: ConnectionPriority,
    options: &EditorOptions,
) -> egui::Color32 {
    let base = match direction {
        ConnectionDirection::Regular => options.connection_color_regular,
        ConnectionDirection::Dual => options.connection_color_dual,
        ConnectionDirection::Reverse => options.connection_color_reverse,
    };

    let color = match priority {
        ConnectionPriority::Regular => base,
        ConnectionPriority::SubPriority => [
            (base[0] + 1.0) * 0.5,
            (base[1] + 1.0) * 0.5,
            (base[2] + 1.0) * 0.5,
            base[3],
        ],
    };

    color32_from_rgba(color)
}

fn color32_from_rgba(color: [f32; 4]) -> egui::Color32 {
    egui::Color32::from_rgba_unmultiplied(
        (color[0].clamp(0.0, 1.0) * 255.0) as u8,
        (color[1].clamp(0.0, 1.0) * 255.0) as u8,
        (color[2].clamp(0.0, 1.0) * 255.0) as u8,
        (color[3].clamp(0.0, 1.0) * 255.0) as u8,
    )
}

/// Zeichnet die Clipboard-Vorschau (Paste-Preview) im Viewport.
///
/// Rendert alle kopierten Nodes und internen Verbindungen an der aktuellen
/// Vorschauposition (`paste_pos`) mit einstellbarer Deckkraft (`opacity`).
pub fn paint_clipboard_preview(
    painter: &egui::Painter,
    rect: egui::Rect,
    camera: &Camera2D,
    viewport_size: Vec2,
    clipboard: &Clipboard,
    paste_pos: Vec2,
    opacity: f32,
) {
    if clipboard.nodes.is_empty() {
        return;
    }

    let alpha = (opacity.clamp(0.0, 1.0) * 255.0) as u8;
    let node_color = egui::Color32::from_rgba_unmultiplied(100, 200, 255, alpha);
    let conn_color =
        egui::Color32::from_rgba_unmultiplied(100, 200, 255, (alpha as u16 * 3 / 4) as u8);
    let marker_color = egui::Color32::from_rgba_unmultiplied(255, 200, 50, alpha);

    let offset = paste_pos - clipboard.center;

    // Positions-Lookup fuer Verbindungen — linearer Scan statt HashMap-Allokation,
    // da Clipboard-Inhalte typischerweise klein sind (< 1000 Nodes).
    for conn in &clipboard.connections {
        let find_pos = |id: u64| {
            clipboard
                .nodes
                .iter()
                .find(|n| n.id == id)
                .map(|n| n.position + offset)
        };
        let Some(start_pos) = find_pos(conn.start_id) else {
            continue;
        };
        let Some(end_pos) = find_pos(conn.end_id) else {
            continue;
        };
        let sa = camera.world_to_screen(start_pos, viewport_size);
        let sb = camera.world_to_screen(end_pos, viewport_size);
        let from = egui::pos2(rect.min.x + sa.x, rect.min.y + sa.y);
        let to = egui::pos2(rect.min.x + sb.x, rect.min.y + sb.y);
        painter.line_segment([from, to], egui::Stroke::new(2.0, conn_color));
    }

    // Nodes zeichnen — Marker-Check per .any() ohne HashSet-Allokation
    for node in &clipboard.nodes {
        let world_pos = node.position + offset;
        let sp = camera.world_to_screen(world_pos, viewport_size);
        let screen_pos = egui::pos2(rect.min.x + sp.x, rect.min.y + sp.y);
        let color = if clipboard.markers.iter().any(|m| m.id == node.id) {
            marker_color
        } else {
            node_color
        };
        painter.circle_filled(screen_pos, 4.0, color);
    }
}
