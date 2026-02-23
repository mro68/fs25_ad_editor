//! GRLE-Decoder (GIANTS Run-Length Encoded).
//!
//! Dekodiert `.grle`-Dateien (InfoLayer) zu Grayscale-Pixeldaten.
//! Format-Dokumentation: <https://github.com/Paint-a-Farm/grleconvert>
//!
//! Basiert auf grleconvert von Kim Brandwijk (MIT-Lizenz).

use anyhow::{ensure, Result};

/// GRLE Magic Bytes
const GRLE_MAGIC: &[u8; 4] = b"GRLE";

/// Header-Größe in Bytes
const GRLE_HEADER_SIZE: usize = 20;

/// Ergebnis einer GRLE-Dekodierung.
#[derive(Debug)]
pub struct GrleImage {
    /// Breite in Pixeln
    pub width: usize,
    /// Höhe in Pixeln
    pub height: usize,
    /// Grayscale-Pixeldaten (1 Byte pro Pixel)
    pub pixels: Vec<u8>,
}

/// Dekodiert GRLE-Daten aus einem Byte-Buffer.
///
/// # Fehler
/// - Datei zu klein oder ungültige Magic Bytes
pub fn decode_grle(data: &[u8]) -> Result<GrleImage> {
    ensure!(
        data.len() >= GRLE_HEADER_SIZE,
        "GRLE-Datei zu klein: {} Bytes (min. {} erwartet)",
        data.len(),
        GRLE_HEADER_SIZE
    );
    ensure!(
        &data[0..4] == GRLE_MAGIC,
        "Ungültige GRLE Magic Bytes: {:?}",
        &data[0..4]
    );

    let _version = u16::from_le_bytes([data[4], data[5]]);
    let width = u16::from_le_bytes([data[6], data[7]]) as usize * 256;
    let height = u16::from_le_bytes([data[10], data[11]]) as usize * 256;

    log::debug!("GRLE: {}x{} Pixel", width, height);

    let compressed_data = &data[GRLE_HEADER_SIZE..];
    let expected_size = width * height;
    let pixels = decode_rle(compressed_data, expected_size);

    Ok(GrleImage {
        width,
        height,
        pixels,
    })
}

/// RLE-Dekompression für GRLE-Daten.
///
/// Algorithmus (aus grleconvert):
/// - Erstes Byte ist 0x00-Padding
/// - Liest Paare (a, b): wenn a == b → Run (Zähler folgt), sonst Transition
/// - Zähler: 0xFF-Bytes addieren je 255, letztes Byte ist Rest, Gesamtpixel = count + 2
fn decode_rle(data: &[u8], expected_size: usize) -> Vec<u8> {
    let mut output = Vec::with_capacity(expected_size);
    let mut i = 1; // Erstes Byte überspringen (0x00 Padding)

    while i + 1 < data.len() && output.len() < expected_size {
        let prev = data[i];
        let new_val = data[i + 1];
        i += 2;

        if prev == new_val {
            // Gleicher Wert: Run mit erweitertem Zähler
            let mut count = 0usize;
            while i < data.len() && data[i] == 0xff {
                count += 255;
                i += 1;
            }
            if i < data.len() {
                count += data[i] as usize;
                i += 1;
            }
            count += 2; // Zähler sind um 2 versetzt

            let to_emit = count.min(expected_size - output.len());
            output.extend(std::iter::repeat_n(prev, to_emit));
        } else {
            // Verschiedene Werte: Transition — erstes Pixel ausgeben, ein Byte zurückgehen
            output.push(prev);
            i -= 1;
        }
    }

    output.resize(expected_size, 0);
    output
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_decode_rle_empty() {
        // Nur Padding-Byte
        let data = [0x00];
        let result = decode_rle(&data, 0);
        assert!(result.is_empty());
    }

    #[test]
    fn test_decode_rle_run() {
        // Padding + Run von 4x Wert 0x42: (0x42, 0x42, 0x02)
        // count = 2 + 2 = 4
        let data = [0x00, 0x42, 0x42, 0x02];
        let result = decode_rle(&data, 4);
        assert_eq!(result, vec![0x42; 4]);
    }

    #[test]
    fn test_decode_rle_transition() {
        // Padding + Transition: (0x10, 0x20) → emit 0x10, backup
        // Dann (0x20, 0x20, 0x00) → Run von 2x 0x20
        let data = [0x00, 0x10, 0x20, 0x20, 0x00];
        let result = decode_rle(&data, 3);
        assert_eq!(result, vec![0x10, 0x20, 0x20]);
    }

    #[test]
    fn test_reject_invalid_magic() {
        let data = [
            b'N', b'O', b'P', b'E', 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
        ];
        let result = decode_grle(&data);
        assert!(result.is_err());
    }
}
