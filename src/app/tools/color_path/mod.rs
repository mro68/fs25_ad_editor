//! Farb-Pfad-Tool: erkennt Wege anhand der Farbe im Hintergrundbild.

mod config_ui;
mod lifecycle;
mod pipeline;
mod preview;
pub(crate) mod sampling;
pub(crate) mod skeleton;
mod state;

pub use state::ColorPathTool;

use crate::app::tools::{RouteToolCore, RouteToolLassoInput};
use crate::core::RoadMap;
use glam::Vec2;
use std::cell::RefCell;
use std::sync::Arc;

#[derive(Clone)]
struct StatsRgbCacheEntry {
    image_id: usize,
    rgb_image: Arc<image::RgbImage>,
}

thread_local! {
    static COLOR_PATH_STATS_RGB_CACHE: RefCell<Option<StatsRgbCacheEntry>> = const { RefCell::new(None) };
}

fn cached_rgb_image_for_stats(image: &image::DynamicImage) -> Arc<image::RgbImage> {
    let image_id = image as *const image::DynamicImage as usize;

    COLOR_PATH_STATS_RGB_CACHE.with(|cache| {
        let mut cache = cache.borrow_mut();
        if let Some(entry) = cache.as_ref() {
            if entry.image_id == image_id {
                return Arc::clone(&entry.rgb_image);
            }
        }

        let rgb_image = Arc::new(image.to_rgb8());
        *cache = Some(StatsRgbCacheEntry {
            image_id,
            rgb_image: Arc::clone(&rgb_image),
        });
        rgb_image
    })
}

/// Fuehrt die Kernpipeline des ColorPathTool fuer Benchmarks und Analysen aus.
///
/// Die Funktion kapselt Flood-Fill und Netzextraktion, ohne interne
/// Skelett-Typen nach aussen zu exponieren. Rueckgabe:
/// `(node_count, segment_count, junction_count, open_end_count)`.
pub fn compute_color_path_network_stats(
    image: &image::DynamicImage,
    palette: &[[u8; 3]],
    tolerance: f32,
    start_pixel: (u32, u32),
    noise_filter: bool,
    map_size: f32,
) -> (usize, usize, usize, usize) {
    let rgb_image = cached_rgb_image_for_stats(image);
    let (mask, width, height) =
        sampling::flood_fill_color_mask_from_rgb(&rgb_image, palette, tolerance, start_pixel);
    let prepared_mask =
        sampling::prepare_mask_for_skeleton(&mask, width as usize, height as usize, noise_filter);
    let start_hint = Some((start_pixel.0 as usize, start_pixel.1 as usize));
    let network =
        skeleton::extract_network_from_mask(&prepared_mask, width, height, map_size, start_hint);

    (
        network.nodes.len(),
        network.segments.len(),
        network.junction_count(),
        network.open_end_count(),
    )
}

/// Beobachtbare Kennzahlen einer einzelnen ColorPath-Benchmark-Aktion.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct ColorPathBenchmarkStats {
    /// Anzahl der Randsegmente der Stage-C-Sampling-Vorschau.
    pub boundary_segments: usize,
    /// Anzahl der Netz-Knoten aus Stage E.
    pub network_nodes: usize,
    /// Anzahl der Netz-Segmente aus Stage E.
    pub network_segments: usize,
    /// Anzahl der vorbereiteten Preview-Segmente aus Stage F.
    pub prepared_segments: usize,
    /// Revision der Sampling-Vorschau nach der Aktion.
    pub sampling_preview_revision: u64,
    /// Revision des Preview-Kerns nach der Aktion.
    pub preview_core_revision: u64,
    /// Revision der PreparedSegments nach der Aktion.
    pub prepared_segments_revision: u64,
}

#[derive(Clone, Copy)]
enum ColorPathBenchmarkKind {
    SamplingPreviewRebuild,
    ComputePipelineFromSampling,
    PreviewCoreRebuild,
    PreparedSegmentsRebuild,
}

/// Vorbereitete Benchmark-Aktion fuer einen konkreten Produktpfad des ColorPathTool.
///
/// Der Setup-Schritt kann ausserhalb des gemessenen Zeitfensters erfolgen.
/// `run()` fuehrt nur den eigentlichen Rebuild-Pfad aus.
#[derive(Clone)]
pub struct ColorPathBenchmarkAction {
    tool: ColorPathTool,
    kind: ColorPathBenchmarkKind,
}

impl ColorPathBenchmarkAction {
    /// Fuehrt die vorbereitete Produktaktion aus und liefert die beobachtbaren Kennzahlen.
    pub fn run(mut self) -> ColorPathBenchmarkStats {
        match self.kind {
            ColorPathBenchmarkKind::SamplingPreviewRebuild => {
                self.tool.on_matching_config_changed();
            }
            ColorPathBenchmarkKind::ComputePipelineFromSampling => {
                self.tool.compute_pipeline();
            }
            ColorPathBenchmarkKind::PreviewCoreRebuild => {
                self.tool.on_preview_core_config_changed();
            }
            ColorPathBenchmarkKind::PreparedSegmentsRebuild => {
                self.tool.on_preview_geometry_config_changed();
            }
        }

        benchmark_stats(&self.tool)
    }
}

/// Reproduzierbarer Setup-Harness fuer Criterion-Benchmarks des ColorPathTool.
///
/// Der Harness laeuft ueber denselben Produktpfad wie im Editor:
/// Hintergrundbild setzen, Sampling starten, per Lasso Farben erfassen und
/// optional die Preview-Pipeline berechnen. Daraus koennen gezielt die
/// relevanten Stage-Rebuild-Pfade vorbereitet werden.
#[derive(Clone)]
pub struct ColorPathBenchmarkHarness {
    sampling_state: ColorPathTool,
    preview_state: ColorPathTool,
    sampling_rebuild_tolerance: f32,
    geometry_rebuild_node_spacing: f32,
    geometry_rebuild_simplify_tolerance: f32,
}

impl ColorPathBenchmarkHarness {
    /// Erstellt Sampling- und Preview-Ausgangszustaende fuer reproduzierbare Benchmarks.
    pub fn new(
        image: Arc<image::DynamicImage>,
        lasso_polygon: Vec<Vec2>,
    ) -> Result<Self, &'static str> {
        if lasso_polygon.len() < 3 {
            return Err("Lasso-Polygon braucht mindestens drei Punkte");
        }

        let mut tool = ColorPathTool::new();
        tool.config.exact_color_match = false;
        tool.config.color_tolerance = 18.0;
        tool.config.node_spacing = 4.0;
        tool.config.simplify_tolerance = 0.75;
        tool.set_background_map_image(Some(image));

        let road_map = RoadMap::new(3);
        let start_pos = lasso_polygon
            .first()
            .copied()
            .ok_or("Lasso-Polygon ist leer")?;
        let _ = tool.on_click(start_pos, &road_map, false);
        let _ = RouteToolLassoInput::on_lasso_completed(&mut tool, lasso_polygon);

        if tool.sampling.sampled_colors.is_empty() {
            return Err("Lasso hat keine Farbsamples erzeugt");
        }
        if tool.sampling_preview.is_none() {
            return Err("Sampling-Preview konnte nicht aufgebaut werden");
        }

        let sampling_state = tool.clone();
        let mut preview_state = tool;
        preview_state.compute_pipeline();

        let has_prepared_segments = preview_state
            .preview_data
            .as_ref()
            .is_some_and(|preview| !preview.prepared_segments.is_empty());
        if !has_prepared_segments {
            return Err("Preview-Pipeline konnte keine PreparedSegments erzeugen");
        }

        Ok(Self {
            sampling_state,
            preview_state,
            sampling_rebuild_tolerance: 24.0,
            geometry_rebuild_node_spacing: 2.0,
            geometry_rebuild_simplify_tolerance: 0.25,
        })
    }

    /// Bereitet einen Matching-getriebenen Sampling-Preview-Rebuild vor (Stages B/C).
    pub fn sampling_preview_rebuild_action(&self) -> ColorPathBenchmarkAction {
        let mut tool = self.sampling_state.clone();
        tool.config.color_tolerance = self.sampling_rebuild_tolerance;
        ColorPathBenchmarkAction {
            tool,
            kind: ColorPathBenchmarkKind::SamplingPreviewRebuild,
        }
    }

    /// Bereitet den Berechnen-Button-Pfad aus der Sampling-Phase vor.
    pub fn compute_pipeline_action(&self) -> ColorPathBenchmarkAction {
        ColorPathBenchmarkAction {
            tool: self.sampling_state.clone(),
            kind: ColorPathBenchmarkKind::ComputePipelineFromSampling,
        }
    }

    /// Bereitet einen Preview-Core-Rebuild via `noise_filter`-Wechsel vor (Stages D-F).
    pub fn preview_core_rebuild_action(&self) -> ColorPathBenchmarkAction {
        let mut tool = self.preview_state.clone();
        tool.config.noise_filter = !tool.config.noise_filter;
        ColorPathBenchmarkAction {
            tool,
            kind: ColorPathBenchmarkKind::PreviewCoreRebuild,
        }
    }

    /// Bereitet einen reinen PreparedSegments-Rebuild via Geometrie-Slider vor (Stage F).
    pub fn prepared_segments_rebuild_action(&self) -> ColorPathBenchmarkAction {
        let mut tool = self.preview_state.clone();
        tool.config.node_spacing = self.geometry_rebuild_node_spacing;
        tool.config.simplify_tolerance = self.geometry_rebuild_simplify_tolerance;
        ColorPathBenchmarkAction {
            tool,
            kind: ColorPathBenchmarkKind::PreparedSegmentsRebuild,
        }
    }
}

fn benchmark_stats(tool: &ColorPathTool) -> ColorPathBenchmarkStats {
    let boundary_segments = tool
        .sampling_preview
        .as_ref()
        .map_or(0, |preview| preview.boundary_segments.len());
    let (network_nodes, network_segments, prepared_segments) = tool
        .preview_data
        .as_ref()
        .map(|preview| {
            (
                preview.network.nodes.len(),
                preview.network.segments.len(),
                preview.prepared_segments.len(),
            )
        })
        .unwrap_or((0, 0, 0));

    ColorPathBenchmarkStats {
        boundary_segments,
        network_nodes,
        network_segments,
        prepared_segments,
        sampling_preview_revision: tool.cache.sampling_preview_revision,
        preview_core_revision: tool.cache.preview_core_revision,
        prepared_segments_revision: tool.cache.prepared_segments_revision,
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use image::{DynamicImage, ImageBuffer, Rgb, RgbImage};

    fn build_test_image(size: u32) -> DynamicImage {
        let half = (size / 2) as i32;
        let thickness = (size / 18).max(6) as i32;
        let img: RgbImage = ImageBuffer::from_fn(size, size, |x, y| {
            let xi = x as i32;
            let yi = y as i32;
            let on_horizontal = (yi - half).abs() <= thickness && x > size / 8 && x < size * 7 / 8;
            let on_vertical = (xi - half).abs() <= thickness && y > size / 6 && y < size * 5 / 6;
            let on_diagonal = ((xi - yi) - half / 3).abs() <= thickness / 2 && x > size / 5;

            if on_horizontal || on_vertical || on_diagonal {
                let tint = ((x + y) % 7) as u8;
                Rgb([150 + tint, 120, 72u8.saturating_sub(tint / 2)])
            } else {
                let noise = ((x * 13 + y * 7) % 9) as u8;
                Rgb([58 + noise, 94 + noise, 56 + noise / 2])
            }
        });
        DynamicImage::ImageRgb8(img)
    }

    fn pixel_rect_to_world_polygon(
        size: u32,
        min_x: u32,
        min_y: u32,
        max_x: u32,
        max_y: u32,
    ) -> Vec<Vec2> {
        let half = size as f32 / 2.0;
        vec![
            Vec2::new(min_x as f32 - half, min_y as f32 - half),
            Vec2::new(max_x as f32 + 1.0 - half, min_y as f32 - half),
            Vec2::new(max_x as f32 + 1.0 - half, max_y as f32 + 1.0 - half),
            Vec2::new(min_x as f32 - half, max_y as f32 + 1.0 - half),
        ]
    }

    fn build_test_lasso(size: u32) -> Vec<Vec2> {
        let band = (size / 18).max(6);
        let center = size / 2;
        let max = size.saturating_sub(1);
        pixel_rect_to_world_polygon(
            size,
            center.saturating_sub(band),
            center.saturating_sub(band),
            (center + band).min(max),
            (center + band).min(max),
        )
    }

    fn build_harness() -> ColorPathBenchmarkHarness {
        ColorPathBenchmarkHarness::new(Arc::new(build_test_image(128)), build_test_lasso(128))
            .expect("Benchmark-Harness sollte aufgebaut werden")
    }

    #[test]
    fn sampling_preview_benchmark_action_rebuilds_stage_b_and_c() {
        let harness = build_harness();
        let action = harness.sampling_preview_rebuild_action();
        let before_sampling_preview = action.tool.cache.sampling_preview_revision;
        let before_preview_core = action.tool.cache.preview_core_revision;
        let before_prepared_segments = action.tool.cache.prepared_segments_revision;

        let stats = action.run();

        assert!(stats.boundary_segments > 0);
        assert!(stats.sampling_preview_revision > before_sampling_preview);
        assert_eq!(stats.preview_core_revision, before_preview_core);
        assert_eq!(stats.prepared_segments_revision, before_prepared_segments);
    }

    #[test]
    fn compute_pipeline_benchmark_action_reuses_stage_c_and_reaches_preview() {
        let harness = build_harness();
        let action = harness.compute_pipeline_action();
        let before_sampling_preview = action.tool.cache.sampling_preview_revision;

        let stats = action.run();

        assert!(stats.network_nodes > 0);
        assert!(stats.network_segments > 0);
        assert!(stats.prepared_segments > 0);
        assert_eq!(stats.sampling_preview_revision, before_sampling_preview);
        assert!(stats.preview_core_revision > 0);
        assert!(stats.prepared_segments_revision > 0);
    }

    #[test]
    fn preview_core_benchmark_action_keeps_stage_c_hot() {
        let harness = build_harness();
        let action = harness.preview_core_rebuild_action();
        let before_sampling_preview = action.tool.cache.sampling_preview_revision;
        let before_preview_core = action.tool.cache.preview_core_revision;
        let before_prepared_segments = action.tool.cache.prepared_segments_revision;

        let stats = action.run();

        assert!(stats.prepared_segments > 0);
        assert_eq!(stats.sampling_preview_revision, before_sampling_preview);
        assert!(stats.preview_core_revision > before_preview_core);
        assert!(stats.prepared_segments_revision > before_prepared_segments);
    }

    #[test]
    fn prepared_segments_benchmark_action_rebuilds_only_stage_f() {
        let harness = build_harness();
        let action = harness.prepared_segments_rebuild_action();
        let before_sampling_preview = action.tool.cache.sampling_preview_revision;
        let before_preview_core = action.tool.cache.preview_core_revision;
        let before_prepared_segments = action.tool.cache.prepared_segments_revision;

        let stats = action.run();

        assert!(stats.prepared_segments > 0);
        assert_eq!(stats.sampling_preview_revision, before_sampling_preview);
        assert_eq!(stats.preview_core_revision, before_preview_core);
        assert!(stats.prepared_segments_revision > before_prepared_segments);
    }
}
