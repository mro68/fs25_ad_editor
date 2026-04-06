//! Offscreen-Canvas-Runtime fuer host-neutrale RGBA-Frames.

use crate::{BackgroundWorldBounds, Renderer, RendererTargetConfig};
use fs25_auto_drive_engine::shared::{RenderAssetsSnapshot, RenderScene};
use std::fmt;
use std::sync::mpsc::{self, RecvTimeoutError};
use std::time::Duration;

const CANVAS_COLOR_FORMAT: wgpu::TextureFormat = wgpu::TextureFormat::Rgba8UnormSrgb;
const CANVAS_SAMPLE_COUNT: u32 = 1;
const CANVAS_BYTES_PER_PIXEL: u32 = 4;
const CANVAS_READBACK_TIMEOUT: Duration = Duration::from_secs(5);

/// Pixel-Format des exportierten Canvas-Frames.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CanvasPixelFormat {
    /// Dicht gepacktes `RGBA8` im sRGB-Farbraum.
    Rgba8Srgb,
}

/// Alpha-Semantik des exportierten Canvas-Frames.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CanvasAlphaMode {
    /// Farbkanaele sind fuer den Export bereits mit Alpha vormultipliziert.
    Premultiplied,
}

/// Explizite Metadaten des zuletzt gerenderten Canvas-Frames.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct CanvasFrameInfo {
    /// Breite des Frames in Pixeln.
    pub width: u32,
    /// Hoehe des Frames in Pixeln.
    pub height: u32,
    /// Dicht gepackte Zeilenlaenge in Bytes.
    pub bytes_per_row: u32,
    /// Exportiertes Pixel-Format.
    pub pixel_format: CanvasPixelFormat,
    /// Exportierter Alpha-Modus.
    pub alpha_mode: CanvasAlphaMode,
}

impl CanvasFrameInfo {
    /// Gesamte Byte-Laenge des Frames.
    pub fn byte_len(&self) -> usize {
        self.bytes_per_row as usize * self.height as usize
    }
}

/// CPU-seitig zwischengespeicherter RGBA-Frame.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CanvasFrame {
    /// Metadaten des Frames.
    pub info: CanvasFrameInfo,
    /// Top-down, dicht gepackte RGBA-Pixel.
    pub pixels: Vec<u8>,
}

/// Fehler der Offscreen-Canvas-Runtime.
#[derive(Debug)]
pub enum CanvasError {
    /// Die angeforderte Canvas-Groesse ist ungueltig.
    InvalidSize { width: u32, height: u32 },
    /// Die angeforderte Canvas-Groesse ueberschreitet die maximale 2D-Textur-Groesse des Devices.
    SizeExceedsTextureLimit {
        width: u32,
        height: u32,
        max_dimension: u32,
    },
    /// Die angeforderte Canvas-Groesse passt nicht in die benoetigten Byte-Berechnungen.
    SizeOverflow { width: u32, height: u32 },
    /// Der benoetigte Readback-Buffer ueberschreitet das Buffer-Limit des Devices.
    FrameExceedsBufferLimit {
        width: u32,
        height: u32,
        required_bytes: u64,
        max_buffer_size: u64,
    },
    /// Die Render-Szene wurde fuer eine andere Viewport-Groesse erzeugt.
    ViewportSizeMismatch {
        expected: [u32; 2],
        actual: [f32; 2],
    },
    /// Das Device-Wait auf den Readback ist in das Timeout gelaufen.
    ReadbackWaitTimedOut,
    /// Das Device lieferte beim Polling des Readbacks einen Fehler.
    ReadbackPollFailed(String),
    /// Der GPU-Readback konnte nicht gemappt werden.
    ReadbackMapFailed(String),
    /// Das Readback-Callback kam nicht rechtzeitig zurueck.
    ReadbackCallbackTimedOut,
    /// Das Readback-Callback lieferte kein Ergebnis.
    ReadbackChannelClosed,
}

impl fmt::Display for CanvasError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::InvalidSize { width, height } => {
                write!(f, "canvas size must be positive, got {width}x{height}")
            }
            Self::SizeExceedsTextureLimit {
                width,
                height,
                max_dimension,
            } => write!(
                f,
                "canvas size {width}x{height} exceeds device texture limit {max_dimension}"
            ),
            Self::SizeOverflow { width, height } => write!(
                f,
                "canvas size {width}x{height} overflows internal byte calculations"
            ),
            Self::FrameExceedsBufferLimit {
                width,
                height,
                required_bytes,
                max_buffer_size,
            } => write!(
                f,
                "canvas size {width}x{height} requires {required_bytes} bytes, exceeding device buffer limit {max_buffer_size}"
            ),
            Self::ViewportSizeMismatch { expected, actual } => write!(
                f,
                "render scene viewport must match canvas size (expected {}x{}, got {}x{})",
                expected[0], expected[1], actual[0], actual[1]
            ),
            Self::ReadbackWaitTimedOut => {
                write!(f, "canvas readback wait timed out")
            }
            Self::ReadbackPollFailed(message) => {
                write!(f, "canvas readback poll failed: {message}")
            }
            Self::ReadbackMapFailed(message) => {
                write!(f, "canvas readback mapping failed: {message}")
            }
            Self::ReadbackCallbackTimedOut => {
                write!(f, "canvas readback callback timed out")
            }
            Self::ReadbackChannelClosed => write!(f, "canvas readback channel closed"),
        }
    }
}

impl std::error::Error for CanvasError {}

/// Gemeinsame Offscreen-Canvas-Runtime fuer Hosts mit RGBA-Readback.
pub struct CanvasRuntime {
    renderer: Renderer,
    layout: CanvasLayout,
    color_texture: wgpu::Texture,
    color_view: wgpu::TextureView,
    readback_buffer: wgpu::Buffer,
    last_background_asset_revision: u64,
    last_background_transform_revision: u64,
    last_frame: Option<CanvasFrame>,
}

impl CanvasRuntime {
    /// Erstellt eine neue Offscreen-Canvas-Runtime.
    pub fn new(
        device: &wgpu::Device,
        queue: &wgpu::Queue,
        size: [u32; 2],
    ) -> Result<Self, CanvasError> {
        let layout = compute_canvas_layout(size, &device.limits())?;

        let renderer = Renderer::new(
            device,
            queue,
            RendererTargetConfig::new(CANVAS_COLOR_FORMAT, CANVAS_SAMPLE_COUNT),
        );
        let (color_texture, color_view) = create_color_target(device, layout.size);
        let readback_buffer = create_readback_buffer(device, layout.readback_buffer_size);

        Ok(Self {
            renderer,
            layout,
            color_texture,
            color_view,
            readback_buffer,
            last_background_asset_revision: 0,
            last_background_transform_revision: 0,
            last_frame: None,
        })
    }

    /// Aendert die Groesse des Offscreen-Targets.
    pub fn resize(&mut self, device: &wgpu::Device, size: [u32; 2]) -> Result<(), CanvasError> {
        let layout = compute_canvas_layout(size, &device.limits())?;
        if self.layout.size == layout.size {
            return Ok(());
        }

        let (color_texture, color_view) = create_color_target(device, layout.size);
        let readback_buffer = create_readback_buffer(device, layout.readback_buffer_size);

        self.layout = layout;
        self.color_texture = color_texture;
        self.color_view = color_view;
        self.readback_buffer = readback_buffer;
        self.last_frame = None;

        Ok(())
    }

    /// Rendert Szene plus Assets in ein dicht gepacktes RGBA-Frame.
    pub fn render_frame(
        &mut self,
        device: &wgpu::Device,
        queue: &wgpu::Queue,
        scene: &RenderScene,
        assets: &RenderAssetsSnapshot,
    ) -> Result<&CanvasFrame, CanvasError> {
        let expected_viewport = [self.layout.size[0] as f32, self.layout.size[1] as f32];
        if scene.viewport_size() != expected_viewport {
            return Err(CanvasError::ViewportSizeMismatch {
                expected: self.layout.size,
                actual: scene.viewport_size(),
            });
        }

        self.sync_background_asset(device, queue, assets);

        let mut encoder = device.create_command_encoder(&wgpu::CommandEncoderDescriptor {
            label: Some("Offscreen Canvas Encoder"),
        });

        {
            let mut render_pass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
                label: Some("Offscreen Canvas Pass"),
                color_attachments: &[Some(wgpu::RenderPassColorAttachment {
                    view: &self.color_view,
                    depth_slice: None,
                    resolve_target: None,
                    ops: wgpu::Operations {
                        load: wgpu::LoadOp::Clear(wgpu::Color::TRANSPARENT),
                        store: wgpu::StoreOp::Store,
                    },
                })],
                depth_stencil_attachment: None,
                occlusion_query_set: None,
                timestamp_writes: None,
                multiview_mask: None,
            });

            self.renderer
                .render_scene(device, queue, &mut render_pass, scene);
        }

        encoder.copy_texture_to_buffer(
            self.color_texture.as_image_copy(),
            wgpu::TexelCopyBufferInfo {
                buffer: &self.readback_buffer,
                layout: wgpu::TexelCopyBufferLayout {
                    offset: 0,
                    bytes_per_row: Some(self.layout.padded_bytes_per_row),
                    rows_per_image: Some(self.layout.size[1]),
                },
            },
            texture_extent(self.layout.size),
        );

        let submission_index = queue.submit(Some(encoder.finish()));

        let layout = self.layout;
        let frame_info = layout.frame_info();
        let readback_buffer = &self.readback_buffer;
        let frame = self.last_frame.get_or_insert_with(|| CanvasFrame {
            info: frame_info,
            pixels: vec![0; layout.frame_byte_len],
        });
        frame.info = frame_info;
        if frame.pixels.len() != layout.frame_byte_len {
            frame.pixels.resize(layout.frame_byte_len, 0);
        }
        read_tight_rgba_frame(device, readback_buffer, frame, layout, submission_index)?;

        Ok(self
            .last_frame
            .as_ref()
            .expect("canvas frame must exist after successful render"))
    }

    /// Liefert den zuletzt erfolgreich gerenderten Frame.
    pub fn frame(&self) -> Option<&CanvasFrame> {
        self.last_frame.as_ref()
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

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
struct CanvasLayout {
    size: [u32; 2],
    tight_bytes_per_row: u32,
    padded_bytes_per_row: u32,
    readback_buffer_size: u64,
    frame_byte_len: usize,
}

impl CanvasLayout {
    fn frame_info(self) -> CanvasFrameInfo {
        CanvasFrameInfo {
            width: self.size[0],
            height: self.size[1],
            bytes_per_row: self.tight_bytes_per_row,
            pixel_format: CanvasPixelFormat::Rgba8Srgb,
            alpha_mode: CanvasAlphaMode::Premultiplied,
        }
    }
}

fn compute_canvas_layout(
    size: [u32; 2],
    limits: &wgpu::Limits,
) -> Result<CanvasLayout, CanvasError> {
    if size[0] == 0 || size[1] == 0 {
        return Err(CanvasError::InvalidSize {
            width: size[0],
            height: size[1],
        });
    }

    if size[0] > limits.max_texture_dimension_2d || size[1] > limits.max_texture_dimension_2d {
        return Err(CanvasError::SizeExceedsTextureLimit {
            width: size[0],
            height: size[1],
            max_dimension: limits.max_texture_dimension_2d,
        });
    }

    let width = u64::from(size[0]);
    let height = u64::from(size[1]);
    let bytes_per_pixel = u64::from(CANVAS_BYTES_PER_PIXEL);
    let alignment = u64::from(wgpu::COPY_BYTES_PER_ROW_ALIGNMENT);

    let tight_bytes_per_row =
        width
            .checked_mul(bytes_per_pixel)
            .ok_or(CanvasError::SizeOverflow {
                width: size[0],
                height: size[1],
            })?;
    let padded_bytes_per_row = tight_bytes_per_row
        .div_ceil(alignment)
        .checked_mul(alignment)
        .ok_or(CanvasError::SizeOverflow {
            width: size[0],
            height: size[1],
        })?;
    let readback_buffer_size =
        padded_bytes_per_row
            .checked_mul(height)
            .ok_or(CanvasError::SizeOverflow {
                width: size[0],
                height: size[1],
            })?;
    let frame_byte_len =
        tight_bytes_per_row
            .checked_mul(height)
            .ok_or(CanvasError::SizeOverflow {
                width: size[0],
                height: size[1],
            })?;

    if readback_buffer_size > limits.max_buffer_size {
        return Err(CanvasError::FrameExceedsBufferLimit {
            width: size[0],
            height: size[1],
            required_bytes: readback_buffer_size,
            max_buffer_size: limits.max_buffer_size,
        });
    }

    Ok(CanvasLayout {
        size,
        tight_bytes_per_row: u32::try_from(tight_bytes_per_row).map_err(|_| {
            CanvasError::SizeOverflow {
                width: size[0],
                height: size[1],
            }
        })?,
        padded_bytes_per_row: u32::try_from(padded_bytes_per_row).map_err(|_| {
            CanvasError::SizeOverflow {
                width: size[0],
                height: size[1],
            }
        })?,
        readback_buffer_size,
        frame_byte_len: usize::try_from(frame_byte_len).map_err(|_| CanvasError::SizeOverflow {
            width: size[0],
            height: size[1],
        })?,
    })
}

fn texture_extent(size: [u32; 2]) -> wgpu::Extent3d {
    wgpu::Extent3d {
        width: size[0],
        height: size[1],
        depth_or_array_layers: 1,
    }
}

fn create_color_target(
    device: &wgpu::Device,
    size: [u32; 2],
) -> (wgpu::Texture, wgpu::TextureView) {
    let texture = device.create_texture(&wgpu::TextureDescriptor {
        label: Some("Offscreen Canvas Texture"),
        size: texture_extent(size),
        mip_level_count: 1,
        sample_count: 1,
        dimension: wgpu::TextureDimension::D2,
        format: CANVAS_COLOR_FORMAT,
        usage: wgpu::TextureUsages::RENDER_ATTACHMENT | wgpu::TextureUsages::COPY_SRC,
        view_formats: &[],
    });
    let view = texture.create_view(&wgpu::TextureViewDescriptor::default());
    (texture, view)
}

fn create_readback_buffer(device: &wgpu::Device, buffer_size: u64) -> wgpu::Buffer {
    device.create_buffer(&wgpu::BufferDescriptor {
        label: Some("Offscreen Canvas Readback Buffer"),
        size: buffer_size,
        usage: wgpu::BufferUsages::COPY_DST | wgpu::BufferUsages::MAP_READ,
        mapped_at_creation: false,
    })
}

fn wait_for_readback_poll(
    device: &wgpu::Device,
    submission_index: wgpu::SubmissionIndex,
) -> Result<(), CanvasError> {
    translate_readback_poll_result(device.poll(wgpu::PollType::Wait {
        submission_index: Some(submission_index),
        timeout: Some(CANVAS_READBACK_TIMEOUT),
    }))
}

fn translate_readback_poll_result(
    result: Result<wgpu::PollStatus, wgpu::PollError>,
) -> Result<(), CanvasError> {
    match result {
        Ok(status) if status.wait_finished() => Ok(()),
        Ok(status) => Err(CanvasError::ReadbackPollFailed(format!(
            "unexpected poll status: {status:?}"
        ))),
        Err(wgpu::PollError::Timeout) => Err(CanvasError::ReadbackWaitTimedOut),
        Err(error) => Err(CanvasError::ReadbackPollFailed(error.to_string())),
    }
}

fn wait_for_readback_callback(
    rx: &mpsc::Receiver<Result<(), wgpu::BufferAsyncError>>,
    timeout: Duration,
) -> Result<(), CanvasError> {
    match rx.recv_timeout(timeout) {
        Ok(Ok(())) => Ok(()),
        Ok(Err(error)) => Err(CanvasError::ReadbackMapFailed(error.to_string())),
        Err(RecvTimeoutError::Timeout) => Err(CanvasError::ReadbackCallbackTimedOut),
        Err(RecvTimeoutError::Disconnected) => Err(CanvasError::ReadbackChannelClosed),
    }
}

fn read_tight_rgba_frame(
    device: &wgpu::Device,
    buffer: &wgpu::Buffer,
    frame: &mut CanvasFrame,
    layout: CanvasLayout,
    submission_index: wgpu::SubmissionIndex,
) -> Result<(), CanvasError> {
    let slice = buffer.slice(..);
    let (tx, rx) = mpsc::channel();
    slice.map_async(wgpu::MapMode::Read, move |result| {
        let _ = tx.send(result);
    });

    wait_for_readback_poll(device, submission_index)?;
    wait_for_readback_callback(&rx, CANVAS_READBACK_TIMEOUT)?;

    let mapped = slice.get_mapped_range();

    for (row_index, dst_row) in frame
        .pixels
        .chunks_exact_mut(layout.tight_bytes_per_row as usize)
        .enumerate()
    {
        let src_start = row_index * layout.padded_bytes_per_row as usize;
        let src_end = src_start + layout.tight_bytes_per_row as usize;
        dst_row.copy_from_slice(&mapped[src_start..src_end]);
    }

    drop(mapped);
    buffer.unmap();

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::{
        compute_canvas_layout, translate_readback_poll_result, wait_for_readback_callback,
        CanvasAlphaMode, CanvasError, CanvasPixelFormat, CanvasRuntime,
    };
    use fs25_auto_drive_engine::shared::{
        EditorOptions, RenderAssetSnapshot, RenderAssetsSnapshot, RenderBackgroundAssetSnapshot,
        RenderBackgroundWorldBounds, RenderCamera, RenderQuality, RenderScene,
        RenderSceneFrameData,
    };
    use image::{DynamicImage, Rgba, RgbaImage};
    use indexmap::IndexSet;
    use std::sync::{mpsc, Arc};
    use std::time::Duration;

    fn create_test_gpu() -> Option<(wgpu::Instance, wgpu::Adapter, wgpu::Device, wgpu::Queue)> {
        let instance = wgpu::Instance::default();
        let adapter = pollster::block_on(instance.request_adapter(&wgpu::RequestAdapterOptions {
            power_preference: wgpu::PowerPreference::LowPower,
            compatible_surface: None,
            force_fallback_adapter: false,
        }))
        .ok()?;

        let (device, queue) = pollster::block_on(adapter.request_device(&wgpu::DeviceDescriptor {
            label: Some("CanvasRuntime Test Device"),
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
    fn canvas_runtime_renders_empty_transparent_frame() {
        let Some((_instance, _adapter, device, queue)) = create_test_gpu() else {
            return;
        };
        let size = [8, 8];
        let mut runtime =
            CanvasRuntime::new(&device, &queue, size).expect("canvas runtime must initialize");
        let scene = empty_scene(size, false);
        let assets = RenderAssetsSnapshot::default();

        let frame = runtime
            .render_frame(&device, &queue, &scene, &assets)
            .expect("empty frame must render");

        assert_eq!(frame.info.width, 8);
        assert_eq!(frame.info.height, 8);
        assert_eq!(frame.info.bytes_per_row, 32);
        assert_eq!(frame.info.pixel_format, CanvasPixelFormat::Rgba8Srgb);
        assert_eq!(frame.info.alpha_mode, CanvasAlphaMode::Premultiplied);
        assert_eq!(frame.info.byte_len(), frame.pixels.len());
        assert!(frame.pixels.iter().all(|byte| *byte == 0));
    }

    #[test]
    fn canvas_runtime_syncs_background_assets_into_render_output() {
        let Some((_instance, _adapter, device, queue)) = create_test_gpu() else {
            return;
        };
        let size = [16, 16];
        let mut runtime =
            CanvasRuntime::new(&device, &queue, size).expect("canvas runtime must initialize");
        let scene = empty_scene(size, true);

        let mut image = RgbaImage::new(1, 1);
        image.put_pixel(0, 0, Rgba([255, 64, 32, 255]));
        let assets = RenderAssetsSnapshot::new(
            1,
            1,
            vec![RenderAssetSnapshot::background(
                RenderBackgroundAssetSnapshot {
                    image: Arc::new(DynamicImage::ImageRgba8(image)),
                    world_bounds: RenderBackgroundWorldBounds::new(-1.0, 1.0, -1.0, 1.0),
                    scale: 1.0,
                    asset_revision: 1,
                    transform_revision: 1,
                },
            )],
        );

        let frame = runtime
            .render_frame(&device, &queue, &scene, &assets)
            .expect("background frame must render");

        assert!(frame.pixels.chunks_exact(4).any(|pixel| pixel[0] > 0));
        assert!(frame
            .pixels
            .chunks_exact(4)
            .any(|pixel| pixel[0] >= pixel[1] && pixel[0] >= pixel[2] && pixel[3] > 0));
    }

    #[test]
    fn canvas_runtime_resize_updates_frame_dimensions() {
        let Some((_instance, _adapter, device, queue)) = create_test_gpu() else {
            return;
        };
        let mut runtime =
            CanvasRuntime::new(&device, &queue, [8, 8]).expect("canvas runtime must initialize");
        runtime
            .resize(&device, [12, 6])
            .expect("canvas runtime must resize");

        let scene = empty_scene([12, 6], false);
        let assets = RenderAssetsSnapshot::default();
        let frame = runtime
            .render_frame(&device, &queue, &scene, &assets)
            .expect("resized frame must render");

        assert_eq!(frame.info.width, 12);
        assert_eq!(frame.info.height, 6);
        assert_eq!(frame.info.bytes_per_row, 48);
        assert_eq!(frame.pixels.len(), 288);
    }

    #[test]
    fn canvas_layout_rejects_zero_oversize_and_overflow() {
        let tight_limit = wgpu::Limits {
            max_texture_dimension_2d: 64,
            max_buffer_size: 4096,
            ..wgpu::Limits::downlevel_defaults()
        };

        assert!(matches!(
            compute_canvas_layout([0, 8], &tight_limit),
            Err(CanvasError::InvalidSize {
                width: 0,
                height: 8
            })
        ));
        assert!(matches!(
            compute_canvas_layout([65, 8], &tight_limit),
            Err(CanvasError::SizeExceedsTextureLimit {
                width: 65,
                height: 8,
                max_dimension: 64,
            })
        ));

        let overflow_limit = wgpu::Limits {
            max_texture_dimension_2d: u32::MAX,
            max_buffer_size: u64::MAX,
            ..wgpu::Limits::downlevel_defaults()
        };
        assert!(matches!(
            compute_canvas_layout([u32::MAX, 1], &overflow_limit),
            Err(CanvasError::SizeOverflow {
                width: u32::MAX,
                height: 1,
            })
        ));
        assert!(matches!(
            compute_canvas_layout([64, 64], &tight_limit),
            Err(CanvasError::FrameExceedsBufferLimit {
                width: 64,
                height: 64,
                ..
            })
        ));
    }

    #[test]
    fn readback_wait_helpers_report_timeout_and_closed_channel() {
        assert!(matches!(
            translate_readback_poll_result(Err(wgpu::PollError::Timeout)),
            Err(CanvasError::ReadbackWaitTimedOut)
        ));
        assert!(matches!(
            translate_readback_poll_result(Ok(wgpu::PollStatus::Poll)),
            Err(CanvasError::ReadbackPollFailed(_))
        ));

        let (_tx, rx) = mpsc::channel();
        assert!(matches!(
            wait_for_readback_callback(&rx, Duration::ZERO),
            Err(CanvasError::ReadbackCallbackTimedOut)
        ));

        let (tx, rx) = mpsc::channel();
        drop(tx);
        assert!(matches!(
            wait_for_readback_callback(&rx, Duration::ZERO),
            Err(CanvasError::ReadbackChannelClosed)
        ));
    }
}
