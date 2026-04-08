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

use crate::{clear_last_error, set_last_error};
use anyhow::{anyhow, Result};
use fs25_auto_drive_host_bridge::HostBridgeSession;
use fs25_auto_drive_render_wgpu::{
    external_texture::vulkan_linux::VulkanDmaBufTexture, ExternalTextureExport,
    SharedTextureRuntime,
};
use std::sync::Mutex;

/// Interner GPU-Runtime-Zustand fuer Flutter-Integration.
///
/// Kapselt wgpu-Device/Queue, den Renderer und die exportierbare Texture.
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
    session: Mutex<HostBridgeSession>,
}

impl GpuRuntimeHandle {
    fn new(width: u32, height: u32) -> Result<Self> {
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
            session: Mutex::new(HostBridgeSession::new()),
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

/// Rendert den naechsten Frame in die interne Shared-Texture.
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
            rt.runtime
                .render_frame(&rt.device, &rt.queue, &frame.scene, &frame.assets)?;
            Ok(())
        })
    })
}

/// Exportiert den nativen Texture-Deskriptor fuer Flutter/Impeller.
///
/// Schreibt den DMA-BUF File-Descriptor in `out_fd`.
/// Gibt `true` bei Erfolg zurueck, `false` bei Fehler.
///
/// # Safety
/// `out_fd` muss ein gueltiger, nicht-null Zeiger auf ein `i32` sein.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn fs25ad_gpu_runtime_export_texture(
    handle: *mut GpuRuntimeHandle,
    out_fd: *mut i32,
) -> bool {
    if out_fd.is_null() {
        set_last_error("fs25ad_gpu_runtime_export_texture: out_fd must not be null");
        return false;
    }
    ffi_guard_bool!({
        with_runtime(handle, |rt| {
            let descriptor = rt.external_texture.export_descriptor()?;
            match descriptor {
                fs25_auto_drive_render_wgpu::PlatformTextureDescriptor::LinuxDmaBuf {
                    fd, ..
                } => {
                    // SAFETY: Gueltigkeit von out_fd wurde oben geprueft.
                    unsafe { *out_fd = fd };
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
            rt.runtime.resize(&rt.device, [width, height])?;
            rt.external_texture.resize(&rt.device, width, height)?;
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

    /// Prueft, dass fs25ad_gpu_runtime_export_texture mit Null-Handle false zurueckgibt.
    #[test]
    fn test_export_texture_rejects_null_handle() {
        let mut fd: i32 = 0;
        // SAFETY: Testaufruf mit Null-Handle; out_fd ist gueltig.
        let result = unsafe { fs25ad_gpu_runtime_export_texture(std::ptr::null_mut(), &mut fd) };
        assert!(!result, "Null-Handle muss false zurueckgeben");
    }

    /// Prueft, dass fs25ad_gpu_runtime_export_texture mit out_fd=null false zurueckgibt.
    #[test]
    fn test_export_texture_rejects_null_out_fd() {
        // Null-Handle + Null-out_fd: set_last_error wird ausgefuehrt, kein Panic.
        // SAFETY: Testaufruf mit Null-out_fd; beide Null-Checks werden geprueft.
        let result = unsafe {
            fs25ad_gpu_runtime_export_texture(std::ptr::null_mut(), std::ptr::null_mut())
        };
        assert!(!result, "Null out_fd muss false zurueckgeben");
    }
}
