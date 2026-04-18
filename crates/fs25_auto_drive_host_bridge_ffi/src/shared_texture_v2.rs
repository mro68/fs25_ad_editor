//! Nativer Shared-Texture-Adapter ueber der kanonischen Host-Bridge-Session.

use crate::{clear_last_error, set_last_error, with_session_mut, HostBridgeSessionHandle};
use anyhow::{anyhow, Result};
use fs25_auto_drive_render_wgpu::{
    SharedTextureAlphaMode, SharedTextureFrame, SharedTextureNativeHandle,
    SharedTexturePixelFormat, SharedTextureRuntime,
};
use std::sync::Mutex;

const FS25AD_SHARED_TEXTURE_CONTRACT_VERSION: u32 = 3;
const FS25AD_SHARED_TEXTURE_PIXEL_FORMAT_RGBA8_SRGB: u32 = 1;
const FS25AD_SHARED_TEXTURE_ALPHA_MODE_PREMULTIPLIED: u32 = 1;
const FS25AD_SHARED_TEXTURE_NATIVE_HANDLE_KIND_OPAQUE_RUNTIME_POINTERS: u32 = 1;

/// Interner Zustand eines Shared-Texture-Handles.
struct HostBridgeSharedTextureState {
    _instance: wgpu::Instance,
    _adapter: wgpu::Adapter,
    device: wgpu::Device,
    queue: wgpu::Queue,
    runtime: SharedTextureRuntime,
    size: [u32; 2],
}

impl HostBridgeSharedTextureState {
    fn new(width: u32, height: u32) -> Result<Self> {
        if width == 0 || height == 0 {
            return Err(anyhow!(
                "shared texture size must be positive, got {}x{}",
                width,
                height
            ));
        }
        let instance = wgpu::Instance::default();
        Self::new_with_instance(instance, width, height)
    }

    /// Erzeugt den State mit einer explizit bereitgestellten wgpu-Instanz.
    ///
    /// Wird von der Flutter-Vulkan-Integration genutzt um eine Vulkan-exklusive
    /// Instanz zu erzwingen und GPU-Sharing mit Impeller zu ermoeglichen.
    ///
    /// TODO(flutter-wiring): Wird aufgerufen sobald der Flutter-GPU-Pfad vollstaendig
    /// mit `flutter_gpu.rs` verbunden ist.
    #[cfg(any(feature = "flutter-linux", feature = "flutter-android"))]
    #[allow(dead_code)]
    fn new_for_flutter(width: u32, height: u32) -> Result<Self> {
        let instance = fs25_auto_drive_render_wgpu::create_vulkan_instance();
        Self::new_with_instance(instance, width, height)
    }

    fn new_with_instance(instance: wgpu::Instance, width: u32, height: u32) -> Result<Self> {
        let adapter = pollster::block_on(instance.request_adapter(&wgpu::RequestAdapterOptions {
            power_preference: wgpu::PowerPreference::LowPower,
            compatible_surface: None,
            force_fallback_adapter: false,
        }))?;

        let (device, queue) =
            pollster::block_on(adapter.request_device(&wgpu::DeviceDescriptor {
                label: Some("fs25ad shared texture v2 device"),
                required_features: wgpu::Features::empty(),
                required_limits: wgpu::Limits::downlevel_defaults(),
                experimental_features: Default::default(),
                memory_hints: wgpu::MemoryHints::Performance,
                trace: wgpu::Trace::Off,
            }))?;
        let runtime = SharedTextureRuntime::new(&device, &queue, [width, height])?;

        Ok(Self {
            _instance: instance,
            _adapter: adapter,
            device,
            queue,
            runtime,
            size: [width, height],
        })
    }

    fn resize(&mut self, width: u32, height: u32) -> Result<()> {
        self.runtime.resize(&self.device, [width, height])?;
        self.size = [width, height];
        Ok(())
    }

    fn render(
        &mut self,
        session: &mut fs25_auto_drive_host_bridge::HostBridgeSession,
    ) -> Result<()> {
        let frame = session.build_render_frame(self.viewport_size_f32());
        self.runtime
            .render_frame(&self.device, &self.queue, &frame.scene, &frame.assets)?;
        Ok(())
    }

    fn acquire(&mut self) -> Result<(SharedTextureFrame, SharedTextureNativeHandle)> {
        let frame = self.runtime.acquire_frame()?;
        let handle = self.runtime.native_handle(frame.frame_token)?;
        Ok((frame, handle))
    }

    fn release(&mut self, frame_token: u64) -> Result<()> {
        self.runtime.release_frame(frame_token)?;
        Ok(())
    }

    fn viewport_size_f32(&self) -> [f32; 2] {
        [self.size[0] as f32, self.size[1] as f32]
    }
}

/// Opaquer Shared-Texture-Handle mit serialisiertem Zugriff.
pub(crate) struct HostBridgeSharedTexture {
    state: Mutex<HostBridgeSharedTextureState>,
}

impl HostBridgeSharedTexture {
    fn new(width: u32, height: u32) -> Result<Self> {
        let state = HostBridgeSharedTextureState::new(width, height)?;
        Ok(Self {
            state: Mutex::new(state),
        })
    }

    fn with_lock<T>(
        &self,
        f: impl FnOnce(&mut HostBridgeSharedTextureState) -> Result<T>,
    ) -> Result<T> {
        let mut guard = self
            .state
            .lock()
            .map_err(|_| anyhow!("HostBridgeSharedTexture lock poisoned"))?;
        f(&mut guard)
    }
}

#[repr(C)]
/// Laufzeitfaehigkeiten fuer den Shared-Texture-Vertrag v3.
pub struct Fs25adSharedTextureCapabilities {
    /// ABI-Konstante fuer das Pixel-Format (`1 = RGBA8 sRGB`).
    pub pixel_format: u32,
    /// ABI-Konstante fuer den Alpha-Modus (`1 = premultiplied`).
    pub alpha_mode: u32,
    /// ABI-Konstante fuer die Handle-Art (`1 = opaque runtime pointers`).
    pub native_handle_kind: u32,
    /// `1` signalisiert, dass Acquire/Release explizit vom Host eingehalten werden muss.
    pub requires_explicit_release: u32,
}

#[repr(C)]
/// Metadaten eines geleasten Shared-Texture-Frames.
pub struct Fs25adSharedTextureFrameInfo {
    /// Frame-Breite in Pixeln.
    pub width: u32,
    /// Frame-Hoehe in Pixeln.
    pub height: u32,
    /// ABI-Konstante fuer das Pixel-Format (`1 = RGBA8 sRGB`).
    pub pixel_format: u32,
    /// ABI-Konstante fuer den Alpha-Modus (`1 = premultiplied`).
    pub alpha_mode: u32,
    /// Runtime-ID der zugrundeliegenden GPU-Textur.
    pub texture_id: u64,
    /// Generation der GPU-Textur.
    pub texture_generation: u64,
    /// Lease-Token fuer den Acquire/Release-Lifecycle.
    pub frame_token: u64,
}

#[repr(C)]
/// Opaque Runtime-Pointerwerte fuer denselben Prozessraum.
///
/// Das sind keine backend-nativen Vulkan-/Metal-/DX-Interop-Handles.
pub struct Fs25adSharedTextureNativeHandle {
    /// Adresse der internen `wgpu::Texture` als `uintptr_t` (nur same-process gueltig).
    pub texture_ptr: usize,
    /// Adresse der internen `wgpu::TextureView` als `uintptr_t` (nur same-process gueltig).
    pub texture_view_ptr: usize,
}

fn pixel_format_abi(format: SharedTexturePixelFormat) -> u32 {
    match format {
        SharedTexturePixelFormat::Rgba8Srgb => FS25AD_SHARED_TEXTURE_PIXEL_FORMAT_RGBA8_SRGB,
    }
}

fn alpha_mode_abi(mode: SharedTextureAlphaMode) -> u32 {
    match mode {
        SharedTextureAlphaMode::Premultiplied => FS25AD_SHARED_TEXTURE_ALPHA_MODE_PREMULTIPLIED,
    }
}

fn with_shared_texture_mut<T>(
    texture: *mut HostBridgeSharedTexture,
    f: impl FnOnce(&mut HostBridgeSharedTextureState) -> Result<T>,
) -> Result<T> {
    if texture.is_null() {
        return Err(anyhow!("HostBridgeSharedTexture pointer must not be null"));
    }

    let texture = unsafe { &*texture };
    texture.with_lock(f)
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

/// Liefert die Version des aktuellen Shared-Texture-Vertrags.
#[unsafe(no_mangle)]
pub extern "C" fn fs25ad_host_bridge_shared_texture_contract_version() -> u32 {
    FS25AD_SHARED_TEXTURE_CONTRACT_VERSION
}

/// Liefert die Runtime-Capabilities des Shared-Texture-Pfads.
///
/// # Safety
///
/// `out_capabilities` muss ein gueltiger, nicht-null Zeiger auf eine initialisierbare
/// `Fs25adSharedTextureCapabilities`-Struktur sein.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn fs25ad_host_bridge_shared_texture_capabilities(
    out_capabilities: *mut Fs25adSharedTextureCapabilities,
) -> bool {
    ffi_guard_bool! {{
        if out_capabilities.is_null() {
            return Err(anyhow::anyhow!("Fs25adSharedTextureCapabilities pointer must not be null"));
        }
        // SAFETY: Aufrufer garantiert gueltigen nicht-null Zeiger.
        unsafe {
            *out_capabilities = Fs25adSharedTextureCapabilities {
                pixel_format: FS25AD_SHARED_TEXTURE_PIXEL_FORMAT_RGBA8_SRGB,
                alpha_mode: FS25AD_SHARED_TEXTURE_ALPHA_MODE_PREMULTIPLIED,
                native_handle_kind:
                    FS25AD_SHARED_TEXTURE_NATIVE_HANDLE_KIND_OPAQUE_RUNTIME_POINTERS,
                requires_explicit_release: 1,
            };
        }
        Ok(())
    }}
}

/// Erstellt einen nativen Shared-Texture-Handle.
#[unsafe(no_mangle)]
pub extern "C" fn fs25ad_host_bridge_shared_texture_new(
    width: u32,
    height: u32,
) -> *mut HostBridgeSharedTexture {
    clear_last_error();

    match HostBridgeSharedTexture::new(width, height) {
        Ok(texture) => Box::into_raw(Box::new(texture)),
        Err(error) => {
            set_last_error(error.to_string());
            std::ptr::null_mut()
        }
    }
}

/// Gibt einen zuvor erstellten Shared-Texture-Handle frei.
///
/// # Safety
///
/// `texture` muss ein durch `fs25ad_host_bridge_shared_texture_new` erzeugter Zeiger sein oder `null`.
/// Nach dem Aufruf ist der Zeiger ungueltig.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn fs25ad_host_bridge_shared_texture_dispose(
    texture: *mut HostBridgeSharedTexture,
) {
    clear_last_error();
    if texture.is_null() {
        return;
    }
    // SAFETY: Aufrufer garantiert durch _new allokierten Zeiger.
    unsafe { drop(Box::from_raw(texture)) };
}

/// Aendert die Zielgroesse eines Shared-Texture-Handles.
///
/// # Safety
///
/// `texture` muss ein gueltiger, durch `fs25ad_host_bridge_shared_texture_new` erzeugter Zeiger sein.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn fs25ad_host_bridge_shared_texture_resize(
    texture: *mut HostBridgeSharedTexture,
    width: u32,
    height: u32,
) -> bool {
    ffi_guard_bool! {
        with_shared_texture_mut(texture, |texture| texture.resize(width, height))
    }
}

/// Rendert den aktuellen Session-Frame in die Shared-Texture.
///
/// # Safety
///
/// `session` und `texture` muessen gueltige, durch die jeweiligen `_new`-Funktionen
/// erzeugte Zeiger sein.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn fs25ad_host_bridge_shared_texture_render(
    session: *mut HostBridgeSessionHandle,
    texture: *mut HostBridgeSharedTexture,
) -> bool {
    ffi_guard_bool! {
        with_session_mut(session, |session| {
            with_shared_texture_mut(texture, |texture| texture.render(session))
        })
    }
}

/// Leased den zuletzt gerenderten Shared-Texture-Frame inklusive nativer Handles.
///
/// # Safety
///
/// `texture` muss ein gueltiger Zeiger sein. `out_frame_info` und `out_native_handle`
/// muessen gueltige, nicht-null Zeiger auf initialisierbare Strukturen sein.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn fs25ad_host_bridge_shared_texture_acquire(
    texture: *mut HostBridgeSharedTexture,
    out_frame_info: *mut Fs25adSharedTextureFrameInfo,
    out_native_handle: *mut Fs25adSharedTextureNativeHandle,
) -> bool {
    ffi_guard_bool! {{
        if out_frame_info.is_null() {
            return Err(anyhow::anyhow!("Fs25adSharedTextureFrameInfo pointer must not be null"));
        }
        if out_native_handle.is_null() {
            return Err(anyhow::anyhow!(
                "Fs25adSharedTextureNativeHandle pointer must not be null"
            ));
        }
        let (frame, native_handle) =
            with_shared_texture_mut(texture, |texture| texture.acquire())?;
        // SAFETY: Aufrufer hat nicht-null Zeiger garantiert.
        unsafe {
            *out_frame_info = Fs25adSharedTextureFrameInfo {
                width: frame.width,
                height: frame.height,
                pixel_format: pixel_format_abi(frame.pixel_format),
                alpha_mode: alpha_mode_abi(frame.alpha_mode),
                texture_id: frame.texture_id,
                texture_generation: frame.texture_generation,
                frame_token: frame.frame_token,
            };
            *out_native_handle = Fs25adSharedTextureNativeHandle {
                texture_ptr: native_handle.texture_ptr,
                texture_view_ptr: native_handle.texture_view_ptr,
            };
        }
        Ok(())
    }}
}

/// Gibt einen zuvor geleasten Shared-Texture-Frame wieder frei.
///
/// # Safety
///
/// `texture` muss ein gueltiger, durch `fs25ad_host_bridge_shared_texture_new` erzeugter Zeiger sein.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn fs25ad_host_bridge_shared_texture_release(
    texture: *mut HostBridgeSharedTexture,
    frame_token: u64,
) -> bool {
    ffi_guard_bool! {
        with_shared_texture_mut(texture, |texture| texture.release(frame_token))
    }
}

#[cfg(test)]
mod tests {
    use super::{
        fs25ad_host_bridge_shared_texture_acquire, fs25ad_host_bridge_shared_texture_capabilities,
        fs25ad_host_bridge_shared_texture_contract_version,
        fs25ad_host_bridge_shared_texture_dispose, fs25ad_host_bridge_shared_texture_new,
        fs25ad_host_bridge_shared_texture_release, fs25ad_host_bridge_shared_texture_render,
        fs25ad_host_bridge_shared_texture_resize, Fs25adSharedTextureCapabilities,
        Fs25adSharedTextureFrameInfo, Fs25adSharedTextureNativeHandle,
        FS25AD_SHARED_TEXTURE_ALPHA_MODE_PREMULTIPLIED, FS25AD_SHARED_TEXTURE_CONTRACT_VERSION,
        FS25AD_SHARED_TEXTURE_NATIVE_HANDLE_KIND_OPAQUE_RUNTIME_POINTERS,
        FS25AD_SHARED_TEXTURE_PIXEL_FORMAT_RGBA8_SRGB,
    };
    use crate::{
        fs25ad_host_bridge_last_error_message, fs25ad_host_bridge_session_dispose,
        fs25ad_host_bridge_session_new, fs25ad_host_bridge_string_free,
    };
    use std::ffi::CStr;

    // Sicherheits-Wrapper fuer unsafe FFI-Funktionen im Testkontext.
    fn string_free(ptr: *mut std::ffi::c_char) {
        unsafe { fs25ad_host_bridge_string_free(ptr) }
    }
    fn session_dispose(s: *mut super::HostBridgeSessionHandle) {
        unsafe { fs25ad_host_bridge_session_dispose(s) }
    }
    fn shared_texture_capabilities(out: *mut Fs25adSharedTextureCapabilities) -> bool {
        unsafe { fs25ad_host_bridge_shared_texture_capabilities(out) }
    }
    fn shared_texture_dispose(t: *mut super::HostBridgeSharedTexture) {
        unsafe { fs25ad_host_bridge_shared_texture_dispose(t) }
    }
    fn shared_texture_resize(t: *mut super::HostBridgeSharedTexture, w: u32, h: u32) -> bool {
        unsafe { fs25ad_host_bridge_shared_texture_resize(t, w, h) }
    }
    fn shared_texture_render(
        s: *mut super::HostBridgeSessionHandle,
        t: *mut super::HostBridgeSharedTexture,
    ) -> bool {
        unsafe { fs25ad_host_bridge_shared_texture_render(s, t) }
    }
    fn shared_texture_acquire(
        t: *mut super::HostBridgeSharedTexture,
        fi: *mut Fs25adSharedTextureFrameInfo,
        nh: *mut Fs25adSharedTextureNativeHandle,
    ) -> bool {
        unsafe { fs25ad_host_bridge_shared_texture_acquire(t, fi, nh) }
    }
    fn shared_texture_release(t: *mut super::HostBridgeSharedTexture, ft: u64) -> bool {
        unsafe { fs25ad_host_bridge_shared_texture_release(t, ft) }
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

    fn is_headless_adapter_error(error: &str) -> bool {
        let lower = error.to_ascii_lowercase();
        lower.contains("no suitable gpu adapters")
            || lower.contains("no adapters found")
            || lower.contains("request adapter")
            || lower.contains("request_adapter")
            || lower.contains("parent device is lost")
            || lower.contains("adapter")
    }

    fn require_shared_texture_or_skip(
        width: u32,
        height: u32,
    ) -> Option<*mut super::HostBridgeSharedTexture> {
        let texture = fs25ad_host_bridge_shared_texture_new(width, height);
        if texture.is_null() {
            let error = read_and_free_error();
            if is_headless_adapter_error(&error) {
                return None;
            }
            panic!("shared texture creation failed unexpectedly: {error}");
        }
        Some(texture)
    }

    #[test]
    fn ffi_shared_texture_reports_contract_and_capabilities() {
        assert_eq!(
            fs25ad_host_bridge_shared_texture_contract_version(),
            FS25AD_SHARED_TEXTURE_CONTRACT_VERSION
        );
        assert_eq!(fs25ad_host_bridge_shared_texture_contract_version(), 3);

        let mut capabilities = Fs25adSharedTextureCapabilities {
            pixel_format: 0,
            alpha_mode: 0,
            native_handle_kind: 0,
            requires_explicit_release: 0,
        };
        assert!(shared_texture_capabilities(&mut capabilities));
        assert_eq!(
            capabilities.pixel_format,
            FS25AD_SHARED_TEXTURE_PIXEL_FORMAT_RGBA8_SRGB
        );
        assert_eq!(
            capabilities.alpha_mode,
            FS25AD_SHARED_TEXTURE_ALPHA_MODE_PREMULTIPLIED
        );
        assert_eq!(
            capabilities.native_handle_kind,
            FS25AD_SHARED_TEXTURE_NATIVE_HANDLE_KIND_OPAQUE_RUNTIME_POINTERS
        );
        assert_eq!(capabilities.requires_explicit_release, 1);
    }

    #[test]
    fn ffi_shared_texture_render_acquire_release_lifecycle() {
        let session = fs25ad_host_bridge_session_new();
        assert!(!session.is_null());

        let Some(texture) = require_shared_texture_or_skip(8, 6) else {
            session_dispose(session);
            return;
        };

        assert!(shared_texture_render(session, texture));

        let mut frame_info = Fs25adSharedTextureFrameInfo {
            width: 0,
            height: 0,
            pixel_format: 0,
            alpha_mode: 0,
            texture_id: 0,
            texture_generation: 0,
            frame_token: 0,
        };
        let mut native_handle = Fs25adSharedTextureNativeHandle {
            texture_ptr: 0,
            texture_view_ptr: 0,
        };
        assert!(shared_texture_acquire(
            texture,
            &mut frame_info,
            &mut native_handle,
        ));

        assert_eq!(frame_info.width, 8);
        assert_eq!(frame_info.height, 6);
        assert_eq!(
            frame_info.pixel_format,
            FS25AD_SHARED_TEXTURE_PIXEL_FORMAT_RGBA8_SRGB
        );
        assert_eq!(
            frame_info.alpha_mode,
            FS25AD_SHARED_TEXTURE_ALPHA_MODE_PREMULTIPLIED
        );
        assert!(frame_info.texture_id > 0);
        assert!(frame_info.texture_generation > 0);
        assert!(frame_info.frame_token > 0);
        assert!(native_handle.texture_ptr > 0);
        assert!(native_handle.texture_view_ptr > 0);

        assert!(shared_texture_release(texture, frame_info.frame_token));

        shared_texture_dispose(texture);
        session_dispose(session);
    }

    #[test]
    fn ffi_shared_texture_rejects_render_while_frame_is_leased() {
        let session = fs25ad_host_bridge_session_new();
        assert!(!session.is_null());

        let Some(texture) = require_shared_texture_or_skip(8, 6) else {
            session_dispose(session);
            return;
        };

        assert!(shared_texture_render(session, texture));

        let mut frame_info = Fs25adSharedTextureFrameInfo {
            width: 0,
            height: 0,
            pixel_format: 0,
            alpha_mode: 0,
            texture_id: 0,
            texture_generation: 0,
            frame_token: 0,
        };
        let mut native_handle = Fs25adSharedTextureNativeHandle {
            texture_ptr: 0,
            texture_view_ptr: 0,
        };
        assert!(shared_texture_acquire(
            texture,
            &mut frame_info,
            &mut native_handle,
        ));

        assert!(!shared_texture_render(session, texture));
        let error = read_and_free_error();
        assert!(error.contains("currently acquired"));

        assert!(shared_texture_release(texture, frame_info.frame_token));
        assert!(shared_texture_render(session, texture));

        shared_texture_dispose(texture);
        session_dispose(session);
    }

    #[test]
    fn ffi_shared_texture_rejects_acquire_before_first_render() {
        let Some(texture) = require_shared_texture_or_skip(8, 6) else {
            return;
        };

        let mut frame_info = Fs25adSharedTextureFrameInfo {
            width: 0,
            height: 0,
            pixel_format: 0,
            alpha_mode: 0,
            texture_id: 0,
            texture_generation: 0,
            frame_token: 0,
        };
        let mut native_handle = Fs25adSharedTextureNativeHandle {
            texture_ptr: 0,
            texture_view_ptr: 0,
        };

        assert!(!shared_texture_acquire(
            texture,
            &mut frame_info,
            &mut native_handle,
        ));
        assert!(read_and_free_error().contains("no rendered frame yet"));

        shared_texture_dispose(texture);
    }

    #[test]
    fn ffi_shared_texture_rejects_double_acquire() {
        let session = fs25ad_host_bridge_session_new();
        assert!(!session.is_null());

        let Some(texture) = require_shared_texture_or_skip(8, 6) else {
            session_dispose(session);
            return;
        };
        assert!(shared_texture_render(session, texture));

        let mut first_frame = Fs25adSharedTextureFrameInfo {
            width: 0,
            height: 0,
            pixel_format: 0,
            alpha_mode: 0,
            texture_id: 0,
            texture_generation: 0,
            frame_token: 0,
        };
        let mut first_handle = Fs25adSharedTextureNativeHandle {
            texture_ptr: 0,
            texture_view_ptr: 0,
        };
        assert!(shared_texture_acquire(
            texture,
            &mut first_frame,
            &mut first_handle,
        ));

        let mut second_frame = Fs25adSharedTextureFrameInfo {
            width: 0,
            height: 0,
            pixel_format: 0,
            alpha_mode: 0,
            texture_id: 0,
            texture_generation: 0,
            frame_token: 0,
        };
        let mut second_handle = Fs25adSharedTextureNativeHandle {
            texture_ptr: 0,
            texture_view_ptr: 0,
        };
        assert!(!shared_texture_acquire(
            texture,
            &mut second_frame,
            &mut second_handle,
        ));
        assert!(read_and_free_error().contains("already acquired"));

        assert!(shared_texture_release(texture, first_frame.frame_token));

        shared_texture_dispose(texture);
        session_dispose(session);
    }

    #[test]
    fn ffi_shared_texture_rejects_resize_while_frame_is_leased() {
        let session = fs25ad_host_bridge_session_new();
        assert!(!session.is_null());

        let Some(texture) = require_shared_texture_or_skip(8, 6) else {
            session_dispose(session);
            return;
        };
        assert!(shared_texture_render(session, texture));

        let mut frame_info = Fs25adSharedTextureFrameInfo {
            width: 0,
            height: 0,
            pixel_format: 0,
            alpha_mode: 0,
            texture_id: 0,
            texture_generation: 0,
            frame_token: 0,
        };
        let mut native_handle = Fs25adSharedTextureNativeHandle {
            texture_ptr: 0,
            texture_view_ptr: 0,
        };
        assert!(shared_texture_acquire(
            texture,
            &mut frame_info,
            &mut native_handle,
        ));

        assert!(!shared_texture_resize(texture, 10, 8));
        assert!(read_and_free_error().contains("currently acquired"));

        assert!(shared_texture_release(texture, frame_info.frame_token));

        shared_texture_dispose(texture);
        session_dispose(session);
    }

    #[test]
    fn ffi_shared_texture_rejects_release_with_wrong_token_while_leased() {
        let session = fs25ad_host_bridge_session_new();
        assert!(!session.is_null());

        let Some(texture) = require_shared_texture_or_skip(8, 6) else {
            session_dispose(session);
            return;
        };
        assert!(shared_texture_render(session, texture));

        let mut frame_info = Fs25adSharedTextureFrameInfo {
            width: 0,
            height: 0,
            pixel_format: 0,
            alpha_mode: 0,
            texture_id: 0,
            texture_generation: 0,
            frame_token: 0,
        };
        let mut native_handle = Fs25adSharedTextureNativeHandle {
            texture_ptr: 0,
            texture_view_ptr: 0,
        };
        assert!(shared_texture_acquire(
            texture,
            &mut frame_info,
            &mut native_handle,
        ));

        assert!(!shared_texture_release(texture, frame_info.frame_token + 1));
        assert!(read_and_free_error().contains("token mismatch"));

        assert!(shared_texture_release(texture, frame_info.frame_token));

        shared_texture_dispose(texture);
        session_dispose(session);
    }

    #[test]
    fn ffi_shared_texture_reports_invalid_calls_and_sizes() {
        let capabilities = Fs25adSharedTextureCapabilities {
            pixel_format: 0,
            alpha_mode: 0,
            native_handle_kind: 0,
            requires_explicit_release: 0,
        };
        assert!(!shared_texture_capabilities(std::ptr::null_mut()));
        assert!(read_and_free_error().contains("Fs25adSharedTextureCapabilities pointer"));

        let texture = fs25ad_host_bridge_shared_texture_new(0, 4);
        assert!(texture.is_null());
        assert!(read_and_free_error().contains("must be positive"));

        let Some(texture) = require_shared_texture_or_skip(4, 4) else {
            return;
        };

        let session = fs25ad_host_bridge_session_new();
        assert!(!session.is_null());

        assert!(!shared_texture_render(std::ptr::null_mut(), texture));
        assert!(read_and_free_error().contains("HostBridgeSession pointer"));

        assert!(!shared_texture_render(session, std::ptr::null_mut()));
        assert!(read_and_free_error().contains("HostBridgeSharedTexture pointer"));

        assert!(!shared_texture_resize(std::ptr::null_mut(), 4, 4));
        assert!(read_and_free_error().contains("HostBridgeSharedTexture pointer"));

        assert!(!shared_texture_resize(texture, 0, 4));
        assert!(read_and_free_error().contains("must be positive"));

        let mut frame_info = Fs25adSharedTextureFrameInfo {
            width: 0,
            height: 0,
            pixel_format: 0,
            alpha_mode: 0,
            texture_id: 0,
            texture_generation: 0,
            frame_token: 0,
        };
        let mut native_handle = Fs25adSharedTextureNativeHandle {
            texture_ptr: 0,
            texture_view_ptr: 0,
        };
        assert!(!shared_texture_acquire(
            std::ptr::null_mut(),
            &mut frame_info,
            &mut native_handle,
        ));
        assert!(read_and_free_error().contains("HostBridgeSharedTexture pointer"));

        assert!(!shared_texture_acquire(
            texture,
            std::ptr::null_mut(),
            &mut native_handle,
        ));
        assert!(read_and_free_error().contains("Fs25adSharedTextureFrameInfo pointer"));

        assert!(!shared_texture_acquire(
            texture,
            &mut frame_info,
            std::ptr::null_mut(),
        ));
        assert!(read_and_free_error().contains("Fs25adSharedTextureNativeHandle pointer"));

        assert!(!shared_texture_release(texture, 1));
        assert!(read_and_free_error().contains("is not acquired"));

        assert!(!shared_texture_release(std::ptr::null_mut(), 1));
        assert!(read_and_free_error().contains("HostBridgeSharedTexture pointer"));

        shared_texture_dispose(std::ptr::null_mut());

        shared_texture_dispose(texture);
        session_dispose(session);

        assert!(capabilities.requires_explicit_release == 0);
    }
}
