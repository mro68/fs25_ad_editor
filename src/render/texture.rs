//! Texture-Utilities für wgpu.

use image::DynamicImage;

/// Erstellt eine wgpu-Texture aus einem DynamicImage
///
/// # Parameter
/// - `device`: wgpu-Device für Texture-Erstellung
/// - `queue`: wgpu-Queue für Daten-Upload
/// - `image`: Bilddaten (wird zu RGBA8 konvertiert)
/// - `label`: Debug-Label für die Texture
///
/// # Returns
/// Die erstellte Texture mit Sampler
pub fn create_texture_from_image(
    device: &wgpu::Device,
    queue: &wgpu::Queue,
    image: &DynamicImage,
    label: &str,
) -> (wgpu::Texture, wgpu::Sampler) {
    let rgba_image = image.to_rgba8();
    let (width, height) = rgba_image.dimensions();

    log::debug!(
        "Erstelle wgpu-Texture '{}': {}x{} Pixel, {} Bytes",
        label,
        width,
        height,
        rgba_image.len()
    );

    // Mip-Level-Count: 1 (keine Mipmaps), da Mip-Generierung noch nicht implementiert ist.
    // Leere Mip-Levels verursachen Fading/Verschwinden bei niedrigem Zoom.
    let mip_level_count = 1;

    // Erstelle Texture
    let texture = device.create_texture(&wgpu::TextureDescriptor {
        label: Some(label),
        size: wgpu::Extent3d {
            width,
            height,
            depth_or_array_layers: 1,
        },
        mip_level_count,
        sample_count: 1,
        dimension: wgpu::TextureDimension::D2,
        format: wgpu::TextureFormat::Rgba8UnormSrgb,
        usage: wgpu::TextureUsages::TEXTURE_BINDING
            | wgpu::TextureUsages::COPY_DST
            | wgpu::TextureUsages::RENDER_ATTACHMENT,
        view_formats: &[],
    });

    // Schreibe Daten in die Texture (nur Mip-Level 0)
    queue.write_texture(
        texture.as_image_copy(),
        &rgba_image,
        wgpu::TexelCopyBufferLayout {
            offset: 0,
            bytes_per_row: Some(4 * width),
            rows_per_image: Some(height),
        },
        wgpu::Extent3d {
            width,
            height,
            depth_or_array_layers: 1,
        },
    );

    // Erstelle Sampler (kein Mipmap-Filter, da nur 1 Mip-Level vorhanden)
    let sampler = device.create_sampler(&wgpu::SamplerDescriptor {
        label: Some(&format!("{}_sampler", label)),
        address_mode_u: wgpu::AddressMode::ClampToEdge,
        address_mode_v: wgpu::AddressMode::ClampToEdge,
        address_mode_w: wgpu::AddressMode::ClampToEdge,
        mag_filter: wgpu::FilterMode::Linear,
        min_filter: wgpu::FilterMode::Linear,
        mipmap_filter: wgpu::FilterMode::Nearest,
        ..Default::default()
    });

    (texture, sampler)
}

// TODO(Mipmap): Bei Bedarf Mipmap-Generierung implementieren (Compute-Pass oder CPU-seitig).\n// Tracker: https://github.com/gfx-rs/wgpu/issues/661

#[cfg(test)]
/// Berechnet die Anzahl der Mip-Levels für eine gegebene Texture-Größe (aktuell nur in Tests)
fn calculate_mip_levels(size: u32) -> u32 {
    // Für sehr kleine Textures (< 256) lohnen sich Mipmaps nicht
    if size <= 256 {
        return 1;
    }

    // Berechne Mip-Levels ab 256px Grenze: floor(log2(size/256)) + 1
    let max_levels = (size as f32 / 256.0).log2().floor() as u32 + 1;

    // Limitiere auf 8 Levels
    max_levels.min(8)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_mip_levels_calculation() {
        // Kleine Textures: keine Mipmaps
        assert_eq!(calculate_mip_levels(128), 1);
        assert_eq!(calculate_mip_levels(255), 1);

        // Mittlere Textures: mehrere Levels
        assert_eq!(calculate_mip_levels(256), 1); // log2(256) = 8, aber 256 ist Grenze
        assert_eq!(calculate_mip_levels(512), 2); // log2(512) = 9
        assert_eq!(calculate_mip_levels(1024), 3); // log2(1024) = 10
        assert_eq!(calculate_mip_levels(2048), 4); // log2(2048) = 11
        assert_eq!(calculate_mip_levels(4096), 5); // log2(4096) = 12

        // Große Textures: limitiert auf 8
        assert_eq!(calculate_mip_levels(8192), 6); // log2(8192) = 13
        assert_eq!(calculate_mip_levels(16384), 7); // log2(16384) = 14
        assert_eq!(calculate_mip_levels(32768), 8); // limitiert
    }

    #[test]
    fn test_mip_levels_edge_cases() {
        assert_eq!(calculate_mip_levels(1), 1);
        assert_eq!(calculate_mip_levels(2), 1);
        assert_eq!(calculate_mip_levels(65536), 8); // sehr groß -> limitiert
    }
}
