//! Interne Stage-Pipeline fuer das ColorPathTool.

use crate::core::simplify_polyline;
use crate::shared::spline_geometry::resample_by_distance;
use std::sync::Arc;

use super::sampling::{
    build_color_palette, build_exact_color_set, extract_boundary_segments_from_mask,
    flood_fill_color_mask_from_rgb, prepare_mask_for_skeleton, world_to_pixel,
};
use super::skeleton::extract_network_from_mask;
use super::state::{
    CachedRgbImage, ColorPathMask, ColorPathTool, MatchingCacheKey, MatchingSpec, PreparedSegment,
    PreparedSegmentsCacheKey, PreviewCoreCacheKey, PreviewData, SamplingPreviewCacheKey,
    SamplingPreviewData, Vec2CacheKey,
};

fn bump_revision(revision: &mut u64) {
    *revision = revision.wrapping_add(1);
    if *revision == 0 {
        *revision = 1;
    }
}

impl ColorPathTool {
    /// Markiert veraenderte Sampling-Eingaben, ohne sofort teure Downstream-Stages zu verwerfen.
    pub(super) fn mark_sampling_input_changed(&mut self) {
        bump_revision(&mut self.cache.sampling_revision);
    }

    /// Berechnet Stage B aus den aktuellen Sampling-Eingaben neu.
    pub(super) fn refresh_matching_spec(&mut self) {
        let key = self.matching_cache_key();
        if self.cache.matching_key == Some(key) {
            return;
        }

        let palette = if self.config.exact_color_match {
            build_exact_color_set(&self.sampling.sampled_colors)
        } else {
            build_color_palette(&self.sampling.sampled_colors, 8)
        };
        let tolerance = if self.config.exact_color_match {
            0.0
        } else {
            self.config.color_tolerance
        };

        let next_matching = MatchingSpec { tolerance, palette };
        let matching_changed = self.matching != next_matching;

        self.matching = next_matching;
        self.cache.matching_key = Some(key);

        if matching_changed {
            bump_revision(&mut self.cache.matching_revision);
            self.clear_sampling_preview();
        }
    }

    /// Berechnet Stage C aus Sampling-Input und Matching-Spezifikation neu.
    pub(super) fn rebuild_sampling_preview(&mut self) {
        self.refresh_matching_spec();

        let Some(key) = self.sampling_preview_cache_key() else {
            self.clear_sampling_preview();
            return;
        };

        if self.cache.sampling_preview_key == Some(key) && self.sampling_preview.is_some() {
            return;
        }

        let Some(sampling_preview) = self.compute_sampling_preview() else {
            self.clear_sampling_preview();
            return;
        };

        self.sampling_preview = Some(sampling_preview);
        self.cache.sampling_preview_key = Some(key);
        bump_revision(&mut self.cache.sampling_preview_revision);
        self.clear_preview_pipeline();
    }

    /// Fuehrt die Stages C-F aus den aktuellen Tool-Eingaben neu aus.
    pub(super) fn rebuild_preview_pipeline(&mut self) -> bool {
        self.ensure_prepared_segments()
    }

    /// Fuehrt die Stages D-F aus den bereits vorliegenden Stage-C-Artefakten aus.
    pub(super) fn rebuild_preview_from_sampling_artifacts(&mut self) -> bool {
        self.ensure_prepared_segments()
    }

    /// Berechnet Stage F aus dem vorhandenen Netz mit neuer Preview-Konfiguration neu.
    pub(super) fn rebuild_prepared_segments(&mut self) {
        let _ = self.ensure_prepared_segments();
    }

    /// Verwirft alle Stage-D-bis-F-Artefakte, behaelt aber Sampling-Input und Stage C.
    pub(super) fn clear_preview_pipeline(&mut self) {
        self.preview_data = None;
        self.cache.preview_core_key = None;
        self.cache.prepared_segments_key = None;
    }

    pub(super) fn clear_sampling_preview(&mut self) {
        self.sampling_preview = None;
        self.cache.sampling_preview_key = None;
        self.clear_preview_pipeline();
    }

    fn matching_cache_key(&self) -> MatchingCacheKey {
        MatchingCacheKey {
            sampling_revision: self.cache.sampling_revision,
            exact_color_match: self.config.exact_color_match,
            color_tolerance_bits: if self.config.exact_color_match {
                0
            } else {
                self.config.color_tolerance.to_bits()
            },
        }
    }

    fn sampling_preview_cache_key(&self) -> Option<SamplingPreviewCacheKey> {
        let background_image_id = self.background_image_id()?;
        let lasso_start_world = Vec2CacheKey::from(self.sampling.lasso_start_world?);
        if self.matching.is_empty() {
            return None;
        }

        Some(SamplingPreviewCacheKey {
            background_image_id,
            map_size_bits: self.map_size.to_bits(),
            lasso_start_world,
            matching_revision: self.cache.matching_revision,
        })
    }

    fn preview_core_cache_key(&self) -> Option<PreviewCoreCacheKey> {
        self.sampling_preview.as_ref()?;
        Some(PreviewCoreCacheKey {
            sampling_preview_revision: self.cache.sampling_preview_revision,
            noise_filter: self.config.noise_filter,
        })
    }

    fn prepared_segments_cache_key(&self) -> Option<PreparedSegmentsCacheKey> {
        self.preview_data.as_ref()?;
        Some(PreparedSegmentsCacheKey {
            preview_core_revision: self.cache.preview_core_revision,
            simplify_tolerance_bits: self.config.simplify_tolerance.to_bits(),
            node_spacing_bits: self.config.node_spacing.to_bits(),
            junction_radius_bits: self.config.junction_radius.to_bits(),
        })
    }

    fn background_image_id(&self) -> Option<usize> {
        self.background_image
            .as_ref()
            .map(|image| Arc::as_ptr(image) as usize)
    }

    fn cached_rgb_image(&mut self) -> Option<Arc<image::RgbImage>> {
        let background_image_id = self.background_image_id()?;

        if let Some(cached) = self.cache.rgb_image.as_ref()
            && cached.background_image_id == background_image_id
        {
            return Some(Arc::clone(&cached.image));
        }

        let image = self.background_image.as_ref()?;
        let rgb_image = Arc::new(image.to_rgb8());
        self.cache.rgb_image = Some(CachedRgbImage {
            background_image_id,
            image: Arc::clone(&rgb_image),
        });
        Some(rgb_image)
    }

    fn ensure_preview_core(&mut self) -> bool {
        self.rebuild_sampling_preview();

        let Some(key) = self.preview_core_cache_key() else {
            self.clear_preview_pipeline();
            return false;
        };

        if self.cache.preview_core_key == Some(key) && self.preview_data.is_some() {
            return true;
        }

        let Some(sampling_preview) = self.sampling_preview.as_ref() else {
            self.clear_preview_pipeline();
            return false;
        };
        if sampling_preview.input_mask.is_empty() {
            self.clear_preview_pipeline();
            return false;
        }

        let prepared_pixels = prepare_mask_for_skeleton(
            &sampling_preview.input_mask.pixels,
            sampling_preview.input_mask.width as usize,
            sampling_preview.input_mask.height as usize,
            self.config.noise_filter,
        );
        let prepared_mask = ColorPathMask::new(
            prepared_pixels,
            sampling_preview.input_mask.width,
            sampling_preview.input_mask.height,
        );
        let start_hint = Some((
            sampling_preview.start_pixel.0 as usize,
            sampling_preview.start_pixel.1 as usize,
        ));
        let network = extract_network_from_mask(
            &prepared_mask.pixels,
            prepared_mask.width,
            prepared_mask.height,
            self.map_size,
            start_hint,
        );

        log::info!(
            "Pipeline complete: {} junctions, {} open ends, {} segments",
            network.junction_count(),
            network.open_end_count(),
            network.segments.len()
        );

        if network.is_empty() {
            self.clear_preview_pipeline();
            return false;
        }

        self.preview_data = Some(PreviewData {
            prepared_mask,
            network,
            prepared_segments: Vec::new(),
        });
        self.cache.preview_core_key = Some(key);
        bump_revision(&mut self.cache.preview_core_revision);
        self.cache.prepared_segments_key = None;
        true
    }

    fn ensure_prepared_segments(&mut self) -> bool {
        if self.preview_data.is_none() {
            if !self.ensure_preview_core() {
                return false;
            }
        } else if let Some(preview_core_key) = self.preview_core_cache_key()
            && self.cache.preview_core_key != Some(preview_core_key)
            && !self.ensure_preview_core()
        {
            return false;
        }

        let Some(key) = self.prepared_segments_cache_key() else {
            self.cache.prepared_segments_key = None;
            return false;
        };

        if self.cache.prepared_segments_key == Some(key) {
            return self
                .preview_data
                .as_ref()
                .is_some_and(|preview| !preview.prepared_segments.is_empty());
        }

        let Some(preview_data) = self.preview_data.as_mut() else {
            self.cache.prepared_segments_key = None;
            return false;
        };

        preview_data.prepared_segments = prepare_segments(
            &preview_data.network,
            self.config.simplify_tolerance,
            self.config.node_spacing,
            self.config.junction_radius,
        );

        log::info!(
            "Netz-Vorschau aktualisiert: {} Knoten, {} Segmente, {} Preview-Segmente",
            preview_data.network.nodes.len(),
            preview_data.network.segments.len(),
            preview_data.prepared_segments.len()
        );

        let has_segments = !preview_data.prepared_segments.is_empty();

        self.cache.prepared_segments_key = Some(key);
        bump_revision(&mut self.cache.prepared_segments_revision);
        has_segments
    }

    fn compute_sampling_preview(&mut self) -> Option<SamplingPreviewData> {
        let (img_w, img_h) = {
            let image = self.background_image.as_ref()?;
            (image.width(), image.height())
        };
        let rgb_image = self.cached_rgb_image()?;
        let lasso_start = self.sampling.lasso_start_world?;
        if self.matching.is_empty() {
            return None;
        }

        let start_pixel = world_to_pixel(lasso_start, self.map_size, img_w, img_h);
        let (mask, width, height) = flood_fill_color_mask_from_rgb(
            &rgb_image,
            &self.matching.palette,
            self.matching.tolerance,
            start_pixel,
        );
        let input_mask = ColorPathMask::new(mask, width, height);
        let boundary_segments = extract_boundary_segments_from_mask(
            &input_mask.pixels,
            input_mask.width,
            input_mask.height,
            self.map_size,
        );

        Some(SamplingPreviewData {
            input_mask,
            boundary_segments,
            start_pixel,
        })
    }
}

fn prepare_segments(
    network: &super::skeleton::SkeletonNetwork,
    simplify_tolerance: f32,
    node_spacing: f32,
    junction_radius: f32,
) -> Vec<PreparedSegment> {
    let mut prepared_segments = Vec::with_capacity(network.segments.len());

    for segment in &network.segments {
        if segment.start_node == segment.end_node {
            continue;
        }

        let simplified = simplify_polyline(&segment.polyline, simplify_tolerance);
        let Some(trimmed_nodes) =
            trim_segment_near_junctions(network, segment, &simplified, junction_radius)
        else {
            continue;
        };
        if trimmed_nodes.len() < 2 {
            continue;
        }
        let resampled_nodes = resample_by_distance(&trimmed_nodes, node_spacing);
        if resampled_nodes.len() < 2 {
            continue;
        }

        prepared_segments.push(PreparedSegment {
            start_node: segment.start_node,
            end_node: segment.end_node,
            resampled_nodes,
        });
    }

    prepared_segments
}

fn trim_segment_near_junctions(
    network: &super::skeleton::SkeletonNetwork,
    segment: &super::skeleton::SkeletonGraphSegment,
    nodes: &[glam::Vec2],
    junction_radius: f32,
) -> Option<Vec<glam::Vec2>> {
    if nodes.len() < 2 {
        return None;
    }

    let mut trimmed = nodes.to_vec();
    if junction_radius > 0.0 {
        if is_junction_node(network, segment.start_node) {
            trim_from_start(&mut trimmed, junction_radius);
        }
        if is_junction_node(network, segment.end_node) {
            trim_from_end(&mut trimmed, junction_radius);
        }
    }

    compact_consecutive_duplicates(&mut trimmed);
    let first = *trimmed.first()?;
    let last = *trimmed.last()?;
    if first.distance_squared(last) <= f32::EPSILON {
        return None;
    }

    Some(trimmed)
}

fn is_junction_node(network: &super::skeleton::SkeletonNetwork, node_index: usize) -> bool {
    network
        .nodes
        .get(node_index)
        .is_some_and(|node| node.kind == super::skeleton::SkeletonGraphNodeKind::Junction)
}

fn trim_from_start(nodes: &mut Vec<glam::Vec2>, radius: f32) {
    if nodes.len() < 3 {
        return;
    }

    let anchor = nodes[0];
    let first_outside = (1..nodes.len() - 1)
        .find(|&idx| nodes[idx].distance(anchor) > radius)
        .unwrap_or(nodes.len() - 1);

    if first_outside > 1 {
        nodes.drain(1..first_outside);
    }
}

fn trim_from_end(nodes: &mut Vec<glam::Vec2>, radius: f32) {
    if nodes.len() < 3 {
        return;
    }

    let anchor = *nodes.last().expect("Segmentende fehlt");
    let last_outside = (1..nodes.len() - 1)
        .rev()
        .find(|&idx| nodes[idx].distance(anchor) > radius)
        .unwrap_or(0);

    let keep_until = if last_outside == 0 {
        1
    } else {
        last_outside + 1
    };
    if keep_until < nodes.len() - 1 {
        nodes.drain(keep_until..nodes.len() - 1);
    }
}

fn compact_consecutive_duplicates(nodes: &mut Vec<glam::Vec2>) {
    nodes.dedup_by(|a, b| a.distance_squared(*b) <= f32::EPSILON);
}

#[cfg(test)]
mod tests {
    use std::sync::Arc;

    use glam::Vec2;
    use image::{DynamicImage, Rgb, RgbImage};

    use super::*;
    use crate::app::tools::color_path::sampling::pixel_to_world;

    fn build_test_image() -> DynamicImage {
        DynamicImage::ImageRgb8(RgbImage::from_fn(10, 10, |x, _| {
            if x < 8 {
                Rgb([200, 0, 0])
            } else {
                Rgb([0, 200, 0])
            }
        }))
    }

    #[test]
    fn preview_pipeline_reuses_sampling_mask_as_stage_input() {
        let image = build_test_image();
        let mut tool = ColorPathTool::new();
        tool.background_image = Some(Arc::new(image));
        tool.map_size = 10.0;
        tool.sampling.sampled_colors = vec![[200, 0, 0]];
        tool.sampling.lasso_start_world = Some(pixel_to_world(0, 0, tool.map_size, 10, 10));
        tool.mark_sampling_input_changed();

        tool.rebuild_sampling_preview();
        let sampling_preview_revision = tool.cache.sampling_preview_revision;
        let stage_c = tool
            .sampling_preview
            .as_ref()
            .expect("Sampling-Preview sollte vorhanden sein")
            .input_mask
            .clone();

        assert!(tool.rebuild_preview_from_sampling_artifacts());

        let preview_data = tool
            .preview_data
            .as_ref()
            .expect("Preview-Daten sollten vorhanden sein");
        let expected_prepared = prepare_mask_for_skeleton(
            &stage_c.pixels,
            stage_c.width as usize,
            stage_c.height as usize,
            tool.config.noise_filter,
        );

        assert_eq!(
            tool.sampling_preview
                .as_ref()
                .expect("Sampling-Preview bleibt erhalten")
                .input_mask,
            stage_c
        );
        assert_eq!(preview_data.prepared_mask.pixels, expected_prepared);
        assert_eq!(
            tool.cache.sampling_preview_revision,
            sampling_preview_revision
        );
    }

    #[test]
    fn rebuild_prepared_segments_uses_existing_network_without_rebuilding_mask() {
        let mut tool = ColorPathTool::new();
        tool.preview_data = Some(PreviewData {
            prepared_mask: ColorPathMask::new(vec![true; 9], 3, 3),
            network: super::super::skeleton::SkeletonNetwork {
                nodes: vec![
                    super::super::skeleton::SkeletonGraphNode {
                        kind: super::super::skeleton::SkeletonGraphNodeKind::OpenEnd,
                        pixel_position: Vec2::new(0.0, 1.0),
                        world_position: Vec2::new(0.0, 0.0),
                    },
                    super::super::skeleton::SkeletonGraphNode {
                        kind: super::super::skeleton::SkeletonGraphNodeKind::OpenEnd,
                        pixel_position: Vec2::new(2.0, 1.0),
                        world_position: Vec2::new(10.0, 0.0),
                    },
                ],
                segments: vec![super::super::skeleton::SkeletonGraphSegment {
                    start_node: 0,
                    end_node: 1,
                    polyline: vec![
                        Vec2::new(0.0, 0.0),
                        Vec2::new(5.0, 0.0),
                        Vec2::new(10.0, 0.0),
                    ],
                }],
            },
            prepared_segments: Vec::new(),
        });
        tool.cache.preview_core_revision = 3;
        tool.config.node_spacing = 20.0;

        tool.rebuild_prepared_segments();

        let preview_data = tool
            .preview_data
            .as_ref()
            .expect("Preview-Daten sollten erhalten bleiben");
        assert_eq!(preview_data.network.segments.len(), 1);
        assert_eq!(preview_data.prepared_mask.width, 3);
        assert_eq!(preview_data.prepared_segments.len(), 1);
        assert_eq!(tool.cache.preview_core_revision, 3);
        assert_eq!(tool.cache.prepared_segments_revision, 1);
    }

    #[test]
    fn equivalent_matching_output_keeps_sampling_preview_cache_hot() {
        let image = build_test_image();
        let mut tool = ColorPathTool::new();
        tool.background_image = Some(Arc::new(image));
        tool.map_size = 10.0;
        tool.config.exact_color_match = false;
        tool.sampling.sampled_colors = vec![[10, 20, 30]];
        tool.sampling.lasso_start_world = Some(pixel_to_world(0, 0, tool.map_size, 10, 10));
        tool.mark_sampling_input_changed();

        tool.rebuild_sampling_preview();
        let matching_revision = tool.cache.matching_revision;
        let sampling_preview_revision = tool.cache.sampling_preview_revision;

        tool.sampling.sampled_colors.push([15, 23, 31]);
        tool.mark_sampling_input_changed();
        tool.rebuild_sampling_preview();

        assert_eq!(tool.cache.matching_revision, matching_revision);
        assert_eq!(
            tool.cache.sampling_preview_revision,
            sampling_preview_revision
        );
    }

    #[test]
    fn noise_filter_change_rebuilds_preview_core_but_not_sampling_preview() {
        let image = build_test_image();
        let mut tool = ColorPathTool::new();
        tool.background_image = Some(Arc::new(image));
        tool.map_size = 10.0;
        tool.sampling.sampled_colors = vec![[200, 0, 0]];
        tool.sampling.lasso_start_world = Some(pixel_to_world(0, 0, tool.map_size, 10, 10));
        tool.mark_sampling_input_changed();

        assert!(tool.rebuild_preview_pipeline());

        let sampling_preview_revision = tool.cache.sampling_preview_revision;
        let preview_core_revision = tool.cache.preview_core_revision;
        let prepared_segments_revision = tool.cache.prepared_segments_revision;

        tool.config.noise_filter = !tool.config.noise_filter;
        let _ = tool.rebuild_preview_from_sampling_artifacts();

        assert_eq!(
            tool.cache.sampling_preview_revision,
            sampling_preview_revision
        );
        assert!(tool.cache.preview_core_revision > preview_core_revision);
        assert!(tool.cache.prepared_segments_revision > prepared_segments_revision);
    }

    #[test]
    fn junction_radius_change_rebuilds_prepared_segments_only() {
        let image = build_test_image();
        let mut tool = ColorPathTool::new();
        tool.background_image = Some(Arc::new(image));
        tool.map_size = 10.0;
        tool.sampling.sampled_colors = vec![[200, 0, 0]];
        tool.sampling.lasso_start_world = Some(pixel_to_world(0, 0, tool.map_size, 10, 10));
        tool.mark_sampling_input_changed();

        assert!(tool.rebuild_preview_pipeline());

        let sampling_preview_revision = tool.cache.sampling_preview_revision;
        let preview_core_revision = tool.cache.preview_core_revision;
        let prepared_segments_revision = tool.cache.prepared_segments_revision;

        tool.config.junction_radius = 8.0;
        tool.rebuild_prepared_segments();

        assert_eq!(
            tool.cache.sampling_preview_revision,
            sampling_preview_revision
        );
        assert_eq!(tool.cache.preview_core_revision, preview_core_revision);
        assert!(tool.cache.prepared_segments_revision > prepared_segments_revision);
    }

    fn network_with_segment(
        start_kind: super::super::skeleton::SkeletonGraphNodeKind,
        end_kind: super::super::skeleton::SkeletonGraphNodeKind,
        polyline: Vec<Vec2>,
    ) -> super::super::skeleton::SkeletonNetwork {
        super::super::skeleton::SkeletonNetwork {
            nodes: vec![
                super::super::skeleton::SkeletonGraphNode {
                    kind: start_kind,
                    pixel_position: Vec2::new(0.0, 0.0),
                    world_position: *polyline.first().expect("Polyline ohne Startpunkt"),
                },
                super::super::skeleton::SkeletonGraphNode {
                    kind: end_kind,
                    pixel_position: Vec2::new(0.0, 0.0),
                    world_position: *polyline.last().expect("Polyline ohne Endpunkt"),
                },
            ],
            segments: vec![super::super::skeleton::SkeletonGraphSegment {
                start_node: 0,
                end_node: 1,
                polyline,
            }],
        }
    }

    #[test]
    fn junction_radius_trims_start_points_inside_radius() {
        let polyline = vec![
            Vec2::new(0.0, 0.0),
            Vec2::new(1.0, 0.0),
            Vec2::new(2.0, 0.0),
            Vec2::new(3.0, 0.0),
            Vec2::new(10.0, 0.0),
        ];
        let network = network_with_segment(
            super::super::skeleton::SkeletonGraphNodeKind::Junction,
            super::super::skeleton::SkeletonGraphNodeKind::OpenEnd,
            polyline,
        );

        let prepared = prepare_segments(&network, 0.0, 1.0, 2.5);

        assert_eq!(prepared.len(), 1);
        assert_eq!(prepared[0].resampled_nodes[0], Vec2::new(0.0, 0.0));
        assert_eq!(prepared[0].resampled_nodes[1], Vec2::new(3.0, 0.0));
        assert_eq!(
            *prepared[0]
                .resampled_nodes
                .last()
                .expect("Endpunkt muss vorhanden sein"),
            Vec2::new(10.0, 0.0)
        );
    }

    #[test]
    fn junction_radius_zero_does_not_trim_junction_segments() {
        let polyline = vec![
            Vec2::new(0.0, 0.0),
            Vec2::new(1.0, 0.0),
            Vec2::new(2.0, 0.0),
            Vec2::new(3.0, 0.0),
            Vec2::new(10.0, 0.0),
        ];
        let junction_network = network_with_segment(
            super::super::skeleton::SkeletonGraphNodeKind::Junction,
            super::super::skeleton::SkeletonGraphNodeKind::Junction,
            polyline.clone(),
        );
        let open_end_network = network_with_segment(
            super::super::skeleton::SkeletonGraphNodeKind::OpenEnd,
            super::super::skeleton::SkeletonGraphNodeKind::OpenEnd,
            polyline,
        );

        let prepared_junction = prepare_segments(&junction_network, 0.0, 1.0, 0.0);
        let prepared_open_end = prepare_segments(&open_end_network, 0.0, 1.0, 0.0);

        assert_eq!(prepared_junction.len(), 1);
        assert_eq!(prepared_open_end.len(), 1);
        assert_eq!(
            prepared_junction[0].resampled_nodes,
            prepared_open_end[0].resampled_nodes
        );
    }

    #[test]
    fn junction_radius_fallback_keeps_direct_connection_when_no_outside_point_exists() {
        let polyline = vec![
            Vec2::new(0.0, 0.0),
            Vec2::new(0.5, 0.0),
            Vec2::new(1.0, 0.0),
        ];
        let network = network_with_segment(
            super::super::skeleton::SkeletonGraphNodeKind::Junction,
            super::super::skeleton::SkeletonGraphNodeKind::OpenEnd,
            polyline,
        );

        let prepared = prepare_segments(&network, 0.0, 1.0, 10.0);

        assert_eq!(prepared.len(), 1);
        assert_eq!(prepared[0].resampled_nodes.len(), 2);
        assert_eq!(prepared[0].resampled_nodes[0], Vec2::new(0.0, 0.0));
        assert_eq!(prepared[0].resampled_nodes[1], Vec2::new(1.0, 0.0));
    }

    #[test]
    fn junction_radius_trims_end_points_inside_radius_for_single_sided_junction() {
        let polyline = vec![
            Vec2::new(0.0, 0.0),
            Vec2::new(7.0, 0.0),
            Vec2::new(8.0, 0.0),
            Vec2::new(9.0, 0.0),
            Vec2::new(10.0, 0.0),
        ];
        let network = network_with_segment(
            super::super::skeleton::SkeletonGraphNodeKind::OpenEnd,
            super::super::skeleton::SkeletonGraphNodeKind::Junction,
            polyline,
        );

        let prepared = prepare_segments(&network, 0.0, 1.0, 2.5);

        assert_eq!(prepared.len(), 1);
        let nodes = &prepared[0].resampled_nodes;
        assert_eq!(nodes[0], Vec2::new(0.0, 0.0));
        assert_eq!(
            *nodes.last().expect("Endpunkt muss vorhanden sein"),
            Vec2::new(10.0, 0.0)
        );
        assert!(
            nodes[nodes.len() - 2].distance(*nodes.last().expect("Endpunkt muss vorhanden sein"))
                > 2.5
        );
    }

    #[test]
    fn junction_radius_trims_both_ends_for_junction_to_junction_segments() {
        let polyline = vec![
            Vec2::new(0.0, 0.0),
            Vec2::new(1.0, 0.0),
            Vec2::new(3.0, 0.0),
            Vec2::new(7.0, 0.0),
            Vec2::new(9.0, 0.0),
            Vec2::new(10.0, 0.0),
        ];
        let network = network_with_segment(
            super::super::skeleton::SkeletonGraphNodeKind::Junction,
            super::super::skeleton::SkeletonGraphNodeKind::Junction,
            polyline,
        );

        let prepared = prepare_segments(&network, 0.0, 1.0, 2.5);

        assert_eq!(prepared.len(), 1);
        let nodes = &prepared[0].resampled_nodes;
        assert!(nodes.len() >= 3);
        assert_eq!(nodes[0], Vec2::new(0.0, 0.0));
        assert_eq!(
            *nodes.last().expect("Endpunkt muss vorhanden sein"),
            Vec2::new(10.0, 0.0)
        );

        // Erster Innenpunkt muss ausserhalb des Radius um die Start-Junction liegen.
        assert!(nodes[1].distance(nodes[0]) > 2.5);
        // Letzter Innenpunkt muss ausserhalb des Radius um die End-Junction liegen.
        assert!(
            nodes[nodes.len() - 2].distance(*nodes.last().expect("Endpunkt muss vorhanden sein"))
                > 2.5
        );
    }

    #[test]
    fn junction_radius_large_with_two_junctions_falls_back_to_direct_segment() {
        let polyline = vec![
            Vec2::new(0.0, 0.0),
            Vec2::new(0.5, 0.0),
            Vec2::new(1.0, 0.0),
        ];
        let network = network_with_segment(
            super::super::skeleton::SkeletonGraphNodeKind::Junction,
            super::super::skeleton::SkeletonGraphNodeKind::Junction,
            polyline,
        );

        let prepared = prepare_segments(&network, 0.0, 1.0, 10.0);

        assert_eq!(prepared.len(), 1);
        assert_eq!(prepared[0].resampled_nodes.len(), 2);
        assert_eq!(prepared[0].resampled_nodes[0], Vec2::new(0.0, 0.0));
        assert_eq!(prepared[0].resampled_nodes[1], Vec2::new(1.0, 0.0));
    }

    #[test]
    fn repeated_sampling_preview_reuses_cached_rgb_image() {
        let image = build_test_image();
        let mut tool = ColorPathTool::new();
        tool.background_image = Some(Arc::new(image));
        tool.map_size = 10.0;
        tool.sampling.sampled_colors = vec![[200, 0, 0]];
        tool.sampling.lasso_start_world = Some(pixel_to_world(0, 0, tool.map_size, 10, 10));
        tool.mark_sampling_input_changed();

        tool.rebuild_sampling_preview();
        let first_rgb = Arc::clone(
            &tool
                .cache
                .rgb_image
                .as_ref()
                .expect("RGB-Cache sollte aufgebaut werden")
                .image,
        );

        tool.rebuild_sampling_preview();
        let second_rgb = Arc::clone(
            &tool
                .cache
                .rgb_image
                .as_ref()
                .expect("RGB-Cache sollte erhalten bleiben")
                .image,
        );

        assert!(Arc::ptr_eq(&first_rgb, &second_rgb));
    }
}
