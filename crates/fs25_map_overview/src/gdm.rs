//! GDM-Decoder (GIANTS Density Map).
//!
//! Dekodiert `.gdm`-Dateien (DetailLayer, FoliageMultiLayer) zu Pixeldaten.
//! Unterstützt 1–8 Kanäle (Grayscale) und 9+ Kanäle (RGB-Encoding).
//! Format-Dokumentation: <https://github.com/Paint-a-Farm/grleconvert>
//!
//! Basiert auf grleconvert von Kim Brandwijk (MIT-Lizenz).

use anyhow::{bail, ensure, Result};

/// GDM Magic Bytes (Version 2)
const GDM_MAGIC_V2: &[u8; 4] = b"\"MDF";

/// GDM Magic Bytes (Version 1)
const GDM_MAGIC_V1: &[u8; 4] = b"!MDF";

/// Ergebnis einer GDM-Dekodierung.
#[derive(Debug)]
pub struct GdmImage {
    /// Dimension (quadratisch: dimension × dimension)
    pub dimension: usize,
    /// Anzahl der Kanäle
    pub num_channels: usize,
    /// Pixeldaten als Grayscale-Bytes (1 Byte/Pixel bei ≤8 Kanälen)
    /// oder RGB-Bytes (3 Bytes/Pixel bei >8 Kanälen)
    pub pixels: Vec<u8>,
    /// True wenn RGB-Encoding (>8 Kanäle)
    pub is_rgb: bool,
}

/// Dekodiert GDM-Daten aus einem Byte-Buffer.
///
/// # Fehler
/// - Datei zu klein, ungültige Magic Bytes oder unerwartetes Datenende
pub fn decode_gdm(data: &[u8]) -> Result<GdmImage> {
    ensure!(data.len() >= 16, "GDM-Datei zu klein: {} Bytes", data.len());

    let magic = &data[0..4];
    let (dimension, num_channels, chunk_size, num_compression_ranges, header_size) =
        if magic == GDM_MAGIC_V2 {
            let version = u32::from_le_bytes([data[4], data[5], data[6], data[7]]);
            ensure!(version == 0, "Nicht unterstützte GDM-Version: {}", version);

            let dim_log2 = data[8] as usize;
            let chunk_log2 = data[9] as usize;
            let num_channels = data[11] as usize;
            let num_compression_ranges = data[12] as usize;
            let dimension = 1 << (dim_log2 + 5);
            let chunk_size = 1 << chunk_log2;
            (
                dimension,
                num_channels,
                chunk_size,
                num_compression_ranges,
                16usize,
            )
        } else if magic == GDM_MAGIC_V1 {
            let dim_log2 = data[4] as usize;
            let chunk_log2 = data[5] as usize;
            let num_channels = data[7] as usize;
            let num_compression_ranges = data[8] as usize;
            let dimension = 1 << (dim_log2 + 5);
            let chunk_size = 1 << chunk_log2;
            (
                dimension,
                num_channels,
                chunk_size,
                num_compression_ranges,
                9usize,
            )
        } else {
            bail!("Ungültige GDM Magic Bytes: {:?}", magic);
        };

    log::debug!(
        "GDM: {}x{}, {} Kanäle, {} Kompressionsranges",
        dimension,
        dimension,
        num_channels,
        num_compression_ranges
    );

    // Kompressionsgrenzen lesen
    let mut compression_boundaries = vec![0u8];
    for i in 0..(num_compression_ranges.saturating_sub(1)) {
        compression_boundaries.push(data[header_size + i]);
    }
    compression_boundaries.push(num_channels as u8);

    let mut bits_per_range = Vec::new();
    for i in 0..num_compression_ranges {
        let start_ch = compression_boundaries[i] as usize;
        let end_ch = compression_boundaries[i + 1] as usize;
        bits_per_range.push(end_ch - start_ch);
    }

    let chunks_per_dim = dimension / chunk_size;
    let total_chunks = chunks_per_dim * chunks_per_dim;
    let compression_boundaries_size = if num_compression_ranges > 1 {
        num_compression_ranges - 1
    } else {
        0
    };
    let data_start = header_size + compression_boundaries_size;

    let use_rgb = num_channels > 8;
    let bytes_per_pixel = if use_rgb { 3 } else { 1 };
    let mut image = vec![0u8; dimension * dimension * bytes_per_pixel];

    let mut pos = data_start;
    for chunk_idx in 0..total_chunks {
        let mut range_values: Vec<Vec<u16>> = Vec::new();
        for _range_idx in 0..num_compression_ranges {
            ensure!(
                pos + 2 <= data.len(),
                "Unerwartetes Datenende bei Chunk {}/{}",
                chunk_idx,
                total_chunks
            );
            let (pixels, block_size) = decode_block(&data, pos, chunk_size);
            range_values.push(pixels);
            pos += block_size;
        }

        let chunk_row = chunk_idx / chunks_per_dim;
        let chunk_col = chunk_idx % chunks_per_dim;
        let base_y = chunk_row * chunk_size;
        let base_x = chunk_col * chunk_size;

        // Ranges zusammenführen (Bit-Shifting)
        let total_pixels = chunk_size * chunk_size;
        for pixel_idx in 0..total_pixels {
            let mut combined: u32 = 0;
            let mut shift = 0;
            for (range_idx, range) in range_values.iter().enumerate() {
                let val = *range.get(pixel_idx).unwrap_or(&0) as u32;
                combined |= val << shift;
                shift += bits_per_range[range_idx];
            }

            let py = pixel_idx / chunk_size;
            let px = pixel_idx % chunk_size;
            let img_y = base_y + py;
            let img_x = base_x + px;

            if img_y < dimension && img_x < dimension {
                let offset = (img_y * dimension + img_x) * bytes_per_pixel;
                if use_rgb {
                    if offset + 2 < image.len() {
                        image[offset] = (combined & 0xFF) as u8;
                        image[offset + 1] = ((combined >> 8) & 0xFF) as u8;
                        image[offset + 2] = ((combined >> 16) & 0xFF) as u8;
                    }
                } else if offset < image.len() {
                    image[offset] = (combined & 0xFF) as u8;
                }
            }
        }
    }

    Ok(GdmImage {
        dimension,
        num_channels,
        pixels: image,
        is_rgb: use_rgb,
    })
}

/// Dekodiert einen einzelnen GDM-Block (Chunk).
///
/// Jeder Block hat:
/// - 1 Byte Bit-Tiefe
/// - 1 Byte Paletten-Einträge
/// - Palette (2 Bytes pro Eintrag)
/// - Bitmap (bit_depth * 128 Bytes)
fn decode_block(data: &[u8], pos: usize, chunk_size: usize) -> (Vec<u16>, usize) {
    let bit_depth = data[pos];
    let palette_count = data[pos + 1] as usize;
    let palette_size = 2 * palette_count;
    let bitmap_size = if bit_depth > 0 {
        (bit_depth as usize) * 128
    } else {
        0
    };
    let block_size = 2 + palette_size + bitmap_size;

    let palette: Vec<u16> = (0..palette_count)
        .map(|i| u16::from_le_bytes([data[pos + 2 + i * 2], data[pos + 3 + i * 2]]))
        .collect();

    let total_pixels = chunk_size * chunk_size;
    let mut pixels = Vec::with_capacity(total_pixels);

    if bit_depth == 0 {
        // Konstanter Wert für alle Pixel
        let value = *palette.first().unwrap_or(&0);
        pixels.resize(total_pixels, value);
    } else {
        let bitmap = &data[pos + 2 + palette_size..pos + 2 + palette_size + bitmap_size];
        let bits_per_pixel = bit_depth as usize;
        let mask = (1u16 << bits_per_pixel) - 1;

        for pixel_idx in 0..total_pixels {
            let bit_pos = pixel_idx * bits_per_pixel;
            let byte_idx = bit_pos / 8;
            let bit_offset = bit_pos % 8;

            let mut raw_value = bitmap[byte_idx] as u16;
            if byte_idx + 1 < bitmap.len() {
                raw_value |= (bitmap[byte_idx + 1] as u16) << 8;
            }
            let idx_or_value = ((raw_value >> bit_offset) & mask) as usize;

            let pixel_value = if bit_depth <= 2 && !palette.is_empty() {
                *palette.get(idx_or_value).unwrap_or(&0)
            } else {
                idx_or_value as u16
            };
            pixels.push(pixel_value);
        }
    }

    (pixels, block_size)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_reject_invalid_magic() {
        let data = [0u8; 20];
        let result = decode_gdm(&data);
        assert!(result.is_err());
    }

    #[test]
    fn test_decode_block_constant() {
        // bit_depth=0, palette_count=1, palette=[0x0042]
        let data = [
            0x00, // bit_depth = 0
            0x01, // palette_count = 1
            0x42, 0x00, // palette[0] = 0x0042
        ];
        let (pixels, block_size) = decode_block(&data, 0, 4);
        assert_eq!(block_size, 4); // 2 + 2 + 0
        assert_eq!(pixels.len(), 16); // 4*4
        assert!(pixels.iter().all(|&p| p == 0x0042));
    }
}
