//! Minimalistisches Bitmap-Text-Rendering.
//!
//! Eingebetteter 5×7 Bitmap-Font für Labels, IDs und Legende.
//! Keine externen Font-Dateien oder Dependencies nötig.

use image::{Rgb, RgbImage};

/// Zeichnet einen Text-String auf ein RGB-Bild.
///
/// Verwendet einen eingebetteten 5×7 Bitmap-Font.
/// Unterstützt ASCII 32–126 (Buchstaben, Ziffern, Satzzeichen).
///
/// # Parameter
/// - `image`: Zielbild
/// - `x`, `y`: Startposition (obere linke Ecke)
/// - `text`: Zu zeichnender Text
/// - `color`: Textfarbe
/// - `scale`: Skalierungsfaktor (1 = 5×7 Pixel, 2 = 10×14, etc.)
pub fn draw_text(image: &mut RgbImage, x: i32, y: i32, text: &str, color: Rgb<u8>, scale: u32) {
    let char_w = (CHAR_WIDTH as i32) * scale as i32;
    let mut cursor_x = x;

    for ch in text.chars() {
        if let Some(glyph) = glyph_for(ch) {
            draw_glyph(image, cursor_x, y, glyph, color, scale);
        }
        cursor_x += char_w + scale as i32; // 1px Spacing pro Scale
    }
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
    // 8 Richtungen für Outline
    for &(dx, dy) in &[
        (-1, -1),
        (0, -1),
        (1, -1),
        (-1, 0),
        (1, 0),
        (-1, 1),
        (0, 1),
        (1, 1),
    ] {
        draw_text(image, x + dx, y + dy, text, outline, scale);
    }
    draw_text(image, x, y, text, color, scale);
}

/// Berechnet die Pixelbreite eines Texts.
pub fn text_width(text: &str, scale: u32) -> u32 {
    let chars = text.len() as u32;
    if chars == 0 {
        return 0;
    }
    chars * CHAR_WIDTH as u32 * scale + (chars - 1) * scale
}

/// Berechnet die Pixelhöhe eines Texts.
pub fn text_height(scale: u32) -> u32 {
    CHAR_HEIGHT as u32 * scale
}

/// Zeichnet ein einzelnes Glyph.
fn draw_glyph(
    image: &mut RgbImage,
    x: i32,
    y: i32,
    glyph: &[u8; CHAR_HEIGHT],
    color: Rgb<u8>,
    scale: u32,
) {
    let img_w = image.width() as i32;
    let img_h = image.height() as i32;

    for (row, &bits) in glyph.iter().enumerate() {
        for col in 0..CHAR_WIDTH {
            if bits & (1 << (CHAR_WIDTH - 1 - col)) != 0 {
                // Skalierter Pixel-Block
                for sy in 0..scale as i32 {
                    for sx in 0..scale as i32 {
                        let px = x + col as i32 * scale as i32 + sx;
                        let py = y + row as i32 * scale as i32 + sy;
                        if px >= 0 && px < img_w && py >= 0 && py < img_h {
                            image.put_pixel(px as u32, py as u32, color);
                        }
                    }
                }
            }
        }
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

// ── 5×7 Bitmap Font ─────────────────────────────────────────────────

const CHAR_WIDTH: usize = 5;
const CHAR_HEIGHT: usize = 7;

/// Gibt das Glyph für ein ASCII-Zeichen zurück.
fn glyph_for(ch: char) -> Option<&'static [u8; CHAR_HEIGHT]> {
    let idx = ch as usize;
    if idx < 32 || idx > 126 {
        return None;
    }
    Some(&FONT_5X7[idx - 32])
}

/// 5×7 Bitmap-Font (ASCII 32–126).
/// Jede Zeile ist ein Byte, Bits 4–0 repräsentieren die 5 Spalten.
#[rustfmt::skip]
static FONT_5X7: [[u8; 7]; 95] = [
    // 32: ' ' (Space)
    [0b00000, 0b00000, 0b00000, 0b00000, 0b00000, 0b00000, 0b00000],
    // 33: '!'
    [0b00100, 0b00100, 0b00100, 0b00100, 0b00000, 0b00100, 0b00000],
    // 34: '"'
    [0b01010, 0b01010, 0b00000, 0b00000, 0b00000, 0b00000, 0b00000],
    // 35: '#'
    [0b01010, 0b11111, 0b01010, 0b01010, 0b11111, 0b01010, 0b00000],
    // 36: '$'
    [0b00100, 0b01111, 0b10100, 0b01110, 0b00101, 0b11110, 0b00100],
    // 37: '%'
    [0b11001, 0b11010, 0b00100, 0b01000, 0b01011, 0b10011, 0b00000],
    // 38: '&'
    [0b01100, 0b10010, 0b01100, 0b10101, 0b10010, 0b01101, 0b00000],
    // 39: '\''
    [0b00100, 0b00100, 0b00000, 0b00000, 0b00000, 0b00000, 0b00000],
    // 40: '('
    [0b00010, 0b00100, 0b01000, 0b01000, 0b00100, 0b00010, 0b00000],
    // 41: ')'
    [0b01000, 0b00100, 0b00010, 0b00010, 0b00100, 0b01000, 0b00000],
    // 42: '*'
    [0b00000, 0b00100, 0b10101, 0b01110, 0b10101, 0b00100, 0b00000],
    // 43: '+'
    [0b00000, 0b00100, 0b00100, 0b11111, 0b00100, 0b00100, 0b00000],
    // 44: ','
    [0b00000, 0b00000, 0b00000, 0b00000, 0b00100, 0b00100, 0b01000],
    // 45: '-'
    [0b00000, 0b00000, 0b00000, 0b11111, 0b00000, 0b00000, 0b00000],
    // 46: '.'
    [0b00000, 0b00000, 0b00000, 0b00000, 0b00000, 0b00100, 0b00000],
    // 47: '/'
    [0b00001, 0b00010, 0b00100, 0b01000, 0b10000, 0b00000, 0b00000],
    // 48: '0'
    [0b01110, 0b10001, 0b10011, 0b10101, 0b11001, 0b10001, 0b01110],
    // 49: '1'
    [0b00100, 0b01100, 0b00100, 0b00100, 0b00100, 0b00100, 0b01110],
    // 50: '2'
    [0b01110, 0b10001, 0b00001, 0b00110, 0b01000, 0b10000, 0b11111],
    // 51: '3'
    [0b01110, 0b10001, 0b00001, 0b00110, 0b00001, 0b10001, 0b01110],
    // 52: '4'
    [0b00010, 0b00110, 0b01010, 0b10010, 0b11111, 0b00010, 0b00010],
    // 53: '5'
    [0b11111, 0b10000, 0b11110, 0b00001, 0b00001, 0b10001, 0b01110],
    // 54: '6'
    [0b00110, 0b01000, 0b10000, 0b11110, 0b10001, 0b10001, 0b01110],
    // 55: '7'
    [0b11111, 0b00001, 0b00010, 0b00100, 0b01000, 0b01000, 0b01000],
    // 56: '8'
    [0b01110, 0b10001, 0b10001, 0b01110, 0b10001, 0b10001, 0b01110],
    // 57: '9'
    [0b01110, 0b10001, 0b10001, 0b01111, 0b00001, 0b00010, 0b01100],
    // 58: ':'
    [0b00000, 0b00100, 0b00000, 0b00000, 0b00100, 0b00000, 0b00000],
    // 59: ';'
    [0b00000, 0b00100, 0b00000, 0b00000, 0b00100, 0b00100, 0b01000],
    // 60: '<'
    [0b00010, 0b00100, 0b01000, 0b10000, 0b01000, 0b00100, 0b00010],
    // 61: '='
    [0b00000, 0b00000, 0b11111, 0b00000, 0b11111, 0b00000, 0b00000],
    // 62: '>'
    [0b10000, 0b01000, 0b00100, 0b00010, 0b00100, 0b01000, 0b10000],
    // 63: '?'
    [0b01110, 0b10001, 0b00010, 0b00100, 0b00000, 0b00100, 0b00000],
    // 64: '@'
    [0b01110, 0b10001, 0b10111, 0b10101, 0b10110, 0b10000, 0b01110],
    // 65: 'A'
    [0b01110, 0b10001, 0b10001, 0b11111, 0b10001, 0b10001, 0b10001],
    // 66: 'B'
    [0b11110, 0b10001, 0b10001, 0b11110, 0b10001, 0b10001, 0b11110],
    // 67: 'C'
    [0b01110, 0b10001, 0b10000, 0b10000, 0b10000, 0b10001, 0b01110],
    // 68: 'D'
    [0b11110, 0b10001, 0b10001, 0b10001, 0b10001, 0b10001, 0b11110],
    // 69: 'E'
    [0b11111, 0b10000, 0b10000, 0b11110, 0b10000, 0b10000, 0b11111],
    // 70: 'F'
    [0b11111, 0b10000, 0b10000, 0b11110, 0b10000, 0b10000, 0b10000],
    // 71: 'G'
    [0b01110, 0b10001, 0b10000, 0b10111, 0b10001, 0b10001, 0b01110],
    // 72: 'H'
    [0b10001, 0b10001, 0b10001, 0b11111, 0b10001, 0b10001, 0b10001],
    // 73: 'I'
    [0b01110, 0b00100, 0b00100, 0b00100, 0b00100, 0b00100, 0b01110],
    // 74: 'J'
    [0b00111, 0b00010, 0b00010, 0b00010, 0b00010, 0b10010, 0b01100],
    // 75: 'K'
    [0b10001, 0b10010, 0b10100, 0b11000, 0b10100, 0b10010, 0b10001],
    // 76: 'L'
    [0b10000, 0b10000, 0b10000, 0b10000, 0b10000, 0b10000, 0b11111],
    // 77: 'M'
    [0b10001, 0b11011, 0b10101, 0b10101, 0b10001, 0b10001, 0b10001],
    // 78: 'N'
    [0b10001, 0b11001, 0b10101, 0b10011, 0b10001, 0b10001, 0b10001],
    // 79: 'O'
    [0b01110, 0b10001, 0b10001, 0b10001, 0b10001, 0b10001, 0b01110],
    // 80: 'P'
    [0b11110, 0b10001, 0b10001, 0b11110, 0b10000, 0b10000, 0b10000],
    // 81: 'Q'
    [0b01110, 0b10001, 0b10001, 0b10001, 0b10101, 0b10010, 0b01101],
    // 82: 'R'
    [0b11110, 0b10001, 0b10001, 0b11110, 0b10100, 0b10010, 0b10001],
    // 83: 'S'
    [0b01110, 0b10001, 0b10000, 0b01110, 0b00001, 0b10001, 0b01110],
    // 84: 'T'
    [0b11111, 0b00100, 0b00100, 0b00100, 0b00100, 0b00100, 0b00100],
    // 85: 'U'
    [0b10001, 0b10001, 0b10001, 0b10001, 0b10001, 0b10001, 0b01110],
    // 86: 'V'
    [0b10001, 0b10001, 0b10001, 0b10001, 0b01010, 0b01010, 0b00100],
    // 87: 'W'
    [0b10001, 0b10001, 0b10001, 0b10101, 0b10101, 0b11011, 0b10001],
    // 88: 'X'
    [0b10001, 0b01010, 0b00100, 0b00100, 0b00100, 0b01010, 0b10001],
    // 89: 'Y'
    [0b10001, 0b10001, 0b01010, 0b00100, 0b00100, 0b00100, 0b00100],
    // 90: 'Z'
    [0b11111, 0b00001, 0b00010, 0b00100, 0b01000, 0b10000, 0b11111],
    // 91: '['
    [0b01110, 0b01000, 0b01000, 0b01000, 0b01000, 0b01000, 0b01110],
    // 92: '\'
    [0b10000, 0b01000, 0b00100, 0b00010, 0b00001, 0b00000, 0b00000],
    // 93: ']'
    [0b01110, 0b00010, 0b00010, 0b00010, 0b00010, 0b00010, 0b01110],
    // 94: '^'
    [0b00100, 0b01010, 0b10001, 0b00000, 0b00000, 0b00000, 0b00000],
    // 95: '_'
    [0b00000, 0b00000, 0b00000, 0b00000, 0b00000, 0b00000, 0b11111],
    // 96: '`'
    [0b01000, 0b00100, 0b00000, 0b00000, 0b00000, 0b00000, 0b00000],
    // 97: 'a'
    [0b00000, 0b00000, 0b01110, 0b00001, 0b01111, 0b10001, 0b01111],
    // 98: 'b'
    [0b10000, 0b10000, 0b11110, 0b10001, 0b10001, 0b10001, 0b11110],
    // 99: 'c'
    [0b00000, 0b00000, 0b01110, 0b10000, 0b10000, 0b10001, 0b01110],
    // 100: 'd'
    [0b00001, 0b00001, 0b01111, 0b10001, 0b10001, 0b10001, 0b01111],
    // 101: 'e'
    [0b00000, 0b00000, 0b01110, 0b10001, 0b11111, 0b10000, 0b01110],
    // 102: 'f'
    [0b00110, 0b01001, 0b01000, 0b11100, 0b01000, 0b01000, 0b01000],
    // 103: 'g'
    [0b00000, 0b01111, 0b10001, 0b10001, 0b01111, 0b00001, 0b01110],
    // 104: 'h'
    [0b10000, 0b10000, 0b10110, 0b11001, 0b10001, 0b10001, 0b10001],
    // 105: 'i'
    [0b00100, 0b00000, 0b01100, 0b00100, 0b00100, 0b00100, 0b01110],
    // 106: 'j'
    [0b00010, 0b00000, 0b00110, 0b00010, 0b00010, 0b10010, 0b01100],
    // 107: 'k'
    [0b10000, 0b10000, 0b10010, 0b10100, 0b11000, 0b10100, 0b10010],
    // 108: 'l'
    [0b01100, 0b00100, 0b00100, 0b00100, 0b00100, 0b00100, 0b01110],
    // 109: 'm'
    [0b00000, 0b00000, 0b11010, 0b10101, 0b10101, 0b10101, 0b10001],
    // 110: 'n'
    [0b00000, 0b00000, 0b10110, 0b11001, 0b10001, 0b10001, 0b10001],
    // 111: 'o'
    [0b00000, 0b00000, 0b01110, 0b10001, 0b10001, 0b10001, 0b01110],
    // 112: 'p'
    [0b00000, 0b00000, 0b11110, 0b10001, 0b11110, 0b10000, 0b10000],
    // 113: 'q'
    [0b00000, 0b00000, 0b01111, 0b10001, 0b01111, 0b00001, 0b00001],
    // 114: 'r'
    [0b00000, 0b00000, 0b10110, 0b11001, 0b10000, 0b10000, 0b10000],
    // 115: 's'
    [0b00000, 0b00000, 0b01111, 0b10000, 0b01110, 0b00001, 0b11110],
    // 116: 't'
    [0b01000, 0b01000, 0b11100, 0b01000, 0b01000, 0b01001, 0b00110],
    // 117: 'u'
    [0b00000, 0b00000, 0b10001, 0b10001, 0b10001, 0b10011, 0b01101],
    // 118: 'v'
    [0b00000, 0b00000, 0b10001, 0b10001, 0b10001, 0b01010, 0b00100],
    // 119: 'w'
    [0b00000, 0b00000, 0b10001, 0b10001, 0b10101, 0b10101, 0b01010],
    // 120: 'x'
    [0b00000, 0b00000, 0b10001, 0b01010, 0b00100, 0b01010, 0b10001],
    // 121: 'y'
    [0b00000, 0b00000, 0b10001, 0b10001, 0b01111, 0b00001, 0b01110],
    // 122: 'z'
    [0b00000, 0b00000, 0b11111, 0b00010, 0b00100, 0b01000, 0b11111],
    // 123: '{'
    [0b00010, 0b00100, 0b00100, 0b01000, 0b00100, 0b00100, 0b00010],
    // 124: '|'
    [0b00100, 0b00100, 0b00100, 0b00100, 0b00100, 0b00100, 0b00100],
    // 125: '}'
    [0b01000, 0b00100, 0b00100, 0b00010, 0b00100, 0b00100, 0b01000],
    // 126: '~'
    [0b00000, 0b00000, 0b01000, 0b10101, 0b00010, 0b00000, 0b00000],
];

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_text_width() {
        assert_eq!(text_width("AB", 1), 11); // 5+1+5
        assert_eq!(text_width("A", 1), 5);
        assert_eq!(text_width("", 1), 0);
        assert_eq!(text_width("AB", 2), 22); // (5*2)+2+(5*2)
    }

    #[test]
    fn test_draw_text_no_panic() {
        let mut img = RgbImage::new(50, 20);
        draw_text(&mut img, 0, 0, "Hi 42!", Rgb([255, 255, 255]), 1);
        // Kein Panic = OK
    }

    #[test]
    fn test_draw_out_of_bounds() {
        // Soll nicht paniken bei negativen Koordinaten
        let mut img = RgbImage::new(10, 10);
        draw_text(&mut img, -5, -5, "X", Rgb([255, 0, 0]), 1);
    }
}
