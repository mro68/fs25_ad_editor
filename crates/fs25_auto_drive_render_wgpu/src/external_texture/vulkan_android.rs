//! Android/Vulkan-Implementierung des GPU-Texture-Exports via AHardwareBuffer.

use super::{ExternalTextureError, ExternalTextureExport, PlatformTextureDescriptor};
use crate::export_core::{EXPORT_COLOR_FORMAT, EXPORT_SAMPLE_COUNT};
use ash::vk;
use std::ffi::CStr;

const EXPORT_TEXTURE_LABEL: &str = "VulkanAhb Export Target";
const EXPORT_TEXTURE_USAGE: wgpu::TextureUsages = wgpu::TextureUsages::RENDER_ATTACHMENT
    .union(wgpu::TextureUsages::COPY_SRC)
    .union(wgpu::TextureUsages::COPY_DST);
const EXPORT_HAL_TEXTURE_USAGE: wgpu::TextureUses = wgpu::TextureUses::COLOR_TARGET
    .union(wgpu::TextureUses::COPY_SRC)
    .union(wgpu::TextureUses::COPY_DST);

struct CreatedTexture {
    texture: wgpu::Texture,
    hardware_buffer: *mut ndk_sys::AHardwareBuffer,
}

/// GPU-Textur mit AHardwareBuffer-Export fuer Android Vulkan Zero-Copy.
///
/// Erstellt eine Vulkan-Textur, die ueber
/// `VK_ANDROID_external_memory_android_hardware_buffer` als `AHardwareBuffer`
/// exportiert werden kann. Der Host kann diesen Buffer anschliessend ueber
/// Android-Nativ- oder EGL-Importpfade weiterverwenden.
pub struct VulkanAhbTexture {
    texture: wgpu::Texture,
    view: wgpu::TextureView,
    /// Persistent gehaltene AHB-Referenz. Wird im Drop released.
    hardware_buffer: *mut ndk_sys::AHardwareBuffer,
    width: u32,
    height: u32,
}

// SAFETY: Android garantiert Thread-Safety fuer AHardwareBuffer-Referenzen.
unsafe impl Send for VulkanAhbTexture {}
// SAFETY: Die Referenzzaehlung des AHardwareBuffer ist thread-safe.
unsafe impl Sync for VulkanAhbTexture {}

impl VulkanAhbTexture {
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

        let mut external_image_info = vk::ExternalMemoryImageCreateInfo::default()
            .handle_types(vk::ExternalMemoryHandleTypeFlags::ANDROID_HARDWARE_BUFFER_ANDROID);
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
            // Android-EGL-Importpfade koennen AHardwareBuffer auch mit OPTIMAL-Tiling nutzen.
            .tiling(vk::ImageTiling::OPTIMAL)
            .usage(
                vk::ImageUsageFlags::COLOR_ATTACHMENT
                    | vk::ImageUsageFlags::TRANSFER_SRC
                    | vk::ImageUsageFlags::TRANSFER_DST,
            )
            .sharing_mode(vk::SharingMode::EXCLUSIVE)
            .initial_layout(vk::ImageLayout::UNDEFINED)
            .push_next(&mut external_image_info);

        // SAFETY: `image_create_info` referenziert nur lokale Builder-Daten, die bis zum
        // Vulkan-Aufruf leben. Das Device stammt direkt aus dem zugehoerigen HAL-Backend.
        let image =
            unsafe { raw_device.create_image(&image_create_info, None) }.map_err(|error| {
                ExternalTextureError::CreationFailed(format!(
                    "VkImage fuer AHardwareBuffer-Export konnte nicht erzeugt werden: {error}"
                ))
            })?;

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

        let mut export_allocate_info = vk::ExportMemoryAllocateInfo::default()
            .handle_types(vk::ExternalMemoryHandleTypeFlags::ANDROID_HARDWARE_BUFFER_ANDROID);
        let mut dedicated_allocate_info = vk::MemoryDedicatedAllocateInfo::default()
            .image(image)
            .buffer(vk::Buffer::null());
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
					"VkDeviceMemory fuer AHardwareBuffer-Image konnte nicht alloziert werden: {error}"
				)));
            }
        };

        // SAFETY: Image und Memory wurden auf demselben Device erzeugt; Offset 0 ist zulaessig,
        // da die dedizierte Allocation exakt dieses Image backed.
        if let Err(error) = unsafe { raw_device.bind_image_memory(image, memory, 0) } {
            cleanup_image(raw_device, image, Some(memory));
            return Err(ExternalTextureError::CreationFailed(format!(
                "VkImageMemory-Bind fuer AHardwareBuffer-Image fehlgeschlagen: {error}"
            )));
        }

        let hardware_buffer = match export_hardware_buffer(raw_instance, raw_device, memory) {
            Ok(hardware_buffer) => hardware_buffer,
            Err(error) => {
                cleanup_image(raw_device, image, Some(memory));
                return Err(error);
            }
        };

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
            hardware_buffer,
        })
    }
}

impl ExternalTextureExport for VulkanAhbTexture {
    fn create_exportable_texture(
        device: &wgpu::Device,
        width: u32,
        height: u32,
    ) -> Result<Self, ExternalTextureError> {
        let CreatedTexture {
            texture,
            hardware_buffer,
        } = Self::create_texture(device, width, height)?;
        let view = texture.create_view(&wgpu::TextureViewDescriptor::default());

        Ok(Self {
            texture,
            view,
            hardware_buffer,
            width,
            height,
        })
    }

    fn export_descriptor(&self) -> Result<PlatformTextureDescriptor, ExternalTextureError> {
        debug_assert!(
            self.width > 0 && self.height > 0,
            "AHardwareBuffer-Export erwartet valide Texturgroessen"
        );
        // SAFETY: Die Instanz haelt eine gueltige persistente AHardwareBuffer-Referenz.
        unsafe { ndk_sys::AHardwareBuffer_acquire(self.hardware_buffer) };
        Ok(PlatformTextureDescriptor::AndroidHardwareBuffer {
            hardware_buffer_ptr: self.hardware_buffer as usize,
        })
    }

    fn texture_view(&self) -> &wgpu::TextureView {
        let _texture_guard = &self.texture;
        &self.view
    }

    fn texture(&self) -> &wgpu::Texture {
        &self.texture
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

impl Drop for VulkanAhbTexture {
    fn drop(&mut self) {
        if !self.hardware_buffer.is_null() {
            // SAFETY: Die Struct besitzt genau eine persistente AHardwareBuffer-Referenz.
            unsafe { ndk_sys::AHardwareBuffer_release(self.hardware_buffer) };
        }
    }
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
    if !has_extension(enabled_extensions, ash::vk::KHR_EXTERNAL_MEMORY_NAME)
        || !has_extension(
            enabled_extensions,
            ash::vk::ANDROID_EXTERNAL_MEMORY_ANDROID_HARDWARE_BUFFER_NAME,
        )
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
			"Texture-Format {format:?} wird fuer Vulkan AHardwareBuffer-Export noch nicht unterstuetzt"
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
    let memory_properties =
        unsafe { raw_instance.get_physical_device_memory_properties(physical_device) };

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

fn export_hardware_buffer(
    raw_instance: &ash::Instance,
    raw_device: &ash::Device,
    memory: vk::DeviceMemory,
) -> Result<*mut ndk_sys::AHardwareBuffer, ExternalTextureError> {
    let ahb_device = ash::android::external_memory_android_hardware_buffer::Device::new(
        raw_instance,
        raw_device,
    );
    let ahb_info = vk::MemoryGetAndroidHardwareBufferInfoANDROID::default().memory(memory);

    // SAFETY: Das Device-Memory wurde gerade mit exportfaehigem AHB-Handle-Typ alloziert.
    let hardware_buffer = unsafe { ahb_device.get_memory_android_hardware_buffer(&ahb_info) }
		.map_err(|error| {
			ExternalTextureError::CreationFailed(format!(
				"AHardwareBuffer konnte nicht ueber vkGetMemoryAndroidHardwareBufferANDROID exportiert werden: {error}"
			))
		})?
		.cast::<ndk_sys::AHardwareBuffer>();

    if hardware_buffer.is_null() {
        return Err(ExternalTextureError::CreationFailed(
            "vkGetMemoryAndroidHardwareBufferANDROID lieferte einen Null-Pointer".into(),
        ));
    }

    Ok(hardware_buffer)
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
