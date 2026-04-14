//! Interner, transportneutraler Render-Export-Kern fuer Offscreen-Ziele.

use crate::{BackgroundWorldBounds, Renderer, RendererTargetConfig};
use fs25_auto_drive_engine::shared::{RenderAssetsSnapshot, RenderScene};
use std::fmt;

pub(crate) const EXPORT_COLOR_FORMAT: wgpu::TextureFormat = wgpu::TextureFormat::Rgba8UnormSrgb;
/// Sample-Count der Export-Zieltexture (DMA-BUF, Readback). Bleibt 1×.
pub(crate) const EXPORT_SAMPLE_COUNT: u32 = 1;
/// MSAA-Sample-Count fuer die Render-Pipelines (wie im egui-Frontend: 4×).
const EXPORT_MSAA_SAMPLES: u32 = 4;

#[derive(Debug, Clone, Copy, PartialEq)]
pub(crate) enum ExportCoreError {
    InvalidSize {
        width: u32,
        height: u32,
    },
    SizeExceedsTextureLimit {
        width: u32,
        height: u32,
        max_dimension: u32,
    },
    ViewportSizeMismatch {
        expected: [u32; 2],
        actual: [f32; 2],
    },
}

impl fmt::Display for ExportCoreError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::InvalidSize { width, height } => {
                write!(
                    f,
                    "export target size must be positive, got {width}x{height}"
                )
            }
            Self::SizeExceedsTextureLimit {
                width,
                height,
                max_dimension,
            } => write!(
                f,
                "export target size {width}x{height} exceeds device texture limit {max_dimension}"
            ),
            Self::ViewportSizeMismatch { expected, actual } => write!(
                f,
                "render scene viewport must match export target size (expected {}x{}, got {}x{})",
                expected[0], expected[1], actual[0], actual[1]
            ),
        }
    }
}

impl std::error::Error for ExportCoreError {}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) struct ExportTargetLayout {
    size: [u32; 2],
}

impl ExportTargetLayout {
    pub(crate) fn size(self) -> [u32; 2] {
        self.size
    }
}

/// Interner Render-Kern fuer Offscreen-Exporte.
///
/// Dieser Typ kapselt nur Renderer-Besitz, Ziel-Validierung und
/// revisionsbasierten Background-Sync. Transportdetails wie CPU-Readback,
/// Frame-Caching oder FFI-Lifecycle leben in separaten Adaptern.
pub(crate) struct RenderExportCore {
    renderer: Renderer,
    layout: ExportTargetLayout,
    color_texture: wgpu::Texture,
    color_view: wgpu::TextureView,
    /// 4× MSAA-Texture als tatsaechliches Render-Ziel. Wird per Resolve in die 1×-Export-Texture aufgeloest.
    msaa_texture: wgpu::Texture,
    msaa_view: wgpu::TextureView,
    last_background_asset_revision: u64,
    last_background_transform_revision: u64,
}

impl RenderExportCore {
    pub(crate) fn new(
        device: &wgpu::Device,
        queue: &wgpu::Queue,
        size: [u32; 2],
    ) -> Result<Self, ExportCoreError> {
        let layout = validate_target_size(size, &device.limits())?;
        let renderer = Renderer::new(
            device,
            queue,
            RendererTargetConfig::new(EXPORT_COLOR_FORMAT, EXPORT_MSAA_SAMPLES),
        );
        let (color_texture, color_view) = create_color_target(device, layout.size());
        let (msaa_texture, msaa_view) = create_msaa_target(device, layout.size());

        Ok(Self {
            renderer,
            layout,
            color_texture,
            color_view,
            msaa_texture,
            msaa_view,
            last_background_asset_revision: 0,
            last_background_transform_revision: 0,
        })
    }

    pub(crate) fn resize(
        &mut self,
        device: &wgpu::Device,
        size: [u32; 2],
    ) -> Result<bool, ExportCoreError> {
        let layout = validate_target_size(size, &device.limits())?;
        if self.layout == layout {
            return Ok(false);
        }

        let (color_texture, color_view) = create_color_target(device, layout.size());
        let (msaa_texture, msaa_view) = create_msaa_target(device, layout.size());
        self.layout = layout;
        self.color_texture = color_texture;
        self.color_view = color_view;
        self.msaa_texture = msaa_texture;
        self.msaa_view = msaa_view;

        Ok(true)
    }

    pub(crate) fn render_scene(
        &mut self,
        device: &wgpu::Device,
        queue: &wgpu::Queue,
        scene: &RenderScene,
        assets: &RenderAssetsSnapshot,
    ) -> Result<(), ExportCoreError> {
        let expected_viewport = [self.layout.size()[0] as f32, self.layout.size()[1] as f32];
        if scene.viewport_size() != expected_viewport {
            return Err(ExportCoreError::ViewportSizeMismatch {
                expected: self.layout.size(),
                actual: scene.viewport_size(),
            });
        }

        self.sync_background_asset(device, queue, assets);
        Self::issue_render_pass(
            &mut self.renderer,
            device,
            queue,
            scene,
            &self.msaa_view,
            &self.color_view,
            "SharedTexture Export Pass",
        );
        Ok(())
    }

    pub(crate) fn size(&self) -> [u32; 2] {
        self.layout.size()
    }

    pub(crate) fn texture(&self) -> &wgpu::Texture {
        &self.color_texture
    }

    pub(crate) fn texture_view(&self) -> &wgpu::TextureView {
        &self.color_view
    }

    /// Rendert die Szene in eine extern bereitgestellte TextureView (Flutter-Integration).
    ///
    /// Im Gegensatz zu [`render_scene`](Self::render_scene) schreibt diese Methode nicht in
    /// die interne Farb-Texture, sondern in einen uebergebenen externen Render-Target.
    /// Hintergrund-Assets werden dabei regulaer synchronisiert.
    ///
    /// # Fehler
    /// Gibt [`ExportCoreError`] zurueck wenn Viewport-Groesse oder Target-Groesse nicht uebereinstimmen.
    ///
    /// TODO(flutter-wiring): Wird aufgerufen sobald der Flutter-GPU-Pfad vollstaendig
    /// mit `flutter_gpu.rs` verbunden ist.
    #[cfg(any(feature = "flutter-linux", feature = "flutter-android"))]
    #[allow(dead_code)]
    pub(crate) fn render_scene_to_view(
        &mut self,
        device: &wgpu::Device,
        queue: &wgpu::Queue,
        scene: &RenderScene,
        assets: &RenderAssetsSnapshot,
        external_texture: &wgpu::Texture,
    ) -> Result<(), ExportCoreError> {
        let expected_viewport = [self.layout.size()[0] as f32, self.layout.size()[1] as f32];
        if scene.viewport_size() != expected_viewport {
            return Err(ExportCoreError::ViewportSizeMismatch {
                expected: self.layout.size(),
                actual: scene.viewport_size(),
            });
        }

        self.sync_background_asset(device, queue, assets);
        // MSAA-Resolve in interne OPTIMAL-Texture, dann Copy in die externe
        // Vulkan-Export-Texture. Direkt-Resolve in LINEAR-Tiling schlaegt
        // im Linux-DMA-BUF-Pfad auf NVIDIA fehl.
        Self::issue_render_pass(
            &mut self.renderer,
            device,
            queue,
            scene,
            &self.msaa_view,
            &self.color_view,
            "Flutter MSAA Resolve Pass",
        );
        Self::copy_to_external(
            device,
            queue,
            &self.color_texture,
            external_texture,
            self.layout.size(),
        );
        Ok(())
    }

    /// Erstellt einen Render-Pass und submittet den Command-Encoder.
    ///
    /// Gemeinsame Render-Infrastruktur fuer [`render_scene`](Self::render_scene) und
    /// [`render_scene_to_view`](Self::render_scene_to_view).
    /// Erstellt einen MSAA-Render-Pass mit automatischem Resolve in die 1×-Zieltexture.
    ///
    /// `msaa_view` ist die 4×-Multisampled-Texture (tatsaechliches Renderziel).
    /// `resolve_view` ist die 1×-Texture (DMA-BUF oder internes Export-Target).
    fn issue_render_pass(
        renderer: &mut Renderer,
        device: &wgpu::Device,
        queue: &wgpu::Queue,
        scene: &RenderScene,
        msaa_view: &wgpu::TextureView,
        resolve_view: &wgpu::TextureView,
        label: &str,
    ) {
        let mut encoder =
            device.create_command_encoder(&wgpu::CommandEncoderDescriptor { label: Some(label) });
        {
            let mut render_pass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
                label: Some(label),
                color_attachments: &[Some(wgpu::RenderPassColorAttachment {
                    view: msaa_view,
                    depth_slice: None,
                    resolve_target: Some(resolve_view),
                    ops: wgpu::Operations {
                        load: wgpu::LoadOp::Clear(wgpu::Color::TRANSPARENT),
                        store: wgpu::StoreOp::Discard,
                    },
                })],
                depth_stencil_attachment: None,
                occlusion_query_set: None,
                timestamp_writes: None,
                multiview_mask: None,
            });
            renderer.render_scene(device, queue, &mut render_pass, scene);
        }
        queue.submit(Some(encoder.finish()));
    }

    /// Kopiert die interne 1x-Farb-Texture in eine externe Vulkan-Export-Texture.
    ///
    /// Noetig weil NVIDIA MSAA-Resolve direkt in LINEAR-Tiling im Linux-DMA-BUF-Pfad
    /// nicht korrekt unterstuetzt.
    #[cfg(any(feature = "flutter-linux", feature = "flutter-android"))]
    fn copy_to_external(
        device: &wgpu::Device,
        queue: &wgpu::Queue,
        src: &wgpu::Texture,
        dst: &wgpu::Texture,
        size: [u32; 2],
    ) {
        let mut encoder = device.create_command_encoder(&wgpu::CommandEncoderDescriptor {
            label: Some("Flutter Copy to External Texture"),
        });
        encoder.copy_texture_to_texture(
            src.as_image_copy(),
            dst.as_image_copy(),
            texture_extent(size),
        );
        queue.submit(Some(encoder.finish()));
    }

    #[cfg(test)]
    pub(crate) fn background_revisions(&self) -> (u64, u64) {
        (
            self.last_background_asset_revision,
            self.last_background_transform_revision,
        )
    }

    fn sync_background_asset(
        &mut self,
        device: &wgpu::Device,
        queue: &wgpu::Queue,
        assets: &RenderAssetsSnapshot,
    ) {
        let asset_revision = assets.background_asset_revision();
        let transform_revision = assets.background_transform_revision();

        if asset_revision == self.last_background_asset_revision
            && transform_revision == self.last_background_transform_revision
        {
            return;
        }

        if let Some(background) = assets.background() {
            self.renderer.set_background(
                device,
                queue,
                background.image.as_ref(),
                BackgroundWorldBounds {
                    min_x: background.world_bounds.min_x,
                    max_x: background.world_bounds.max_x,
                    min_y: background.world_bounds.min_z,
                    max_y: background.world_bounds.max_z,
                },
                background.scale,
            );
        } else {
            self.renderer.clear_background();
        }

        self.last_background_asset_revision = asset_revision;
        self.last_background_transform_revision = transform_revision;
    }
}

fn validate_target_size(
    size: [u32; 2],
    limits: &wgpu::Limits,
) -> Result<ExportTargetLayout, ExportCoreError> {
    if size[0] == 0 || size[1] == 0 {
        return Err(ExportCoreError::InvalidSize {
            width: size[0],
            height: size[1],
        });
    }

    if size[0] > limits.max_texture_dimension_2d || size[1] > limits.max_texture_dimension_2d {
        return Err(ExportCoreError::SizeExceedsTextureLimit {
            width: size[0],
            height: size[1],
            max_dimension: limits.max_texture_dimension_2d,
        });
    }

    Ok(ExportTargetLayout { size })
}

fn texture_extent(size: [u32; 2]) -> wgpu::Extent3d {
    wgpu::Extent3d {
        width: size[0],
        height: size[1],
        depth_or_array_layers: 1,
    }
}

/// Erstellt die 1×-Resolve-/Export-Zieltexture.
fn create_color_target(
    device: &wgpu::Device,
    size: [u32; 2],
) -> (wgpu::Texture, wgpu::TextureView) {
    let texture = device.create_texture(&wgpu::TextureDescriptor {
        label: Some("SharedTexture Export Target (1x resolve)"),
        size: texture_extent(size),
        mip_level_count: 1,
        sample_count: EXPORT_SAMPLE_COUNT,
        dimension: wgpu::TextureDimension::D2,
        format: EXPORT_COLOR_FORMAT,
        usage: wgpu::TextureUsages::RENDER_ATTACHMENT
            | wgpu::TextureUsages::TEXTURE_BINDING
            | wgpu::TextureUsages::COPY_SRC,
        view_formats: &[],
    });
    let view = texture.create_view(&wgpu::TextureViewDescriptor::default());
    (texture, view)
}

/// Erstellt die 4×-MSAA-Texture als tatsaechliches Renderziel.
fn create_msaa_target(device: &wgpu::Device, size: [u32; 2]) -> (wgpu::Texture, wgpu::TextureView) {
    let texture = device.create_texture(&wgpu::TextureDescriptor {
        label: Some("SharedTexture MSAA Target (4x)"),
        size: texture_extent(size),
        mip_level_count: 1,
        sample_count: EXPORT_MSAA_SAMPLES,
        dimension: wgpu::TextureDimension::D2,
        format: EXPORT_COLOR_FORMAT,
        usage: wgpu::TextureUsages::RENDER_ATTACHMENT,
        view_formats: &[],
    });
    let view = texture.create_view(&wgpu::TextureViewDescriptor::default());
    (texture, view)
}

#[cfg(test)]
mod tests {
    use super::{validate_target_size, ExportCoreError, RenderExportCore};
    use fs25_auto_drive_engine::shared::{
        EditorOptions, RenderAssetSnapshot, RenderAssetsSnapshot, RenderBackgroundAssetSnapshot,
        RenderBackgroundWorldBounds, RenderCamera, RenderQuality, RenderScene,
        RenderSceneFrameData,
    };
    use image::{DynamicImage, Rgba, RgbaImage};
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
            label: Some("RenderExportCore Test Device"),
            required_features: wgpu::Features::empty(),
            required_limits: wgpu::Limits::downlevel_defaults(),
            experimental_features: Default::default(),
            memory_hints: wgpu::MemoryHints::Performance,
            trace: wgpu::Trace::Off,
        }))
        .ok()?;

        Some((instance, adapter, device, queue))
    }

    fn empty_scene(size: [u32; 2], has_background: bool) -> RenderScene {
        RenderScene::new(
            None,
            RenderSceneFrameData {
                camera: RenderCamera::new(glam::Vec2::ZERO, 2048.0),
                viewport_size: [size[0] as f32, size[1] as f32],
                render_quality: RenderQuality::High,
                selected_node_ids: Arc::new(IndexSet::new()),
                selected_node_ids_revision: 0,
                has_background,
                background_visible: has_background,
                options: Arc::new(EditorOptions::default()),
                hidden_node_ids: Arc::new(IndexSet::new()),
                hidden_node_ids_revision: 0,
                dimmed_node_ids: Arc::new(IndexSet::new()),
                dimmed_node_ids_revision: 0,
            },
        )
    }

    #[test]
    fn export_core_rejects_invalid_and_oversized_targets() {
        let tight_limit = wgpu::Limits {
            max_texture_dimension_2d: 64,
            ..wgpu::Limits::downlevel_defaults()
        };

        assert!(matches!(
            validate_target_size([0, 8], &tight_limit),
            Err(ExportCoreError::InvalidSize {
                width: 0,
                height: 8
            })
        ));
        assert!(matches!(
            validate_target_size([8, 0], &tight_limit),
            Err(ExportCoreError::InvalidSize {
                width: 8,
                height: 0
            })
        ));
        assert!(matches!(
            validate_target_size([65, 8], &tight_limit),
            Err(ExportCoreError::SizeExceedsTextureLimit {
                width: 65,
                height: 8,
                max_dimension: 64,
            })
        ));
    }

    #[test]
    fn export_core_tracks_background_revisions() {
        let Some((_instance, _adapter, device, queue)) = create_test_gpu() else {
            return;
        };

        let mut core =
            RenderExportCore::new(&device, &queue, [16, 16]).expect("export core must initialize");
        assert_eq!(core.background_revisions(), (0, 0));

        let scene = empty_scene([16, 16], true);
        let mut image = RgbaImage::new(1, 1);
        image.put_pixel(0, 0, Rgba([255, 64, 32, 255]));
        let assets = RenderAssetsSnapshot::new(
            3,
            5,
            vec![RenderAssetSnapshot::background(
                RenderBackgroundAssetSnapshot {
                    image: Arc::new(DynamicImage::ImageRgba8(image)),
                    world_bounds: RenderBackgroundWorldBounds::new(-1.0, 1.0, -1.0, 1.0),
                    scale: 1.0,
                    asset_revision: 3,
                    transform_revision: 5,
                },
            )],
        );

        core.render_scene(&device, &queue, &scene, &assets)
            .expect("render with background must succeed");
        assert_eq!(core.background_revisions(), (3, 5));

        core.render_scene(&device, &queue, &scene, &assets)
            .expect("second render with same revisions must succeed");
        assert_eq!(core.background_revisions(), (3, 5));
    }

    #[test]
    fn export_core_rejects_viewport_mismatch() {
        let Some((_instance, _adapter, device, queue)) = create_test_gpu() else {
            return;
        };

        let mut core =
            RenderExportCore::new(&device, &queue, [16, 16]).expect("export core must initialize");
        let mismatched_scene = empty_scene([8, 8], false);

        let error = core
            .render_scene(
                &device,
                &queue,
                &mismatched_scene,
                &RenderAssetsSnapshot::default(),
            )
            .expect_err("viewport mismatch must fail");
        assert!(matches!(
            error,
            ExportCoreError::ViewportSizeMismatch {
                expected: [16, 16],
                actual,
            } if actual == [8.0, 8.0]
        ));
    }
}
