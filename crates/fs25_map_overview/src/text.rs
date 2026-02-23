//! TTF-basiertes Text-Rendering mit eingebetteter DejaVu-Sans-Schrift.
//!
//! Unterstützt vollständiges Latin (inkl. Umlaute ä, ö, ü, ß etc.).
//! Font wird per `include_bytes!` eingebettet — keine externen Dateien nötig.
//!
//! Lizenz DejaVu Sans: Bitstream Vera License (siehe `assets/LICENSE-DejaVuSans.txt`)

use ab_glyph::{Font, FontRef, PxScale, ScaleFont};
use image::{Rgb, RgbImage};

/// Eingebettete DejaVu Sans Schrift.
static FONT_DATA: &[u8] = include_bytes!("../assets/DejaVuSans.ttf");

fn font() -> FontRef<'static> {
    FontRef::try_from_slice(FONT_DATA).expect("Eingebettete Schrift ungültig")
}

/// Rechnet den `scale`-Faktor in eine Pixel-Größe um.
///
/// Kompatibel zum alten Bitmap-Font: scale=1 ≈ 8px, scale=2 ≈ 16px, etc.
fn px_scale(scale: u32) -> PxScale {
    PxScale::from(scale as f32 * 8.0)
}

/// Zeichnet einen Text-String auf ein RGB-Bild.
///
/// Verwendet die eingebettete DejaVu-Sans-Schrift mit Anti-Aliasing.
/// Unterstützt alle Latin-Zeichen inkl. Umlaute (ä, ö, ü, ß etc.).
///
/// # Parameter
/// - `image`: Zielbild
/// - `x`, `y`: Startposition (obere linke Ecke)
/// - `text`: Zu zeichnender Text
/// - `color`: Textfarbe
/// - `scale`: Skalierungsfaktor (1 ≈ 8px, 2 ≈ 16px, etc.)
pub fn draw_text(image: &mut RgbImage, x: i32, y: i32, text: &str, color: Rgb<u8>, scale: u32) {
    draw_text_internal(image, x, y, text, color, scale, true);
}

/// Zeichnet Text mit schwarzem Outline (Schatteneffekt).
///
/// Für bessere Lesbarkeit auf variablem Hintergrund.
pub fn draw_text_outlined(
    image: &mut RgbImage,
    x: i32,
    y: i32,
    text: &str,
    color: Rgb<u8>,
    scale: u32,
) {
    let outline = Rgb([0, 0, 0]);
    let offset = (scale as i32).max(1);
    // 8 Richtungen für Outline (solid, ohne Blending)
    for &(dx, dy) in &[
        (-offset, -offset),
        (0, -offset),
        (offset, -offset),
        (-offset, 0),
        (offset, 0),
        (-offset, offset),
        (0, offset),
        (offset, offset),
    ] {
        draw_text_internal(image, x + dx, y + dy, text, outline, scale, false);
    }
    // Vordergrund mit Anti-Aliasing
    draw_text_internal(image, x, y, text, color, scale, true);
}

/// Berechnet die Pixelbreite eines Texts.
pub fn text_width(text: &str, scale: u32) -> u32 {
    let font = font();
    let scaled = font.as_scaled(px_scale(scale));
    let mut width = 0.0_f32;
    let mut last_glyph_id = None;
    for ch in text.chars() {
        let glyph_id = font.glyph_id(ch);
        if let Some(last) = last_glyph_id {
            width += scaled.kern(last, glyph_id);
        }
        width += scaled.h_advance(glyph_id);
        last_glyph_id = Some(glyph_id);
    }
    width.ceil() as u32
}

/// Berechnet die Pixelhöhe eines Texts.
pub fn text_height(scale: u32) -> u32 {
    let font = font();
    let scaled = font.as_scaled(px_scale(scale));
    (scaled.ascent() - scaled.descent()).ceil() as u32
}

/// Interne Zeichen-Funktion.
///
/// `blend`: true = Anti-Aliased Blending, false = Solid (für Outline)
fn draw_text_internal(
    image: &mut RgbImage,
    x: i32,
    y: i32,
    text: &str,
    color: Rgb<u8>,
    scale: u32,
    blend: bool,
) {
    let font = font();
    let scaled = font.as_scaled(px_scale(scale));
    let img_w = image.width() as i32;
    let img_h = image.height() as i32;

    let mut cursor_x = x as f32;
    let ascent = scaled.ascent();
    let mut last_glyph_id = None;

    for ch in text.chars() {
        let glyph_id = font.glyph_id(ch);
        if let Some(last) = last_glyph_id {
            cursor_x += scaled.kern(last, glyph_id);
        }

        let glyph = glyph_id.with_scale_and_position(
            px_scale(scale),
            ab_glyph::point(cursor_x, y as f32 + ascent),
        );

        if let Some(outlined) = font.outline_glyph(glyph) {
            let bounds = outlined.px_bounds();
            outlined.draw(|gx, gy, coverage| {
                if coverage < 0.05 {
                    return;
                }
                let px = bounds.min.x as i32 + gx as i32;
                let py = bounds.min.y as i32 + gy as i32;
                if px < 0 || py < 0 || px >= img_w || py >= img_h {
                    return;
                }
                if blend {
                    let bg = image.get_pixel(px as u32, py as u32);
                    let alpha = coverage.min(1.0);
                    let inv = 1.0 - alpha;
                    image.put_pixel(
                        px as u32,
                        py as u32,
                        Rgb([
                            (bg[0] as f32 * inv + color[0] as f32 * alpha) as u8,
                            (bg[1] as f32 * inv + color[1] as f32 * alpha) as u8,
                            (bg[2] as f32 * inv + color[2] as f32 * alpha) as u8,
                        ]),
                    );
                } else {
                    image.put_pixel(px as u32, py as u32, color);
                }
            });
        }

        cursor_x += scaled.h_advance(glyph_id);
        last_glyph_id = Some(glyph_id);
    }
}

/// Zeichnet ein gefülltes Rechteck.
pub fn draw_rect_filled(image: &mut RgbImage, x: i32, y: i32, w: u32, h: u32, color: Rgb<u8>) {
    let img_w = image.width() as i32;
    let img_h = image.height() as i32;

    for dy in 0..h as i32 {
        for dx in 0..w as i32 {
            let px = x + dx;
            let py = y + dy;
            if px >= 0 && px < img_w && py >= 0 && py < img_h {
                image.put_pixel(px as u32, py as u32, color);
            }
        }
    }
}

/// Zeichnet ein gefülltes Rechteck mit Alpha-Blending.
pub fn draw_rect_blended(
    image: &mut RgbImage,
    x: i32,
    y: i32,
    w: u32,
    h: u32,
    color: Rgb<u8>,
    alpha: f32,
) {
    let img_w = image.width() as i32;
    let img_h = image.height() as i32;
    let inv_alpha = 1.0 - alpha;

    for dy in 0..h as i32 {
        for dx in 0..w as i32 {
            let px = x + dx;
            let py = y + dy;
            if px >= 0 && px < img_w && py >= 0 && py < img_h {
                let bg = image.get_pixel(px as u32, py as u32);
                let blended = Rgb([
                    (bg[0] as f32 * inv_alpha + color[0] as f32 * alpha) as u8,
                    (bg[1] as f32 * inv_alpha + color[1] as f32 * alpha) as u8,
                    (bg[2] as f32 * inv_alpha + color[2] as f32 * alpha) as u8,
                ]);
                image.put_pixel(px as u32, py as u32, blended);
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_text_width_nonzero() {
        let w = text_width("AB", 1);
        assert!(w > 0, "Textbreite muss > 0 sein, war {}", w);
    }

    #[test]
    fn test_text_width_empty() {
        assert_eq!(text_width("", 1), 0);
    }

    #[test]
    fn test_text_width_scales() {
        let w1 = text_width("Test", 1);
        let w2 = text_width("Test", 2);
        assert!(w2 > w1, "Scale 2 muss breiter sein als Scale 1");
    }

    #[test]
    fn test_text_height_scales() {
        let h1 = text_height(1);
        let h2 = text_height(2);
        assert!(h1 > 0);
        assert!(h2 > h1, "Scale 2 muss höher sein als Scale 1");
    }

    #[test]
    fn test_draw_text_no_panic() {
        let mut img = RgbImage::new(100, 30);
        draw_text(&mut img, 0, 0, "Hi 42!", Rgb([255, 255, 255]), 1);
    }

    #[test]
    fn test_draw_text_umlauts() {
        // Umlaute dürfen nicht paniken und sollen sichtbare Pixel erzeugen
        let mut img = RgbImage::new(200, 30);
        draw_text(&mut img, 0, 0, "Höflingen Straße", Rgb([255, 255, 255]), 1);
        // Prüfe dass mindestens ein Pixel gesetzt wurde
        let has_white = img.pixels().any(|p| p[0] > 200);
        assert!(has_white, "Umlaute müssen sichtbare Pixel erzeugen");
    }

    #[test]
    fn test_draw_out_of_bounds() {
        let mut img = RgbImage::new(10, 10);
        draw_text(&mut img, -5, -5, "X", Rgb([255, 0, 0]), 1);
    }

    #[test]
    fn test_draw_text_outlined_no_panic() {
        let mut img = RgbImage::new(200, 30);
        draw_text_outlined(&mut img, 5, 5, "Ölfeld", Rgb([255, 255, 200]), 2);
    }
}
