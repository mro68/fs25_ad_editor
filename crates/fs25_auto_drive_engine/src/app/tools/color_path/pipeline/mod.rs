//! Interne Stage-Pipeline fuer das ColorPathTool.
//!
//! Die Pipeline ist in Teilmodule zerlegt, deren `impl ColorPathTool`-Bloecke sich
//! die Stages teilen:
//!
//! - [`matching`] — Stage B: Matching-Spezifikation aus Sampling-Farben.
//! - [`sampling_stage`] — Stage C: Sampling-Preview (Flood-Fill-Maske + Randsegmente).
//! - [`preview_core`] — Stages D/E: Maskenvorbereitung + Skelett-Extraktion.
//! - [`prepared`] — Stage F: Simplify + Junction-Trim + Resample.

mod matching;
mod prepared;
mod preview_core;
mod sampling_stage;

use crate::app::tools::color_path::state::ColorPathTool;

/// Erhoeht eine Revisionszaehler-Variable und ueberspringt dabei den Wert 0.
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

    /// Fuehrt die Stages D-F aus den bereits vorliegenden Stage-C-Artefakten aus.
    #[cfg(test)]
    pub(super) fn rebuild_preview_from_sampling_artifacts(&mut self) -> bool {
        self.ensure_prepared_segments()
    }

    /// Fuehrt nur Stage C-E aus und ueberspringt den Junction-Trim der Stage F.
    ///
    /// Zweck: CP-03 trennt die Wizard-Phasen `CenterlinePreview`/`JunctionEdit`
    /// (reine Skelett-Vorschau) von `Finalize` (mit Junction-Radius-Begradigung).
    /// Die Methode liefert `true`, wenn mindestens Stage E aufgebaut werden konnte.
    pub(super) fn rebuild_preview_core_only(&mut self) -> bool {
        self.rebuild_sampling_preview();
        self.ensure_preview_core()
    }

    /// Baut Stage F auf den bereits vorliegenden Stage-E-Artefakten neu auf.
    ///
    /// Wird beim Uebergang `JunctionEdit → Finalize` genutzt, wenn Stage E bereits
    /// aktuell ist und nur der Junction-Trim ergaenzt werden muss.
    pub(super) fn rebuild_stage_f_only(&mut self) -> bool {
        self.ensure_prepared_segments()
    }

    /// Verwirft alle Stage-D-bis-F-Artefakte, behaelt aber Sampling-Input und Stage C.
    pub(super) fn clear_preview_pipeline(&mut self) {
        self.preview_data = None;
        self.cache.preview_core_key = None;
        self.cache.prepared_segments_key = None;
    }
}

#[cfg(test)]
mod tests {
    use std::sync::Arc;

    use glam::Vec2;
    use image::{DynamicImage, Rgb, RgbImage};

    use super::prepared::{prepare_segments, trim_segment_near_junctions};
    use crate::app::tools::color_path::sampling::{pixel_to_world, prepare_mask_for_skeleton};
    use crate::app::tools::color_path::skeleton::{
        SkeletonGraphNode, SkeletonGraphNodeKind, SkeletonGraphSegment, SkeletonNetwork,
    };
    use crate::app::tools::color_path::state::{ColorPathMask, ColorPathTool, PreviewData};

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
            network: SkeletonNetwork {
                nodes: vec![
                    SkeletonGraphNode {
                        kind: SkeletonGraphNodeKind::OpenEnd,
                        pixel_position: Vec2::new(0.0, 1.0),
                        world_position: Vec2::new(0.0, 0.0),
                    },
                    SkeletonGraphNode {
                        kind: SkeletonGraphNodeKind::OpenEnd,
                        pixel_position: Vec2::new(2.0, 1.0),
                        world_position: Vec2::new(10.0, 0.0),
                    },
                ],
                segments: vec![SkeletonGraphSegment {
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

        assert!(tool.rebuild_stage_f_only());

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

        assert!(tool.rebuild_stage_f_only());

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
        start_kind: SkeletonGraphNodeKind,
        end_kind: SkeletonGraphNodeKind,
        polyline: Vec<Vec2>,
    ) -> SkeletonNetwork {
        SkeletonNetwork {
            nodes: vec![
                SkeletonGraphNode {
                    kind: start_kind,
                    pixel_position: Vec2::new(0.0, 0.0),
                    world_position: *polyline.first().expect("Polyline ohne Startpunkt"),
                },
                SkeletonGraphNode {
                    kind: end_kind,
                    pixel_position: Vec2::new(0.0, 0.0),
                    world_position: *polyline.last().expect("Polyline ohne Endpunkt"),
                },
            ],
            segments: vec![SkeletonGraphSegment {
                start_node: 0,
                end_node: 1,
                polyline,
            }],
        }
    }

    fn assert_resampled_spacing(nodes: &[Vec2], node_spacing: f32) {
        assert!(
            nodes.len() >= 2,
            "Resample muss mindestens Start und Ende enthalten"
        );
        for pair in nodes.windows(2) {
            let distance = pair[0].distance(pair[1]);
            assert!(
                distance > 0.0,
                "Aufeinanderfolgende Punkte duerfen nicht identisch sein"
            );
            assert!(
                distance <= node_spacing + 1e-4,
                "Abstand {distance} verletzt node_spacing {node_spacing}"
            );
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
            SkeletonGraphNodeKind::Junction,
            SkeletonGraphNodeKind::OpenEnd,
            polyline,
        );

        let trimmed = trim_segment_near_junctions(
            &network,
            &network.segments[0],
            &network.segments[0].polyline,
            2.5,
        )
        .expect("Getrimmtes Segment sollte gueltig bleiben");

        assert_eq!(trimmed[0], Vec2::new(0.0, 0.0));
        assert_eq!(trimmed[1], Vec2::new(3.0, 0.0));
        assert_eq!(
            *trimmed.last().expect("Endpunkt muss vorhanden sein"),
            Vec2::new(10.0, 0.0)
        );

        let prepared = prepare_segments(&network, None, 0.0, 1.0, 2.5);

        assert_eq!(prepared.len(), 1);
        assert_eq!(prepared[0].resampled_nodes[0], Vec2::new(0.0, 0.0));
        assert_eq!(
            *prepared[0]
                .resampled_nodes
                .last()
                .expect("Endpunkt muss vorhanden sein"),
            Vec2::new(10.0, 0.0)
        );
        assert_resampled_spacing(&prepared[0].resampled_nodes, 1.0);
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
            SkeletonGraphNodeKind::Junction,
            SkeletonGraphNodeKind::Junction,
            polyline.clone(),
        );
        let open_end_network = network_with_segment(
            SkeletonGraphNodeKind::OpenEnd,
            SkeletonGraphNodeKind::OpenEnd,
            polyline,
        );

        let prepared_junction = prepare_segments(&junction_network, None, 0.0, 1.0, 0.0);
        let prepared_open_end = prepare_segments(&open_end_network, None, 0.0, 1.0, 0.0);

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
            SkeletonGraphNodeKind::Junction,
            SkeletonGraphNodeKind::OpenEnd,
            polyline,
        );

        let prepared = prepare_segments(&network, None, 0.0, 1.0, 10.0);

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
            SkeletonGraphNodeKind::OpenEnd,
            SkeletonGraphNodeKind::Junction,
            polyline,
        );

        let trimmed = trim_segment_near_junctions(
            &network,
            &network.segments[0],
            &network.segments[0].polyline,
            2.5,
        )
        .expect("Getrimmtes Segment sollte gueltig bleiben");

        assert_eq!(trimmed[0], Vec2::new(0.0, 0.0));
        assert_eq!(
            *trimmed.last().expect("Endpunkt muss vorhanden sein"),
            Vec2::new(10.0, 0.0)
        );
        assert_eq!(trimmed[trimmed.len() - 2], Vec2::new(7.0, 0.0));

        let prepared = prepare_segments(&network, None, 0.0, 1.0, 2.5);

        assert_eq!(prepared.len(), 1);
        let nodes = &prepared[0].resampled_nodes;
        assert_eq!(nodes[0], Vec2::new(0.0, 0.0));
        assert_eq!(
            *nodes.last().expect("Endpunkt muss vorhanden sein"),
            Vec2::new(10.0, 0.0)
        );
        assert_resampled_spacing(nodes, 1.0);
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
            SkeletonGraphNodeKind::Junction,
            SkeletonGraphNodeKind::Junction,
            polyline,
        );

        let trimmed = trim_segment_near_junctions(
            &network,
            &network.segments[0],
            &network.segments[0].polyline,
            2.5,
        )
        .expect("Getrimmtes Segment sollte gueltig bleiben");

        assert_eq!(
            trimmed,
            vec![
                Vec2::new(0.0, 0.0),
                Vec2::new(3.0, 0.0),
                Vec2::new(7.0, 0.0),
                Vec2::new(10.0, 0.0),
            ]
        );

        let prepared = prepare_segments(&network, None, 0.0, 1.0, 2.5);

        assert_eq!(prepared.len(), 1);
        let nodes = &prepared[0].resampled_nodes;
        assert_eq!(nodes[0], Vec2::new(0.0, 0.0));
        assert_eq!(
            *nodes.last().expect("Endpunkt muss vorhanden sein"),
            Vec2::new(10.0, 0.0)
        );
        assert_resampled_spacing(nodes, 1.0);
    }

    #[test]
    fn junction_radius_large_with_two_junctions_falls_back_to_direct_segment() {
        let polyline = vec![
            Vec2::new(0.0, 0.0),
            Vec2::new(0.5, 0.0),
            Vec2::new(1.0, 0.0),
        ];
        let network = network_with_segment(
            SkeletonGraphNodeKind::Junction,
            SkeletonGraphNodeKind::Junction,
            polyline,
        );

        let prepared = prepare_segments(&network, None, 0.0, 1.0, 10.0);

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

    // ── CP-07: Stage F auf EditableCenterlines ────────────────────────────

    fn tool_with_editable_network(polyline: Vec<Vec2>) -> ColorPathTool {
        use crate::app::tools::color_path::editable::EditableCenterlines;

        let network = network_with_segment(
            SkeletonGraphNodeKind::Junction,
            SkeletonGraphNodeKind::Junction,
            polyline,
        );
        let mut tool = ColorPathTool::new();
        tool.editable = Some(EditableCenterlines::from_skeleton_network(&network));
        tool.preview_data = Some(PreviewData {
            prepared_mask: ColorPathMask::default(),
            network,
            prepared_segments: Vec::new(),
        });
        // Stage E als "bereits aktuell" markieren, damit `ensure_prepared_segments`
        // direkt in Stage F geht.
        tool.cache.preview_core_revision = 1;
        tool.cache.preview_core_key = tool.preview_core_cache_key();
        tool
    }

    #[test]
    fn prepare_segments_pulls_polyline_endpoints_to_editable_junctions() {
        use crate::app::tools::color_path::editable::{EditableCenterlines, EditableJunctionId};

        let network = network_with_segment(
            SkeletonGraphNodeKind::Junction,
            SkeletonGraphNodeKind::Junction,
            vec![
                Vec2::new(0.0, 0.0),
                Vec2::new(5.0, 0.0),
                Vec2::new(10.0, 0.0),
            ],
        );
        let mut editable = EditableCenterlines::from_skeleton_network(&network);
        // Start-Junction verschieben — simuliert CP-08-Drag.
        editable.move_junction(EditableJunctionId(0), Vec2::new(-2.0, 3.0));

        let prepared = prepare_segments(&network, Some(&editable), 0.0, 2.0, 0.0);

        assert_eq!(prepared.len(), 1);
        let nodes = &prepared[0].resampled_nodes;
        assert_eq!(
            nodes[0],
            Vec2::new(-2.0, 3.0),
            "Polyline-Start muss an die gedraggte Junction-Position gezogen werden"
        );
        assert_eq!(
            *nodes.last().expect("Endpunkt muss vorhanden sein"),
            Vec2::new(10.0, 0.0),
            "End-Junction wurde nicht bewegt und bleibt unveraendert"
        );
        assert_resampled_spacing(nodes, 2.0);
    }

    #[test]
    fn editable_revision_bump_invalidates_prepared_segments_cache() {
        use crate::app::tools::color_path::editable::EditableJunctionId;

        let mut tool = tool_with_editable_network(vec![
            Vec2::new(0.0, 0.0),
            Vec2::new(5.0, 0.0),
            Vec2::new(10.0, 0.0),
        ]);
        tool.config.simplify_tolerance = 0.0;
        tool.config.node_spacing = 2.0;
        tool.config.junction_radius = 0.0;

        assert!(tool.ensure_prepared_segments());
        let first_rev = tool.cache.prepared_segments_revision;
        let first_key = tool
            .cache
            .prepared_segments_key
            .expect("Stage-F-Cache-Key muss nach erstem Build vorhanden sein");

        // Identischer Aufruf → Cache-Hit.
        assert!(tool.ensure_prepared_segments());
        assert_eq!(tool.cache.prepared_segments_revision, first_rev);

        // Junction-Drag auf Editable bumpt revision → Stage F muss neu rechnen.
        let editable = tool.editable.as_mut().expect("Editable muss existieren");
        editable.move_junction(EditableJunctionId(1), Vec2::new(12.0, 0.0));
        let new_editable_rev = editable.revision;

        assert!(tool.ensure_prepared_segments());
        assert!(tool.cache.prepared_segments_revision > first_rev);
        let new_key = tool
            .cache
            .prepared_segments_key
            .expect("Stage-F-Cache-Key bleibt vorhanden");
        assert_ne!(new_key, first_key);
        assert_eq!(new_key.editable_revision, new_editable_rev);

        let resampled = &tool
            .preview_data
            .as_ref()
            .expect("Preview-Daten")
            .prepared_segments[0]
            .resampled_nodes;
        assert_eq!(
            *resampled.last().expect("Endpunkt"),
            Vec2::new(12.0, 0.0),
            "End-Junction-Drag muss in Stage F sichtbar werden"
        );
    }

    #[test]
    fn junction_radius_still_only_trims_junction_nodes_with_editable() {
        use crate::app::tools::color_path::editable::EditableCenterlines;

        // OpenEnd ↔ OpenEnd: Junction-Radius darf nicht greifen, auch nicht mit Editable.
        let network = network_with_segment(
            SkeletonGraphNodeKind::OpenEnd,
            SkeletonGraphNodeKind::OpenEnd,
            vec![
                Vec2::new(0.0, 0.0),
                Vec2::new(1.0, 0.0),
                Vec2::new(5.0, 0.0),
                Vec2::new(9.0, 0.0),
                Vec2::new(10.0, 0.0),
            ],
        );
        let editable = EditableCenterlines::from_skeleton_network(&network);

        let prepared_with_editable = prepare_segments(&network, Some(&editable), 0.0, 1.0, 3.0);
        let prepared_without_editable = prepare_segments(&network, None, 0.0, 1.0, 3.0);

        assert_eq!(prepared_with_editable.len(), 1);
        assert_eq!(prepared_without_editable.len(), 1);
        assert_eq!(
            prepared_with_editable[0].resampled_nodes, prepared_without_editable[0].resampled_nodes,
            "Junction-Radius bleibt auf Junction-Kind beschraenkt — OpenEnds unveraendert"
        );
    }

    #[test]
    fn prepare_segments_resample_still_honors_node_spacing_with_editable() {
        use crate::app::tools::color_path::editable::{EditableCenterlines, EditableJunctionId};

        let network = network_with_segment(
            SkeletonGraphNodeKind::Junction,
            SkeletonGraphNodeKind::Junction,
            vec![Vec2::new(0.0, 0.0), Vec2::new(10.0, 0.0)],
        );
        let mut editable = EditableCenterlines::from_skeleton_network(&network);
        editable.move_junction(EditableJunctionId(1), Vec2::new(20.0, 0.0));

        let prepared = prepare_segments(&network, Some(&editable), 0.0, 2.5, 0.0);

        assert_eq!(prepared.len(), 1);
        let nodes = &prepared[0].resampled_nodes;
        assert_eq!(nodes[0], Vec2::new(0.0, 0.0));
        assert_eq!(
            *nodes.last().expect("Endpunkt"),
            Vec2::new(20.0, 0.0),
            "End-Junction-Drag muss in resample eingehen"
        );
        assert_resampled_spacing(nodes, 2.5);
    }
}
