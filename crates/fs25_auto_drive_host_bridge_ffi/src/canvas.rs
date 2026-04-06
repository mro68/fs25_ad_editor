//! Native-Canvas-Adapter ueber der kanonischen Host-Bridge-Session.

use crate::{clear_last_error, set_last_error, with_session_mut};
use anyhow::{anyhow, Result};
use fs25_auto_drive_host_bridge::HostBridgeSession;
use fs25_auto_drive_render_wgpu::{CanvasAlphaMode, CanvasFrame, CanvasPixelFormat, CanvasRuntime};

const FS25AD_CANVAS_CONTRACT_VERSION: u32 = 1;
const FS25AD_PIXEL_FORMAT_RGBA8_SRGB: u32 = 1;
const FS25AD_ALPHA_MODE_PREMULTIPLIED: u32 = 1;

/// Opaquer nativer Canvas-Handle fuer Offscreen-Rendering.
pub(crate) struct HostBridgeNativeCanvas {
    _instance: wgpu::Instance,
    _adapter: wgpu::Adapter,
    device: wgpu::Device,
    queue: wgpu::Queue,
    runtime: CanvasRuntime,
    size: [u32; 2],
}

impl HostBridgeNativeCanvas {
    fn new(width: u32, height: u32) -> Result<Self> {
        if width == 0 || height == 0 {
            return Err(anyhow!(
                "canvas size must be positive, got {width}x{height}"
            ));
        }

        let instance = wgpu::Instance::default();
        let adapter = pollster::block_on(instance.request_adapter(&wgpu::RequestAdapterOptions {
            power_preference: wgpu::PowerPreference::LowPower,
            compatible_surface: None,
            force_fallback_adapter: false,
        }))?;

        let (device, queue) =
            pollster::block_on(adapter.request_device(&wgpu::DeviceDescriptor {
                label: Some("fs25ad native canvas device"),
                required_features: wgpu::Features::empty(),
                required_limits: wgpu::Limits::downlevel_defaults(),
                experimental_features: Default::default(),
                memory_hints: wgpu::MemoryHints::Performance,
                trace: wgpu::Trace::Off,
            }))?;
        let runtime = CanvasRuntime::new(&device, &queue, [width, height])?;

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

    fn render_rgba(&mut self, session: &mut HostBridgeSession) -> Result<()> {
        let frame = session.build_render_frame(self.viewport_size_f32());
        self.runtime
            .render_frame(&self.device, &self.queue, &frame.scene, &frame.assets)?;
        Ok(())
    }

    fn frame(&self) -> Option<&CanvasFrame> {
        self.runtime.frame()
    }

    fn viewport_size_f32(&self) -> [f32; 2] {
        [self.size[0] as f32, self.size[1] as f32]
    }
}

#[repr(C)]
/// Metadatenstruktur fuer den zuletzt gerenderten RGBA-Frame im C-ABI.
pub struct Fs25adRgbaFrameInfo {
    /// Frame-Breite in Pixeln.
    pub width: u32,
    /// Frame-Hoehe in Pixeln.
    pub height: u32,
    /// Dicht gepackte Zeilenlaenge in Bytes (`width * 4`).
    pub bytes_per_row: u32,
    /// ABI-Konstante fuer das Pixel-Format (`1 = RGBA8 sRGB`).
    pub pixel_format: u32,
    /// ABI-Konstante fuer den Alpha-Modus (`1 = premultiplied`).
    pub alpha_mode: u32,
    /// Gesamte Byte-Laenge des Frames.
    pub byte_len: usize,
}

fn pixel_format_abi(format: CanvasPixelFormat) -> u32 {
    match format {
        CanvasPixelFormat::Rgba8Srgb => FS25AD_PIXEL_FORMAT_RGBA8_SRGB,
    }
}

fn alpha_mode_abi(mode: CanvasAlphaMode) -> u32 {
    match mode {
        CanvasAlphaMode::Premultiplied => FS25AD_ALPHA_MODE_PREMULTIPLIED,
    }
}

fn with_canvas_mut<T>(
    canvas: *mut HostBridgeNativeCanvas,
    f: impl FnOnce(&mut HostBridgeNativeCanvas) -> Result<T>,
) -> Result<T> {
    if canvas.is_null() {
        return Err(anyhow!("HostBridgeNativeCanvas pointer must not be null"));
    }

    let canvas = unsafe { &mut *canvas };
    f(canvas)
}

fn with_canvas<T>(
    canvas: *const HostBridgeNativeCanvas,
    f: impl FnOnce(&HostBridgeNativeCanvas) -> Result<T>,
) -> Result<T> {
    if canvas.is_null() {
        return Err(anyhow!("HostBridgeNativeCanvas pointer must not be null"));
    }

    let canvas = unsafe { &*canvas };
    f(canvas)
}

/// Liefert die Version des nativen Canvas-Vertrags.
#[unsafe(no_mangle)]
pub extern "C" fn fs25ad_host_bridge_canvas_contract_version() -> u32 {
    FS25AD_CANVAS_CONTRACT_VERSION
}

/// Erstellt einen nativen Offscreen-Canvas fuer RGBA-Frames.
#[unsafe(no_mangle)]
pub extern "C" fn fs25ad_host_bridge_canvas_new(
    width: u32,
    height: u32,
) -> *mut HostBridgeNativeCanvas {
    clear_last_error();

    match HostBridgeNativeCanvas::new(width, height) {
        Ok(canvas) => Box::into_raw(Box::new(canvas)),
        Err(error) => {
            set_last_error(error.to_string());
            std::ptr::null_mut()
        }
    }
}

/// Gibt einen zuvor erstellten nativen Canvas-Handle frei.
#[allow(clippy::not_unsafe_ptr_arg_deref)]
#[unsafe(no_mangle)]
pub extern "C" fn fs25ad_host_bridge_canvas_dispose(canvas: *mut HostBridgeNativeCanvas) {
    clear_last_error();
    if canvas.is_null() {
        return;
    }

    unsafe {
        drop(Box::from_raw(canvas));
    }
}

/// Aendert die Zielgroesse eines nativen Canvas.
#[unsafe(no_mangle)]
pub extern "C" fn fs25ad_host_bridge_canvas_resize(
    canvas: *mut HostBridgeNativeCanvas,
    width: u32,
    height: u32,
) -> bool {
    clear_last_error();

    match with_canvas_mut(canvas, |canvas| canvas.resize(width, height)) {
        Ok(()) => true,
        Err(error) => {
            set_last_error(error.to_string());
            false
        }
    }
}

/// Rendert den aktuellen Session-Frame in den nativen Canvas und puffert RGBA-Pixel.
#[unsafe(no_mangle)]
pub extern "C" fn fs25ad_host_bridge_canvas_render_rgba(
    session: *mut HostBridgeSession,
    canvas: *mut HostBridgeNativeCanvas,
) -> bool {
    clear_last_error();

    match with_session_mut(session, |session| {
        with_canvas_mut(canvas, |canvas| canvas.render_rgba(session))
    }) {
        Ok(()) => true,
        Err(error) => {
            set_last_error(error.to_string());
            false
        }
    }
}

/// Liefert Metadaten des zuletzt erfolgreich gerenderten RGBA-Frames.
#[allow(clippy::not_unsafe_ptr_arg_deref)]
#[unsafe(no_mangle)]
pub extern "C" fn fs25ad_host_bridge_canvas_last_frame_info(
    canvas: *const HostBridgeNativeCanvas,
    out_info: *mut Fs25adRgbaFrameInfo,
) -> bool {
    clear_last_error();

    if out_info.is_null() {
        set_last_error("Fs25adRgbaFrameInfo pointer must not be null");
        return false;
    }

    match with_canvas(canvas, |canvas| {
        let frame = canvas
            .frame()
            .ok_or_else(|| anyhow!("canvas has no rendered frame yet"))?;
        let info = &frame.info;
        Ok(Fs25adRgbaFrameInfo {
            width: info.width,
            height: info.height,
            bytes_per_row: info.bytes_per_row,
            pixel_format: pixel_format_abi(info.pixel_format),
            alpha_mode: alpha_mode_abi(info.alpha_mode),
            byte_len: info.byte_len(),
        })
    }) {
        Ok(info) => {
            unsafe {
                *out_info = info;
            }
            true
        }
        Err(error) => {
            set_last_error(error.to_string());
            false
        }
    }
}

/// Kopiert den zuletzt erfolgreich gerenderten RGBA-Frame in einen Host-Buffer.
#[allow(clippy::not_unsafe_ptr_arg_deref)]
#[unsafe(no_mangle)]
pub extern "C" fn fs25ad_host_bridge_canvas_copy_last_frame_rgba(
    canvas: *const HostBridgeNativeCanvas,
    dst: *mut u8,
    dst_len: usize,
) -> bool {
    clear_last_error();

    if dst.is_null() {
        set_last_error("destination buffer pointer must not be null");
        return false;
    }

    match with_canvas(canvas, |canvas| {
        let frame = canvas
            .frame()
            .ok_or_else(|| anyhow!("canvas has no rendered frame yet"))?;
        if dst_len < frame.pixels.len() {
            return Err(anyhow!(
                "destination buffer too small: need {}, got {}",
                frame.pixels.len(),
                dst_len
            ));
        }

        unsafe {
            std::ptr::copy_nonoverlapping(frame.pixels.as_ptr(), dst, frame.pixels.len());
        }
        Ok(())
    }) {
        Ok(()) => true,
        Err(error) => {
            set_last_error(error.to_string());
            false
        }
    }
}

#[cfg(test)]
mod tests {
    use super::{
        fs25ad_host_bridge_canvas_copy_last_frame_rgba, fs25ad_host_bridge_canvas_dispose,
        fs25ad_host_bridge_canvas_last_frame_info, fs25ad_host_bridge_canvas_new,
        fs25ad_host_bridge_canvas_render_rgba, fs25ad_host_bridge_canvas_resize,
        Fs25adRgbaFrameInfo,
    };
    use crate::{
        fs25ad_host_bridge_last_error_message, fs25ad_host_bridge_session_dispose,
        fs25ad_host_bridge_session_new, fs25ad_host_bridge_string_free,
    };
    use std::ffi::CStr;

    fn read_and_free_error() -> String {
        let ptr = fs25ad_host_bridge_last_error_message();
        assert!(!ptr.is_null());
        let value = unsafe { CStr::from_ptr(ptr) }
            .to_str()
            .expect("error string must be valid UTF-8")
            .to_string();
        fs25ad_host_bridge_string_free(ptr);
        value
    }

    #[test]
    fn ffi_canvas_renders_and_copies_last_frame() {
        let session = fs25ad_host_bridge_session_new();
        assert!(!session.is_null());

        let canvas = fs25ad_host_bridge_canvas_new(8, 6);
        assert!(!canvas.is_null());
        assert!(fs25ad_host_bridge_canvas_render_rgba(session, canvas));

        let mut info = Fs25adRgbaFrameInfo {
            width: 0,
            height: 0,
            bytes_per_row: 0,
            pixel_format: 0,
            alpha_mode: 0,
            byte_len: 0,
        };
        assert!(fs25ad_host_bridge_canvas_last_frame_info(canvas, &mut info));
        assert_eq!(info.width, 8);
        assert_eq!(info.height, 6);
        assert_eq!(info.bytes_per_row, 32);
        assert_eq!(info.pixel_format, 1);
        assert_eq!(info.alpha_mode, 1);
        assert_eq!(info.byte_len, 192);

        let mut pixels = vec![255_u8; info.byte_len];
        assert!(fs25ad_host_bridge_canvas_copy_last_frame_rgba(
            canvas,
            pixels.as_mut_ptr(),
            pixels.len()
        ));
        assert!(pixels.iter().all(|byte| *byte == 0));

        fs25ad_host_bridge_canvas_dispose(canvas);
        fs25ad_host_bridge_session_dispose(session);
    }

    #[test]
    fn ffi_canvas_resize_changes_reported_frame_dimensions() {
        let session = fs25ad_host_bridge_session_new();
        assert!(!session.is_null());
        let canvas = fs25ad_host_bridge_canvas_new(4, 4);
        assert!(!canvas.is_null());

        assert!(fs25ad_host_bridge_canvas_resize(canvas, 10, 3));
        assert!(fs25ad_host_bridge_canvas_render_rgba(session, canvas));

        let mut info = Fs25adRgbaFrameInfo {
            width: 0,
            height: 0,
            bytes_per_row: 0,
            pixel_format: 0,
            alpha_mode: 0,
            byte_len: 0,
        };
        assert!(fs25ad_host_bridge_canvas_last_frame_info(canvas, &mut info));
        assert_eq!(info.width, 10);
        assert_eq!(info.height, 3);
        assert_eq!(info.bytes_per_row, 40);
        assert_eq!(info.byte_len, 120);

        fs25ad_host_bridge_canvas_dispose(canvas);
        fs25ad_host_bridge_session_dispose(session);
    }

    #[test]
    fn ffi_canvas_copy_reports_small_destination_buffer() {
        let session = fs25ad_host_bridge_session_new();
        assert!(!session.is_null());
        let canvas = fs25ad_host_bridge_canvas_new(4, 4);
        assert!(!canvas.is_null());
        assert!(fs25ad_host_bridge_canvas_render_rgba(session, canvas));

        let mut tiny = vec![0_u8; 8];
        assert!(!fs25ad_host_bridge_canvas_copy_last_frame_rgba(
            canvas,
            tiny.as_mut_ptr(),
            tiny.len()
        ));
        let error = read_and_free_error();
        assert!(error.contains("destination buffer too small"));

        fs25ad_host_bridge_canvas_dispose(canvas);
        fs25ad_host_bridge_session_dispose(session);
    }

    #[test]
    fn ffi_canvas_rejects_invalid_and_oversized_sizes() {
        let canvas = fs25ad_host_bridge_canvas_new(0, 4);
        assert!(canvas.is_null());
        assert!(read_and_free_error().contains("canvas size must be positive"));

        let canvas = fs25ad_host_bridge_canvas_new(u32::MAX, 1);
        assert!(canvas.is_null());
        assert!(read_and_free_error().contains("exceeds device texture limit"));

        let canvas = fs25ad_host_bridge_canvas_new(4, 4);
        assert!(!canvas.is_null());

        assert!(!fs25ad_host_bridge_canvas_resize(canvas, 0, 4));
        assert!(read_and_free_error().contains("canvas size must be positive"));

        assert!(!fs25ad_host_bridge_canvas_resize(canvas, u32::MAX, 1));
        assert!(read_and_free_error().contains("exceeds device texture limit"));

        fs25ad_host_bridge_canvas_dispose(canvas);
    }

    #[test]
    fn ffi_canvas_reports_missing_frame_and_null_output_pointers() {
        let canvas = fs25ad_host_bridge_canvas_new(4, 4);
        assert!(!canvas.is_null());

        let mut info = Fs25adRgbaFrameInfo {
            width: 0,
            height: 0,
            bytes_per_row: 0,
            pixel_format: 0,
            alpha_mode: 0,
            byte_len: 0,
        };
        assert!(!fs25ad_host_bridge_canvas_last_frame_info(
            canvas, &mut info
        ));
        assert!(read_and_free_error().contains("canvas has no rendered frame yet"));

        let mut pixels = vec![0_u8; 16];
        assert!(!fs25ad_host_bridge_canvas_copy_last_frame_rgba(
            canvas,
            pixels.as_mut_ptr(),
            pixels.len()
        ));
        assert!(read_and_free_error().contains("canvas has no rendered frame yet"));

        assert!(!fs25ad_host_bridge_canvas_last_frame_info(
            canvas,
            std::ptr::null_mut()
        ));
        assert!(read_and_free_error().contains("Fs25adRgbaFrameInfo pointer must not be null"));

        assert!(!fs25ad_host_bridge_canvas_copy_last_frame_rgba(
            canvas,
            std::ptr::null_mut(),
            pixels.len()
        ));
        assert!(read_and_free_error().contains("destination buffer pointer must not be null"));

        fs25ad_host_bridge_canvas_dispose(canvas);
    }
}
