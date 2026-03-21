//! Gruppen-Boundary-Overlay: Zeichnet Ein-/Ausfahrt-Icons ueber Boundary-Nodes einer Gruppe.
//!
//! Fuer jeden selektierten Node wird geprueft, ob er zu einer Gruppe gehoert.
//! Pro Gruppe werden die Boundary-Nodes (Nodes mit externen Verbindungen) mit
//! Richtungs-Icons markiert: Eingang, Ausgang oder bidirektional.

use eframe::egui::{self, ColorImage, TextureHandle, TextureOptions};
use glam::Vec2;
use indexmap::IndexSet;

use crate::app::{BoundaryDirection, Camera2D, GroupRegistry, RoadMap};

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

    /// Gibt den passenden TextureHandle fuer eine `BoundaryDirection` zurueck.
    fn icon_for_direction(&self, direction: BoundaryDirection) -> &TextureHandle {
        match direction {
            BoundaryDirection::Entry => &self.entry,
            BoundaryDirection::Exit => &self.exit,
            BoundaryDirection::Bidirectional => &self.bidirectional,
        }
    }
}

/// Zeichnet Boundary-Icons unterhalb der Nodes aller selektierten Gruppen.
///
/// Fuer jede selektierte Gruppe werden alle gecachten Boundary-Nodes mit
/// dem passenden Icon (Eingang/Ausgang/Bidirektional) beschriftet. Das Icon wird
/// **unterhalb** des Nodes platziert, sodass es nicht mit dem Lock-Icon (oberhalb)
/// kollidiert.
///
/// # Parameter
/// - `painter`: egui-Painter fuer den Viewport
/// - `rect`: Viewport-Rechteck in Screen-Koordinaten
/// - `camera`: Kamera fuer Welt→Screen-Transformation
/// - `viewport_size`: Viewport-Abmessungen (Pixel)
/// - `registry`: Gruppen-Registry mit allen gespeicherten Gruppen
/// - `road_map`: RoadMap fuer Node-Positionen
/// - `selected_node_ids`: Aktuell selektierte Node-IDs
/// - `icons`: Gecachte Textur-Handles fuer die Icon-Typen
/// - `icon_size_px`: Groesse des Icons in Pixeln
/// - `show_all`: true = Icons fuer alle Boundary-Nodes; false = nur Nodes mit Verbindungen
///   zu Nodes ausserhalb JEDER registrierten Gruppe
#[allow(clippy::too_many_arguments)]
pub fn render_group_boundary_overlays(
    painter: &egui::Painter,
    rect: egui::Rect,
    camera: &Camera2D,
    viewport_size: Vec2,
    registry: &GroupRegistry,
    road_map: &RoadMap,
    selected_node_ids: &IndexSet<u64>,
    icons: &GroupBoundaryIcons,
    icon_size_px: f32,
    show_all: bool,
) {
    if selected_node_ids.is_empty() {
        return;
    }

    let icon_size = icon_size_px.max(8.0);
    // Unterhalb des Nodes platzieren (positiver Y-Offset)
    let icon_offset_y = icon_size + 12.0;
    let half = icon_size * 0.5;

    // Alle Gruppen finden, die mindestens einen selektierten Node enthalten
    let records = registry.find_by_node_ids(selected_node_ids);

    for record in records {
        if !registry.is_group_valid(record, road_map) {
            continue;
        }

        let Some(boundary_infos) = registry.boundary_cache_for(record.id) else {
            continue;
        };

        for bi in boundary_infos {
            // Eingangs-Icon-Filter:
            // show_all=true  → Icons an allen Grenzknoten anzeigen
            // show_all=false → Eingangsknoten mit gueltigem Eingangswinkel (≤90°) ausblenden,
            //                  nur ungueltige (>90°) oder fehlende Eingaenge anzeigen
            if !show_all {
                let is_entry = matches!(
                    bi.direction,
                    BoundaryDirection::Entry | BoundaryDirection::Bidirectional
                );
                if is_entry {
                    const ANGLE_THRESHOLD: f32 = std::f32::consts::FRAC_PI_2;
                    if let Some(max_dev) = bi.max_external_angle_deviation {
                        if max_dev <= ANGLE_THRESHOLD {
                            continue; // Eingangswinkel OK → kein Icon noetig
                        }
                    }
                    // >90° oder None → Icon anzeigen (ungueltig/fehlend)
                }
                // Exit-Nodes: immer anzeigen (Ausfahrt-Info bleibt sichtbar)
            }

            let Some(node) = road_map.nodes.get(&bi.node_id) else {
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

            let texture_id = icons.icon_for_direction(bi.direction).id();
            let uv = egui::Rect::from_min_max(egui::pos2(0.0, 0.0), egui::pos2(1.0, 1.0));
            painter.image(texture_id, icon_rect, uv, egui::Color32::WHITE);

            // Unicode-Fallback bei sehr kleinen Icons
            if half < 6.0 {
                painter.text(
                    center,
                    egui::Align2::CENTER_CENTER,
                    match bi.direction {
                        BoundaryDirection::Bidirectional => "↔",
                        BoundaryDirection::Entry => "→",
                        BoundaryDirection::Exit => "←",
                    },
                    egui::FontId::proportional(icon_size),
                    egui::Color32::from_rgb(255, 162, 0),
                );
            }
        }
    }
}
