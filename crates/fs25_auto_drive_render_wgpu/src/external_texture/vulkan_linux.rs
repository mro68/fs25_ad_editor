//! Linux/Vulkan-Implementierung des GPU-Texture-Exports via DMA-BUF.
//!
//! Diese Implementierung nutzt wgpu mit dem Vulkan-Backend und bereitet die
//! Architektur fuer den vollstaendigen DMA-BUF-Export vor. Der eigentliche
//! HAL-Zugriff (VK_KHR_external_memory_fd, vkGetMemoryFdKHR) ist als TODO
//! markiert bis die Flutter-Seite fuer Tests verfuegbar ist.

use super::{ExternalTextureError, ExternalTextureExport, PlatformTextureDescriptor};
use crate::export_core::{EXPORT_COLOR_FORMAT, EXPORT_SAMPLE_COUNT};

/// Vulkan-basierte Texture fuer den Zero-Copy-Export an Flutter/Impeller via DMA-BUF.
///
/// # Architektur
/// Die Texture wird mit `wgpu` erzeugt. Der naechste Schritt (TODO) ist der HAL-Zugriff
/// via `device.as_hal::<wgpu::hal::vulkan::Api>()` um ein `VkImage` mit
/// `VK_EXTERNAL_MEMORY_HANDLE_TYPE_DMA_BUF_BIT_EXT` zu erzeugen und per
/// `vkGetMemoryFdKHR` einen exportierbaren File-Descriptor zu erhalten.
pub struct VulkanDmaBufTexture {
    texture: wgpu::Texture,
    view: wgpu::TextureView,
    width: u32,
    height: u32,
}

impl VulkanDmaBufTexture {
    /// Interne Hilfsfunktion: Erzeugt Texture + View fuer die gegebene Groesse.
    fn create_texture(
        device: &wgpu::Device,
        width: u32,
        height: u32,
    ) -> (wgpu::Texture, wgpu::TextureView) {
        let texture = device.create_texture(&wgpu::TextureDescriptor {
            label: Some("VulkanDmaBuf Export Target"),
            size: wgpu::Extent3d {
                width,
                height,
                depth_or_array_layers: 1,
            },
            mip_level_count: 1,
            sample_count: EXPORT_SAMPLE_COUNT,
            dimension: wgpu::TextureDimension::D2,
            // TODO(flutter-linux-dmabuf): Format muss VK_FORMAT_R8G8B8A8_SRGB entsprechen
            format: EXPORT_COLOR_FORMAT,
            // TODO(flutter-linux-dmabuf): TEXTURE_BINDING ggf. durch externe Memory-Flags ersetzen
            usage: wgpu::TextureUsages::RENDER_ATTACHMENT | wgpu::TextureUsages::TEXTURE_BINDING,
            view_formats: &[],
        });
        let view = texture.create_view(&wgpu::TextureViewDescriptor::default());
        (texture, view)
    }
}

impl ExternalTextureExport for VulkanDmaBufTexture {
    fn create_exportable_texture(
        device: &wgpu::Device,
        width: u32,
        height: u32,
    ) -> Result<Self, ExternalTextureError> {
        if width == 0 || height == 0 {
            return Err(ExternalTextureError::CreationFailed(format!(
                "Texturgroesse muss positiv sein, erhalten {width}x{height}"
            )));
        }
        let (texture, view) = Self::create_texture(device, width, height);
        Ok(Self {
            texture,
            view,
            width,
            height,
        })
    }

    fn export_descriptor(&self) -> Result<PlatformTextureDescriptor, ExternalTextureError> {
        // TODO(flutter-linux-dmabuf): Echter DMA-BUF-Export via wgpu HAL:
        //
        //   1. unsafe { device.as_hal::<wgpu::hal::vulkan::Api, _, _>(|hal_device| { ... }) }
        //   2. Vom hal_device: raw_device().get_memory_fd(&vk::MemoryGetFdInfoKHR { ... })
        //   3. vkGetMemoryFdKHR → DMA-BUF fd
        //   4. fd + Stride + Format → PlatformTextureDescriptor::LinuxDmaBuf { ... }
        //
        // Voraussetzung: VkImage muss mit VK_EXTERNAL_MEMORY_HANDLE_TYPE_DMA_BUF_BIT_EXT erzeugt
        // werden (erfordert wgpu HAL-Texture-Erzeugung statt des normalen create_texture-Pfads).
        Err(ExternalTextureError::ExportFailed(
            "DMA-BUF-Export noch nicht implementiert (TODO flutter-linux-dmabuf)".into(),
        ))
    }

    fn texture_view(&self) -> &wgpu::TextureView {
        &self.view
    }

    fn resize(
        &mut self,
        device: &wgpu::Device,
        width: u32,
        height: u32,
    ) -> Result<(), ExternalTextureError> {
        if width == 0 || height == 0 {
            return Err(ExternalTextureError::CreationFailed(format!(
                "Texturgroesse muss positiv sein, erhalten {width}x{height}"
            )));
        }
        let (texture, view) = Self::create_texture(device, width, height);
        self.texture = texture;
        self.view = view;
        self.width = width;
        self.height = height;
        Ok(())
    }
}

/// Gibt die Breite der exportierbaren Texture zurueck.
pub fn dmabuf_texture_width(t: &VulkanDmaBufTexture) -> u32 {
    t.width
}

/// Gibt die Hoehe der exportierbaren Texture zurueck.
pub fn dmabuf_texture_height(t: &VulkanDmaBufTexture) -> u32 {
    t.height
}

#[cfg(test)]
mod tests {
    use super::*;

    fn create_test_gpu() -> Option<(wgpu::Instance, wgpu::Adapter, wgpu::Device, wgpu::Queue)> {
        let instance = wgpu::Instance::default();
        let adapter = pollster::block_on(instance.request_adapter(&wgpu::RequestAdapterOptions {
            power_preference: wgpu::PowerPreference::LowPower,
            compatible_surface: None,
            force_fallback_adapter: false,
        }))
        .ok()?;
        let (device, queue) = pollster::block_on(adapter.request_device(&wgpu::DeviceDescriptor {
            label: Some("VulkanDmaBufTexture Test Device"),
            required_features: wgpu::Features::empty(),
            required_limits: wgpu::Limits::downlevel_defaults(),
            experimental_features: Default::default(),
            memory_hints: wgpu::MemoryHints::Performance,
            trace: wgpu::Trace::Off,
        }))
        .ok()?;
        Some((instance, adapter, device, queue))
    }

    /// Prueft, dass eine Nullbreite korrekt abgelehnt wird.
    #[test]
    fn test_create_exportable_texture_rejects_zero_width() {
        let Some((_inst, _adp, device, _queue)) = create_test_gpu() else {
            return;
        };
        let result = VulkanDmaBufTexture::create_exportable_texture(&device, 0, 64);
        assert!(result.is_err(), "Breite 0 muss CreationFailed zurueckgeben");
    }

    /// Prueft, dass eine Nullhoehe korrekt abgelehnt wird.
    #[test]
    fn test_create_exportable_texture_rejects_zero_height() {
        let Some((_inst, _adp, device, _queue)) = create_test_gpu() else {
            return;
        };
        let result = VulkanDmaBufTexture::create_exportable_texture(&device, 64, 0);
        assert!(result.is_err(), "Hoehe 0 muss CreationFailed zurueckgeben");
    }

    /// Prueft, dass export_descriptor den erwarteten TODO-Fehler zurueckgibt.
    #[test]
    fn test_export_descriptor_returns_not_implemented() {
        let Some((_inst, _adp, device, _queue)) = create_test_gpu() else {
            return;
        };
        let texture = VulkanDmaBufTexture::create_exportable_texture(&device, 16, 16)
            .expect("Texture-Erzeugung fuer Testgroesse muss gelingen");
        let result = texture.export_descriptor();
        assert!(
            result.is_err(),
            "DMA-BUF-Export muss Err zurueckgeben (TODO flutter-linux-dmabuf)"
        );
    }

    /// Prueft, dass resize() die Dimensionen korrekt aktualisiert.
    #[test]
    fn test_resize_updates_dimensions() {
        let Some((_inst, _adp, device, _queue)) = create_test_gpu() else {
            return;
        };
        let mut texture = VulkanDmaBufTexture::create_exportable_texture(&device, 16, 16)
            .expect("Initiale Texture-Erzeugung muss gelingen");
        assert_eq!(dmabuf_texture_width(&texture), 16);
        assert_eq!(dmabuf_texture_height(&texture), 16);

        texture
            .resize(&device, 32, 64)
            .expect("Resize muss gelingen");
        assert_eq!(dmabuf_texture_width(&texture), 32);
        assert_eq!(dmabuf_texture_height(&texture), 64);
    }
}
