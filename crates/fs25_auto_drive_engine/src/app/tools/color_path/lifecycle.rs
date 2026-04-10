//! RouteTool-Implementierung fuer das ColorPathTool.
//!
//! Enthaelt nur Orchestrierung, Phasenwechsel und den RouteTool-Adapter.

use image::GenericImageView;
use std::sync::Arc;

use crate::app::tools::common::sync_tool_host;
use crate::app::tools::{
    RouteTool, RouteToolCore, RouteToolHostSync, RouteToolLassoInput, RouteToolPanelBridge,
    ToolAction, ToolHostContext, ToolPreview, ToolResult,
};
use crate::app::ui_contract::{RouteToolConfigState, RouteToolPanelAction, RouteToolPanelEffect};
use crate::core::{FarmlandGrid, RoadMap};
use glam::Vec2;

use super::state::{ColorPathPhase, ColorPathTool};

impl ColorPathTool {
    fn sample_color_from_click(&mut self, pos: Vec2) -> bool {
        let Some(image) = self.background_image.as_ref() else {
            log::warn!(
                "ColorPathTool: Kein Hintergrundbild vorhanden — Klick-Sampling wird ignoriert"
            );
            return false;
        };
        let Some(color) = super::sampling::sample_color_at_world(pos, image, self.map_size) else {
            return false;
        };

        self.sampling.sampled_colors.push(color);
        if self.sampling.lasso_start_world.is_none() {
            self.sampling.lasso_start_world = Some(pos);
        }
        self.sampling.avg_color = Some(super::sampling::compute_average_color(
            &self.sampling.sampled_colors,
        ));
        self.mark_sampling_input_changed();
        self.rebuild_sampling_preview();
        true
    }

    /// Reagiert auf Aenderungen am Farb-Matching.
    pub(super) fn on_matching_config_changed(&mut self) {
        match self.phase {
            ColorPathPhase::Idle => self.refresh_matching_spec(),
            ColorPathPhase::Sampling => self.rebuild_sampling_preview(),
            ColorPathPhase::Preview => self.compute_pipeline(),
        }
    }

    /// Reagiert auf Aenderungen der Stage-D/E-Konfiguration.
    pub(super) fn on_preview_core_config_changed(&mut self) {
        if self.phase == ColorPathPhase::Preview {
            let _ = self.rebuild_preview_from_sampling_artifacts();
        }
    }

    /// Reagiert auf Aenderungen der Stage-F-Konfiguration.
    pub(super) fn on_preview_geometry_config_changed(&mut self) {
        if self.phase == ColorPathPhase::Preview {
            self.rebuild_prepared_segments();
        }
    }

    /// Reagiert auf Aenderungen am Bild-/Map-Kontext der Sampling-Pipeline.
    pub(super) fn on_sampling_context_changed(&mut self) {
        match self.phase {
            ColorPathPhase::Idle => self.clear_sampling_preview(),
            ColorPathPhase::Sampling => self.rebuild_sampling_preview(),
            ColorPathPhase::Preview => self.compute_pipeline(),
        }
    }

    /// Fuehrt die Stages C-F der Farb-Pfad-Erkennung aus und schaltet bei Erfolg auf Preview.
    pub(super) fn compute_pipeline(&mut self) {
        let Some(_image) = self.background_image.as_ref() else {
            log::warn!("ColorPathTool: Pipeline abgebrochen — kein Hintergrundbild");
            self.clear_sampling_preview();
            return;
        };
        if self.sampling.sampled_colors.is_empty() {
            log::warn!("ColorPathTool: Pipeline abgebrochen — keine Farbsamples");
            self.refresh_matching_spec();
            self.clear_sampling_preview();
            return;
        }
        if self.sampling.lasso_start_world.is_none() {
            log::warn!("ColorPathTool: Pipeline abgebrochen — kein Lasso-Startpunkt");
            self.clear_sampling_preview();
            return;
        }

        let preview_ready = self.rebuild_preview_pipeline();
        if self.preview_data.is_none() {
            log::warn!("ColorPathTool: Kein exportierbares Netz gefunden — Phase bleibt Sampling");
            return;
        }
        if !preview_ready {
            log::warn!(
                "ColorPathTool: Netz extrahiert, aber keine gueltigen Preview-Segmente erzeugt"
            );
            return;
        }

        self.phase = ColorPathPhase::Preview;
    }

    /// Setzt das Hintergrundbild fuer die Sampling-Pipeline.
    pub(crate) fn set_background_map_image(&mut self, image: Option<Arc<image::DynamicImage>>) {
        let previous_image_id = self
            .background_image
            .as_ref()
            .map(|current| Arc::as_ptr(current) as usize);
        let previous_map_size_bits = self.map_size.to_bits();

        if let Some(ref img) = image {
            let (w, h) = img.dimensions();
            let img_map_size = w.min(h) as f32;
            if self.map_size == 2048.0 || (self.map_size - img_map_size).abs() > 1.0 {
                log::info!(
                    "ColorPathTool: map_size aus Bild abgeleitet: {} (war {})",
                    img_map_size,
                    self.map_size
                );
                self.map_size = img_map_size;
            }
        }
        self.background_image = image;

        let current_image_id = self
            .background_image
            .as_ref()
            .map(|current| Arc::as_ptr(current) as usize);
        let context_changed = previous_image_id != current_image_id
            || previous_map_size_bits != self.map_size.to_bits();
        if context_changed {
            self.cache.rgb_image = None;
            self.on_sampling_context_changed();
        }
    }

    /// Leitet optionale Farmland-Grid-Infos in die Sampling-Pipeline weiter.
    pub(crate) fn set_farmland_grid(&mut self, grid: Option<Arc<FarmlandGrid>>) {
        if let Some(g) = &grid {
            let previous_map_size_bits = self.map_size.to_bits();
            self.map_size = g.map_size;
            if previous_map_size_bits != self.map_size.to_bits() {
                self.on_sampling_context_changed();
            }
        }
    }
}

impl RouteToolPanelBridge for ColorPathTool {
    fn status_text(&self) -> &str {
        match self.phase {
            ColorPathPhase::Idle => "Klick oder Alt+Lasso fuer Farbsample",
            ColorPathPhase::Sampling if self.sampling.sampled_colors.is_empty() => {
                "Klick oder Alt+Lasso fuer Farbsample"
            }
            ColorPathPhase::Sampling => "Berechnen fuer Wegenetz",
            ColorPathPhase::Preview => "Enter zum Einfuegen, Reset zum Zuruecksetzen",
        }
    }

    fn panel_state(&self) -> RouteToolConfigState {
        RouteToolConfigState::ColorPath(self.panel_state())
    }

    fn apply_panel_action(&mut self, action: RouteToolPanelAction) -> RouteToolPanelEffect {
        let RouteToolPanelAction::ColorPath(action) = action else {
            return RouteToolPanelEffect::default();
        };

        self.apply_panel_action(action)
    }
}

impl RouteToolCore for ColorPathTool {
    fn on_click(&mut self, pos: Vec2, _road_map: &RoadMap, _ctrl: bool) -> ToolAction {
        match self.phase {
            ColorPathPhase::Idle => {
                self.phase = ColorPathPhase::Sampling;
                let _ = self.sample_color_from_click(pos);
                ToolAction::Continue
            }
            ColorPathPhase::Sampling => {
                let _ = self.sample_color_from_click(pos);
                ToolAction::Continue
            }
            ColorPathPhase::Preview => {
                if self.sample_color_from_click(pos) {
                    self.phase = ColorPathPhase::Sampling;
                }
                ToolAction::Continue
            }
        }
    }

    fn preview(&self, _cursor_pos: Vec2, _road_map: &RoadMap) -> ToolPreview {
        match self.phase {
            ColorPathPhase::Idle => ToolPreview::default(),
            ColorPathPhase::Sampling => self.build_sampling_preview(),
            ColorPathPhase::Preview => self.build_network_preview(),
        }
    }

    fn execute(&self, road_map: &RoadMap) -> Option<ToolResult> {
        if self.phase != ColorPathPhase::Preview {
            return None;
        }

        self.execute_result(road_map)
    }

    fn reset(&mut self) {
        self.phase = ColorPathPhase::Idle;
        self.sampling = super::state::SamplingInput::default();
        self.matching = super::state::MatchingSpec::default();
        self.sampling_preview = None;
        self.preview_data = None;
        self.cache = super::state::ColorPathCacheState::default();
    }

    fn is_ready(&self) -> bool {
        self.phase == ColorPathPhase::Preview
            && self
                .preview_data
                .as_ref()
                .is_some_and(|preview| !preview.prepared_segments.is_empty())
    }

    fn has_pending_input(&self) -> bool {
        self.phase != ColorPathPhase::Idle
    }
}

impl RouteToolHostSync for ColorPathTool {
    fn sync_host(&mut self, context: &ToolHostContext) {
        sync_tool_host(
            &mut self.direction,
            &mut self.priority,
            &mut self.lifecycle,
            context,
        );
        self.set_background_map_image(context.background_image.clone());
        self.set_farmland_grid(context.farmland_grid.clone());
    }
}

impl RouteToolLassoInput for ColorPathTool {
    fn is_lasso_input_active(&self) -> bool {
        self.phase == ColorPathPhase::Sampling
    }

    fn on_lasso_completed(&mut self, polygon: Vec<Vec2>) -> ToolAction {
        if self.phase != ColorPathPhase::Sampling {
            return ToolAction::Continue;
        }
        let Some(image) = &self.background_image else {
            log::warn!("ColorPathTool: Kein Hintergrundbild vorhanden — Lasso wird ignoriert");
            return ToolAction::Continue;
        };

        let new_colors = super::sampling::sample_colors_in_polygon(&polygon, image, self.map_size);
        let new_count = new_colors.len();
        if self.sampling.lasso_regions.is_empty() {
            self.sampling.lasso_start_world = polygon.first().copied();
        }
        self.sampling.sampled_colors.extend(new_colors);
        self.sampling.lasso_regions.push(polygon);
        self.sampling.avg_color = Some(super::sampling::compute_average_color(
            &self.sampling.sampled_colors,
        ));
        self.mark_sampling_input_changed();
        self.rebuild_sampling_preview();

        log::info!(
            "Color sampling: {} new pixels, {} total, match colors: {}, avg color: {:?}",
            new_count,
            self.sampling.sampled_colors.len(),
            self.matching.palette.len(),
            self.sampling.avg_color
        );
        log::info!(
            "Flood-Fill Vorschau: {} Randsegmente",
            self.sampling_preview
                .as_ref()
                .map_or(0, |preview| preview.boundary_segments.len())
        );
        ToolAction::Continue
    }
}

impl RouteTool for ColorPathTool {
    fn as_lasso_input(&self) -> Option<&dyn RouteToolLassoInput> {
        Some(self)
    }

    fn as_lasso_input_mut(&mut self) -> Option<&mut dyn RouteToolLassoInput> {
        Some(self)
    }
}

#[cfg(test)]
mod tests {
    use std::sync::Arc;

    use super::*;
    use crate::app::tools::color_path::sampling::pixel_to_world;
    use crate::app::tools::color_path::skeleton::{
        SkeletonGraphNode, SkeletonGraphNodeKind, SkeletonNetwork,
    };
    use crate::app::tools::color_path::state::{
        ColorPathMask, ExistingConnectionMode, PreparedSegment, PreviewData,
    };
    use crate::app::tools::RouteToolCore;
    use crate::core::{ConnectionDirection, ConnectionPriority, MapNode, NodeFlag};
    use image::{DynamicImage, Rgb, RgbImage};

    fn build_test_image() -> DynamicImage {
        DynamicImage::ImageRgb8(RgbImage::from_fn(10, 10, |x, _| {
            if x < 8 {
                Rgb([200, 0, 0])
            } else {
                Rgb([0, 200, 0])
            }
        }))
    }

    fn sample_network() -> SkeletonNetwork {
        SkeletonNetwork {
            nodes: vec![
                SkeletonGraphNode {
                    kind: SkeletonGraphNodeKind::Junction,
                    pixel_position: Vec2::new(10.0, 10.0),
                    world_position: Vec2::ZERO,
                },
                SkeletonGraphNode {
                    kind: SkeletonGraphNodeKind::OpenEnd,
                    pixel_position: Vec2::new(0.0, 10.0),
                    world_position: Vec2::new(-10.0, 0.0),
                },
                SkeletonGraphNode {
                    kind: SkeletonGraphNodeKind::OpenEnd,
                    pixel_position: Vec2::new(20.0, 10.0),
                    world_position: Vec2::new(10.0, 0.0),
                },
            ],
            segments: vec![],
        }
    }

    fn sample_prepared_segments() -> Vec<PreparedSegment> {
        vec![
            PreparedSegment {
                start_node: 1,
                end_node: 0,
                resampled_nodes: vec![Vec2::new(-10.0, 0.0), Vec2::new(-5.0, 0.0), Vec2::ZERO],
            },
            PreparedSegment {
                start_node: 0,
                end_node: 2,
                resampled_nodes: vec![Vec2::ZERO, Vec2::new(5.0, 0.0), Vec2::new(10.0, 0.0)],
            },
        ]
    }

    fn build_preview_tool(mode: ExistingConnectionMode) -> ColorPathTool {
        let mut tool = ColorPathTool::new();
        tool.phase = ColorPathPhase::Preview;
        tool.direction = ConnectionDirection::Regular;
        tool.priority = ConnectionPriority::Regular;
        tool.config.existing_connection_mode = mode;
        tool.preview_data = Some(PreviewData {
            prepared_mask: ColorPathMask::default(),
            network: sample_network(),
            prepared_segments: sample_prepared_segments(),
        });
        tool.lifecycle.snap_radius = 1.0;
        tool
    }

    #[test]
    fn click_from_idle_samples_first_color_and_enters_sampling() {
        let image = Arc::new(build_test_image());
        let road_map = RoadMap::default();
        let mut tool = ColorPathTool::new();
        tool.phase = ColorPathPhase::Idle;
        tool.set_background_map_image(Some(image));

        let click_pos = pixel_to_world(0, 0, tool.map_size, 10, 10);
        let _ = tool.on_click(click_pos, &road_map, false);

        assert_eq!(tool.phase, ColorPathPhase::Sampling);
        assert_eq!(tool.sampling.sampled_colors, vec![[200, 0, 0]]);
        assert_eq!(tool.sampling.avg_color, Some([200, 0, 0]));
        assert_eq!(tool.sampling.lasso_start_world, Some(click_pos));
        assert!(tool.sampling_preview.is_some());
    }

    #[test]
    fn click_from_preview_adds_color_and_returns_to_sampling() {
        let image = Arc::new(build_test_image());
        let road_map = RoadMap::default();
        let mut tool = build_preview_tool(ExistingConnectionMode::Never);
        tool.set_background_map_image(Some(image));

        let click_pos = pixel_to_world(9, 0, tool.map_size, 10, 10);
        let _ = tool.on_click(click_pos, &road_map, false);

        assert_eq!(tool.phase, ColorPathPhase::Sampling);
        assert_eq!(tool.sampling.sampled_colors, vec![[0, 200, 0]]);
        assert_eq!(tool.sampling.avg_color, Some([0, 200, 0]));
        assert_eq!(tool.sampling.lasso_start_world, Some(click_pos));
        assert!(tool.sampling_preview.is_some());
        assert!(tool.preview_data.is_none());
    }

    #[test]
    fn execute_reuses_shared_junction_node_for_multiple_segments() {
        let tool = build_preview_tool(ExistingConnectionMode::Never);
        let road_map = RoadMap::new(3);

        let result = tool
            .execute(&road_map)
            .expect("Preview-Netz sollte exportierbar sein");

        assert_eq!(
            result.new_nodes.len(),
            5,
            "3 Graph-Knoten + 2 Zwischenknoten"
        );
        assert_eq!(result.internal_connections.len(), 4);
        assert_eq!(
            result.new_nodes[0].0,
            Vec2::ZERO,
            "Junction nur einmal anlegen"
        );
        assert!(
            result
                .internal_connections
                .iter()
                .any(|&(from, to, _, _)| from == 3 && to == 0),
            "Erstes Segment muss in denselben Junction-Knoten muenden"
        );
        assert!(
            result
                .internal_connections
                .iter()
                .any(|&(from, to, _, _)| from == 0 && to == 4),
            "Zweites Segment muss denselben Junction-Knoten wiederverwenden"
        );
    }

    #[test]
    fn execute_snap_modes_limit_existing_connections() {
        let mut road_map = RoadMap::new(3);
        road_map.add_node(MapNode::new(100, Vec2::new(-10.4, 0.0), NodeFlag::Regular));
        road_map.add_node(MapNode::new(200, Vec2::new(0.3, 0.0), NodeFlag::Regular));
        road_map.ensure_spatial_index();

        let tool_open_ends = build_preview_tool(ExistingConnectionMode::OpenEnds);
        let result_open_ends = tool_open_ends
            .execute(&road_map)
            .expect("Open-End-Modus sollte exportierbar sein");
        assert_eq!(result_open_ends.external_connections.len(), 1);
        assert_eq!(result_open_ends.external_connections[0].1, 100);

        let tool_with_junctions = build_preview_tool(ExistingConnectionMode::OpenEndsAndJunctions);
        let result_with_junctions = tool_with_junctions
            .execute(&road_map)
            .expect("Junction-Modus sollte exportierbar sein");
        assert_eq!(result_with_junctions.external_connections.len(), 2);
        assert!(result_with_junctions.external_connections.iter().any(
            |&(idx, existing_id, _, direction, _)| {
                idx == 0 && existing_id == 200 && direction == ConnectionDirection::Dual
            }
        ));
    }

    #[test]
    fn set_background_map_image_with_same_arc_keeps_sampling_preview_cache() {
        let image = Arc::new(build_test_image());
        let mut tool = ColorPathTool::new();
        tool.phase = ColorPathPhase::Sampling;
        tool.set_background_map_image(Some(Arc::clone(&image)));
        tool.sampling.sampled_colors = vec![[200, 0, 0]];
        tool.sampling.lasso_start_world = Some(pixel_to_world(0, 0, tool.map_size, 10, 10));
        tool.mark_sampling_input_changed();

        tool.rebuild_sampling_preview();
        let sampling_preview_revision = tool.cache.sampling_preview_revision;

        tool.set_background_map_image(Some(Arc::clone(&image)));

        assert_eq!(
            tool.cache.sampling_preview_revision,
            sampling_preview_revision
        );
        assert!(tool.sampling_preview.is_some());
    }
}
