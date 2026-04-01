//! Interne Stage-Pipeline fuer das ColorPathTool.

use crate::core::simplify_polyline;
use crate::shared::spline_geometry::resample_by_distance;

use super::sampling::{
    build_color_palette, build_exact_color_set, extract_boundary_segments_from_mask,
    flood_fill_color_mask, prepare_mask_for_skeleton, world_to_pixel,
};
use super::skeleton::extract_network_from_mask;
use super::state::{
    ColorPathMask, ColorPathTool, MatchingSpec, PreparedSegment, PreviewData, SamplingPreviewData,
};

impl ColorPathTool {
    /// Berechnet Stage B aus den aktuellen Sampling-Eingaben neu.
    pub(super) fn refresh_matching_spec(&mut self) {
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

        self.matching = MatchingSpec { tolerance, palette };
    }

    /// Berechnet Stage C aus Sampling-Input und Matching-Spezifikation neu.
    pub(super) fn rebuild_sampling_preview(&mut self) {
        self.refresh_matching_spec();
        self.sampling_preview = self.compute_sampling_preview();
    }

    /// Fuehrt die Stages C-F aus den aktuellen Tool-Eingaben neu aus.
    pub(super) fn rebuild_preview_pipeline(&mut self) -> bool {
        self.rebuild_sampling_preview();
        self.rebuild_preview_from_sampling_artifacts()
    }

    /// Fuehrt die Stages D-F aus den bereits vorliegenden Stage-C-Artefakten aus.
    pub(super) fn rebuild_preview_from_sampling_artifacts(&mut self) -> bool {
        let Some(sampling_preview) = self.sampling_preview.as_ref() else {
            self.preview_data = None;
            return false;
        };
        if sampling_preview.input_mask.is_empty() {
            self.preview_data = None;
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
            self.preview_data = None;
            return false;
        }

        let prepared_segments = prepare_segments(
            &network,
            self.config.simplify_tolerance,
            self.config.node_spacing,
        );
        log::info!(
            "Netz-Vorschau: {} Knoten, {} Segmente, {} Preview-Segmente",
            network.nodes.len(),
            network.segments.len(),
            prepared_segments.len()
        );

        let has_segments = !prepared_segments.is_empty();
        self.preview_data = Some(PreviewData {
            prepared_mask,
            network,
            prepared_segments,
        });
        has_segments
    }

    /// Berechnet Stage F aus dem vorhandenen Netz mit neuer Preview-Konfiguration neu.
    pub(super) fn rebuild_prepared_segments(&mut self) {
        let Some(preview_data) = self.preview_data.as_mut() else {
            return;
        };

        preview_data.prepared_segments = prepare_segments(
            &preview_data.network,
            self.config.simplify_tolerance,
            self.config.node_spacing,
        );

        log::info!(
            "Netz-Vorschau aktualisiert: {} Knoten, {} Segmente, {} Preview-Segmente",
            preview_data.network.nodes.len(),
            preview_data.network.segments.len(),
            preview_data.prepared_segments.len()
        );
    }

    /// Verwirft alle Stage-D-bis-F-Artefakte, behaelt aber Sampling-Input und Stage C.
    pub(super) fn clear_preview_pipeline(&mut self) {
        self.preview_data = None;
    }

    fn compute_sampling_preview(&self) -> Option<SamplingPreviewData> {
        let image = self.background_image.as_ref()?;
        let lasso_start = self.sampling.lasso_start_world?;
        if self.matching.is_empty() {
            return None;
        }

        let img_w = image.width();
        let img_h = image.height();
        let start_pixel = world_to_pixel(lasso_start, self.map_size, img_w, img_h);
        let (mask, width, height) = flood_fill_color_mask(
            image,
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
) -> Vec<PreparedSegment> {
    let mut prepared_segments = Vec::with_capacity(network.segments.len());

    for segment in &network.segments {
        let simplified = simplify_polyline(&segment.polyline, simplify_tolerance);
        let resampled_nodes = resample_by_distance(&simplified, node_spacing);
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

        tool.rebuild_sampling_preview();
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
        tool.config.node_spacing = 20.0;

        tool.rebuild_prepared_segments();

        let preview_data = tool
            .preview_data
            .as_ref()
            .expect("Preview-Daten sollten erhalten bleiben");
        assert_eq!(preview_data.network.segments.len(), 1);
        assert_eq!(preview_data.prepared_mask.width, 3);
        assert_eq!(preview_data.prepared_segments.len(), 1);
    }
}
