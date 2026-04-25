//! RouteTool-Implementierung fuer das ColorPathTool.
//!
//! Enthaelt nur Orchestrierung, Phasenwechsel und den RouteTool-Adapter.

use image::GenericImageView;
use std::sync::Arc;

use crate::app::tools::common::sync_tool_host;
use crate::app::tools::{
    RouteTool, RouteToolCore, RouteToolDrag, RouteToolHostSync, RouteToolLassoInput,
    RouteToolPanelBridge, ToolAction, ToolHostContext, ToolPreview, ToolResult,
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
    ///
    /// In Editing-Phasen (`CenterlinePreview`/`JunctionEdit`/`Finalize`) darf
    /// ein Matching-Wechsel die Wizard-Phase niemals veraendern. Das Netz wird
    /// in-place neu aufgebaut und das Editable-Modell nachgezogen; aktive
    /// Junction-Drags (Handle) werden verworfen, weil die alten IDs nach dem
    /// Rebuild nicht mehr stabil sind.
    pub(super) fn on_matching_config_changed(&mut self) {
        match self.phase {
            ColorPathPhase::Idle => self.refresh_matching_spec(),
            ColorPathPhase::Sampling => self.rebuild_sampling_preview(),
            ColorPathPhase::CenterlinePreview | ColorPathPhase::JunctionEdit => {
                self.rebuild_editing_preview_in_place(false);
            }
            ColorPathPhase::Finalize => {
                self.rebuild_editing_preview_in_place(true);
            }
        }
    }

    /// Reagiert auf Aenderungen der Stage-D/E-Konfiguration.
    ///
    /// Wie bei [`Self::on_matching_config_changed`] bleibt die Phase in allen
    /// Editing-Phasen erhalten; das Editable-Modell wird nach dem Rebuild
    /// konsistent neu synchronisiert (siehe R1/F1).
    pub(super) fn on_preview_core_config_changed(&mut self) {
        match self.phase {
            ColorPathPhase::Idle | ColorPathPhase::Sampling => {}
            ColorPathPhase::CenterlinePreview | ColorPathPhase::JunctionEdit => {
                self.rebuild_editing_preview_in_place(false);
            }
            ColorPathPhase::Finalize => {
                self.rebuild_editing_preview_in_place(true);
            }
        }
    }

    /// Reagiert auf Aenderungen der Stage-F-Konfiguration.
    ///
    /// In `Finalize` wird Stage F direkt neu berechnet. In den Editing-Phasen
    /// `CenterlinePreview`/`JunctionEdit` wirken Geometrie-Slider erst beim
    /// naechsten Finalize-Eintritt — hier reicht es, den Stage-F-Cache zu
    /// invalidieren, ohne Phase oder Drags zu beruehren (F3).
    pub(super) fn on_preview_geometry_config_changed(&mut self) {
        match self.phase {
            ColorPathPhase::Finalize => self.rebuild_prepared_segments(),
            ColorPathPhase::CenterlinePreview | ColorPathPhase::JunctionEdit => {
                self.cache.prepared_segments_key = None;
            }
            ColorPathPhase::Idle | ColorPathPhase::Sampling => {}
        }
    }

    /// Baut das Preview-Netz in einer Editing-Phase neu auf, ohne die Phase
    /// zu veraendern.
    ///
    /// Invalidiert den aktiven Junction-Drag-Handle, weil die Editable-IDs
    /// nach dem Rebuild nicht mehr garantiert zum vorherigen Snapshot
    /// passen. Wenn `include_stage_f` gesetzt ist (Phase `Finalize`), wird
    /// anschliessend Stage F direkt neu berechnet.
    fn rebuild_editing_preview_in_place(&mut self, include_stage_f: bool) {
        if !self.rebuild_preview_core_only() {
            return;
        }
        self.sync_editable_from_network();
        self.bump_editable_revision();
        self.dragging_junction = None;
        if include_stage_f {
            let _ = self.rebuild_stage_f_only();
        }
    }

    /// Reagiert auf Aenderungen am Bild-/Map-Kontext der Sampling-Pipeline.
    pub(super) fn on_sampling_context_changed(&mut self) {
        match self.phase {
            ColorPathPhase::Idle => self.clear_sampling_preview(),
            ColorPathPhase::Sampling => self.rebuild_sampling_preview(),
            ColorPathPhase::CenterlinePreview
            | ColorPathPhase::JunctionEdit
            | ColorPathPhase::Finalize => self.compute_pipeline(),
        }
    }

    /// Fuehrt die Stages C-F der Farb-Pfad-Erkennung aus und schaltet bei Erfolg auf Finalize.
    ///
    /// Seit CP-03 laeuft die Pipeline entlang der drei Wizard-Phasen:
    /// Stage E → `CenterlinePreview`, danach der Platzhalter-Uebergang
    /// nach `JunctionEdit` (in CP-03 noch ohne Drag-Logik) und schliesslich
    /// Stage F → `Finalize`. Schlaegt eine Stufe fehl, bleibt die Phase auf
    /// dem zuletzt erreichten Zwischenschritt stehen.
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

        // Stage C-E: Centerline-Preview ohne Junction-Trim.
        if !self.rebuild_preview_core_only() {
            log::warn!("ColorPathTool: Kein exportierbares Netz gefunden — Phase bleibt Sampling");
            return;
        }
        self.sync_editable_from_network();
        self.phase = ColorPathPhase::CenterlinePreview;

        // JunctionEdit: in CP-03 noch ohne echte Drag-Logik, aber als eigener Wizard-Schritt sichtbar.
        self.bump_editable_revision();
        self.phase = ColorPathPhase::JunctionEdit;

        // Stage F: Junction-Trim und Resampling; bei Fehlschlag bleibt Phase auf JunctionEdit.
        if !self.rebuild_stage_f_only() {
            log::warn!(
                "ColorPathTool: Netz extrahiert, aber keine gueltigen Preview-Segmente erzeugt"
            );
            return;
        }

        self.bump_editable_revision();
        self.phase = ColorPathPhase::Finalize;
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

    /// Setzt das ColorPathTool vollstaendig in den Ausgangszustand zurueck.
    ///
    /// Invariante: Einziger autorisierter Reset-Pfad fuer das ColorPathTool.
    /// Alle Aufrufer (RouteToolCore::reset, ColorPathPanelAction::Reset,
    /// kuenftige Wizard-Phasen) muessen ueber diese Routine gehen, damit Phase,
    /// Sampling-Input, Matching-Spec, Previews und Cache-State konsistent
    /// zurueckgesetzt werden.
    pub(super) fn reset_all(&mut self) {
        self.phase = ColorPathPhase::Idle;
        self.sampling = super::state::SamplingInput::default();
        self.matching = super::state::MatchingSpec::default();
        self.sampling_preview = None;
        self.preview_data = None;
        self.editable = None;
        self.dragging_junction = None;
        self.cache = super::state::ColorPathCacheState::default();
    }

    /// Synchronisiert das editierbare Zwischenmodell mit dem aktuellen Stage-E-Netz.
    ///
    /// Wird beim Eintritt in `CenterlinePreview` aufgerufen und rekonstruiert
    /// [`super::editable::EditableCenterlines`] aus `preview_data.network`. Fehlt
    /// das Netz (z. B. vor dem ersten erfolgreichen Stage-E-Durchlauf), wird
    /// das Editable-Feld geleert. Spaetere Commit-Punkte (CP-07/08) lesen die
    /// Junction-Positionen hieraus, CP-06 selbst nutzt es noch nicht fuer Stage F.
    pub(super) fn sync_editable_from_network(&mut self) {
        let Some(preview) = self.preview_data.as_ref() else {
            self.editable = None;
            return;
        };
        if preview.network.is_empty() {
            self.editable = None;
            return;
        }
        self.editable = Some(super::editable::EditableCenterlines::from_skeleton_network(
            &preview.network,
        ));
    }

    /// Bumpt die Editable-Revision, falls ein Editable-Modell vorliegt.
    ///
    /// Wird bei Phase-Wechseln (CP-06) und spaeter beim Junction-Drag (CP-08)
    /// aufgerufen, damit abgeleitete Cache-Keys kuenftiger Stages invalidiert
    /// werden. Ohne aktives Editable-Modell ist der Aufruf ein No-Op.
    pub(super) fn bump_editable_revision(&mut self) {
        if let Some(editable) = self.editable.as_mut() {
            editable.bump_revision();
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
            ColorPathPhase::CenterlinePreview => {
                "Centerline-Vorschau — weiter zur Kreuzungsbearbeitung"
            }
            ColorPathPhase::JunctionEdit => "Kreuzungen bearbeiten — weiter zu Finalize",
            ColorPathPhase::Finalize => "Enter zum Einfuegen, Reset zum Zuruecksetzen",
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
            ColorPathPhase::CenterlinePreview
            | ColorPathPhase::JunctionEdit
            | ColorPathPhase::Finalize => {
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
            ColorPathPhase::CenterlinePreview
            | ColorPathPhase::JunctionEdit
            | ColorPathPhase::Finalize => self.build_network_preview(),
        }
    }

    fn execute(&self, road_map: &RoadMap) -> Option<ToolResult> {
        if !self.phase.is_finalized() {
            return None;
        }

        self.execute_result(road_map)
    }

    fn reset(&mut self) {
        self.reset_all();
    }

    fn is_ready(&self) -> bool {
        self.phase.is_finalized()
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

    fn as_drag(&self) -> Option<&dyn RouteToolDrag> {
        if self.phase == ColorPathPhase::JunctionEdit {
            Some(self)
        } else {
            None
        }
    }

    fn as_drag_mut(&mut self) -> Option<&mut dyn RouteToolDrag> {
        if self.phase == ColorPathPhase::JunctionEdit {
            Some(self)
        } else {
            None
        }
    }
}

impl RouteToolDrag for ColorPathTool {
    fn drag_targets(&self) -> Vec<Vec2> {
        super::drag::drag_targets(self)
    }

    fn on_drag_start(&mut self, pos: Vec2, road_map: &RoadMap, pick_radius: f32) -> bool {
        super::drag::on_drag_start(self, pos, road_map, pick_radius)
    }

    fn on_drag_update(&mut self, pos: Vec2) {
        super::drag::on_drag_update(self, pos);
    }

    fn on_drag_end(&mut self, road_map: &RoadMap) {
        super::drag::on_drag_end(self, road_map);
    }
}

#[cfg(test)]
mod tests {
    use std::sync::Arc;

    use super::*;
    use crate::app::tools::color_path::sampling::pixel_to_world;
    use crate::app::tools::color_path::skeleton::{
        SkeletonGraphNode, SkeletonGraphNodeKind, SkeletonGraphSegment, SkeletonNetwork,
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
        tool.phase = ColorPathPhase::Finalize;
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

    #[test]
    fn preview_geometry_change_keeps_preview_phase_and_updates_execute_consistently() {
        let mut tool = ColorPathTool::new();
        tool.phase = ColorPathPhase::Finalize;
        tool.direction = ConnectionDirection::Regular;
        tool.priority = ConnectionPriority::Regular;
        tool.config.existing_connection_mode = ExistingConnectionMode::Never;
        tool.config.simplify_tolerance = 0.0;
        tool.config.node_spacing = 1.0;
        tool.config.junction_radius = 0.0;
        tool.cache.preview_core_revision = 11;

        tool.preview_data = Some(PreviewData {
            prepared_mask: ColorPathMask::default(),
            network: SkeletonNetwork {
                nodes: vec![
                    SkeletonGraphNode {
                        kind: SkeletonGraphNodeKind::Junction,
                        pixel_position: Vec2::new(0.0, 0.0),
                        world_position: Vec2::new(0.0, 0.0),
                    },
                    SkeletonGraphNode {
                        kind: SkeletonGraphNodeKind::Junction,
                        pixel_position: Vec2::new(10.0, 0.0),
                        world_position: Vec2::new(10.0, 0.0),
                    },
                ],
                segments: vec![SkeletonGraphSegment {
                    start_node: 0,
                    end_node: 1,
                    polyline: vec![
                        Vec2::new(0.0, 0.0),
                        Vec2::new(1.0, 0.0),
                        Vec2::new(2.0, 0.0),
                        Vec2::new(3.0, 0.0),
                        Vec2::new(7.0, 0.0),
                        Vec2::new(8.0, 0.0),
                        Vec2::new(9.0, 0.0),
                        Vec2::new(10.0, 0.0),
                    ],
                }],
            },
            prepared_segments: Vec::new(),
        });

        tool.rebuild_prepared_segments();
        let road_map = RoadMap::new(3);
        let before_execute = tool
            .execute(&road_map)
            .expect("Preview mit Segmenten sollte ausfuehrbar sein");
        let before_edges = before_execute.internal_connections.len();
        let before_prepared_revision = tool.cache.prepared_segments_revision;
        let preview_core_revision = tool.cache.preview_core_revision;

        tool.config.junction_radius = 2.5;
        tool.on_preview_geometry_config_changed();

        let after_execute = tool
            .execute(&road_map)
            .expect("Getrimmte Preview sollte weiterhin ausfuehrbar sein");
        let after_edges = after_execute.internal_connections.len();
        let max_after_edge_length = after_execute
            .internal_connections
            .iter()
            .map(|&(from, to, _, _)| {
                let start = after_execute
                    .new_nodes
                    .get(from)
                    .expect("Startknoten der Kante muss existieren")
                    .0;
                let end = after_execute
                    .new_nodes
                    .get(to)
                    .expect("Endknoten der Kante muss existieren")
                    .0;
                start.distance(end)
            })
            .fold(0.0_f32, f32::max);

        assert_eq!(tool.phase, ColorPathPhase::Finalize);
        assert_eq!(tool.cache.preview_core_revision, preview_core_revision);
        assert!(tool.cache.prepared_segments_revision > before_prepared_revision);
        assert_eq!(
            after_edges, before_edges,
            "Junction-Radius wirkt nur auf Kreuzungsbegradigung; finale Kantenanzahl folgt node_spacing"
        );
        assert!(
            max_after_edge_length <= tool.config.node_spacing + 1e-4,
            "Finale Geometrie muss nach Begradigung auf node_spacing resampled sein"
        );
    }

    // ---------------------------------------------------------------------
    // CP-05 — Wizard-Transition-Tests (Next/Prev/Accept).
    // ---------------------------------------------------------------------

    /// Baut ein Tool auf, das an der Schwelle JunctionEdit → Finalize steht:
    /// Netz ist vorhanden, Stage F aber noch nicht berechnet.
    #[allow(deprecated)]
    fn build_junction_edit_tool() -> ColorPathTool {
        // Netz mit echten Polyline-Segmenten, damit Stage F beim Rebuild
        // auch tatsaechlich `PreparedSegment`s produziert.
        let mut tool = ColorPathTool::new();
        tool.phase = ColorPathPhase::JunctionEdit;
        tool.direction = ConnectionDirection::Regular;
        tool.priority = ConnectionPriority::Regular;
        tool.config.existing_connection_mode = ExistingConnectionMode::Never;
        tool.config.simplify_tolerance = 0.0;
        tool.config.node_spacing = 1.0;
        tool.config.junction_radius = 0.0;
        tool.preview_data = Some(PreviewData {
            prepared_mask: ColorPathMask::default(),
            network: SkeletonNetwork {
                nodes: vec![
                    SkeletonGraphNode {
                        kind: SkeletonGraphNodeKind::OpenEnd,
                        pixel_position: Vec2::new(0.0, 0.0),
                        world_position: Vec2::new(0.0, 0.0),
                    },
                    SkeletonGraphNode {
                        kind: SkeletonGraphNodeKind::OpenEnd,
                        pixel_position: Vec2::new(10.0, 0.0),
                        world_position: Vec2::new(10.0, 0.0),
                    },
                ],
                segments: vec![SkeletonGraphSegment {
                    start_node: 0,
                    end_node: 1,
                    polyline: vec![
                        Vec2::new(0.0, 0.0),
                        Vec2::new(2.5, 0.0),
                        Vec2::new(5.0, 0.0),
                        Vec2::new(7.5, 0.0),
                        Vec2::new(10.0, 0.0),
                    ],
                }],
            },
            prepared_segments: Vec::new(),
        });
        tool.lifecycle.snap_radius = 1.0;
        tool
    }

    #[test]
    #[allow(deprecated)]
    fn wizard_next_phase_junction_edit_rebuilds_stage_f_into_finalize() {
        use crate::app::ui_contract::ColorPathPanelAction;

        let mut tool = build_junction_edit_tool();
        assert_eq!(tool.phase, ColorPathPhase::JunctionEdit);
        assert!(
            tool.preview_data
                .as_ref()
                .is_some_and(|p| p.prepared_segments.is_empty()),
            "Stage F muss in JunctionEdit leer sein"
        );

        let effect = tool.apply_panel_action(ColorPathPanelAction::NextPhase);

        assert!(effect.changed, "NextPhase muss einen Uebergang melden");
        assert!(effect.next_action.is_none());
        assert_eq!(tool.phase, ColorPathPhase::Finalize);
        assert!(
            tool.preview_data
                .as_ref()
                .is_some_and(|p| !p.prepared_segments.is_empty()),
            "Stage F muss nach Finalize-Eintritt befuellt sein"
        );
    }

    #[test]
    #[allow(deprecated)]
    fn wizard_prev_phase_finalize_drops_stage_f_and_keeps_network() {
        use crate::app::ui_contract::ColorPathPanelAction;

        let mut tool = build_preview_tool(ExistingConnectionMode::Never);
        let network_segments_before = tool
            .preview_data
            .as_ref()
            .map(|p| p.network.nodes.len())
            .unwrap_or_default();

        let effect = tool.apply_panel_action(ColorPathPanelAction::PrevPhase);

        assert!(effect.changed);
        assert_eq!(tool.phase, ColorPathPhase::JunctionEdit);
        let preview = tool
            .preview_data
            .as_ref()
            .expect("Netz darf beim PrevPhase aus Finalize nicht verworfen werden");
        assert!(preview.prepared_segments.is_empty());
        assert_eq!(preview.network.nodes.len(), network_segments_before);
        assert!(tool.cache.prepared_segments_key.is_none());
    }

    #[test]
    #[allow(deprecated)]
    fn wizard_prev_phase_centerline_clears_preview_and_returns_to_sampling() {
        use crate::app::ui_contract::ColorPathPanelAction;

        let mut tool = build_preview_tool(ExistingConnectionMode::Never);
        // Finalize → JunctionEdit → CenterlinePreview.
        assert!(
            tool.apply_panel_action(ColorPathPanelAction::PrevPhase)
                .changed
        );
        assert!(
            tool.apply_panel_action(ColorPathPanelAction::PrevPhase)
                .changed
        );
        assert_eq!(tool.phase, ColorPathPhase::CenterlinePreview);

        let effect = tool.apply_panel_action(ColorPathPanelAction::PrevPhase);

        assert!(effect.changed);
        assert_eq!(tool.phase, ColorPathPhase::Sampling);
        assert!(
            tool.preview_data.is_none(),
            "CenterlinePreview → Sampling muss Preview-Pipeline verwerfen"
        );
    }

    #[test]
    #[allow(deprecated)]
    fn wizard_accept_in_finalize_emits_ready_to_execute() {
        use crate::app::ui_contract::{ColorPathPanelAction, RouteToolPanelFollowUp};

        let mut tool = build_preview_tool(ExistingConnectionMode::Never);
        let effect = tool.apply_panel_action(ColorPathPanelAction::Accept);

        assert_eq!(
            effect.next_action,
            Some(RouteToolPanelFollowUp::ReadyToExecute),
            "Accept im Finalize muss den Apply-Pfad anstossen"
        );
        assert_eq!(tool.phase, ColorPathPhase::Finalize);
    }

    #[test]
    #[allow(deprecated)]
    fn wizard_accept_outside_finalize_is_noop() {
        use crate::app::ui_contract::ColorPathPanelAction;

        let mut tool = build_junction_edit_tool();
        let effect = tool.apply_panel_action(ColorPathPanelAction::Accept);
        assert!(effect.next_action.is_none());
        assert!(!effect.changed);
        assert_eq!(tool.phase, ColorPathPhase::JunctionEdit);
    }

    #[test]
    #[allow(deprecated)]
    fn wizard_legacy_actions_alias_to_wizard_transitions() {
        use crate::app::ui_contract::ColorPathPanelAction;

        // Ein Tool in Finalize mit echter Netz-Polyline (nicht der leere
        // `sample_network`-Helper, damit Stage F beim erneuten Rebuild auch
        // wirklich Segmente produziert).
        let mut tool = build_junction_edit_tool();
        assert!(
            tool.apply_panel_action(ColorPathPanelAction::NextPhase)
                .changed
        );
        assert_eq!(tool.phase, ColorPathPhase::Finalize);

        // BackToSampling wirkt jetzt wie PrevPhase — aus Finalize fuehrt das
        // in den JunctionEdit und laesst das Netz unberuehrt.
        let effect = tool.apply_panel_action(ColorPathPanelAction::BackToSampling);
        assert!(effect.changed);
        assert_eq!(tool.phase, ColorPathPhase::JunctionEdit);
        assert!(tool.preview_data.is_some());

        // ComputePreview wirkt jetzt wie NextPhase — aus JunctionEdit fuehrt
        // das in den Finalize mit frisch berechneter Stage F.
        let effect = tool.apply_panel_action(ColorPathPanelAction::ComputePreview);
        assert!(effect.changed);
        assert_eq!(tool.phase, ColorPathPhase::Finalize);
        assert!(
            tool.preview_data
                .as_ref()
                .is_some_and(|p| !p.prepared_segments.is_empty())
        );
    }

    // ---------------------------------------------------------------------
    // R1/T1 — Config-Change in Editing-Phase darf Editable nicht veralten lassen.
    // ---------------------------------------------------------------------

    /// Fuehrt ein Tool von Idle bis `JunctionEdit` ueber den echten Wizard-Pfad.
    fn drive_tool_to_junction_edit() -> ColorPathTool {
        let image = Arc::new(build_test_image());
        let road_map = RoadMap::default();
        let mut tool = ColorPathTool::new();
        tool.phase = ColorPathPhase::Idle;
        tool.set_background_map_image(Some(Arc::clone(&image)));

        let click_pos = pixel_to_world(0, 0, tool.map_size, 10, 10);
        let _ = tool.on_click(click_pos, &road_map, false);
        assert_eq!(tool.phase, ColorPathPhase::Sampling);

        use crate::app::ui_contract::ColorPathPanelAction;
        assert!(
            tool.apply_panel_action(ColorPathPanelAction::NextPhase)
                .changed,
            "Sampling → CenterlinePreview muss gelingen"
        );
        assert_eq!(tool.phase, ColorPathPhase::CenterlinePreview);
        assert!(
            tool.apply_panel_action(ColorPathPanelAction::NextPhase)
                .changed,
            "CenterlinePreview → JunctionEdit muss gelingen"
        );
        assert_eq!(tool.phase, ColorPathPhase::JunctionEdit);
        assert!(tool.editable.is_some());
        tool
    }

    #[test]
    fn noise_filter_change_in_junction_edit_resyncs_editable() {
        let mut tool = drive_tool_to_junction_edit();

        // Drag-Artefakt simulieren: eine Junction verschieben und den Drag-Handle setzen.
        let first_id = {
            let editable = tool
                .editable
                .as_ref()
                .expect("Editable muss in JunctionEdit vorhanden sein");
            *editable
                .junctions
                .keys()
                .min_by_key(|id| id.0)
                .expect("Netz braucht mindestens eine Junction")
        };
        let original_pos = tool.editable.as_ref().unwrap().junctions[&first_id].world_pos;
        let dragged_pos = original_pos + Vec2::new(100.0, 100.0);
        tool.editable
            .as_mut()
            .unwrap()
            .move_junction(first_id, dragged_pos);
        tool.dragging_junction = Some(first_id);

        // Preview-Core-Config aendern: noise_filter toggeln.
        tool.config.noise_filter = !tool.config.noise_filter;
        tool.on_preview_core_config_changed();

        // R1: Phase bleibt JunctionEdit, Editable wurde neu synchronisiert.
        assert_eq!(tool.phase, ColorPathPhase::JunctionEdit);
        let editable = tool
            .editable
            .as_ref()
            .expect("Editable muss nach Re-Sync erneut vorhanden sein");
        let network = &tool
            .preview_data
            .as_ref()
            .expect("Netz muss nach Rebuild vorhanden sein")
            .network;
        for id in editable.junctions.keys() {
            assert!(
                (id.0 as usize) < network.nodes.len(),
                "Editable-ID {id:?} muss auf einen gueltigen Netz-Knoten zeigen"
            );
        }
        // Gedraggte Junction wurde durch den Re-Sync auf die Netz-Position zurueckgesetzt.
        if let Some(refreshed) = editable.junctions.get(&first_id) {
            assert_ne!(
                refreshed.world_pos, dragged_pos,
                "Re-Sync muss die verschobene Junction verwerfen"
            );
        }
        // F3/R2: Drag-Handle wurde verworfen, um Zugriff auf veraltete IDs zu vermeiden.
        assert!(
            tool.dragging_junction.is_none(),
            "dragging_junction muss nach Strukturaenderung geleert sein"
        );
    }

    #[test]
    fn matching_config_change_in_junction_edit_keeps_phase() {
        let mut tool = drive_tool_to_junction_edit();

        // Farb-Matching aendern (R2): darf Phase nicht auf Finalize schieben.
        tool.config.color_tolerance = (tool.config.color_tolerance + 5.0).clamp(1.0, 80.0);
        tool.on_matching_config_changed();

        assert_eq!(
            tool.phase,
            ColorPathPhase::JunctionEdit,
            "Matching-Change darf die Wizard-Phase nicht veraendern"
        );
    }
}
