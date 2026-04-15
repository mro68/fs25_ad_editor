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

struct HardwareBufferProperties {
    allocation_size: vk::DeviceSize,
    memory_type_bits: u32,
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
        let hardware_buffer = allocate_hardware_buffer(width, height)?;
        let hardware_buffer_properties =
            match query_hardware_buffer_properties(raw_instance, raw_device, hardware_buffer) {
                Ok(properties) => properties,
                Err(error) => {
                    release_hardware_buffer(hardware_buffer);
                    return Err(error);
                }
            };

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
                release_hardware_buffer(hardware_buffer);
                ExternalTextureError::CreationFailed(format!(
                    "VkImage fuer importierten AHardwareBuffer konnte nicht erzeugt werden: {error}"
                ))
            })?;

        let memory_type_index = match find_memory_type_index(
            raw_instance,
            physical_device,
            hardware_buffer_properties.memory_type_bits,
            vk::MemoryPropertyFlags::DEVICE_LOCAL,
        ) {
            Some(index) => index,
            None => {
                cleanup_image_and_hardware_buffer(raw_device, image, None, hardware_buffer);
                return Err(ExternalTextureError::CreationFailed(
                    "Kein DEVICE_LOCAL-Memory-Type fuer den importierten AHardwareBuffer gefunden"
                        .into(),
                ));
            }
        };

        let mut import_hardware_buffer_info =
            vk::ImportAndroidHardwareBufferInfoANDROID::default().buffer(hardware_buffer.cast());
        let mut dedicated_allocate_info = vk::MemoryDedicatedAllocateInfo::default()
            .image(image)
            .buffer(vk::Buffer::null());
        let memory_allocate_info = vk::MemoryAllocateInfo::default()
            .allocation_size(hardware_buffer_properties.allocation_size)
            .memory_type_index(memory_type_index)
            .push_next(&mut dedicated_allocate_info)
            .push_next(&mut import_hardware_buffer_info);

        // SAFETY: Die Allocation importiert den zuvor allozierten AHardwareBuffer auf demselben
        // Device. Groesse und Memory-Type stammen direkt aus den Vulkan-AHB-Properties.
        let memory = match unsafe { raw_device.allocate_memory(&memory_allocate_info, None) } {
            Ok(memory) => memory,
            Err(error) => {
                cleanup_image_and_hardware_buffer(raw_device, image, None, hardware_buffer);
                return Err(ExternalTextureError::CreationFailed(format!(
					"VkDeviceMemory fuer importierten AHardwareBuffer konnte nicht alloziert werden: {error}"
				)));
            }
        };

        // SAFETY: Image und Memory wurden auf demselben Device erzeugt; Offset 0 ist zulaessig,
        // da die dedizierte Allocation exakt dieses Image backed.
        if let Err(error) = unsafe { raw_device.bind_image_memory(image, memory, 0) } {
            cleanup_image_and_hardware_buffer(raw_device, image, Some(memory), hardware_buffer);
            return Err(ExternalTextureError::CreationFailed(format!(
                "VkImageMemory-Bind fuer importierten AHardwareBuffer fehlgeschlagen: {error}"
            )));
        }

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

fn allocate_hardware_buffer(
    width: u32,
    height: u32,
) -> Result<*mut ndk_sys::AHardwareBuffer, ExternalTextureError> {
    let hardware_buffer_desc = ndk_sys::AHardwareBuffer_Desc {
        width,
        height,
        layers: 1,
        format: ndk_sys::AHardwareBuffer_Format::AHARDWAREBUFFER_FORMAT_R8G8B8A8_UNORM.0,
        usage: ndk_sys::AHardwareBuffer_UsageFlags::AHARDWAREBUFFER_USAGE_GPU_COLOR_OUTPUT.0
            | ndk_sys::AHardwareBuffer_UsageFlags::AHARDWAREBUFFER_USAGE_GPU_SAMPLED_IMAGE.0,
        stride: 0,
        rfu0: 0,
        rfu1: 0,
    };
    let mut hardware_buffer = std::ptr::null_mut();

    // SAFETY: Der Deskriptor ist vollstaendig initialisiert und `outBuffer` zeigt auf lokalen
    // Speicher fuer den vom Android-NDK gelieferten AHardwareBuffer-Pointer.
    let result =
        unsafe { ndk_sys::AHardwareBuffer_allocate(&hardware_buffer_desc, &mut hardware_buffer) };
    if result != 0 {
        return Err(ExternalTextureError::CreationFailed(format!(
            "AHardwareBuffer konnte nicht alloziert werden: AHardwareBuffer_allocate lieferte Fehlercode {result}"
        )));
    }
    if hardware_buffer.is_null() {
        return Err(ExternalTextureError::CreationFailed(
            "AHardwareBuffer_allocate lieferte einen Null-Pointer".into(),
        ));
    }

    Ok(hardware_buffer)
}

fn query_hardware_buffer_properties(
    raw_instance: &ash::Instance,
    raw_device: &ash::Device,
    hardware_buffer: *mut ndk_sys::AHardwareBuffer,
) -> Result<HardwareBufferProperties, ExternalTextureError> {
    let ahb_device = ash::android::external_memory_android_hardware_buffer::Device::new(
        raw_instance,
        raw_device,
    );
    let mut ahb_format_properties = vk::AndroidHardwareBufferFormatPropertiesANDROID::default();
    let mut ahb_properties =
        vk::AndroidHardwareBufferPropertiesANDROID::default().push_next(&mut ahb_format_properties);

    // SAFETY: `hardware_buffer` stammt aus `AHardwareBuffer_allocate` und ist bis zum Ende dieses
    // Aufrufs gueltig. Vulkan liest nur die Metadaten des nativen Buffers aus.
    unsafe {
        ahb_device.get_android_hardware_buffer_properties(
            hardware_buffer.cast(),
            &mut ahb_properties,
        )
    }
    .map_err(|error| {
        ExternalTextureError::CreationFailed(format!(
            "VkAndroidHardwareBufferProperties konnten fuer den AHardwareBuffer nicht abgefragt werden: {error}"
        ))
    })?;

    let allocation_size = ahb_properties.allocation_size;
    let memory_type_bits = ahb_properties.memory_type_bits;
    let imported_format = ahb_format_properties.format;

    if allocation_size == 0 {
        return Err(ExternalTextureError::CreationFailed(
            "VkAndroidHardwareBufferProperties meldeten allocation_size = 0".into(),
        ));
    }
    if memory_type_bits == 0 {
        return Err(ExternalTextureError::CreationFailed(
            "VkAndroidHardwareBufferProperties meldeten keine kompatiblen Memory-Types".into(),
        ));
    }
    if !matches!(
        imported_format,
        vk::Format::R8G8B8A8_UNORM | vk::Format::R8G8B8A8_SRGB
    ) {
        return Err(ExternalTextureError::CreationFailed(format!(
            "AHardwareBuffer-Format {:?} wird fuer den Vulkan-Importpfad noch nicht unterstuetzt",
            imported_format
        )));
    }

    Ok(HardwareBufferProperties {
        allocation_size,
        memory_type_bits,
    })
}

fn release_hardware_buffer(hardware_buffer: *mut ndk_sys::AHardwareBuffer) {
    if !hardware_buffer.is_null() {
        // SAFETY: Der Aufrufer uebergibt nur AHardwareBuffer-Referenzen, die lokal gehalten oder
        // frisch alloziert wurden und hier gezielt freigegeben werden sollen.
        unsafe { ndk_sys::AHardwareBuffer_release(hardware_buffer) };
    }
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

fn cleanup_image_and_hardware_buffer(
    raw_device: &ash::Device,
    image: vk::Image,
    memory: Option<vk::DeviceMemory>,
    hardware_buffer: *mut ndk_sys::AHardwareBuffer,
) {
    cleanup_image(raw_device, image, memory);
    release_hardware_buffer(hardware_buffer);
}
