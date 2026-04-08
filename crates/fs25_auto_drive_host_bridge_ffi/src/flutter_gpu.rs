//! Raw C-FFI fuer den Flutter GPU-Runtime-Stack (Linux/Vulkan).
//!
//! Dieses Modul exportiert Low-Level C-Funktionen fuer den GPU-Hot-Path.
//! Alle Funktionen sind Panic-isoliert via [`ffi_guard_bool!`] und [`ffi_guard_ptr!`]
//! aus dem Parent-Modul.
//!
//! # Lebenszyklus
//! ```text
//! fs25ad_gpu_runtime_new()
//!   → fs25ad_gpu_runtime_resize()      // optional bei Groessenaenderung
//!   → fs25ad_gpu_runtime_render()      // pro Frame
//!   → fs25ad_gpu_runtime_export_texture() // pro Frame nach render
//!   → fs25ad_gpu_runtime_dispose()    // am Ende
//! ```
//!
//! # Safety
//! Alle Funktionen sind `extern "C"` und koennen von C/Dart aufgerufen werden.
//! Der Aufrufer ist fuer korrekte Pointer-Gueltigkeit und Lebensdauer verantwortlich.

use crate::flutter_api::FlutterSessionHandle;
use crate::texture_registration_v4::{
    Fs25adTextureRegistrationV4LinuxDmabufDescriptor, Fs25adTextureRegistrationV4LinuxDmabufPlane,
};
use crate::{clear_last_error, set_last_error};
use anyhow::{anyhow, Result};
use fs25_auto_drive_host_bridge::HostBridgeSession;
use fs25_auto_drive_render_wgpu::{
    external_texture::vulkan_linux::VulkanDmaBufTexture, ExternalTextureExport,
    LinuxDmabufDescriptor, LinuxDmabufPlane, SharedTextureRuntime, MAX_LINUX_DMABUF_PLANES,
};
use std::sync::{Arc, Mutex};

/// Interner GPU-Runtime-Zustand fuer Flutter-Integration.
///
/// Kapselt wgpu-Device/Queue, den Renderer-Zustand und die exportierbare Texture.
pub struct GpuRuntimeHandle {
    /// Gehalten damit die Instanz nicht vorzeitig gedroppt wird (Device wuerde orphan).
    _instance: wgpu::Instance,
    /// Gehalten damit Adapter und Device synchron leben.
    _adapter: wgpu::Adapter,
    device: wgpu::Device,
    queue: wgpu::Queue,
    runtime: SharedTextureRuntime,
    external_texture: VulkanDmaBufTexture,
    size: [u32; 2],
    session: Arc<Mutex<HostBridgeSession>>,
}

impl GpuRuntimeHandle {
    #[allow(clippy::arc_with_non_send_sync)] // HostBridgeSession ist !Send, aber FFI-Zugriff ist seriell
    #[allow(clippy::arc_with_non_send_sync)] // HostBridgeSession ist !Send, aber FFI-Zugriff ist seriell
    fn new(width: u32, height: u32) -> Result<Self> {
        Self::new_with_session(
            Arc::new(Mutex::new(HostBridgeSession::new())),
            width,
            height,
        )
    }

    fn new_with_session(
        session: Arc<Mutex<HostBridgeSession>>,
        width: u32,
        height: u32,
    ) -> Result<Self> {
        let instance = fs25_auto_drive_render_wgpu::create_vulkan_instance();
        // HINWEIS: pollster::block_on blockiert den aufrufenden Thread fuer die gesamte
        // GPU-Adapter-Initialisierung. Dies ist einmaliger Init-Code (nicht pro Frame).
        // Empfehlung: fs25ad_gpu_runtime_new() ausschliesslich von einem dedizierten
        // Worker-Thread aufrufen, niemals vom Flutter-Platform-Thread.
        // TODO(flutter-async): Async-Konstruktor oder Thread-Spawn in GpuRuntimeHandle::new().
        let adapter = pollster::block_on(instance.request_adapter(&wgpu::RequestAdapterOptions {
            power_preference: wgpu::PowerPreference::HighPerformance,
            compatible_surface: None,
            force_fallback_adapter: false,
        }))
        .map_err(|e| anyhow!("Kein Vulkan-Adapter gefunden: {e}"))?;

        let (device, queue) =
            pollster::block_on(adapter.request_device(&wgpu::DeviceDescriptor {
                label: Some("fs25ad flutter gpu runtime"),
                required_features: wgpu::Features::empty(),
                required_limits: wgpu::Limits::default(),
                experimental_features: Default::default(),
                memory_hints: wgpu::MemoryHints::Performance,
                trace: wgpu::Trace::Off,
            }))?;

        let runtime = SharedTextureRuntime::new(&device, &queue, [width, height])?;
        let external_texture =
            VulkanDmaBufTexture::create_exportable_texture(&device, width, height)?;

        Ok(Self {
            _instance: instance,
            _adapter: adapter,
            device,
            queue,
            runtime,
            external_texture,
            size: [width, height],
            session,
        })
    }
}

fn with_runtime<T>(
    handle: *mut GpuRuntimeHandle,
    f: impl FnOnce(&mut GpuRuntimeHandle) -> Result<T>,
) -> Result<T> {
    if handle.is_null() {
        return Err(anyhow!("GpuRuntimeHandle pointer must not be null"));
    }
    // SAFETY: Aufrufer garantiert gueltigen, nicht-aliasenden Pointer mit ausreichender Lifetime.
    let runtime = unsafe { &mut *handle };
    f(runtime)
}

fn map_linux_dmabuf_descriptor(
    descriptor: LinuxDmabufDescriptor,
) -> Fs25adTextureRegistrationV4LinuxDmabufDescriptor {
    let mut planes = [Fs25adTextureRegistrationV4LinuxDmabufPlane {
        fd: -1,
        offset_bytes: 0,
        stride_bytes: 0,
    }; MAX_LINUX_DMABUF_PLANES];

    for (index, plane) in descriptor
        .planes
        .iter()
        .take(descriptor.plane_count as usize)
        .enumerate()
    {
        planes[index] = Fs25adTextureRegistrationV4LinuxDmabufPlane {
            fd: plane.fd,
            offset_bytes: plane.offset_bytes,
            stride_bytes: plane.stride_bytes,
        };
    }

    Fs25adTextureRegistrationV4LinuxDmabufDescriptor {
        drm_fourcc: descriptor.drm_fourcc,
        drm_modifier_hi: (descriptor.drm_modifier >> 32) as u32,
        drm_modifier_lo: descriptor.drm_modifier as u32,
        plane_count: descriptor.plane_count,
        planes,
    }
}

/// Erzeugt einen neuen GPU-Runtime-Handle fuer Flutter.
///
/// Gibt einen opaques Handle zurueck der mit `fs25ad_gpu_runtime_dispose` freizugeben ist.
/// Gibt bei Fehler `NULL` zurueck; Fehlertext via `fs25ad_host_bridge_last_error_message`.
#[unsafe(no_mangle)]
pub extern "C" fn fs25ad_gpu_runtime_new(width: u32, height: u32) -> *mut GpuRuntimeHandle {
    clear_last_error();
    match std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
        GpuRuntimeHandle::new(width, height)
    })) {
        Ok(Ok(handle)) => Box::into_raw(Box::new(handle)),
        Ok(Err(e)) => {
            set_last_error(e.to_string());
            std::ptr::null_mut()
        }
        Err(_) => {
            set_last_error("internal panic in fs25ad_gpu_runtime_new");
            std::ptr::null_mut()
        }
    }
}

/// Erzeugt einen GPU-Runtime-Handle, der dieselbe Session wie die Flutter-Control-Plane teilt.
///
/// Gibt einen opaques Handle zurueck der mit `fs25ad_gpu_runtime_dispose` freizugeben ist.
/// Gibt bei Fehler `NULL` zurueck; Fehlertext via `fs25ad_host_bridge_last_error_message`.
///
/// # Safety
/// `session_handle` muss auf einen gueltigen `FlutterSessionHandle` zeigen.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn fs25ad_gpu_runtime_new_with_session(
    session_handle: *const FlutterSessionHandle,
    width: u32,
    height: u32,
) -> *mut GpuRuntimeHandle {
    ffi_guard_ptr!({
        if session_handle.is_null() {
            return Err(anyhow!(
                "fs25ad_gpu_runtime_new_with_session: session_handle must not be null"
            ));
        }

        // SAFETY: Der Aufrufer garantiert einen gueltigen FlutterSessionHandle-Pointer
        // mit ausreichender Lifetime fuer das Klonen des internen Arc-Owners.
        let shared_session = unsafe { (&*session_handle).session_arc() };
        GpuRuntimeHandle::new_with_session(shared_session, width, height)
            .map(|handle| Box::into_raw(Box::new(handle)))
    })
}

/// Rendert den naechsten Frame direkt in die exportierbare Vulkan-Texture.
///
/// Gibt `true` bei Erfolg zurueck, `false` bei Fehler.
#[unsafe(no_mangle)]
pub extern "C" fn fs25ad_gpu_runtime_render(handle: *mut GpuRuntimeHandle) -> bool {
    ffi_guard_bool!({
        with_runtime(handle, |rt| {
            // Lock nur fuer build_render_frame halten, danach sofort freigeben.
            // GPU-Submit laeuft lock-frei, damit Flutter-Isolate-Aufrufe nicht blockieren.
            let frame = {
                let session = rt
                    .session
                    .lock()
                    .map_err(|_| anyhow!("GPU runtime session lock poisoned"))?;
                session.build_render_frame([rt.size[0] as f32, rt.size[1] as f32])
            }; // Lock wird hier freigegeben
            rt.runtime.render_to_view(
                &rt.device,
                &rt.queue,
                &frame.scene,
                &frame.assets,
                rt.external_texture.texture_view(),
            )?;
            Ok(())
        })
    })
}

/// Exportiert den nativen Texture-Deskriptor fuer Flutter/Impeller.
///
/// Schreibt den Linux-DMA-BUF-v4-Descriptor in `out_descriptor`.
/// Gibt `true` bei Erfolg zurueck, `false` bei Fehler.
///
/// # Safety
/// `out_descriptor` muss ein gueltiger, nicht-null Zeiger auf einen
/// `Fs25adTextureRegistrationV4LinuxDmabufDescriptor` sein.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn fs25ad_gpu_runtime_export_texture(
    handle: *mut GpuRuntimeHandle,
    out_descriptor: *mut Fs25adTextureRegistrationV4LinuxDmabufDescriptor,
) -> bool {
    if out_descriptor.is_null() {
        set_last_error("fs25ad_gpu_runtime_export_texture: out_descriptor must not be null");
        return false;
    }
    ffi_guard_bool!({
        with_runtime(handle, |rt| {
            let descriptor = rt.external_texture.export_descriptor()?;
            match descriptor {
                fs25_auto_drive_render_wgpu::PlatformTextureDescriptor::LinuxDmaBuf {
                    fd,
                    stride,
                    format,
                    modifier,
                    ..
                } => {
                    let descriptor = LinuxDmabufDescriptor::single_plane(
                        format,
                        modifier,
                        LinuxDmabufPlane::new(fd, 0, stride),
                    );
                    // SAFETY: Gueltigkeit von out_descriptor wurde oben geprueft.
                    unsafe { *out_descriptor = map_linux_dmabuf_descriptor(descriptor) };
                    Ok(())
                }
            }
        })
    })
}

/// Passt die Groesse des GPU-Runtime-Render-Targets an.
///
/// Gibt `true` bei Erfolg zurueck, `false` bei Fehler.
#[unsafe(no_mangle)]
pub extern "C" fn fs25ad_gpu_runtime_resize(
    handle: *mut GpuRuntimeHandle,
    width: u32,
    height: u32,
) -> bool {
    ffi_guard_bool!({
        with_runtime(handle, |rt| {
            let new_external_texture =
                VulkanDmaBufTexture::create_exportable_texture(&rt.device, width, height)?;
            rt.runtime.resize(&rt.device, [width, height])?;
            rt.external_texture = new_external_texture;
            rt.size = [width, height];
            Ok(())
        })
    })
}

/// Gibt einen GPU-Runtime-Handle frei.
///
/// Nach diesem Aufruf darf `handle` nicht mehr verwendet werden.
///
/// # Safety
/// `handle` muss durch `fs25ad_gpu_runtime_new` alloziert worden sein.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn fs25ad_gpu_runtime_dispose(handle: *mut GpuRuntimeHandle) {
    if !handle.is_null() {
        // SAFETY: handle wurde via Box::into_raw erzeugt und ist hier der Eigentuemer.
        let _ = unsafe { Box::from_raw(handle) };
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// Prueft, dass fs25ad_gpu_runtime_render mit Null-Pointer false zurueckgibt.
    #[test]
    fn test_render_rejects_null_handle() {
        let result = fs25ad_gpu_runtime_render(std::ptr::null_mut());
        assert!(!result, "Null-Pointer muss false zurueckgeben");
    }

    /// Prueft, dass fs25ad_gpu_runtime_resize mit Null-Pointer false zurueckgibt.
    #[test]
    fn test_resize_rejects_null_handle() {
        let result = fs25ad_gpu_runtime_resize(std::ptr::null_mut(), 640, 480);
        assert!(!result, "Null-Pointer muss false zurueckgeben");
    }

    /// Prueft, dass fs25ad_gpu_runtime_new_with_session mit Null-Session null zurueckgibt.
    #[test]
    fn test_new_with_session_rejects_null_handle() {
        // SAFETY: Testaufruf mit Null-Session-Pointer; die Funktion muss den Pointer vor jeder
        // weiteren Nutzung validieren und NULL zurueckgeben.
        let handle = unsafe { fs25ad_gpu_runtime_new_with_session(std::ptr::null(), 640, 480) };
        assert!(handle.is_null(), "Null-Session muss NULL zurueckgeben");
    }

    /// Prueft, dass fs25ad_gpu_runtime_export_texture mit Null-Handle false zurueckgibt.
    #[test]
    fn test_export_texture_rejects_null_handle() {
        let mut descriptor = Fs25adTextureRegistrationV4LinuxDmabufDescriptor {
            drm_fourcc: 0,
            drm_modifier_hi: 0,
            drm_modifier_lo: 0,
            plane_count: 0,
            planes: [Fs25adTextureRegistrationV4LinuxDmabufPlane {
                fd: -1,
                offset_bytes: 0,
                stride_bytes: 0,
            }; MAX_LINUX_DMABUF_PLANES],
        };
        // SAFETY: Testaufruf mit Null-Handle; out_descriptor ist gueltig.
        let result =
            unsafe { fs25ad_gpu_runtime_export_texture(std::ptr::null_mut(), &mut descriptor) };
        assert!(!result, "Null-Handle muss false zurueckgeben");
    }

    /// Prueft, dass fs25ad_gpu_runtime_export_texture mit out_descriptor=null false zurueckgibt.
    #[test]
    fn test_export_texture_rejects_null_out_descriptor() {
        // Null-Handle + Null-out_descriptor: set_last_error wird ausgefuehrt, kein Panic.
        // SAFETY: Testaufruf mit Null-out_descriptor; beide Null-Checks werden geprueft.
        let result = unsafe {
            fs25ad_gpu_runtime_export_texture(std::ptr::null_mut(), std::ptr::null_mut())
        };
        assert!(!result, "Null out_descriptor muss false zurueckgeben");
    }

    /// Prueft die Abbildung eines Single-Plane-DMA-BUF-Descriptors auf den v4-ABI-Typ.
    #[test]
    fn test_map_linux_dmabuf_descriptor_builds_v4_shape() {
        let descriptor = LinuxDmabufDescriptor::single_plane(
            0x3432_4241,
            0x1122_3344_5566_7788,
            LinuxDmabufPlane::new(42, 0, 512),
        );

        let ffi_descriptor = map_linux_dmabuf_descriptor(descriptor);

        assert_eq!(ffi_descriptor.drm_fourcc, 0x3432_4241);
        assert_eq!(ffi_descriptor.drm_modifier_hi, 0x1122_3344);
        assert_eq!(ffi_descriptor.drm_modifier_lo, 0x5566_7788);
        assert_eq!(ffi_descriptor.plane_count, 1);
        assert_eq!(ffi_descriptor.planes[0].fd, 42);
        assert_eq!(ffi_descriptor.planes[0].offset_bytes, 0);
        assert_eq!(ffi_descriptor.planes[0].stride_bytes, 512);
        assert_eq!(ffi_descriptor.planes[1].fd, -1);
    }
}
