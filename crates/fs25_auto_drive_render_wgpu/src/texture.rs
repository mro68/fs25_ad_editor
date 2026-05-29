//! Texture-Utilities fuer wgpu.

use image::DynamicImage;

/// Erstellt einen Textur-Sampler mit einheitlichem Filtermodus fuer Min-/Mag-Filter.
pub(crate) fn create_sampler(
    device: &wgpu::Device,
    label: &str,
    filter_mode: wgpu::FilterMode,
) -> wgpu::Sampler {
    device.create_sampler(&wgpu::SamplerDescriptor {
        label: Some(label),
        address_mode_u: wgpu::AddressMode::ClampToEdge,
        address_mode_v: wgpu::AddressMode::ClampToEdge,
        address_mode_w: wgpu::AddressMode::ClampToEdge,
        mag_filter: filter_mode,
        min_filter: filter_mode,
        mipmap_filter: wgpu::MipmapFilterMode::Nearest,
        ..Default::default()
    })
}

/// Erstellt eine wgpu-Texture aus einem DynamicImage mit automatischer Mipmap-Generierung.
///
/// Fuer Texturen groesser als 256 px werden Mip-Levels per CPU-seitigem Downsampling
/// (Triangle-Filter) erzeugt. Die Anzahl der Levels ist auf maximal 8 begrenzt.
/// Texturen bis einschliesslich 256 px erhalten nur Level 0 (kein Mipmap).
///
/// # Parameter
/// - `device`: wgpu-Device fuer Texture-Erstellung
/// - `queue`: wgpu-Queue fuer Daten-Upload
/// - `image`: Bilddaten (wird zu RGBA8 konvertiert)
/// - `label`: Debug-Label fuer die Texture
///
/// # Rueckgabe
/// Tuple aus Texture und Sampler; bei mehreren Mip-Levels wird `mipmap_filter: Linear`
/// verwendet, sonst `Nearest`.
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

    // Berechne Mip-Level-Anzahl basierend auf der groessten Dimension.
    // Texturen <= 256 px bekommen Level 1 (keine Mipmaps).
    let mip_level_count = calculate_mip_levels(width.max(height));

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

    // Schreibe Daten in die Texture (Mip-Level 0)
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

    // Erzeuge Mip-Levels 1..N per CPU-seitigem Downsampling (image::Triangle-Filter).
    // Fuer Texturen <= 256 px entfaellt die Schleife (mip_level_count == 1).
    if mip_level_count > 1 {
        let mut current_image = rgba_image;
        for level in 1..mip_level_count {
            let mip_width = (width >> level).max(1);
            let mip_height = (height >> level).max(1);
            let mip_image = image::DynamicImage::ImageRgba8(current_image)
                .resize_exact(mip_width, mip_height, image::imageops::FilterType::Triangle)
                .to_rgba8();
            queue.write_texture(
                wgpu::TexelCopyTextureInfo {
                    texture: &texture,
                    mip_level: level,
                    origin: wgpu::Origin3d::ZERO,
                    aspect: wgpu::TextureAspect::All,
                },
                &mip_image,
                wgpu::TexelCopyBufferLayout {
                    offset: 0,
                    bytes_per_row: Some(4 * mip_width),
                    rows_per_image: Some(mip_height),
                },
                wgpu::Extent3d {
                    width: mip_width,
                    height: mip_height,
                    depth_or_array_layers: 1,
                },
            );
            current_image = mip_image;
        }
    }

    // Erstelle Sampler — Mipmap-Filter Linear wenn mehrere Levels vorhanden
    let sampler_label = format!("{}_sampler", label);
    let sampler = if mip_level_count > 1 {
        device.create_sampler(&wgpu::SamplerDescriptor {
            label: Some(&sampler_label),
            address_mode_u: wgpu::AddressMode::ClampToEdge,
            address_mode_v: wgpu::AddressMode::ClampToEdge,
            address_mode_w: wgpu::AddressMode::ClampToEdge,
            mag_filter: wgpu::FilterMode::Linear,
            min_filter: wgpu::FilterMode::Linear,
            mipmap_filter: wgpu::MipmapFilterMode::Linear,
            ..Default::default()
        })
    } else {
        create_sampler(device, &sampler_label, wgpu::FilterMode::Linear)
    };

    (texture, sampler)
}

/// Berechnet die Anzahl der Mip-Levels fuer eine gegebene Texturgroesse.
///
/// Texturen kleiner oder gleich 256 Pixel erhalten kein Mipmap (Level 1).
/// Fuer groessere Texturen wird die Levelanzahl auf maximal 8 begrenzt.
fn calculate_mip_levels(size: u32) -> u32 {
    // Fuer sehr kleine Textures (< 256) lohnen sich Mipmaps nicht
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

        // Grosse Textures: limitiert auf 8
        assert_eq!(calculate_mip_levels(8192), 6); // log2(8192) = 13
        assert_eq!(calculate_mip_levels(16384), 7); // log2(16384) = 14
        assert_eq!(calculate_mip_levels(32768), 8); // limitiert
    }

    #[test]
    fn test_mip_levels_edge_cases() {
        assert_eq!(calculate_mip_levels(1), 1);
        assert_eq!(calculate_mip_levels(2), 1);
        assert_eq!(calculate_mip_levels(65536), 8); // sehr gross -> limitiert
    }
}
