use image::{Rgb, RgbImage};

use super::OverviewOptions;
use crate::text;

/// Legende-Einträge: (Farbe, Beschreibung).
const LEGEND_ITEMS: &[([u8; 3], &str)] = &[
    ([100, 100, 105], "Asphalt / Straße"),
    ([165, 165, 160], "Beton / Zement"),
    ([90, 150, 60], "Grasland"),
    ([50, 110, 45], "Wald"),
    ([140, 110, 70], "Erde / Boden"),
    ([160, 150, 130], "Kies"),
    ([95, 75, 50], "Schlamm"),
    ([195, 180, 140], "Sand"),
    ([70, 95, 120], "Wasser"),
    ([150, 145, 140], "Gehweg / Pflaster"),
];

/// Zeichnet die Farbschlüssel-Legende auf das Bild.
///
/// Die Legende wird unten links als halbtransparente Box dargestellt.
/// Enthält Terrain-Farben, POI-Markierung und Farmland-Grenzfarbe.
pub fn draw_legend(image: &mut RgbImage, options: &OverviewOptions) {
    let scale = (image.width() / 1200).clamp(1, 4);
    let padding = 15i32 * scale as i32 / 2;
    let row_h = 20i32 * scale as i32 / 2 + padding;
    let swatch_size = 14u32 * scale / 2;

    let mut rows = LEGEND_ITEMS.len();
    if options.pois {
        rows += 1;
    }
    if options.farmlands {
        rows += 1;
    }
    rows += 1;

    let legend_w = (200 * scale / 2).max(180);
    let legend_h = (rows as i32 * row_h + padding * 2) as u32;

    let lx = 20i32;
    let ly = image.height() as i32 - legend_h as i32 - 20;

    text::draw_rect_blended(image, lx, ly, legend_w, legend_h, Rgb([30, 30, 30]), 0.78);

    let mut yo = ly + padding;
    text::draw_text(
        image,
        lx + padding,
        yo,
        "Legende",
        Rgb([255, 255, 255]),
        scale,
    );
    yo += row_h + padding / 2;

    for &(color, label) in LEGEND_ITEMS {
        text::draw_rect_filled(
            image,
            lx + padding,
            yo,
            swatch_size,
            swatch_size,
            Rgb(color),
        );
        text::draw_text(
            image,
            lx + padding + swatch_size as i32 + 8,
            yo + 2,
            label,
            Rgb([255, 255, 255]),
            scale.max(1),
        );
        yo += row_h;
    }

    if options.pois {
        let r = (swatch_size / 2) as i32;
        let center_x = lx + padding + r;
        let center_y = yo + r;
        draw_filled_circle(image, center_x, center_y, r, Rgb([220, 50, 50]));
        text::draw_text(
            image,
            lx + padding + swatch_size as i32 + 8,
            yo + 2,
            "Gebäude / POI",
            Rgb([255, 255, 255]),
            scale,
        );
        yo += row_h;
    }

    if options.farmlands {
        text::draw_rect_filled(
            image,
            lx + padding,
            yo + swatch_size as i32 / 2 - 1,
            swatch_size,
            3,
            Rgb([255, 220, 50]),
        );
        text::draw_text(
            image,
            lx + padding + swatch_size as i32 + 8,
            yo + 2,
            "Farmland-Grenze",
            Rgb([255, 255, 255]),
            scale,
        );
    }
}

/// Zeichnet eine Titel-Bar am oberen Bildrand.
///
/// Halbtransparenter Hintergrund mit dem Kartennamen.
pub fn draw_title_bar(image: &mut RgbImage, title: &str) {
    let scale = (image.width() / 800).clamp(2, 6);
    let bar_h = text::text_height(scale) + scale * 6;
    let bar_w = image.width();

    text::draw_rect_blended(image, 0, 0, bar_w, bar_h, Rgb([30, 30, 30]), 0.7);

    let label = format!("{} - Overview", title);
    let tx = (scale * 4) as i32;
    let ty = (scale * 3) as i32;
    text::draw_text(image, tx, ty, &label, Rgb([255, 255, 255]), scale);
}

fn draw_filled_circle(image: &mut RgbImage, cx: i32, cy: i32, radius: i32, color: Rgb<u8>) {
    let w = image.width() as i32;
    let h = image.height() as i32;

    for dy in -radius..=radius {
        for dx in -radius..=radius {
            if dx * dx + dy * dy <= radius * radius {
                let x = cx + dx;
                let y = cy + dy;
                if x >= 0 && x < w && y >= 0 && y < h {
                    image.put_pixel(x as u32, y as u32, color);
                }
            }
        }
    }
}
