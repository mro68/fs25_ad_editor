//! Linux/Vulkan-Implementierung des GPU-Texture-Exports via DMA-BUF.

use super::{ExternalTextureError, ExternalTextureExport, PlatformTextureDescriptor};
use crate::export_core::{EXPORT_COLOR_FORMAT, EXPORT_SAMPLE_COUNT};
use ash::vk;
use std::ffi::CStr;
use std::os::fd::{FromRawFd, IntoRawFd, OwnedFd};

const EXPORT_TEXTURE_LABEL: &str = "VulkanDmaBuf Export Target";
const EXPORT_TEXTURE_USAGE: wgpu::TextureUsages = wgpu::TextureUsages::RENDER_ATTACHMENT
    .union(wgpu::TextureUsages::TEXTURE_BINDING)
    .union(wgpu::TextureUsages::COPY_SRC)
    .union(wgpu::TextureUsages::COPY_DST);
const EXPORT_HAL_TEXTURE_USAGE: wgpu::TextureUses = wgpu::TextureUses::COLOR_TARGET
    .union(wgpu::TextureUses::RESOURCE)
    .union(wgpu::TextureUses::COPY_SRC)
    .union(wgpu::TextureUses::COPY_DST);
const DRM_FORMAT_RESERVED: u64 = (1_u64 << 56) - 1;
const DRM_FORMAT_ABGR8888: u32 = fourcc_code(b'A', b'B', b'2', b'4');
const DRM_FORMAT_MOD_INVALID: u64 = fourcc_mod_code_none(DRM_FORMAT_RESERVED);

struct CreatedTexture {
    texture: wgpu::Texture,
    dma_buf_fd: OwnedFd,
    modifier: u64,
}

/// Vulkan-basierte Texture fuer den Zero-Copy-Export an Flutter/Impeller via DMA-BUF.
///
/// Die zugrundeliegende Vulkan-Image-/Memory-Allocation wird explizit mit
/// `VK_EXTERNAL_MEMORY_HANDLE_TYPE_DMA_BUF_BIT_EXT` erzeugt und anschliessend
/// via `wgpu` HAL in ein regulaeres `wgpu::Texture`-Objekt ueberfuehrt.
pub struct VulkanDmaBufTexture {
    texture: wgpu::Texture,
    view: wgpu::TextureView,
    dma_buf_fd: OwnedFd,
    width: u32,
    height: u32,
    stride: u32,
    modifier: u64,
    drm_format: u32,
}

impl VulkanDmaBufTexture {
    fn create_texture(
        device: &wgpu::Device,
        width: u32,
        height: u32,
    ) -> Result<CreatedTexture, ExternalTextureError> {
        validate_size(device, width, height)?;

        let wgpu_desc = wgpu_texture_descriptor(width, height);
        let hal_desc = hal_texture_descriptor(width, height)?;

        // SAFETY: Der Aufrufer hat ein `wgpu::Device` aus dem Vulkan-Backend erzeugt.
        // Wir holen nur den HAL-Handle fuer genau dieses Device ab und erzeugen damit
        // ein passendes Vulkan-Image samt dedizierter Memory-Allocation.
        let hal_device = unsafe { device.as_hal::<wgpu::hal::vulkan::Api>() }
            .ok_or(ExternalTextureError::PlatformNotSupported)?;
        ensure_required_extensions(&hal_device)?;

        let raw_device = hal_device.raw_device();
        let raw_instance = hal_device.shared_instance().raw_instance();
        let physical_device = hal_device.raw_physical_device();

        let mut external_image_info =
            vk::ExternalMemoryImageCreateInfo::default().handle_types(
                vk::ExternalMemoryHandleTypeFlags::DMA_BUF_EXT,
            );
        let image_create_info = vk::ImageCreateInfo::default()
            .image_type(vk::ImageType::TYPE_2D)
            .format(export_vk_format()?)
            .extent(vk::Extent3D {
                width,
                height,
                depth: 1,
            })
            .mip_levels(1)
            .array_layers(1)
            .samples(vk::SampleCountFlags::TYPE_1)
            .tiling(vk::ImageTiling::OPTIMAL)
            .usage(
                vk::ImageUsageFlags::COLOR_ATTACHMENT
                    | vk::ImageUsageFlags::SAMPLED
                    | vk::ImageUsageFlags::TRANSFER_SRC
                    | vk::ImageUsageFlags::TRANSFER_DST,
            )
            .sharing_mode(vk::SharingMode::EXCLUSIVE)
            .initial_layout(vk::ImageLayout::UNDEFINED)
            .push_next(&mut external_image_info);

        // SAFETY: `image_create_info` referenziert nur lokale Builder-Daten, die bis zum
        // Vulkan-Aufruf leben. Das Device stammt direkt aus dem zugehoerigen HAL-Backend.
        let image = unsafe { raw_device.create_image(&image_create_info, None) }
            .map_err(|error| ExternalTextureError::CreationFailed(format!(
                "VkImage fuer DMA-BUF-Export konnte nicht erzeugt werden: {error}"
            )))?;

        // SAFETY: Das Image wurde unmittelbar zuvor auf demselben Device erzeugt.
        let memory_requirements = unsafe { raw_device.get_image_memory_requirements(image) };
        let memory_type_index = match find_memory_type_index(
            raw_instance,
            physical_device,
            memory_requirements.memory_type_bits,
            vk::MemoryPropertyFlags::DEVICE_LOCAL,
        ) {
            Some(index) => index,
            None => {
                cleanup_image(raw_device, image, None);
                return Err(ExternalTextureError::CreationFailed(
                    "Kein DEVICE_LOCAL-Memory-Type fuer exportierbares Vulkan-Image gefunden"
                        .into(),
                ));
            }
        };

        let mut export_allocate_info =
            vk::ExportMemoryAllocateInfo::default().handle_types(
                vk::ExternalMemoryHandleTypeFlags::DMA_BUF_EXT,
            );
        let mut dedicated_allocate_info = vk::MemoryDedicatedAllocateInfo::default().image(image);
        let memory_allocate_info = vk::MemoryAllocateInfo::default()
            .allocation_size(memory_requirements.size)
            .memory_type_index(memory_type_index)
            .push_next(&mut dedicated_allocate_info)
            .push_next(&mut export_allocate_info);

        // SAFETY: Die Speicheranforderungen stammen vom gerade erzeugten Image; die Allocation
        // wird auf demselben Device mit passendem Memory-Type angefordert.
        let memory = match unsafe { raw_device.allocate_memory(&memory_allocate_info, None) } {
            Ok(memory) => memory,
            Err(error) => {
                cleanup_image(raw_device, image, None);
                return Err(ExternalTextureError::CreationFailed(format!(
                    "VkDeviceMemory fuer DMA-BUF-Image konnte nicht alloziert werden: {error}"
                )));
            }
        };

        // SAFETY: Image und Memory wurden auf demselben Device erzeugt; Offset 0 ist zulaessig,
        // da die dedizierte Allocation exakt dieses Image backed.
        if let Err(error) = unsafe { raw_device.bind_image_memory(image, memory, 0) } {
            cleanup_image(raw_device, image, Some(memory));
            return Err(ExternalTextureError::CreationFailed(format!(
                "VkImageMemory-Bind fuer DMA-BUF-Image fehlgeschlagen: {error}"
            )));
        }

        let fd_loader = ash::khr::external_memory_fd::Device::new(raw_instance, raw_device);
        let get_fd_info = vk::MemoryGetFdInfoKHR::default()
            .memory(memory)
            .handle_type(vk::ExternalMemoryHandleTypeFlags::DMA_BUF_EXT);

        // SAFETY: Das Device-Memory wurde gerade mit exportfaehigem Handle-Typ alloziert und
        // ist weiterhin an dieses Image gebunden.
        let exported_fd = match unsafe { fd_loader.get_memory_fd(&get_fd_info) } {
            Ok(fd) => fd,
            Err(error) => {
                cleanup_image(raw_device, image, Some(memory));
                return Err(ExternalTextureError::CreationFailed(format!(
                    "DMA-BUF-FD konnte nicht ueber vkGetMemoryFdKHR exportiert werden: {error}"
                )));
            }
        };

        // SAFETY: `exported_fd` stammt direkt aus `vkGetMemoryFdKHR` und geht hier in den
        // alleinigen Besitz von `OwnedFd` ueber.
        let dma_buf_fd = unsafe { OwnedFd::from_raw_fd(exported_fd) };
        let modifier = query_drm_modifier(
            raw_instance,
            raw_device,
            image,
            hal_device.enabled_device_extensions(),
        );

        // SAFETY: `image` und `memory` wurden auf genau diesem HAL-Device erzeugt und respektieren
        // den uebergebenen HAL-Deskriptor. Mit `TextureMemory::Dedicated` uebernimmt wgpu-hal den
        // Besitz von Vulkan-Image und DeviceMemory beim Drop der resultierenden `wgpu::Texture`.
        let hal_texture = unsafe {
            hal_device.texture_from_raw(
                image,
                &hal_desc,
                None,
                wgpu::hal::vulkan::TextureMemory::Dedicated(memory),
            )
        };
        // SAFETY: Das HAL-Texture-Objekt wurde unmittelbar zuvor aus demselben Device erzeugt und
        // nutzt denselben WGPU-Deskriptor wie spaeteres View-/Render-Handling.
        let texture = unsafe {
            device.create_texture_from_hal::<wgpu::hal::vulkan::Api>(hal_texture, &wgpu_desc)
        };

        Ok(CreatedTexture {
            texture,
            dma_buf_fd,
            modifier,
        })
    }
}

impl ExternalTextureExport for VulkanDmaBufTexture {
    fn create_exportable_texture(
        device: &wgpu::Device,
        width: u32,
        height: u32,
    ) -> Result<Self, ExternalTextureError> {
        let CreatedTexture {
            texture,
            dma_buf_fd,
            modifier,
        } = Self::create_texture(device, width, height)?;
        let view = texture.create_view(&wgpu::TextureViewDescriptor::default());

        Ok(Self {
            texture,
            view,
            dma_buf_fd,
            width,
            height,
            stride: 0,
            modifier,
            drm_format: export_drm_format()?,
        })
    }

    fn export_descriptor(&self) -> Result<PlatformTextureDescriptor, ExternalTextureError> {
        let exported_fd = self
            .dma_buf_fd
            .try_clone()
            .map_err(|error| {
                ExternalTextureError::ExportFailed(format!(
                    "DMA-BUF-Dateideskriptor konnte nicht dupliziert werden: {error}"
                ))
            })?
            .into_raw_fd();

        Ok(PlatformTextureDescriptor::LinuxDmaBuf {
            fd: exported_fd,
            width: self.width,
            height: self.height,
            stride: self.stride,
            format: self.drm_format,
            modifier: self.modifier,
        })
    }

    fn texture_view(&self) -> &wgpu::TextureView {
        let _texture_guard = &self.texture;
        &self.view
    }

    fn resize(
        &mut self,
        device: &wgpu::Device,
        width: u32,
        height: u32,
    ) -> Result<(), ExternalTextureError> {
        *self = Self::create_exportable_texture(device, width, height)?;
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

fn validate_size(
    device: &wgpu::Device,
    width: u32,
    height: u32,
) -> Result<(), ExternalTextureError> {
    if width == 0 || height == 0 {
        return Err(ExternalTextureError::CreationFailed(format!(
            "Texturgroesse muss positiv sein, erhalten {width}x{height}"
        )));
    }

    let max_dimension = device.limits().max_texture_dimension_2d;
    if width > max_dimension || height > max_dimension {
        return Err(ExternalTextureError::CreationFailed(format!(
            "Texturgroesse {width}x{height} ueberschreitet Device-Limit {max_dimension}"
        )));
    }

    Ok(())
}

fn wgpu_texture_descriptor(width: u32, height: u32) -> wgpu::TextureDescriptor<'static> {
    wgpu::TextureDescriptor {
        label: Some(EXPORT_TEXTURE_LABEL),
        size: wgpu::Extent3d {
            width,
            height,
            depth_or_array_layers: 1,
        },
        mip_level_count: 1,
        sample_count: EXPORT_SAMPLE_COUNT,
        dimension: wgpu::TextureDimension::D2,
        format: EXPORT_COLOR_FORMAT,
        usage: EXPORT_TEXTURE_USAGE,
        view_formats: &[],
    }
}

fn hal_texture_descriptor(
    width: u32,
    height: u32,
) -> Result<wgpu::hal::TextureDescriptor<'static>, ExternalTextureError> {
    Ok(wgpu::hal::TextureDescriptor {
        label: Some(EXPORT_TEXTURE_LABEL),
        size: wgpu::Extent3d {
            width,
            height,
            depth_or_array_layers: 1,
        },
        mip_level_count: 1,
        sample_count: EXPORT_SAMPLE_COUNT,
        dimension: wgpu::TextureDimension::D2,
        format: EXPORT_COLOR_FORMAT,
        usage: EXPORT_HAL_TEXTURE_USAGE,
        memory_flags: wgpu::hal::MemoryFlags::empty(),
        view_formats: Vec::new(),
    })
}

fn ensure_required_extensions(
    hal_device: &wgpu::hal::vulkan::Device,
) -> Result<(), ExternalTextureError> {
    let enabled_extensions = hal_device.enabled_device_extensions();
    if !has_extension(enabled_extensions, ash::khr::external_memory_fd::NAME)
        || !has_extension(enabled_extensions, ash::ext::external_memory_dma_buf::NAME)
    {
        return Err(ExternalTextureError::ExtensionNotAvailable);
    }

    Ok(())
}

fn has_extension(enabled_extensions: &[&'static CStr], required_extension: &'static CStr) -> bool {
    enabled_extensions
        .iter()
        .copied()
        .any(|extension| extension == required_extension)
}

fn export_vk_format() -> Result<vk::Format, ExternalTextureError> {
    match EXPORT_COLOR_FORMAT {
        wgpu::TextureFormat::Rgba8UnormSrgb => Ok(vk::Format::R8G8B8A8_SRGB),
        format => Err(ExternalTextureError::CreationFailed(format!(
            "Texture-Format {format:?} wird fuer Vulkan DMA-BUF-Export noch nicht unterstuetzt"
        ))),
    }
}

fn export_drm_format() -> Result<u32, ExternalTextureError> {
    match EXPORT_COLOR_FORMAT {
        wgpu::TextureFormat::Rgba8UnormSrgb => Ok(DRM_FORMAT_ABGR8888),
        format => Err(ExternalTextureError::CreationFailed(format!(
            "DRM-FourCC fuer Texture-Format {format:?} ist nicht definiert"
        ))),
    }
}

fn find_memory_type_index(
    raw_instance: &ash::Instance,
    physical_device: vk::PhysicalDevice,
    memory_type_bits: u32,
    required_flags: vk::MemoryPropertyFlags,
) -> Option<u32> {
    // SAFETY: `physical_device` stammt aus demselben Vulkan-Instance-Objekt; der Aufruf liest nur
    // statische Memory-Properties des Adapters.
    let memory_properties = unsafe {
        raw_instance.get_physical_device_memory_properties(physical_device)
    };

    memory_properties
        .memory_types_as_slice()
        .iter()
        .enumerate()
        .find_map(|(index, memory_type)| {
            let type_mask = 1_u32 << index;
            let is_compatible = memory_type_bits & type_mask != 0;
            let has_required_flags = memory_type.property_flags.contains(required_flags);
            if is_compatible && has_required_flags {
                Some(index as u32)
            } else {
                None
            }
        })
}

fn cleanup_image(raw_device: &ash::Device, image: vk::Image, memory: Option<vk::DeviceMemory>) {
    if let Some(memory) = memory {
        // SAFETY: Dieser Cleanup-Pfad laeuft nur fuer lokal erzeugte, noch nicht an wgpu-hal
        // uebergebene Ressourcen. `memory` gehoert exklusiv diesem Device.
        unsafe { raw_device.free_memory(memory, None) };
    }

    // SAFETY: Dieser Cleanup-Pfad laeuft nur fuer lokal erzeugte, noch nicht an wgpu-hal
    // uebergebene Images. `image` gehoert exklusiv diesem Device.
    unsafe { raw_device.destroy_image(image, None) };
}

fn query_drm_modifier(
    raw_instance: &ash::Instance,
    raw_device: &ash::Device,
    image: vk::Image,
    enabled_extensions: &[&'static CStr],
) -> u64 {
    if !has_extension(enabled_extensions, ash::ext::image_drm_format_modifier::NAME) {
        return DRM_FORMAT_MOD_INVALID;
    }

    let drm_modifier_loader =
        ash::ext::image_drm_format_modifier::Device::new(raw_instance, raw_device);
    let mut modifier_properties = vk::ImageDrmFormatModifierPropertiesEXT::default();
    // SAFETY: Das Image wurde auf diesem Device erzeugt; die Query liest nur Metadaten aus dem
    // Treiber und veraendert weder Image noch Allocation.
    match unsafe {
        drm_modifier_loader.get_image_drm_format_modifier_properties(
            image,
            &mut modifier_properties,
        )
    } {
        Ok(()) => modifier_properties.drm_format_modifier,
        Err(_) => DRM_FORMAT_MOD_INVALID,
    }
}

const fn fourcc_code(a: u8, b: u8, c: u8, d: u8) -> u32 {
    (a as u32) | ((b as u32) << 8) | ((c as u32) << 16) | ((d as u32) << 24)
}

const fn fourcc_mod_code_none(value: u64) -> u64 {
    value & 0x00ff_ffff_ffff_ffff
}

#[cfg(test)]
mod tests {
    use super::*;

    fn create_test_gpu() -> Option<(wgpu::Instance, wgpu::Adapter, wgpu::Device, wgpu::Queue)> {
        let instance = crate::create_vulkan_instance();
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

    fn create_export_texture_or_skip(device: &wgpu::Device) -> Option<VulkanDmaBufTexture> {
        match VulkanDmaBufTexture::create_exportable_texture(device, 16, 16) {
            Ok(texture) => Some(texture),
            Err(ExternalTextureError::ExtensionNotAvailable) => None,
            Err(error) => panic!("Export-Texture-Erzeugung darf nicht fehlschlagen: {error}"),
        }
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

    /// Prueft, dass export_descriptor bei verfuegbaren Extensions einen DMA-BUF beschreibt.
    #[test]
    fn test_export_descriptor_returns_dmabuf_descriptor() {
        let Some((_inst, _adp, device, _queue)) = create_test_gpu() else {
            return;
        };
        let Some(texture) = create_export_texture_or_skip(&device) else {
            return;
        };

        let first = texture
            .export_descriptor()
            .expect("DMA-BUF-Descriptor-Export muss gelingen");
        let second = texture
            .export_descriptor()
            .expect("Wiederholter DMA-BUF-Descriptor-Export muss gelingen");

        let (first_fd, second_fd) = match (first, second) {
            (
                PlatformTextureDescriptor::LinuxDmaBuf {
                    fd: first_fd,
                    width,
                    height,
                    stride,
                    format,
                    modifier: _,
                },
                PlatformTextureDescriptor::LinuxDmaBuf {
                    fd: second_fd,
                    width: second_width,
                    height: second_height,
                    ..
                },
            ) => {
                assert_eq!(width, 16);
                assert_eq!(height, 16);
                assert_eq!(second_width, 16);
                assert_eq!(second_height, 16);
                assert_eq!(stride, 0);
                assert_eq!(format, DRM_FORMAT_ABGR8888);
                (first_fd, second_fd)
            }
        };

        assert_ne!(first_fd, second_fd, "Jeder Export muss einen neuen FD liefern");

        // SAFETY: Beide FDs wurden ueber `export_descriptor()` an den Test uebertragen und werden
        // genau einmal in `OwnedFd` ueberfuehrt, damit sie am Testende geschlossen werden.
        let _first_fd = unsafe { OwnedFd::from_raw_fd(first_fd) };
        // SAFETY: Siehe oben fuer den ersten FD; der zweite FD wird identisch behandelt.
        let _second_fd = unsafe { OwnedFd::from_raw_fd(second_fd) };
    }

    /// Prueft, dass resize() die Dimensionen korrekt aktualisiert.
    #[test]
    fn test_resize_updates_dimensions() {
        let Some((_inst, _adp, device, _queue)) = create_test_gpu() else {
            return;
        };
        let Some(mut texture) = create_export_texture_or_skip(&device) else {
            return;
        };
        assert_eq!(dmabuf_texture_width(&texture), 16);
        assert_eq!(dmabuf_texture_height(&texture), 16);

        texture
            .resize(&device, 32, 64)
            .expect("Resize muss gelingen");
        assert_eq!(dmabuf_texture_width(&texture), 32);
        assert_eq!(dmabuf_texture_height(&texture), 64);
    }
}
