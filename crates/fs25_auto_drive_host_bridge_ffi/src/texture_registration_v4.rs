//! Additiver Texture-Registration-v4-Adapter ueber der kanonischen Host-Bridge-Session.
//!
//! Dieser Slice friert den C-ABI-Vertrag, die Capability-Matrix und die
//! plattformspezifischen Payload-Familien additiv neben Shared-Texture-v3 ein.
//! Echte externe Host-Registration ist damit aber noch nicht produktiv: Dafuer
//! braucht es zusaetzlich backend-native Export-/Attach-Pfade im Renderer und
//! native Host-Import-/Surface-Pfade im jeweiligen Flutter-/C++-Consumer.

use crate::{clear_last_error, set_last_error, HostBridgeSessionHandle};
use anyhow::{anyhow, Result};
use fs25_auto_drive_render_wgpu::{
    query_texture_registration_v4_capabilities, AndroidAttachmentKind,
    AndroidHardwareBufferDescriptor,
    TextureRegistrationAlphaMode, TextureRegistrationAvailability, TextureRegistrationModel,
    TextureRegistrationPayloadFamily, TextureRegistrationPixelFormat, TextureRegistrationPlatform,
    TextureRegistrationPlatformCapabilities, MAX_LINUX_DMABUF_PLANES,
    TEXTURE_REGISTRATION_V4_CONTRACT_VERSION,
};

const FS25AD_TEXTURE_REGISTRATION_V4_CONTRACT_VERSION: u32 =
    TEXTURE_REGISTRATION_V4_CONTRACT_VERSION;

const FS25AD_TEXTURE_REGISTRATION_V4_PIXEL_FORMAT_RGBA8_SRGB: u32 = 1;
const FS25AD_TEXTURE_REGISTRATION_V4_ALPHA_MODE_PREMULTIPLIED: u32 = 1;

const FS25AD_TEXTURE_REGISTRATION_V4_PLATFORM_WINDOWS: u32 = 1;
const FS25AD_TEXTURE_REGISTRATION_V4_PLATFORM_LINUX: u32 = 2;
const FS25AD_TEXTURE_REGISTRATION_V4_PLATFORM_ANDROID: u32 = 3;

const FS25AD_TEXTURE_REGISTRATION_V4_MODEL_EXPORT_LEASE: u32 = 1;
const FS25AD_TEXTURE_REGISTRATION_V4_MODEL_HOST_ATTACHED_SURFACE: u32 = 2;

const FS25AD_TEXTURE_REGISTRATION_V4_PAYLOAD_WINDOWS_DESCRIPTOR: u32 = 1;
const FS25AD_TEXTURE_REGISTRATION_V4_PAYLOAD_LINUX_DMABUF: u32 = 2;
const FS25AD_TEXTURE_REGISTRATION_V4_PAYLOAD_ANDROID_SURFACE_ATTACHMENT: u32 = 3;
const FS25AD_TEXTURE_REGISTRATION_V4_PAYLOAD_ANDROID_HARDWARE_BUFFER: u32 = 4;

const FS25AD_TEXTURE_REGISTRATION_V4_AVAILABILITY_SUPPORTED: u32 = 1;
const FS25AD_TEXTURE_REGISTRATION_V4_AVAILABILITY_NOT_YET_IMPLEMENTED: u32 = 2;
const FS25AD_TEXTURE_REGISTRATION_V4_AVAILABILITY_UNSUPPORTED: u32 = 3;

const FS25AD_TEXTURE_REGISTRATION_V4_WINDOWS_DESCRIPTOR_DXGI_SHARED_HANDLE: u32 = 1;
const FS25AD_TEXTURE_REGISTRATION_V4_WINDOWS_DESCRIPTOR_D3D11_TEXTURE2D: u32 = 2;

const FS25AD_TEXTURE_REGISTRATION_V4_ANDROID_ATTACHMENT_NATIVE_WINDOW: u32 = 1;
const FS25AD_TEXTURE_REGISTRATION_V4_ANDROID_ATTACHMENT_SURFACE_PRODUCER: u32 = 2;

const V4_GENERAL_BLOCKER_DETAIL: &str = "current host-bridge v4 runtime still uses a placeholder handle and does not plumb backend-native export descriptors from the renderer into the ABI";

/// Opaquer Platzhalter-Handle fuer den additiven v4-Vertrag.
pub(crate) struct HostBridgeTextureRegistrationV4 {
    _private: (),
}

fn platform_blocker_detail(platform: TextureRegistrationPlatform) -> &'static str {
    match platform {
        TextureRegistrationPlatform::Windows => {
            "windows blocker: the renderer does not create exportable DXGI/D3D resources and this repo has no native host registration path for DXGI shared handles or ID3D11Texture2D"
        }
        TextureRegistrationPlatform::Linux => {
            "linux blocker: the v4 host-bridge runtime is not yet wired to the renderer's DMA-BUF export path"
        }
        TextureRegistrationPlatform::Android => {
            "android blocker: the v4 host-bridge runtime is not yet wired to the renderer's AHardwareBuffer export-lease path"
        }
    }
}

#[repr(C)]
#[derive(Clone, Copy)]
/// Plattformspezifische Capability-Zeile des v4-Vertrags.
pub struct Fs25adTextureRegistrationV4PlatformCapabilities {
    /// ABI-Konstante der Plattform (`1=Windows`, `2=Linux`, `3=Android`).
    pub platform: u32,
    /// ABI-Konstante des Registrierungsmodells (`1=ExportLease`, `2=HostAttachedSurface`).
    pub registration_model: u32,
    /// ABI-Konstante der Payload-Familie.
    pub payload_family: u32,
    /// ABI-Konstante der Verfuegbarkeit (`1=Supported`, `2=NotYetImplemented`, `3=Unsupported`).
    pub availability: u32,
}

#[repr(C)]
#[derive(Clone, Copy)]
/// Gemeinsame Runtime-Capabilities des additiven v4-Vertrags.
///
/// `availability = NotYetImplemented` markiert explizit, dass fuer echte
/// externe Host-Registration noch backend-native oder host-native Pfade fehlen.
pub struct Fs25adTextureRegistrationV4Capabilities {
    /// Vertragsversion des v4-Pfads.
    pub contract_version: u32,
    /// ABI-Konstante des Pixel-Formats (`1 = RGBA8 sRGB`).
    pub pixel_format: u32,
    /// ABI-Konstante des Alpha-Modus (`1 = premultiplied`).
    pub alpha_mode: u32,
    /// `1`, wenn Acquire/Release vom Host explizit eingehalten werden muss.
    pub requires_explicit_release: u32,
    /// Plattformzeile fuer Windows.
    pub windows: Fs25adTextureRegistrationV4PlatformCapabilities,
    /// Plattformzeile fuer Linux.
    pub linux: Fs25adTextureRegistrationV4PlatformCapabilities,
    /// Plattformzeile fuer Android.
    pub android: Fs25adTextureRegistrationV4PlatformCapabilities,
}

#[repr(C)]
#[derive(Clone, Copy)]
/// Gemeinsame Frame-Metadaten fuer den v4-Lifecycle.
pub struct Fs25adTextureRegistrationV4FrameInfo {
    /// Frame-Breite in Pixeln.
    pub width: u32,
    /// Frame-Hoehe in Pixeln.
    pub height: u32,
    /// ABI-Konstante des Pixel-Formats (`1 = RGBA8 sRGB`).
    pub pixel_format: u32,
    /// ABI-Konstante des Alpha-Modus (`1 = premultiplied`).
    pub alpha_mode: u32,
    /// Runtime-ID der zugrundeliegenden GPU-Textur.
    pub texture_id: u64,
    /// Generation der GPU-Textur.
    pub texture_generation: u64,
    /// Lease-Token fuer Acquire/Release.
    pub frame_token: u64,
}

#[repr(C)]
#[derive(Clone, Copy)]
/// Windows-spezifischer Descriptor der v4-Payload-Familie.
pub struct Fs25adTextureRegistrationV4WindowsDescriptor {
    /// ABI-Konstante des Untertyps (`1 = DXGI Shared Handle`, `2 = D3D11 Texture2D`).
    pub descriptor_kind: u32,
    /// DXGI Shared Handle (nur fuer `descriptor_kind = 1`).
    pub dxgi_shared_handle: u64,
    /// Opaque `ID3D11Texture2D`-Pointerwert (nur fuer `descriptor_kind = 2`).
    pub d3d11_texture_ptr: usize,
    /// Opaque D3D11-Device-Pointerwert (nur fuer `descriptor_kind = 2`).
    pub d3d11_device_ptr: usize,
}

#[repr(C)]
#[derive(Clone, Copy)]
/// Linux-DMA-BUF-Plane des v4-Vertrags.
pub struct Fs25adTextureRegistrationV4LinuxDmabufPlane {
    /// Dateideskriptor der Plane.
    pub fd: i32,
    /// Plane-Offset in Bytes.
    pub offset_bytes: u32,
    /// Plane-Stride in Bytes.
    pub stride_bytes: u32,
}

#[repr(C)]
#[derive(Clone, Copy)]
/// Linux-DMA-BUF-Descriptor des v4-Vertrags.
pub struct Fs25adTextureRegistrationV4LinuxDmabufDescriptor {
    /// DRM FourCC-Format.
    pub drm_fourcc: u32,
    /// Oberes `u32` des DRM-Modifiers.
    pub drm_modifier_hi: u32,
    /// Unteres `u32` des DRM-Modifiers.
    pub drm_modifier_lo: u32,
    /// Anzahl gueltiger Planes.
    pub plane_count: u32,
    /// Feste Plane-Liste mit ABI-stabiler Groesse.
    pub planes: [Fs25adTextureRegistrationV4LinuxDmabufPlane; MAX_LINUX_DMABUF_PLANES],
}

#[repr(C)]
#[derive(Clone, Copy)]
/// C-ABI-Descriptor fuer Android-AHardwareBuffer-Export.
pub struct Fs25adTextureRegistrationV4AndroidHardwareBufferDescriptor {
    /// Opaker Pointer auf einen `AHardwareBuffer*`, bereits `AHardwareBuffer_acquire()`-t.
    /// Der Empfaenger muss `AHardwareBuffer_release()` aufrufen.
    pub hardware_buffer_ptr: usize,
}

#[repr(C)]
#[derive(Clone, Copy)]
/// Android-Surface-Descriptor des v4-Vertrags.
pub struct Fs25adTextureRegistrationV4AndroidSurfaceDescriptor {
    /// ABI-Konstante des Untertyps (`1 = NativeWindow`, `2 = SurfaceProducer`).
    pub attachment_kind: u32,
    /// Opaque Pointerwert auf ein NativeWindow-Aequivalent.
    pub native_window_ptr: usize,
    /// Opaque Pointerwert auf ein hostseitiges Surface-Objekt.
    pub surface_handle_ptr: usize,
}

fn platform_from_abi(platform: u32) -> Result<TextureRegistrationPlatform> {
    match platform {
        FS25AD_TEXTURE_REGISTRATION_V4_PLATFORM_WINDOWS => Ok(TextureRegistrationPlatform::Windows),
        FS25AD_TEXTURE_REGISTRATION_V4_PLATFORM_LINUX => Ok(TextureRegistrationPlatform::Linux),
        FS25AD_TEXTURE_REGISTRATION_V4_PLATFORM_ANDROID => Ok(TextureRegistrationPlatform::Android),
        value => Err(anyhow!(
            "unknown texture registration v4 platform value: {value}"
        )),
    }
}

fn platform_abi(platform: TextureRegistrationPlatform) -> u32 {
    match platform {
        TextureRegistrationPlatform::Windows => FS25AD_TEXTURE_REGISTRATION_V4_PLATFORM_WINDOWS,
        TextureRegistrationPlatform::Linux => FS25AD_TEXTURE_REGISTRATION_V4_PLATFORM_LINUX,
        TextureRegistrationPlatform::Android => FS25AD_TEXTURE_REGISTRATION_V4_PLATFORM_ANDROID,
    }
}

fn model_abi(model: TextureRegistrationModel) -> u32 {
    match model {
        TextureRegistrationModel::ExportLease => FS25AD_TEXTURE_REGISTRATION_V4_MODEL_EXPORT_LEASE,
        TextureRegistrationModel::HostAttachedSurface => {
            FS25AD_TEXTURE_REGISTRATION_V4_MODEL_HOST_ATTACHED_SURFACE
        }
    }
}

fn payload_family_abi(payload_family: TextureRegistrationPayloadFamily) -> u32 {
    match payload_family {
        TextureRegistrationPayloadFamily::WindowsDescriptor => {
            FS25AD_TEXTURE_REGISTRATION_V4_PAYLOAD_WINDOWS_DESCRIPTOR
        }
        TextureRegistrationPayloadFamily::LinuxDmabuf => {
            FS25AD_TEXTURE_REGISTRATION_V4_PAYLOAD_LINUX_DMABUF
        }
        TextureRegistrationPayloadFamily::AndroidSurfaceAttachment => {
            FS25AD_TEXTURE_REGISTRATION_V4_PAYLOAD_ANDROID_SURFACE_ATTACHMENT
        }
        TextureRegistrationPayloadFamily::AndroidHardwareBuffer => {
            FS25AD_TEXTURE_REGISTRATION_V4_PAYLOAD_ANDROID_HARDWARE_BUFFER
        }
    }
}

fn availability_abi(availability: TextureRegistrationAvailability) -> u32 {
    match availability {
        TextureRegistrationAvailability::Supported => {
            FS25AD_TEXTURE_REGISTRATION_V4_AVAILABILITY_SUPPORTED
        }
        TextureRegistrationAvailability::NotYetImplemented => {
            FS25AD_TEXTURE_REGISTRATION_V4_AVAILABILITY_NOT_YET_IMPLEMENTED
        }
        TextureRegistrationAvailability::Unsupported => {
            FS25AD_TEXTURE_REGISTRATION_V4_AVAILABILITY_UNSUPPORTED
        }
    }
}

fn pixel_format_abi(format: TextureRegistrationPixelFormat) -> u32 {
    match format {
        TextureRegistrationPixelFormat::Rgba8Srgb => {
            FS25AD_TEXTURE_REGISTRATION_V4_PIXEL_FORMAT_RGBA8_SRGB
        }
    }
}

fn alpha_mode_abi(mode: TextureRegistrationAlphaMode) -> u32 {
    match mode {
        TextureRegistrationAlphaMode::Premultiplied => {
            FS25AD_TEXTURE_REGISTRATION_V4_ALPHA_MODE_PREMULTIPLIED
        }
    }
}

fn android_attachment_kind_from_abi(kind: u32) -> Result<AndroidAttachmentKind> {
    match kind {
        FS25AD_TEXTURE_REGISTRATION_V4_ANDROID_ATTACHMENT_NATIVE_WINDOW => {
            Ok(AndroidAttachmentKind::NativeWindow)
        }
        FS25AD_TEXTURE_REGISTRATION_V4_ANDROID_ATTACHMENT_SURFACE_PRODUCER => {
            Ok(AndroidAttachmentKind::SurfaceProducer)
        }
        value => Err(anyhow!(
            "unknown texture registration v4 android attachment kind value: {value}"
        )),
    }
}

fn platform_capability_to_abi(
    capability: TextureRegistrationPlatformCapabilities,
) -> Fs25adTextureRegistrationV4PlatformCapabilities {
    Fs25adTextureRegistrationV4PlatformCapabilities {
        platform: platform_abi(capability.platform),
        registration_model: model_abi(capability.model),
        payload_family: payload_family_abi(capability.payload_family),
        availability: availability_abi(capability.availability),
    }
}

fn android_hardware_buffer_descriptor_to_abi(
    descriptor: AndroidHardwareBufferDescriptor,
) -> Fs25adTextureRegistrationV4AndroidHardwareBufferDescriptor {
    Fs25adTextureRegistrationV4AndroidHardwareBufferDescriptor {
        hardware_buffer_ptr: descriptor.hardware_buffer_ptr,
    }
}

fn ensure_texture_pointer(texture: *mut HostBridgeTextureRegistrationV4) -> Result<()> {
    if texture.is_null() {
        return Err(anyhow!(
            "HostBridgeTextureRegistrationV4 pointer must not be null"
        ));
    }

    Ok(())
}

fn ensure_session_pointer(session: *mut HostBridgeSessionHandle) -> Result<()> {
    if session.is_null() {
        return Err(anyhow!("HostBridgeSession pointer must not be null"));
    }

    Ok(())
}

fn fail_not_implemented(function_name: &str) -> Result<()> {
    Err(anyhow!(
        "{function_name} is not implemented for texture registration v4 in this build; {V4_GENERAL_BLOCKER_DETAIL}"
    ))
}

fn fail_android_hardware_buffer_descriptor_unavailable(frame_token: u64) -> Result<()> {
    eprintln!(
        "Android AHardwareBuffer descriptor getter: v4 runtime not yet connected to AHardwareBuffer export (frame_token={frame_token})"
    );
    Err(anyhow!(
        "fs25ad_host_bridge_texture_registration_v4_get_android_hardware_buffer_descriptor is not yet connected to the v4 runtime in this build; {}",
        platform_blocker_detail(TextureRegistrationPlatform::Android)
    ))
}

fn fail_legacy_android_surface_path(function_name: &str) -> Result<()> {
    eprintln!(
        "Legacy Android surface path is deprecated; use the export-lease AHardwareBuffer descriptor path instead ({function_name})"
    );
    Err(anyhow!(
        "{function_name} is deprecated for texture registration v4; use the export-lease Android AHardwareBuffer descriptor path instead"
    ))
}

/// Hilfsmakro: bool-FFI-Aufruf mit Panic-Isolation.
macro_rules! ffi_guard_bool {
    ($body:expr) => {{
        clear_last_error();
        match std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| $body)) {
            Ok(Ok(())) => true,
            Ok(Err(e)) => {
                set_last_error(e.to_string());
                false
            }
            Err(_) => {
                set_last_error("internal panic in FFI call");
                false
            }
        }
    }};
}

/// Liefert die Version des additiven Texture-Registration-v4-Vertrags.
#[unsafe(no_mangle)]
pub extern "C" fn fs25ad_host_bridge_texture_registration_v4_contract_version() -> u32 {
    FS25AD_TEXTURE_REGISTRATION_V4_CONTRACT_VERSION
}

/// Liefert die Runtime-Capabilities des additiven Texture-Registration-v4-Pfads.
///
/// # Safety
///
/// `out_capabilities` muss ein gueltiger, nicht-null Zeiger auf eine initialisierbare
/// `Fs25adTextureRegistrationV4Capabilities`-Struktur sein.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn fs25ad_host_bridge_texture_registration_v4_capabilities(
    out_capabilities: *mut Fs25adTextureRegistrationV4Capabilities,
) -> bool {
    ffi_guard_bool! {{
        if out_capabilities.is_null() {
            return Err(anyhow::anyhow!(
                "Fs25adTextureRegistrationV4Capabilities pointer must not be null"
            ));
        }
        let capabilities = query_texture_registration_v4_capabilities();
        // SAFETY: Aufrufer hat nicht-null Zeiger garantiert.
        unsafe {
            *out_capabilities = Fs25adTextureRegistrationV4Capabilities {
                contract_version: capabilities.contract_version,
                pixel_format: pixel_format_abi(capabilities.pixel_format),
                alpha_mode: alpha_mode_abi(capabilities.alpha_mode),
                requires_explicit_release: u32::from(capabilities.requires_explicit_release),
                windows: platform_capability_to_abi(capabilities.windows),
                linux: platform_capability_to_abi(capabilities.linux),
                android: platform_capability_to_abi(capabilities.android),
            };
        }
        Ok(())
    }}
}

/// Erstellt einen v4-Texture-Registration-Handle fuer eine Zielplattform.
#[unsafe(no_mangle)]
pub extern "C" fn fs25ad_host_bridge_texture_registration_v4_new(
    platform: u32,
    width: u32,
    height: u32,
) -> *mut HostBridgeTextureRegistrationV4 {
    clear_last_error();

    match (|| {
        if width == 0 || height == 0 {
            return Err(anyhow!(
                "texture registration v4 size must be positive, got {width}x{height}"
            ));
        }

        let platform = platform_from_abi(platform)?;
        let capabilities = query_texture_registration_v4_capabilities();
        let platform_capability = capabilities.platform(platform);

        match platform_capability.availability {
            TextureRegistrationAvailability::Supported => Err(anyhow!(
                "texture registration v4 backend for {platform} is currently not wired in this build; {}",
                platform_blocker_detail(platform)
            )),
            TextureRegistrationAvailability::NotYetImplemented => Err(anyhow!(
                "texture registration v4 backend for {platform} is not yet implemented in this build; {}",
                platform_blocker_detail(platform)
            )),
            TextureRegistrationAvailability::Unsupported => Err(anyhow!(
                "texture registration v4 backend for {platform} is unsupported on this target; {}",
                platform_blocker_detail(platform)
            )),
        }
    })() {
        Ok(()) => Box::into_raw(Box::new(HostBridgeTextureRegistrationV4 { _private: () })),
        Err(error) => {
            set_last_error(error.to_string());
            std::ptr::null_mut()
        }
    }
}

/// Gibt einen zuvor erstellten v4-Texture-Registration-Handle frei.
///
/// # Safety
///
/// `texture` muss ein durch `fs25ad_host_bridge_texture_registration_v4_new` erzeugter
/// Zeiger sein oder `null`. Nach dem Aufruf ist der Zeiger ungueltig.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn fs25ad_host_bridge_texture_registration_v4_dispose(
    texture: *mut HostBridgeTextureRegistrationV4,
) {
    clear_last_error();
    if texture.is_null() {
        return;
    }
    // SAFETY: Aufrufer garantiert durch _new allokierten Zeiger.
    unsafe { drop(Box::from_raw(texture)) };
}

/// Aendert die Zielgroesse eines v4-Texture-Registration-Handles.
///
/// # Safety
///
/// `texture` muss ein gueltiger, durch `fs25ad_host_bridge_texture_registration_v4_new`
/// erzeugter Zeiger sein.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn fs25ad_host_bridge_texture_registration_v4_resize(
    texture: *mut HostBridgeTextureRegistrationV4,
    width: u32,
    height: u32,
) -> bool {
    ffi_guard_bool! {{
        ensure_texture_pointer(texture)?;
        if width == 0 || height == 0 {
            return Err(anyhow!(
                "texture registration v4 size must be positive, got {width}x{height}"
            ));
        }
        fail_not_implemented("fs25ad_host_bridge_texture_registration_v4_resize")
    }}
}

/// Rendert den aktuellen Session-Frame fuer einen v4-Texture-Registration-Handle.
///
/// # Safety
///
/// `session` und `texture` muessen gueltige, durch die jeweiligen `_new`-Funktionen
/// erzeugte Zeiger sein.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn fs25ad_host_bridge_texture_registration_v4_render(
    session: *mut HostBridgeSessionHandle,
    texture: *mut HostBridgeTextureRegistrationV4,
) -> bool {
    ffi_guard_bool! {{
        ensure_session_pointer(session)?;
        ensure_texture_pointer(texture)?;
        fail_not_implemented("fs25ad_host_bridge_texture_registration_v4_render")
    }}
}

/// Leased den zuletzt gerenderten v4-Frame und liefert gemeinsame Metadaten.
///
/// # Safety
///
/// `texture` muss ein gueltiger Zeiger sein. `out_frame_info` muss ein gueltiger,
/// nicht-null Zeiger auf eine initialisierbare Struktur sein.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn fs25ad_host_bridge_texture_registration_v4_acquire(
    texture: *mut HostBridgeTextureRegistrationV4,
    out_frame_info: *mut Fs25adTextureRegistrationV4FrameInfo,
) -> bool {
    ffi_guard_bool! {{
        if out_frame_info.is_null() {
            return Err(anyhow::anyhow!(
                "Fs25adTextureRegistrationV4FrameInfo pointer must not be null"
            ));
        }
        ensure_texture_pointer(texture)?;
        fail_not_implemented("fs25ad_host_bridge_texture_registration_v4_acquire")
    }}
}

/// Gibt einen zuvor geleasten v4-Frame wieder frei.
///
/// # Safety
///
/// `texture` muss ein gueltiger, durch `fs25ad_host_bridge_texture_registration_v4_new`
/// erzeugter Zeiger sein.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn fs25ad_host_bridge_texture_registration_v4_release(
    texture: *mut HostBridgeTextureRegistrationV4,
    _frame_token: u64,
) -> bool {
    ffi_guard_bool! {{
        ensure_texture_pointer(texture)?;
        fail_not_implemented("fs25ad_host_bridge_texture_registration_v4_release")
    }}
}

/// Liefert den Windows-Descriptor fuer den aktiven v4-Frame-Lease.
///
/// # Safety
///
/// `texture` muss ein gueltiger Zeiger sein. `out_descriptor` muss ein gueltiger,
/// nicht-null Zeiger sein.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn fs25ad_host_bridge_texture_registration_v4_get_windows_descriptor(
    texture: *mut HostBridgeTextureRegistrationV4,
    _frame_token: u64,
    out_descriptor: *mut Fs25adTextureRegistrationV4WindowsDescriptor,
) -> bool {
    ffi_guard_bool! {{
        if out_descriptor.is_null() {
            return Err(anyhow::anyhow!(
                "Fs25adTextureRegistrationV4WindowsDescriptor pointer must not be null"
            ));
        }
        ensure_texture_pointer(texture)?;
        let _supported_descriptor_kinds = [
            FS25AD_TEXTURE_REGISTRATION_V4_WINDOWS_DESCRIPTOR_DXGI_SHARED_HANDLE,
            FS25AD_TEXTURE_REGISTRATION_V4_WINDOWS_DESCRIPTOR_D3D11_TEXTURE2D,
        ];
        fail_not_implemented(
            "fs25ad_host_bridge_texture_registration_v4_get_windows_descriptor",
        )
    }}
}

/// Liefert den Linux-DMA-BUF-Descriptor fuer den aktiven v4-Frame-Lease.
///
/// # Safety
///
/// `texture` muss ein gueltiger Zeiger sein. `out_descriptor` muss ein gueltiger,
/// nicht-null Zeiger sein.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn fs25ad_host_bridge_texture_registration_v4_get_linux_dmabuf_descriptor(
    texture: *mut HostBridgeTextureRegistrationV4,
    _frame_token: u64,
    out_descriptor: *mut Fs25adTextureRegistrationV4LinuxDmabufDescriptor,
) -> bool {
    ffi_guard_bool! {{
        if out_descriptor.is_null() {
            return Err(anyhow::anyhow!(
                "Fs25adTextureRegistrationV4LinuxDmabufDescriptor pointer must not be null"
            ));
        }
        ensure_texture_pointer(texture)?;
        fail_not_implemented(
            "fs25ad_host_bridge_texture_registration_v4_get_linux_dmabuf_descriptor",
        )
    }}
}

/// Liefert den Android-AHardwareBuffer-Descriptor fuer den aktiven v4-Frame-Lease.
///
/// Der ABI-Vertrag ist bereits eingefroren, aber die aktuelle v4-Runtime ist in
/// diesem Build noch nicht an den produktiven AHardwareBuffer-Exportpfad verdrahtet.
///
/// # Safety
///
/// `texture` muss ein gueltiger Zeiger sein. `out_descriptor` muss ein gueltiger,
/// nicht-null Zeiger sein.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn fs25ad_host_bridge_texture_registration_v4_get_android_hardware_buffer_descriptor(
    texture: *mut HostBridgeTextureRegistrationV4,
    frame_token: u64,
    out_descriptor: *mut Fs25adTextureRegistrationV4AndroidHardwareBufferDescriptor,
) -> bool {
    ffi_guard_bool! {{
        if out_descriptor.is_null() {
            return Err(anyhow::anyhow!(
                "Fs25adTextureRegistrationV4AndroidHardwareBufferDescriptor pointer must not be null"
            ));
        }
        ensure_texture_pointer(texture)?;
        // SAFETY: Zeiger wurde oben auf Nicht-Null validiert.
        unsafe {
            *out_descriptor = android_hardware_buffer_descriptor_to_abi(
                AndroidHardwareBufferDescriptor {
                    hardware_buffer_ptr: 0,
                },
            );
        }
        fail_android_hardware_buffer_descriptor_unavailable(frame_token)
    }}
}

/// Liefert den veralteten Legacy-Android-Surface-Descriptor fuer den aktiven v4-Frame-Lease.
///
/// Dieser ABI-Pfad bleibt nur fuer Alt-Consumer exportiert. Neue Consumer sollen
/// stattdessen den Android-AHardwareBuffer-Descriptorpfad verwenden.
///
/// # Safety
///
/// `texture` muss ein gueltiger Zeiger sein. `out_descriptor` muss ein gueltiger,
/// nicht-null Zeiger sein.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn fs25ad_host_bridge_texture_registration_v4_get_android_surface_descriptor(
    texture: *mut HostBridgeTextureRegistrationV4,
    _frame_token: u64,
    out_descriptor: *mut Fs25adTextureRegistrationV4AndroidSurfaceDescriptor,
) -> bool {
    ffi_guard_bool! {{
        if out_descriptor.is_null() {
            return Err(anyhow::anyhow!(
                "Fs25adTextureRegistrationV4AndroidSurfaceDescriptor pointer must not be null"
            ));
        }
        ensure_texture_pointer(texture)?;
        fail_legacy_android_surface_path(
            "fs25ad_host_bridge_texture_registration_v4_get_android_surface_descriptor",
        )
    }}
}

/// Haengt fuer Android den veralteten Legacy-Surface-Descriptor an den v4-Handle.
///
/// Dieser ABI-Pfad bleibt nur fuer Alt-Consumer exportiert. Neue Consumer sollen
/// stattdessen den ExportLease-AHardwareBuffer-Pfad verwenden.
///
/// # Safety
///
/// `texture` muss ein gueltiger Zeiger sein. `surface_descriptor` muss ein gueltiger,
/// nicht-null Zeiger auf einen gueltigen Descriptor sein.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn fs25ad_host_bridge_texture_registration_v4_attach_android_surface(
    texture: *mut HostBridgeTextureRegistrationV4,
    surface_descriptor: *const Fs25adTextureRegistrationV4AndroidSurfaceDescriptor,
) -> bool {
    ffi_guard_bool! {{
        if surface_descriptor.is_null() {
            return Err(anyhow::anyhow!(
                "Fs25adTextureRegistrationV4AndroidSurfaceDescriptor pointer must not be null"
            ));
        }
        ensure_texture_pointer(texture)?;
        // SAFETY: Aufrufer garantiert gueltigen nicht-null Descriptor-Zeiger.
        let descriptor = unsafe { &*surface_descriptor };
        let _attachment_kind = android_attachment_kind_from_abi(descriptor.attachment_kind)?;
        if descriptor.native_window_ptr == 0 {
            return Err(anyhow!(
                "android surface descriptor native_window_ptr must not be null"
            ));
        }
        fail_legacy_android_surface_path(
            "fs25ad_host_bridge_texture_registration_v4_attach_android_surface",
        )
    }}
}

/// Trennt fuer Android den zuvor attached Legacy-Surface-Descriptor wieder.
///
/// Dieser ABI-Pfad bleibt nur fuer Alt-Consumer exportiert. Neue Consumer sollen
/// stattdessen den ExportLease-AHardwareBuffer-Pfad verwenden.
///
/// # Safety
///
/// `texture` muss ein gueltiger, durch `fs25ad_host_bridge_texture_registration_v4_new`
/// erzeugter Zeiger sein.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn fs25ad_host_bridge_texture_registration_v4_detach_android_surface(
    texture: *mut HostBridgeTextureRegistrationV4,
) -> bool {
    ffi_guard_bool! {{
        ensure_texture_pointer(texture)?;
        fail_legacy_android_surface_path(
            "fs25ad_host_bridge_texture_registration_v4_detach_android_surface",
        )
    }}
}

#[cfg(test)]
mod tests {
    use super::{
        android_hardware_buffer_descriptor_to_abi,
        android_attachment_kind_from_abi, fs25ad_host_bridge_texture_registration_v4_acquire,
        fs25ad_host_bridge_texture_registration_v4_attach_android_surface,
        fs25ad_host_bridge_texture_registration_v4_capabilities,
        fs25ad_host_bridge_texture_registration_v4_contract_version,
        fs25ad_host_bridge_texture_registration_v4_detach_android_surface,
        fs25ad_host_bridge_texture_registration_v4_dispose,
        fs25ad_host_bridge_texture_registration_v4_get_android_hardware_buffer_descriptor,
        fs25ad_host_bridge_texture_registration_v4_get_android_surface_descriptor,
        fs25ad_host_bridge_texture_registration_v4_get_linux_dmabuf_descriptor,
        fs25ad_host_bridge_texture_registration_v4_get_windows_descriptor,
        fs25ad_host_bridge_texture_registration_v4_release,
        fs25ad_host_bridge_texture_registration_v4_render,
        fs25ad_host_bridge_texture_registration_v4_resize,
        Fs25adTextureRegistrationV4AndroidHardwareBufferDescriptor,
        Fs25adTextureRegistrationV4AndroidSurfaceDescriptor,
        Fs25adTextureRegistrationV4Capabilities, Fs25adTextureRegistrationV4FrameInfo,
        Fs25adTextureRegistrationV4LinuxDmabufDescriptor,
        Fs25adTextureRegistrationV4LinuxDmabufPlane,
        Fs25adTextureRegistrationV4PlatformCapabilities,
        Fs25adTextureRegistrationV4WindowsDescriptor, HostBridgeTextureRegistrationV4,
        FS25AD_TEXTURE_REGISTRATION_V4_ALPHA_MODE_PREMULTIPLIED,
        FS25AD_TEXTURE_REGISTRATION_V4_ANDROID_ATTACHMENT_NATIVE_WINDOW,
        FS25AD_TEXTURE_REGISTRATION_V4_ANDROID_ATTACHMENT_SURFACE_PRODUCER,
        FS25AD_TEXTURE_REGISTRATION_V4_AVAILABILITY_NOT_YET_IMPLEMENTED,
        FS25AD_TEXTURE_REGISTRATION_V4_AVAILABILITY_SUPPORTED,
        FS25AD_TEXTURE_REGISTRATION_V4_AVAILABILITY_UNSUPPORTED,
        FS25AD_TEXTURE_REGISTRATION_V4_CONTRACT_VERSION,
        FS25AD_TEXTURE_REGISTRATION_V4_MODEL_EXPORT_LEASE,
        FS25AD_TEXTURE_REGISTRATION_V4_PAYLOAD_ANDROID_HARDWARE_BUFFER,
        FS25AD_TEXTURE_REGISTRATION_V4_PAYLOAD_LINUX_DMABUF,
        FS25AD_TEXTURE_REGISTRATION_V4_PAYLOAD_WINDOWS_DESCRIPTOR,
        FS25AD_TEXTURE_REGISTRATION_V4_PIXEL_FORMAT_RGBA8_SRGB,
        FS25AD_TEXTURE_REGISTRATION_V4_PLATFORM_ANDROID,
        FS25AD_TEXTURE_REGISTRATION_V4_PLATFORM_LINUX,
        FS25AD_TEXTURE_REGISTRATION_V4_PLATFORM_WINDOWS,
        FS25AD_TEXTURE_REGISTRATION_V4_WINDOWS_DESCRIPTOR_D3D11_TEXTURE2D,
        FS25AD_TEXTURE_REGISTRATION_V4_WINDOWS_DESCRIPTOR_DXGI_SHARED_HANDLE,
    };
    use crate::{
        fs25ad_host_bridge_last_error_message, fs25ad_host_bridge_session_dispose,
        fs25ad_host_bridge_session_new, fs25ad_host_bridge_string_free,
    };
    use fs25_auto_drive_render_wgpu::{AndroidAttachmentKind, AndroidHardwareBufferDescriptor};
    use std::ffi::CStr;

    // Sicherheits-Wrapper fuer unsafe FFI-Funktionen im Testkontext.
    fn string_free(ptr: *mut std::ffi::c_char) {
        unsafe { fs25ad_host_bridge_string_free(ptr) }
    }
    fn session_dispose(s: *mut super::HostBridgeSessionHandle) {
        unsafe { fs25ad_host_bridge_session_dispose(s) }
    }
    fn reg_capabilities(out: *mut Fs25adTextureRegistrationV4Capabilities) -> bool {
        unsafe { fs25ad_host_bridge_texture_registration_v4_capabilities(out) }
    }
    fn reg_new(platform: u32, w: u32, h: u32) -> *mut super::HostBridgeTextureRegistrationV4 {
        // _new ist sicher (kein Zeiger-Deref als Input)
        super::fs25ad_host_bridge_texture_registration_v4_new(platform, w, h)
    }
    fn reg_dispose(t: *mut super::HostBridgeTextureRegistrationV4) {
        unsafe { fs25ad_host_bridge_texture_registration_v4_dispose(t) }
    }
    fn reg_resize(t: *mut super::HostBridgeTextureRegistrationV4, w: u32, h: u32) -> bool {
        unsafe { fs25ad_host_bridge_texture_registration_v4_resize(t, w, h) }
    }
    fn reg_render(
        s: *mut super::HostBridgeSessionHandle,
        t: *mut super::HostBridgeTextureRegistrationV4,
    ) -> bool {
        unsafe { fs25ad_host_bridge_texture_registration_v4_render(s, t) }
    }
    fn reg_acquire(
        t: *mut super::HostBridgeTextureRegistrationV4,
        fi: *mut Fs25adTextureRegistrationV4FrameInfo,
    ) -> bool {
        unsafe { fs25ad_host_bridge_texture_registration_v4_acquire(t, fi) }
    }
    fn reg_release(t: *mut super::HostBridgeTextureRegistrationV4, ft: u64) -> bool {
        unsafe { fs25ad_host_bridge_texture_registration_v4_release(t, ft) }
    }
    fn get_windows_descriptor(
        t: *mut super::HostBridgeTextureRegistrationV4,
        ft: u64,
        out: *mut Fs25adTextureRegistrationV4WindowsDescriptor,
    ) -> bool {
        unsafe { fs25ad_host_bridge_texture_registration_v4_get_windows_descriptor(t, ft, out) }
    }
    fn get_linux_dmabuf_descriptor(
        t: *mut super::HostBridgeTextureRegistrationV4,
        ft: u64,
        out: *mut Fs25adTextureRegistrationV4LinuxDmabufDescriptor,
    ) -> bool {
        unsafe {
            fs25ad_host_bridge_texture_registration_v4_get_linux_dmabuf_descriptor(t, ft, out)
        }
    }
    fn get_android_hardware_buffer_descriptor(
        t: *mut super::HostBridgeTextureRegistrationV4,
        ft: u64,
        out: *mut Fs25adTextureRegistrationV4AndroidHardwareBufferDescriptor,
    ) -> bool {
        unsafe {
            fs25ad_host_bridge_texture_registration_v4_get_android_hardware_buffer_descriptor(
                t, ft, out,
            )
        }
    }
    fn get_android_surface_descriptor(
        t: *mut super::HostBridgeTextureRegistrationV4,
        ft: u64,
        out: *mut Fs25adTextureRegistrationV4AndroidSurfaceDescriptor,
    ) -> bool {
        unsafe {
            fs25ad_host_bridge_texture_registration_v4_get_android_surface_descriptor(t, ft, out)
        }
    }
    fn attach_android_surface(
        t: *mut super::HostBridgeTextureRegistrationV4,
        desc: *const Fs25adTextureRegistrationV4AndroidSurfaceDescriptor,
    ) -> bool {
        unsafe { fs25ad_host_bridge_texture_registration_v4_attach_android_surface(t, desc) }
    }
    fn detach_android_surface(t: *mut super::HostBridgeTextureRegistrationV4) -> bool {
        unsafe { fs25ad_host_bridge_texture_registration_v4_detach_android_surface(t) }
    }

    fn read_and_free_error() -> String {
        let ptr = fs25ad_host_bridge_last_error_message();
        assert!(!ptr.is_null());
        let value = unsafe { CStr::from_ptr(ptr) }
            .to_str()
            .expect("error string must be valid UTF-8")
            .to_string();
        string_free(ptr);
        value
    }

    fn empty_platform_capability() -> Fs25adTextureRegistrationV4PlatformCapabilities {
        Fs25adTextureRegistrationV4PlatformCapabilities {
            platform: 0,
            registration_model: 0,
            payload_family: 0,
            availability: 0,
        }
    }

    fn make_dummy_registration() -> *mut HostBridgeTextureRegistrationV4 {
        Box::into_raw(Box::new(HostBridgeTextureRegistrationV4 { _private: () }))
    }

    #[test]
    fn ffi_v4_reports_contract_and_capabilities() {
        assert_eq!(
            fs25ad_host_bridge_texture_registration_v4_contract_version(),
            FS25AD_TEXTURE_REGISTRATION_V4_CONTRACT_VERSION
        );

        let mut capabilities = Fs25adTextureRegistrationV4Capabilities {
            contract_version: 0,
            pixel_format: 0,
            alpha_mode: 0,
            requires_explicit_release: 0,
            windows: empty_platform_capability(),
            linux: empty_platform_capability(),
            android: empty_platform_capability(),
        };

        assert!(reg_capabilities(&mut capabilities));
        assert_eq!(
            capabilities.contract_version,
            FS25AD_TEXTURE_REGISTRATION_V4_CONTRACT_VERSION
        );
        assert_eq!(
            capabilities.pixel_format,
            FS25AD_TEXTURE_REGISTRATION_V4_PIXEL_FORMAT_RGBA8_SRGB
        );
        assert_eq!(
            capabilities.alpha_mode,
            FS25AD_TEXTURE_REGISTRATION_V4_ALPHA_MODE_PREMULTIPLIED
        );
        assert_eq!(capabilities.requires_explicit_release, 1);

        assert_eq!(
            capabilities.windows.platform,
            FS25AD_TEXTURE_REGISTRATION_V4_PLATFORM_WINDOWS
        );
        assert_eq!(
            capabilities.windows.registration_model,
            FS25AD_TEXTURE_REGISTRATION_V4_MODEL_EXPORT_LEASE
        );
        assert_eq!(
            capabilities.windows.payload_family,
            FS25AD_TEXTURE_REGISTRATION_V4_PAYLOAD_WINDOWS_DESCRIPTOR
        );

        assert_eq!(
            capabilities.linux.platform,
            FS25AD_TEXTURE_REGISTRATION_V4_PLATFORM_LINUX
        );
        assert_eq!(
            capabilities.linux.registration_model,
            FS25AD_TEXTURE_REGISTRATION_V4_MODEL_EXPORT_LEASE
        );
        assert_eq!(
            capabilities.linux.payload_family,
            FS25AD_TEXTURE_REGISTRATION_V4_PAYLOAD_LINUX_DMABUF
        );

        assert_eq!(
            capabilities.android.platform,
            FS25AD_TEXTURE_REGISTRATION_V4_PLATFORM_ANDROID
        );
        assert_eq!(
            capabilities.android.registration_model,
            FS25AD_TEXTURE_REGISTRATION_V4_MODEL_EXPORT_LEASE
        );
        assert_eq!(
            capabilities.android.payload_family,
            FS25AD_TEXTURE_REGISTRATION_V4_PAYLOAD_ANDROID_HARDWARE_BUFFER
        );

        if cfg!(target_os = "windows") {
            assert_eq!(
                capabilities.windows.availability,
                FS25AD_TEXTURE_REGISTRATION_V4_AVAILABILITY_NOT_YET_IMPLEMENTED
            );
        } else {
            assert_eq!(
                capabilities.windows.availability,
                FS25AD_TEXTURE_REGISTRATION_V4_AVAILABILITY_UNSUPPORTED
            );
        }

        if cfg!(target_os = "linux") {
            assert_eq!(
                capabilities.linux.availability,
                FS25AD_TEXTURE_REGISTRATION_V4_AVAILABILITY_NOT_YET_IMPLEMENTED
            );
        } else {
            assert_eq!(
                capabilities.linux.availability,
                FS25AD_TEXTURE_REGISTRATION_V4_AVAILABILITY_UNSUPPORTED
            );
        }

        if cfg!(target_os = "android") {
            assert_eq!(
                capabilities.android.availability,
                FS25AD_TEXTURE_REGISTRATION_V4_AVAILABILITY_SUPPORTED
            );
        } else {
            assert_eq!(
                capabilities.android.availability,
                FS25AD_TEXTURE_REGISTRATION_V4_AVAILABILITY_UNSUPPORTED
            );
        }
    }

    #[test]
    fn ffi_v4_new_reports_explicit_platform_blockers() {
        let windows = reg_new(FS25AD_TEXTURE_REGISTRATION_V4_PLATFORM_WINDOWS, 8, 6);
        assert!(windows.is_null());
        let windows_error = read_and_free_error();
        if cfg!(target_os = "windows") {
            assert!(windows_error.contains("not yet implemented"));
        } else {
            assert!(windows_error.contains("unsupported"));
        }

        let linux = reg_new(FS25AD_TEXTURE_REGISTRATION_V4_PLATFORM_LINUX, 8, 6);
        assert!(linux.is_null());
        let linux_error = read_and_free_error();
        if cfg!(target_os = "linux") {
            assert!(linux_error.contains("not yet implemented"));
        } else {
            assert!(linux_error.contains("unsupported"));
        }

        let android = reg_new(FS25AD_TEXTURE_REGISTRATION_V4_PLATFORM_ANDROID, 8, 6);
        assert!(android.is_null());
        let android_error = read_and_free_error();
        if cfg!(target_os = "android") {
            assert!(android_error.contains("currently not wired"));
        } else {
            assert!(android_error.contains("unsupported"));
        }

        let invalid_platform = reg_new(99, 8, 6);
        assert!(invalid_platform.is_null());
        assert!(read_and_free_error().contains("unknown texture registration v4 platform"));
    }

    #[test]
    fn ffi_v4_rejects_invalid_calls_and_null_pointers() {
        assert!(!reg_capabilities(std::ptr::null_mut()));
        assert!(read_and_free_error().contains("Fs25adTextureRegistrationV4Capabilities pointer"));

        assert!(!reg_resize(std::ptr::null_mut(), 8, 6));
        assert!(read_and_free_error().contains("HostBridgeTextureRegistrationV4 pointer"));

        let mut frame = Fs25adTextureRegistrationV4FrameInfo {
            width: 0,
            height: 0,
            pixel_format: 0,
            alpha_mode: 0,
            texture_id: 0,
            texture_generation: 0,
            frame_token: 0,
        };

        assert!(!reg_acquire(std::ptr::null_mut(), &mut frame));
        assert!(read_and_free_error().contains("HostBridgeTextureRegistrationV4 pointer"));

        assert!(!reg_acquire(std::ptr::null_mut(), std::ptr::null_mut()));
        assert!(read_and_free_error().contains("Fs25adTextureRegistrationV4FrameInfo pointer"));

        assert!(!reg_release(std::ptr::null_mut(), 1));
        assert!(read_and_free_error().contains("HostBridgeTextureRegistrationV4 pointer"));

        let mut windows = Fs25adTextureRegistrationV4WindowsDescriptor {
            descriptor_kind: FS25AD_TEXTURE_REGISTRATION_V4_WINDOWS_DESCRIPTOR_D3D11_TEXTURE2D,
            dxgi_shared_handle: 0,
            d3d11_texture_ptr: 0,
            d3d11_device_ptr: 0,
        };
        assert!(!get_windows_descriptor(
            std::ptr::null_mut(),
            1,
            &mut windows
        ));
        assert!(read_and_free_error().contains("HostBridgeTextureRegistrationV4 pointer"));

        let mut linux = Fs25adTextureRegistrationV4LinuxDmabufDescriptor {
            drm_fourcc: 0,
            drm_modifier_hi: 0,
            drm_modifier_lo: 0,
            plane_count: 0,
            planes: [Fs25adTextureRegistrationV4LinuxDmabufPlane {
                fd: -1,
                offset_bytes: 0,
                stride_bytes: 0,
            }; 4],
        };
        assert!(!get_linux_dmabuf_descriptor(
            std::ptr::null_mut(),
            1,
            &mut linux
        ));
        assert!(read_and_free_error().contains("HostBridgeTextureRegistrationV4 pointer"));

        let mut android_ahb = Fs25adTextureRegistrationV4AndroidHardwareBufferDescriptor {
            hardware_buffer_ptr: 0,
        };
        assert!(!get_android_hardware_buffer_descriptor(
            std::ptr::null_mut(),
            1,
            &mut android_ahb
        ));
        assert!(read_and_free_error().contains("HostBridgeTextureRegistrationV4 pointer"));

        let mut android = Fs25adTextureRegistrationV4AndroidSurfaceDescriptor {
            attachment_kind: FS25AD_TEXTURE_REGISTRATION_V4_ANDROID_ATTACHMENT_SURFACE_PRODUCER,
            native_window_ptr: 0,
            surface_handle_ptr: 0,
        };
        assert!(!get_android_surface_descriptor(
            std::ptr::null_mut(),
            1,
            &mut android
        ));
        assert!(read_and_free_error().contains("HostBridgeTextureRegistrationV4 pointer"));

        assert!(!attach_android_surface(std::ptr::null_mut(), &android));
        assert!(read_and_free_error().contains("HostBridgeTextureRegistrationV4 pointer"));

        assert!(!attach_android_surface(
            std::ptr::null_mut(),
            std::ptr::null()
        ));
        assert!(read_and_free_error()
            .contains("Fs25adTextureRegistrationV4AndroidSurfaceDescriptor pointer"));

        assert!(!detach_android_surface(std::ptr::null_mut()));
        assert!(read_and_free_error().contains("HostBridgeTextureRegistrationV4 pointer"));

        reg_dispose(std::ptr::null_mut());
    }

    #[test]
    fn ffi_v4_android_attachment_kind_mapping_is_stable() {
        assert_eq!(
            android_attachment_kind_from_abi(
                FS25AD_TEXTURE_REGISTRATION_V4_ANDROID_ATTACHMENT_NATIVE_WINDOW
            )
            .expect("native window kind must parse"),
            AndroidAttachmentKind::NativeWindow
        );
        assert_eq!(
            android_attachment_kind_from_abi(
                FS25AD_TEXTURE_REGISTRATION_V4_ANDROID_ATTACHMENT_SURFACE_PRODUCER
            )
            .expect("surface producer kind must parse"),
            AndroidAttachmentKind::SurfaceProducer
        );
        assert!(android_attachment_kind_from_abi(999).is_err());
    }

    #[test]
    fn ffi_v4_android_hardware_buffer_descriptor_mapping_is_stable() {
        let descriptor = android_hardware_buffer_descriptor_to_abi(AndroidHardwareBufferDescriptor {
            hardware_buffer_ptr: 0x44,
        });

        assert_eq!(descriptor.hardware_buffer_ptr, 0x44);
    }

    #[test]
    fn ffi_v4_attach_android_surface_validates_native_window_pointer() {
        let registration = make_dummy_registration();

        let invalid_android = Fs25adTextureRegistrationV4AndroidSurfaceDescriptor {
            attachment_kind: FS25AD_TEXTURE_REGISTRATION_V4_ANDROID_ATTACHMENT_SURFACE_PRODUCER,
            native_window_ptr: 0,
            surface_handle_ptr: 0x22,
        };
        assert!(!attach_android_surface(registration, &invalid_android));
        assert!(read_and_free_error().contains("native_window_ptr"));

        reg_dispose(registration);
    }

    #[test]
    fn ffi_v4_windows_and_legacy_android_paths_report_expected_errors() {
        let registration = make_dummy_registration();

        let mut windows = Fs25adTextureRegistrationV4WindowsDescriptor {
            descriptor_kind: FS25AD_TEXTURE_REGISTRATION_V4_WINDOWS_DESCRIPTOR_DXGI_SHARED_HANDLE,
            dxgi_shared_handle: 0x44,
            d3d11_texture_ptr: 0,
            d3d11_device_ptr: 0,
        };
        assert!(!get_windows_descriptor(registration, 1, &mut windows));
        assert!(read_and_free_error().contains("not implemented"));

        let android = Fs25adTextureRegistrationV4AndroidSurfaceDescriptor {
            attachment_kind: FS25AD_TEXTURE_REGISTRATION_V4_ANDROID_ATTACHMENT_SURFACE_PRODUCER,
            native_window_ptr: 0x11,
            surface_handle_ptr: 0x22,
        };
        let mut android_surface = android;
        assert!(!get_android_surface_descriptor(
            registration,
            1,
            &mut android_surface
        ));
        assert!(read_and_free_error().contains("deprecated"));

        assert!(!attach_android_surface(registration, &android));
        assert!(read_and_free_error().contains("deprecated"));

        assert!(!detach_android_surface(registration));
        assert!(read_and_free_error().contains("deprecated"));

        reg_dispose(registration);
    }

    #[test]
    fn ffi_v4_valid_handle_reports_not_implemented_for_remaining_lifecycle_calls() {
        let registration = make_dummy_registration();

        assert!(!reg_resize(registration, 8, 6));
        assert!(read_and_free_error().contains("not implemented"));

        let mut frame = Fs25adTextureRegistrationV4FrameInfo {
            width: 0,
            height: 0,
            pixel_format: 0,
            alpha_mode: 0,
            texture_id: 0,
            texture_generation: 0,
            frame_token: 0,
        };
        assert!(!reg_acquire(registration, &mut frame));
        assert!(read_and_free_error().contains("not implemented"));

        assert!(!reg_release(registration, 1));
        assert!(read_and_free_error().contains("not implemented"));

        reg_dispose(registration);
    }

    #[test]
    fn ffi_v4_valid_handle_reports_expected_errors_for_linux_and_android_payload_getters() {
        let registration = make_dummy_registration();

        let mut linux = Fs25adTextureRegistrationV4LinuxDmabufDescriptor {
            drm_fourcc: 0,
            drm_modifier_hi: 0,
            drm_modifier_lo: 0,
            plane_count: 0,
            planes: [Fs25adTextureRegistrationV4LinuxDmabufPlane {
                fd: -1,
                offset_bytes: 0,
                stride_bytes: 0,
            }; 4],
        };
        assert!(!get_linux_dmabuf_descriptor(registration, 1, &mut linux));
        assert!(read_and_free_error().contains("not implemented"));

        let mut android_ahb = Fs25adTextureRegistrationV4AndroidHardwareBufferDescriptor {
            hardware_buffer_ptr: 0,
        };
        assert!(!get_android_hardware_buffer_descriptor(
            registration,
            1,
            &mut android_ahb
        ));
        let android_ahb_error = read_and_free_error();
        assert!(android_ahb_error.contains("not yet connected"));
        assert!(android_ahb_error.contains("AHardwareBuffer"));

        let mut android = Fs25adTextureRegistrationV4AndroidSurfaceDescriptor {
            attachment_kind: FS25AD_TEXTURE_REGISTRATION_V4_ANDROID_ATTACHMENT_NATIVE_WINDOW,
            native_window_ptr: 0x11,
            surface_handle_ptr: 0,
        };
        assert!(!get_android_surface_descriptor(
            registration,
            1,
            &mut android
        ));
        assert!(read_and_free_error().contains("deprecated"));

        reg_dispose(registration);
    }

    #[test]
    fn ffi_v4_android_hardware_buffer_getter_rejects_null_output_pointer() {
        let registration = make_dummy_registration();

        assert!(!get_android_hardware_buffer_descriptor(
            registration,
            1,
            std::ptr::null_mut()
        ));
        assert!(read_and_free_error()
            .contains("Fs25adTextureRegistrationV4AndroidHardwareBufferDescriptor pointer"));

        reg_dispose(registration);
    }

    #[test]
    fn ffi_v4_render_requires_session_pointer_before_runtime_guard() {
        let registration = make_dummy_registration();

        assert!(!reg_render(std::ptr::null_mut(), registration));
        assert!(read_and_free_error().contains("HostBridgeSession pointer"));

        let session = fs25ad_host_bridge_session_new();
        assert!(!session.is_null());

        assert!(!reg_render(session, registration));
        assert!(read_and_free_error().contains("not implemented"));

        reg_dispose(registration);
        session_dispose(session);
    }
}
