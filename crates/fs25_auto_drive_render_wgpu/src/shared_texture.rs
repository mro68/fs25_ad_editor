//! Shared-Texture-Runtime fuer hostseitige GPU-Handles ohne Pixelbuffer-Readback.

use crate::export_core::{ExportCoreError, RenderExportCore};
use fs25_auto_drive_engine::shared::{RenderAssetsSnapshot, RenderScene};
use std::fmt;
use std::sync::atomic::{AtomicU64, Ordering};

static NEXT_SHARED_TEXTURE_ID: AtomicU64 = AtomicU64::new(1);

/// Pixel-Format einer exportierten Shared-Texture.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SharedTexturePixelFormat {
    /// `RGBA8` im sRGB-Farbraum.
    Rgba8Srgb,
}

/// Alpha-Semantik einer exportierten Shared-Texture.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SharedTextureAlphaMode {
    /// Farbwerte sind premultiplied gespeichert.
    Premultiplied,
}

/// Metadaten eines gerenderten Shared-Texture-Frames.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct SharedTextureFrame {
    /// Breite in Pixeln.
    pub width: u32,
    /// Hoehe in Pixeln.
    pub height: u32,
    /// Exportiertes Pixel-Format.
    pub pixel_format: SharedTexturePixelFormat,
    /// Exportierter Alpha-Modus.
    pub alpha_mode: SharedTextureAlphaMode,
    /// Stabile Runtime-ID der zugrundeliegenden GPU-Textur.
    pub texture_id: u64,
    /// Generation der GPU-Textur (inkrementiert bei Recreate/Resize).
    pub texture_generation: u64,
    /// Lease-Token fuer Acquire/Release-Lifecycle.
    pub frame_token: u64,
}

/// Opaque Runtime-Pointerwerte fuer denselben Prozessraum.
///
/// Diese Werte sind keine backend-nativen Vulkan-/Metal-/DX-Interop-Handles.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct SharedTextureNativeHandle {
    /// Adresse der `wgpu::Texture` als opaque Pointerwert.
    pub texture_ptr: usize,
    /// Adresse der `wgpu::TextureView` als opaque Pointerwert.
    pub texture_view_ptr: usize,
}

/// Fehler der Shared-Texture-Runtime.
#[derive(Debug, Clone, PartialEq)]
pub enum SharedTextureError {
    /// Die angeforderte Zielgroesse ist ungueltig.
    InvalidSize { width: u32, height: u32 },
    /// Die Zielgroesse ueberschreitet die maximale 2D-Textur-Groesse des Devices.
    SizeExceedsTextureLimit {
        width: u32,
        height: u32,
        max_dimension: u32,
    },
    /// Die Render-Szene wurde fuer eine andere Viewport-Groesse gebaut.
    ViewportSizeMismatch {
        expected: [u32; 2],
        actual: [f32; 2],
    },
    /// Es wurde noch kein Frame gerendert.
    FrameUnavailable,
    /// Ein Frame ist bereits geleast und muss zuerst freigegeben werden.
    FrameAlreadyAcquired { frame_token: u64 },
    /// Es existiert kein aktiver Lease fuer den angeforderten Zugriff.
    FrameLeaseMissing,
    /// Das uebergebene Token passt nicht zum aktiven Lease.
    FrameLeaseMismatch { expected: u64, actual: u64 },
    /// Rendern/Resize ist waehrend eines aktiven Leases nicht erlaubt.
    FrameInUse { frame_token: u64 },
}

impl fmt::Display for SharedTextureError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::InvalidSize { width, height } => {
                write!(f, "shared texture size must be positive, got {width}x{height}")
            }
            Self::SizeExceedsTextureLimit {
                width,
                height,
                max_dimension,
            } => write!(
                f,
                "shared texture size {width}x{height} exceeds device texture limit {max_dimension}"
            ),
            Self::ViewportSizeMismatch { expected, actual } => write!(
                f,
                "render scene viewport must match shared texture size (expected {}x{}, got {}x{})",
                expected[0], expected[1], actual[0], actual[1]
            ),
            Self::FrameUnavailable => write!(f, "shared texture has no rendered frame yet"),
            Self::FrameAlreadyAcquired { frame_token } => write!(
                f,
                "shared texture frame {frame_token} is already acquired and must be released first"
            ),
            Self::FrameLeaseMissing => write!(f, "shared texture frame is not acquired"),
            Self::FrameLeaseMismatch { expected, actual } => write!(
                f,
                "shared texture frame token mismatch: expected {expected}, got {actual}"
            ),
            Self::FrameInUse { frame_token } => write!(
                f,
                "shared texture frame {frame_token} is currently acquired; release before render or resize"
            ),
        }
    }
}

impl std::error::Error for SharedTextureError {}

/// Offscreen-Runtime fuer Shared-Texture ohne CPU-Readback.
///
/// Die Runtime rendert `RenderScene + RenderAssetsSnapshot` in eine interne
/// GPU-Textur und erzwingt einen expliziten Acquire/Release-Lifecycle.
pub struct SharedTextureRuntime {
    core: RenderExportCore,
    texture_id: u64,
    texture_generation: u64,
    next_frame_token: u64,
    last_frame: Option<SharedTextureFrame>,
    acquired_frame_token: Option<u64>,
}

impl SharedTextureRuntime {
    /// Erstellt eine neue Shared-Texture-Runtime mit gegebener Zielgroesse.
    pub fn new(
        device: &wgpu::Device,
        queue: &wgpu::Queue,
        size: [u32; 2],
    ) -> Result<Self, SharedTextureError> {
        let core = RenderExportCore::new(device, queue, size).map_err(Self::map_core_error)?;

        Ok(Self {
            core,
            texture_id: NEXT_SHARED_TEXTURE_ID.fetch_add(1, Ordering::Relaxed),
            texture_generation: 1,
            next_frame_token: 1,
            last_frame: None,
            acquired_frame_token: None,
        })
    }

    /// Aendert die Zielgroesse der Shared-Texture.
    ///
    /// Der Aufruf ist nur erlaubt, wenn kein Frame aktiv geleast ist.
    pub fn resize(
        &mut self,
        device: &wgpu::Device,
        size: [u32; 2],
    ) -> Result<(), SharedTextureError> {
        self.ensure_not_acquired()?;

        if self
            .core
            .resize(device, size)
            .map_err(Self::map_core_error)?
        {
            self.texture_generation = self.texture_generation.saturating_add(1);
            self.last_frame = None;
        }

        Ok(())
    }

    /// Rendert Szene plus Assets in die Shared-Texture.
    ///
    /// Solange ein Frame geleast ist, wird kein neues Rendern zugelassen.
    pub fn render_frame(
        &mut self,
        device: &wgpu::Device,
        queue: &wgpu::Queue,
        scene: &RenderScene,
        assets: &RenderAssetsSnapshot,
    ) -> Result<SharedTextureFrame, SharedTextureError> {
        self.ensure_not_acquired()?;

        self.core
            .render_scene(device, queue, scene, assets)
            .map_err(Self::map_core_error)?;

        let size = self.core.size();
        let frame = SharedTextureFrame {
            width: size[0],
            height: size[1],
            pixel_format: SharedTexturePixelFormat::Rgba8Srgb,
            alpha_mode: SharedTextureAlphaMode::Premultiplied,
            texture_id: self.texture_id,
            texture_generation: self.texture_generation,
            frame_token: self.next_frame_token(),
        };

        self.last_frame = Some(frame);
        Ok(frame)
    }

    /// Rendert Szene plus Assets in eine extern bereitgestellte `TextureView`.
    ///
    /// Diese Methode nutzt denselben Renderer- und Background-Sync-Zustand wie
    /// [`render_frame`](Self::render_frame), schreibt aber nicht in die interne
    /// Shared-Texture. Der Acquire/Release-Lifecycle der internen Shared-Texture
    /// bleibt dabei unveraendert.
    #[cfg(any(feature = "flutter-linux", feature = "flutter-android"))]
    pub fn render_to_view(
        &mut self,
        device: &wgpu::Device,
        queue: &wgpu::Queue,
        scene: &RenderScene,
        assets: &RenderAssetsSnapshot,
        target_texture: &wgpu::Texture,
    ) -> Result<(), SharedTextureError> {
        self.ensure_not_acquired()?;
        self.core
            .render_scene_to_view(device, queue, scene, assets, target_texture)
            .map_err(Self::map_core_error)
    }

    /// Leased den zuletzt gerenderten Frame fuer den Host.
    pub fn acquire_frame(&mut self) -> Result<SharedTextureFrame, SharedTextureError> {
        if let Some(frame_token) = self.acquired_frame_token {
            return Err(SharedTextureError::FrameAlreadyAcquired { frame_token });
        }

        let frame = self
            .last_frame
            .ok_or(SharedTextureError::FrameUnavailable)?;
        self.acquired_frame_token = Some(frame.frame_token);

        Ok(frame)
    }

    /// Gibt einen zuvor geleasten Frame wieder frei.
    pub fn release_frame(&mut self, frame_token: u64) -> Result<(), SharedTextureError> {
        match self.acquired_frame_token {
            Some(active_token) if active_token == frame_token => {
                self.acquired_frame_token = None;
                Ok(())
            }
            Some(active_token) => Err(SharedTextureError::FrameLeaseMismatch {
                expected: active_token,
                actual: frame_token,
            }),
            None => Err(SharedTextureError::FrameLeaseMissing),
        }
    }

    /// Liefert die Metadaten des zuletzt gerenderten Frames ohne Lease-Aenderung.
    pub fn frame(&self) -> Option<SharedTextureFrame> {
        self.last_frame
    }

    /// Liefert opaque Runtime-Pointerwerte fuer den aktiven Lease.
    ///
    /// Die Pointer sind nur im selben Prozessraum gueltig und keine backend-
    /// nativen Vulkan-/Metal-/DX-Interop-Handles.
    pub fn native_handle(
        &self,
        frame_token: u64,
    ) -> Result<SharedTextureNativeHandle, SharedTextureError> {
        match self.acquired_frame_token {
            Some(active_token) if active_token == frame_token => Ok(SharedTextureNativeHandle {
                texture_ptr: self.core.texture() as *const wgpu::Texture as usize,
                texture_view_ptr: self.core.texture_view() as *const wgpu::TextureView as usize,
            }),
            Some(active_token) => Err(SharedTextureError::FrameLeaseMismatch {
                expected: active_token,
                actual: frame_token,
            }),
            None => Err(SharedTextureError::FrameLeaseMissing),
        }
    }

    fn map_core_error(error: ExportCoreError) -> SharedTextureError {
        match error {
            ExportCoreError::InvalidSize { width, height } => {
                SharedTextureError::InvalidSize { width, height }
            }
            ExportCoreError::SizeExceedsTextureLimit {
                width,
                height,
                max_dimension,
            } => SharedTextureError::SizeExceedsTextureLimit {
                width,
                height,
                max_dimension,
            },
            ExportCoreError::ViewportSizeMismatch { expected, actual } => {
                SharedTextureError::ViewportSizeMismatch { expected, actual }
            }
        }
    }

    fn ensure_not_acquired(&self) -> Result<(), SharedTextureError> {
        if let Some(frame_token) = self.acquired_frame_token {
            return Err(SharedTextureError::FrameInUse { frame_token });
        }

        Ok(())
    }

    fn next_frame_token(&mut self) -> u64 {
        let token = self.next_frame_token;
        self.next_frame_token = if token == u64::MAX { 1 } else { token + 1 };
        token
    }
}

#[cfg(test)]
mod tests {
    use super::{SharedTextureError, SharedTextureRuntime};
    use fs25_auto_drive_engine::shared::{
        EditorOptions, RenderCamera, RenderQuality, RenderScene, RenderSceneFrameData,
    };
    use indexmap::IndexSet;
    use std::sync::Arc;

    fn create_test_gpu() -> Option<(wgpu::Instance, wgpu::Adapter, wgpu::Device, wgpu::Queue)> {
        let instance = wgpu::Instance::default();
        let adapter = pollster::block_on(instance.request_adapter(&wgpu::RequestAdapterOptions {
            power_preference: wgpu::PowerPreference::LowPower,
            compatible_surface: None,
            force_fallback_adapter: false,
        }))
        .ok()?;

        let (device, queue) = pollster::block_on(adapter.request_device(&wgpu::DeviceDescriptor {
            label: Some("SharedTextureRuntime Test Device"),
            required_features: wgpu::Features::empty(),
            required_limits: wgpu::Limits::downlevel_defaults(),
            experimental_features: Default::default(),
            memory_hints: wgpu::MemoryHints::Performance,
            trace: wgpu::Trace::Off,
        }))
        .ok()?;

        Some((instance, adapter, device, queue))
    }

    fn empty_scene(size: [u32; 2]) -> RenderScene {
        RenderScene::new(
            None,
            RenderSceneFrameData {
                camera: RenderCamera::new(glam::Vec2::ZERO, 2048.0),
                viewport_size: [size[0] as f32, size[1] as f32],
                render_quality: RenderQuality::High,
                selected_node_ids: Arc::new(IndexSet::new()),
                selected_node_ids_revision: 0,
                has_background: false,
                background_visible: false,
                options: Arc::new(EditorOptions::default()),
                hidden_node_ids: Arc::new(IndexSet::new()),
                hidden_node_ids_revision: 0,
                dimmed_node_ids: Arc::new(IndexSet::new()),
                dimmed_node_ids_revision: 0,
            },
        )
    }

    #[test]
    fn shared_texture_runtime_renders_and_manages_frame_lease() {
        let Some((_instance, _adapter, device, queue)) = create_test_gpu() else {
            return;
        };

        let mut runtime =
            SharedTextureRuntime::new(&device, &queue, [8, 6]).expect("runtime must initialize");
        let frame = runtime
            .render_frame(
                &device,
                &queue,
                &empty_scene([8, 6]),
                &fs25_auto_drive_engine::shared::RenderAssetsSnapshot::default(),
            )
            .expect("render must succeed");

        assert_eq!(frame.width, 8);
        assert_eq!(frame.height, 6);

        let leased = runtime.acquire_frame().expect("frame lease must succeed");
        assert_eq!(leased.frame_token, frame.frame_token);

        let handle = runtime
            .native_handle(leased.frame_token)
            .expect("native handle lookup must succeed");
        assert_ne!(handle.texture_ptr, 0);
        assert_ne!(handle.texture_view_ptr, 0);

        runtime
            .release_frame(leased.frame_token)
            .expect("frame release must succeed");
    }

    #[test]
    fn shared_texture_runtime_blocks_render_and_resize_while_leased() {
        let Some((_instance, _adapter, device, queue)) = create_test_gpu() else {
            return;
        };

        let mut runtime =
            SharedTextureRuntime::new(&device, &queue, [8, 6]).expect("runtime must initialize");
        let frame = runtime
            .render_frame(
                &device,
                &queue,
                &empty_scene([8, 6]),
                &fs25_auto_drive_engine::shared::RenderAssetsSnapshot::default(),
            )
            .expect("render must succeed");
        let leased = runtime.acquire_frame().expect("frame lease must succeed");

        assert!(matches!(
            runtime.render_frame(
                &device,
                &queue,
                &empty_scene([8, 6]),
                &fs25_auto_drive_engine::shared::RenderAssetsSnapshot::default(),
            ),
            Err(SharedTextureError::FrameInUse { frame_token }) if frame_token == leased.frame_token
        ));

        assert!(matches!(
            runtime.resize(&device, [10, 4]),
            Err(SharedTextureError::FrameInUse { frame_token }) if frame_token == leased.frame_token
        ));

        runtime
            .release_frame(frame.frame_token)
            .expect("frame release must succeed");
        runtime
            .resize(&device, [10, 4])
            .expect("resize after release must succeed");
    }

    #[test]
    fn shared_texture_runtime_reports_lease_mismatch() {
        let Some((_instance, _adapter, device, queue)) = create_test_gpu() else {
            return;
        };

        let mut runtime =
            SharedTextureRuntime::new(&device, &queue, [8, 6]).expect("runtime must initialize");
        let frame = runtime
            .render_frame(
                &device,
                &queue,
                &empty_scene([8, 6]),
                &fs25_auto_drive_engine::shared::RenderAssetsSnapshot::default(),
            )
            .expect("render must succeed");
        let leased = runtime.acquire_frame().expect("frame lease must succeed");

        assert!(matches!(
            runtime.release_frame(leased.frame_token + 1),
            Err(SharedTextureError::FrameLeaseMismatch {
                expected,
                actual,
            }) if expected == leased.frame_token && actual == leased.frame_token + 1
        ));

        runtime
            .release_frame(frame.frame_token)
            .expect("release with correct token must succeed");
    }

    #[test]
    fn shared_texture_runtime_rejects_invalid_size_and_viewport_mismatch() {
        let Some((_instance, _adapter, device, queue)) = create_test_gpu() else {
            return;
        };

        assert!(matches!(
            SharedTextureRuntime::new(&device, &queue, [0, 8]),
            Err(SharedTextureError::InvalidSize {
                width: 0,
                height: 8,
            })
        ));

        let mut runtime =
            SharedTextureRuntime::new(&device, &queue, [8, 8]).expect("runtime must initialize");
        let error = runtime
            .render_frame(
                &device,
                &queue,
                &empty_scene([4, 4]),
                &fs25_auto_drive_engine::shared::RenderAssetsSnapshot::default(),
            )
            .expect_err("viewport mismatch must fail");

        assert!(matches!(
            error,
            SharedTextureError::ViewportSizeMismatch {
                expected: [8, 8],
                actual,
            } if actual == [4.0, 4.0]
        ));
    }
}
