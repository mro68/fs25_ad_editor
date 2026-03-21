//! Gruppen-Boundary-Overlay: Zeichnet Ein-/Ausfahrt-Icons ueber Boundary-Nodes einer Gruppe.
//!
//! Fuer jeden selektierten Node wird geprueft, ob er zu einem Segment gehoert.
//! Pro Segment werden die Boundary-Nodes (Nodes mit externen Verbindungen) mit
//! Richtungs-Icons markiert: Eingang, Ausgang oder bidirektional.

use eframe::egui::{self, ColorImage, TextureHandle, TextureOptions};
use glam::Vec2;
use indexmap::IndexSet;

use crate::app::{BoundaryNode, Camera2D, RoadMap, SegmentRegistry};

const ICON_SIZE_PX: u32 = 32;

/// Rasterisiert ein SVG zu einem `ColorImage` mit der angegebenen Pixelgroesse.
fn rasterize_svg_to_color_image(svg_bytes: &[u8], size: u32) -> ColorImage {
    use resvg::{tiny_skia, usvg};
    let svg_str = std::str::from_utf8(svg_bytes).expect("SVG-Bytes sind kein valides UTF-8");
    let options = usvg::Options::default();
    let tree = usvg::Tree::from_str(svg_str, &options)
        .expect("Gruppen-Icon-SVG konnte nicht geparst werden");

    let mut pixmap = tiny_skia::Pixmap::new(size, size)
        .expect("Pixmap fuer Gruppen-Icon konnte nicht erstellt werden");

    let svg_size = tree.size();
    let scale_x = size as f32 / svg_size.width();
    let scale_y = size as f32 / svg_size.height();
    let transform = tiny_skia::Transform::from_scale(scale_x, scale_y);
    resvg::render(&tree, transform, &mut pixmap.as_mut());

    // tiny-skia liefert prae-multipliziertes RGBA → in normales RGBA umrechnen
    let mut rgba = pixmap.data().to_vec();
    for pixel in rgba.chunks_mut(4) {
        let a = pixel[3];
        if a > 0 && a < 255 {
            pixel[0] = ((pixel[0] as u32 * 255 + a as u32 / 2) / a as u32).min(255) as u8;
            pixel[1] = ((pixel[1] as u32 * 255 + a as u32 / 2) / a as u32).min(255) as u8;
            pixel[2] = ((pixel[2] as u32 * 255 + a as u32 / 2) / a as u32).min(255) as u8;
        }
    }

    ColorImage::from_rgba_unmultiplied([size as usize, size as usize], &rgba)
}

/// Gecachte egui-Textur-Handles fuer die drei Gruppen-Boundary-Icon-Typen.
pub struct GroupBoundaryIcons {
    /// Icon fuer Eingang-Nodes (externe eingehende Verbindung)
    pub entry: TextureHandle,
    /// Icon fuer Ausgang-Nodes (externe ausgehende Verbindung)
    pub exit: TextureHandle,
    /// Icon fuer bidirektionale Nodes (ein- und ausgehende externe Verbindungen)
    pub bidirectional: TextureHandle,
}

impl GroupBoundaryIcons {
    /// Laedt und rasterisiert die drei SVG-Icons als egui-Texturen.
    ///
    /// Soll einmal pro App-Lifetime aufgerufen werden (lazily beim ersten `update()`).
    pub fn load(ctx: &egui::Context) -> Self {
        let entry_bytes = include_bytes!("../../assets/icons/group_entry.svg");
        let exit_bytes = include_bytes!("../../assets/icons/group_exit.svg");
        let bidi_bytes = include_bytes!("../../assets/icons/group_bidirectional.svg");

        let entry_img = rasterize_svg_to_color_image(entry_bytes, ICON_SIZE_PX);
        let exit_img = rasterize_svg_to_color_image(exit_bytes, ICON_SIZE_PX);
        let bidi_img = rasterize_svg_to_color_image(bidi_bytes, ICON_SIZE_PX);

        Self {
            entry: ctx.load_texture("group_entry_icon", entry_img, TextureOptions::LINEAR),
            exit: ctx.load_texture("group_exit_icon", exit_img, TextureOptions::LINEAR),
            bidirectional: ctx.load_texture(
                "group_bidirectional_icon",
                bidi_img,
                TextureOptions::LINEAR,
            ),
        }
    }

    /// Gibt den passenden TextureHandle fuer einen `BoundaryNode` zurueck.
    fn icon_for(&self, node: &BoundaryNode) -> &TextureHandle {
        match (node.has_external_incoming, node.has_external_outgoing) {
            (true, true) => &self.bidirectional,
            (true, false) => &self.entry,
            (false, true) => &self.exit,
            (false, false) => &self.entry, // Fallback — sollte nicht auftreten
        }
    }
}

/// Zeichnet Boundary-Icons unterhalb der Nodes aller selektierten Segmente.
///
/// Fuer jedes selektierte Segment werden alle Boundary-Nodes ermittelt und mit
/// dem passenden Icon (Eingang/Ausgang/Bidirektional) beschriftet. Das Icon wird
/// **unterhalb** des Nodes platziert, sodass es nicht mit dem Lock-Icon (oberhalb)
/// kollidiert.
///
/// # Parameter
/// - `painter`: egui-Painter fuer den Viewport
/// - `rect`: Viewport-Rechteck in Screen-Koordinaten
/// - `camera`: Kamera fuer Welt→Screen-Transformation
/// - `viewport_size`: Viewport-Abmessungen (Pixel)
/// - `registry`: Segment-Registry mit allen gespeicherten Segmenten
/// - `road_map`: RoadMap fuer Node-Positionen
/// - `selected_node_ids`: Aktuell selektierte Node-IDs
/// - `icons`: Gecachte Textur-Handles fuer die Icon-Typen
/// - `icon_size_px`: Groesse des Icons in Pixeln
#[allow(clippy::too_many_arguments)]
pub fn render_group_boundary_overlays(
    painter: &egui::Painter,
    rect: egui::Rect,
    camera: &Camera2D,
    viewport_size: Vec2,
    registry: &SegmentRegistry,
    road_map: &RoadMap,
    selected_node_ids: &IndexSet<u64>,
    icons: &GroupBoundaryIcons,
    icon_size_px: f32,
) {
    if selected_node_ids.is_empty() {
        return;
    }

    let icon_size = icon_size_px.max(8.0);
    // Unterhalb des Nodes platzieren (positiver Y-Offset)
    let icon_offset_y = icon_size + 12.0;
    let half = icon_size * 0.5;

    let mut seen_segment_ids = std::collections::HashSet::new();

    for &selected_id in selected_node_ids.iter() {
        let Some(record) = registry.find_first_by_node_id(selected_id) else {
            continue;
        };

        if !seen_segment_ids.insert(record.id) {
            continue;
        }

        if !registry.is_segment_valid(record, road_map) {
            continue;
        }

        let Some(boundary_nodes) = registry.open_nodes(record.id, road_map) else {
            continue;
        };

        for bn in &boundary_nodes {
            let Some(node) = road_map.nodes.get(&bn.node_id) else {
                continue;
            };

            let screen_local = camera.world_to_screen(node.position, viewport_size);
            let center = egui::pos2(
                rect.min.x + screen_local.x,
                rect.min.y + screen_local.y + icon_offset_y,
            );

            let icon_rect = egui::Rect::from_center_size(center, egui::vec2(icon_size, icon_size));

            // Hintergrund-Tint damit das Icon auf jedem Untergrund sichtbar ist
            painter.rect_filled(
                icon_rect.expand(2.0),
                3.0,
                egui::Color32::from_rgba_unmultiplied(20, 20, 20, 180),
            );

            let texture_id = icons.icon_for(bn).id();
            let uv = egui::Rect::from_min_max(egui::pos2(0.0, 0.0), egui::pos2(1.0, 1.0));
            painter.image(texture_id, icon_rect, uv, egui::Color32::WHITE);

            // Tooltip (Debug-Info) nur bei sehr kleinen Icons als Unicode-Fallback
            if half < 6.0 {
                painter.text(
                    center,
                    egui::Align2::CENTER_CENTER,
                    if bn.has_external_incoming && bn.has_external_outgoing {
                        "↔"
                    } else if bn.has_external_incoming {
                        "→"
                    } else {
                        "←"
                    },
                    egui::FontId::proportional(icon_size),
                    egui::Color32::from_rgb(255, 162, 0),
                );
            }
        }
    }
}
