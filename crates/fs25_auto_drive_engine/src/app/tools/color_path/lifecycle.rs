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
    /// Liefert `true`, wenn das Tool aktuell ausfuehrbar ist.
    ///
    /// Ersetzt das alte `phase.is_finalized()`-Idiom: gilt genau dann, wenn der
    /// Wizard in der Editing-Phase steht und Stage F bereits exportierbare
    /// `prepared_segments` erzeugt hat. Wird sowohl von [`RouteToolCore`] als
    /// auch von der Panel-Bruecke (`can_accept`) konsumiert.
    pub(super) fn can_execute(&self) -> bool {
        matches!(self.phase, ColorPathPhase::Editing)
            && self
                .preview_data
                .as_ref()
                .is_some_and(|preview| !preview.prepared_segments.is_empty())
    }

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
    /// In der Editing-Phase darf ein Matching-Wechsel die Wizard-Phase niemals
    /// veraendern. Das Netz wird in-place neu aufgebaut und das
    /// Editable-Modell nachgezogen; aktive Junction-Drags (Handle) werden
    /// verworfen, weil die alten IDs nach dem Rebuild nicht mehr stabil sind.
    pub(super) fn on_matching_config_changed(&mut self) {
        match self.phase {
            ColorPathPhase::Idle => self.refresh_matching_spec(),
            ColorPathPhase::Sampling => self.rebuild_sampling_preview(),
            ColorPathPhase::Editing => {
                self.rebuild_editing_preview_in_place(true);
            }
        }
    }

    /// Reagiert auf Aenderungen der Stage-D/E-Konfiguration.
    ///
    /// Wie bei [`Self::on_matching_config_changed`] bleibt die Phase in der
    /// Editing-Phase erhalten; das Editable-Modell wird nach dem Rebuild
    /// konsistent neu synchronisiert (siehe R1/F1).
    pub(super) fn on_preview_core_config_changed(&mut self) {
        match self.phase {
            ColorPathPhase::Idle | ColorPathPhase::Sampling => {}
            ColorPathPhase::Editing => {
                self.rebuild_editing_preview_in_place(true);
            }
        }
    }

    /// Reagiert auf Aenderungen der Stage-F-Konfiguration.
    ///
    /// In der Editing-Phase wird Stage F direkt neu berechnet (Live-Slider,
    /// keine Phase-Aenderung); in den Phasen `Idle`/`Sampling` ist der Aufruf
    /// ein No-Op, da ohne fertiges Netz auch kein Stage-F-Cache existiert,
    /// der invalidiert werden muesste.
    pub(super) fn on_preview_geometry_config_changed(&mut self) {
        match self.phase {
            ColorPathPhase::Editing => {
                let _ = self.rebuild_stage_f_only();
            }
            ColorPathPhase::Idle | ColorPathPhase::Sampling => {}
        }
    }

    /// Baut das Preview-Netz in einer Editing-Phase neu auf, ohne die Phase
    /// zu veraendern.
    ///
    /// Invalidiert den aktiven Junction-Drag-Handle, weil die Editable-IDs
    /// nach dem Rebuild nicht mehr garantiert zum vorherigen Snapshot
    /// passen. Wenn `include_stage_f` gesetzt ist, wird anschliessend Stage F
    /// direkt neu berechnet.
    ///
    /// CP-03: Wenn `editable_user_dirty` gesetzt ist, werden die alten
    /// Junction-Positionen (per stabiler [`super::editable::EditableJunctionId`])
    /// vor dem Resync gesnapshottet und danach auf das frisch synchronisierte
    /// Editable-Modell zurueckgemappt. Existiert keine der alten IDs mehr im
    /// neuen Skelett, faellt die Routine auf eine reine Resynchronisation
    /// zurueck, leert das Dirty-Flag und loggt eine Warnung.
    fn rebuild_editing_preview_in_place(&mut self, include_stage_f: bool) {
        // Vor dem Rebuild des Preview-Cores die User-Edits sichern, damit ein
        // Slider-Change ungewollten Drag nicht ueberschreibt.
        let preserved_user_positions: Option<Vec<(super::editable::EditableJunctionId, Vec2)>> =
            if self.editable_user_dirty {
                self.editable.as_ref().map(|editable| {
                    editable
                        .junctions
                        .iter()
                        .map(|(id, junction)| (*id, junction.world_pos))
                        .collect()
                })
            } else {
                None
            };

        if !self.rebuild_preview_core_only() {
            return;
        }
        self.sync_editable_from_network();
        self.bump_editable_revision();
        self.dragging_junction = None;

        // Re-Apply der User-Drags auf das frisch synchronisierte Editable-Modell.
        if let Some(saved) = preserved_user_positions {
            let mut applied = 0usize;
            if let Some(editable) = self.editable.as_mut() {
                for (id, pos) in &saved {
                    if editable.move_junction(*id, *pos) {
                        applied += 1;
                    }
                }
            }
            if applied == 0 && !saved.is_empty() {
                log::warn!(
                    "ColorPathTool: User-Drags konnten nach Resync nicht erhalten werden — Topologie hat sich geaendert; Editable-Modell zurueckgesetzt"
                );
                self.editable_user_dirty = false;
            }
        }

        if include_stage_f {
            let _ = self.rebuild_stage_f_only();
        }
    }

    /// Reagiert auf Aenderungen am Bild-/Map-Kontext der Sampling-Pipeline.
    pub(super) fn on_sampling_context_changed(&mut self) {
        match self.phase {
            ColorPathPhase::Idle => self.clear_sampling_preview(),
            ColorPathPhase::Sampling => self.rebuild_sampling_preview(),
            ColorPathPhase::Editing => self.compute_pipeline(),
        }
    }

    /// Fuehrt aus der Sampling-Phase die volle Stage-C-bis-F-Pipeline aus und
    /// wechselt in die Editing-Phase.
    ///
    /// Zentrale Routine seit CP-02 (Single-Step): laeuft strikt in der
    /// Reihenfolge `rebuild_preview_core_only` → `sync_editable_from_network`
    /// → `bump_editable_revision` → `rebuild_stage_f_only` → `Phase = Editing`.
    /// Schlaegt Stage E fehl, bleibt die Phase auf `Sampling`. Schlaegt nur
    /// Stage F fehl, geht das Tool trotzdem nach `Editing`; `prepared_segments`
    /// ist dann leer und [`ColorPathTool::can_execute`] liefert `false`.
    pub(super) fn compute_to_editing(&mut self) {
        // Stage C-E: Centerline-Netz aufbauen.
        if !self.rebuild_preview_core_only() {
            log::warn!("ColorPathTool: Kein exportierbares Netz gefunden — Phase bleibt Sampling");
            return;
        }
        self.sync_editable_from_network();
        self.bump_editable_revision();

        // CP-03: Frischer Compute = frische Editable-Basis ohne User-Drags.
        self.editable_user_dirty = false;

        // Stage F: Junction-Trim und Resampling; bei Fehlschlag bleibt Phase
        // Editing mit leeren `prepared_segments`.
        if !self.rebuild_stage_f_only() {
            log::warn!(
                "ColorPathTool: Netz extrahiert, aber keine gueltigen Preview-Segmente erzeugt"
            );
        }

        self.phase = ColorPathPhase::Editing;
    }

    /// Pruefen-Wrapper um [`Self::compute_to_editing`].
    ///
    /// Validiert die Sampling-Eingaben (Hintergrundbild, Farbsamples,
    /// Lasso-Start) und delegiert dann unveraendert an die zentrale Routine.
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

        self.compute_to_editing();
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
        self.editable_user_dirty = false;
        self.cache = super::state::ColorPathCacheState::default();
    }

    /// Synchronisiert das editierbare Zwischenmodell mit dem aktuellen Stage-E-Netz.
    ///
    /// Wird beim Eintritt in die Editing-Phase aufgerufen und rekonstruiert
    /// [`super::editable::EditableCenterlines`] aus `preview_data.network`. Fehlt
    /// das Netz (z. B. vor dem ersten erfolgreichen Stage-E-Durchlauf), wird
    /// das Editable-Feld geleert.
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
            ColorPathPhase::Editing => "Enter zum Einfuegen, Reset zum Zuruecksetzen",
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
            ColorPathPhase::Editing => {
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
            ColorPathPhase::Editing => self.build_network_preview(),
        }
    }

    fn execute(&self, road_map: &RoadMap) -> Option<ToolResult> {
        if !self.can_execute() {
            return None;
        }

        self.execute_result(road_map)
    }

    fn reset(&mut self) {
        self.reset_all();
    }

    fn is_ready(&self) -> bool {
        self.can_execute()
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
        if self.phase.is_editing() {
            Some(self)
        } else {
            None
        }
    }

    fn as_drag_mut(&mut self) -> Option<&mut dyn RouteToolDrag> {
        if self.phase.is_editing() {
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
        tool.phase = ColorPathPhase::Editing;
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
        tool.phase = ColorPathPhase::Editing;
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

        assert_eq!(tool.phase, ColorPathPhase::Editing);
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
    // CP-01 — Wizard-Transition-Tests (Compute/Accept) im Single-Step-Modell.
    // ---------------------------------------------------------------------

    /// Baut ein Tool auf, das in `Editing` ohne fertige Stage F steht:
    /// Netz ist vorhanden, `prepared_segments` aber noch leer.
    fn build_editing_tool_without_stage_f() -> ColorPathTool {
        // Netz mit echten Polyline-Segmenten, damit Stage F beim Rebuild
        // auch tatsaechlich `PreparedSegment`s produziert.
        let mut tool = ColorPathTool::new();
        tool.phase = ColorPathPhase::Editing;
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
    fn wizard_accept_in_editing_emits_ready_to_execute() {
        use crate::app::ui_contract::{ColorPathPanelAction, RouteToolPanelFollowUp};

        let mut tool = build_preview_tool(ExistingConnectionMode::Never);
        let effect = tool.apply_panel_action(ColorPathPanelAction::Accept);

        assert_eq!(
            effect.next_action,
            Some(RouteToolPanelFollowUp::ReadyToExecute),
            "Accept im Editing mit Stage F muss den Apply-Pfad anstossen"
        );
        assert_eq!(tool.phase, ColorPathPhase::Editing);
    }

    #[test]
    fn wizard_accept_without_stage_f_is_noop() {
        use crate::app::ui_contract::ColorPathPanelAction;

        let mut tool = build_editing_tool_without_stage_f();
        let effect = tool.apply_panel_action(ColorPathPanelAction::Accept);
        assert!(effect.next_action.is_none());
        assert!(!effect.changed);
        assert_eq!(tool.phase, ColorPathPhase::Editing);
    }

    #[test]
    fn legacy_prev_phase_action_in_editing_resets_to_idle() {
        use crate::app::ui_contract::ColorPathPanelAction;

        let mut tool = build_preview_tool(ExistingConnectionMode::Never);
        #[allow(deprecated)] // CP-06: PrevPhase mappt auf Reset → Phase wird Idle.
        let effect = tool.apply_panel_action(ColorPathPanelAction::PrevPhase);

        assert!(effect.changed);
        // Single-Step (CP-06): PrevPhase ist Legacy-Alias fuer Reset → vollstaendiger
        // Reset statt Rueckweg in Sampling.
        assert_eq!(tool.phase, ColorPathPhase::Idle);
        assert!(
            tool.preview_data.is_none(),
            "PrevPhase aus Editing muss die Preview-Pipeline verwerfen"
        );
        assert!(tool.editable.is_none());
    }

    // ---------------------------------------------------------------------
    // R1/T1 — Config-Change in Editing-Phase darf Editable nicht veralten lassen.
    // ---------------------------------------------------------------------

    /// Fuehrt ein Tool von Idle bis `Editing` ueber den echten Compute-Pfad.
    fn drive_tool_to_editing() -> ColorPathTool {
        let image = Arc::new(build_test_image());
        let road_map = RoadMap::default();
        let mut tool = ColorPathTool::new();
        tool.phase = ColorPathPhase::Idle;
        tool.set_background_map_image(Some(Arc::clone(&image)));

        let click_pos = pixel_to_world(0, 0, tool.map_size, 10, 10);
        let _ = tool.on_click(click_pos, &road_map, false);
        assert_eq!(tool.phase, ColorPathPhase::Sampling);

        use crate::app::ui_contract::ColorPathPanelAction;
        let advanced = tool
            .apply_panel_action(ColorPathPanelAction::Compute)
            .changed;
        assert!(advanced, "Sampling → Editing muss gelingen");
        assert_eq!(tool.phase, ColorPathPhase::Editing);
        assert!(tool.editable.is_some());
        tool
    }

    #[test]
    fn noise_filter_change_in_editing_resyncs_editable() {
        let mut tool = drive_tool_to_editing();

        // Drag-Artefakt simulieren: eine Junction verschieben und den Drag-Handle setzen.
        let first_id = {
            let editable = tool
                .editable
                .as_ref()
                .expect("Editable muss in Editing vorhanden sein");
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

        // R1: Phase bleibt Editing, Editable wurde neu synchronisiert.
        assert_eq!(tool.phase, ColorPathPhase::Editing);
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
    fn matching_config_change_in_editing_keeps_phase() {
        let mut tool = drive_tool_to_editing();

        // Farb-Matching aendern (R2): darf Phase nicht aus Editing schubsen.
        tool.config.color_tolerance = (tool.config.color_tolerance + 5.0).clamp(1.0, 80.0);
        tool.on_matching_config_changed();

        assert_eq!(
            tool.phase,
            ColorPathPhase::Editing,
            "Matching-Change darf die Wizard-Phase nicht veraendern"
        );
    }

    // ---------------------------------------------------------------------
    // CP-02 — Single-Step-Pipeline + Live-Slider in Editing.
    // ---------------------------------------------------------------------

    /// `compute_to_editing()` ueber den vollen Sampling-Pfad: Phase landet in
    /// `Editing` und Stage F erzeugt `prepared_segments`, sodass das Tool
    /// direkt ausfuehrbar ist.
    #[test]
    fn compute_action_enters_editing_with_stage_f() {
        let tool = drive_tool_to_editing();

        assert_eq!(tool.phase, ColorPathPhase::Editing);
        let preview = tool
            .preview_data
            .as_ref()
            .expect("Preview-Daten muessen nach compute_to_editing existieren");
        assert!(
            !preview.prepared_segments.is_empty(),
            "Stage F muss `prepared_segments` produzieren"
        );
        assert!(tool.can_execute());
    }

    /// Geometry-Slider (Stage F) in Editing: Phase bleibt Editing, nur Stage F
    /// wird neu gerechnet — der Stage-E-Cache (`preview_core_revision`) bleibt
    /// unveraendert.
    #[test]
    fn geometry_slider_in_editing_triggers_stage_f() {
        let mut tool = drive_tool_to_editing();
        let core_revision_before = tool.cache.preview_core_revision;
        let prepared_revision_before = tool.cache.prepared_segments_revision;

        tool.config.junction_radius = (tool.config.junction_radius + 1.5).clamp(0.0, 100.0);
        tool.on_preview_geometry_config_changed();

        assert_eq!(tool.phase, ColorPathPhase::Editing);
        assert_eq!(
            tool.cache.preview_core_revision, core_revision_before,
            "Stage E darf bei reinem Geometry-Change nicht neu laufen"
        );
        assert!(
            tool.cache.prepared_segments_revision > prepared_revision_before,
            "Stage F muss live neu berechnet werden"
        );
    }

    /// Matching-Slider in Editing: Phase bleibt Editing, Stage E *und* Stage F
    /// werden neu aufgebaut, ein aktiver Drag-Handle wird verworfen.
    #[test]
    fn matching_slider_in_editing_rebuilds_core_and_stage_f() {
        let mut tool = drive_tool_to_editing();
        let core_revision_before = tool.cache.preview_core_revision;
        let prepared_revision_before = tool.cache.prepared_segments_revision;
        // Drag-Handle simulieren, um das Clearen nachzuweisen.
        let some_id = *tool
            .editable
            .as_ref()
            .unwrap()
            .junctions
            .keys()
            .next()
            .expect("Editable braucht mindestens eine Junction");
        tool.dragging_junction = Some(some_id);

        // exact_color_match toggeln invalidiert die Matching-Spezifikation
        // unabhaengig von Tolerance-Defaults und triggert Stage C–E neu.
        tool.config.exact_color_match = !tool.config.exact_color_match;
        tool.on_matching_config_changed();

        assert_eq!(tool.phase, ColorPathPhase::Editing);
        assert!(
            tool.cache.preview_core_revision > core_revision_before,
            "Matching-Change muss Stage E neu aufbauen"
        );
        assert!(
            tool.cache.prepared_segments_revision > prepared_revision_before,
            "Matching-Change muss anschliessend auch Stage F neu erzeugen"
        );
        assert!(
            tool.dragging_junction.is_none(),
            "Aktiver Drag-Handle muss verworfen werden"
        );
    }

    /// Preview-Core-Slider (Noise/Skeleton) in Editing: Phase bleibt Editing,
    /// Stage E und Stage F werden neu gerechnet, Drag-Handle geleert.
    #[test]
    fn preview_core_slider_in_editing_rebuilds_core_and_stage_f() {
        let mut tool = drive_tool_to_editing();
        let core_revision_before = tool.cache.preview_core_revision;
        let prepared_revision_before = tool.cache.prepared_segments_revision;
        let some_id = *tool
            .editable
            .as_ref()
            .unwrap()
            .junctions
            .keys()
            .next()
            .expect("Editable braucht mindestens eine Junction");
        tool.dragging_junction = Some(some_id);

        tool.config.noise_filter = !tool.config.noise_filter;
        tool.on_preview_core_config_changed();

        assert_eq!(tool.phase, ColorPathPhase::Editing);
        assert!(
            tool.cache.preview_core_revision > core_revision_before,
            "Core-Change muss Stage E neu aufbauen"
        );
        assert!(
            tool.cache.prepared_segments_revision > prepared_revision_before,
            "Core-Change muss anschliessend auch Stage F neu erzeugen"
        );
        assert!(
            tool.dragging_junction.is_none(),
            "Aktiver Drag-Handle muss verworfen werden"
        );
    }

    // ---------------------------------------------------------------------
    // CP-03 — User-Drag-Schutz gegen Matching-/Preview-Core-Resync.
    // ---------------------------------------------------------------------

    /// `on_drag_update` muss das `editable_user_dirty`-Flag setzen, sobald die
    /// Junction tatsaechlich verschoben wurde.
    #[test]
    fn editable_user_dirty_set_by_drag_update() {
        use crate::app::tools::color_path::drag::{on_drag_start, on_drag_update};

        let mut tool = drive_tool_to_editing();
        let road_map = RoadMap::default();
        assert!(
            !tool.editable_user_dirty,
            "Frisch berechnete Editable-Basis darf nicht dirty sein"
        );

        let (first_id, pick_pos) = {
            let editable = tool.editable.as_ref().expect("Editable muss existieren");
            let id = *editable
                .junctions
                .keys()
                .next()
                .expect("Editable braucht mindestens eine Junction");
            (id, editable.junctions[&id].world_pos)
        };

        // Pick-Radius bewusst gross, damit der Test unabhaengig von der konkreten
        // Junction-Position bleibt.
        assert!(on_drag_start(&mut tool, pick_pos, &road_map, 1_000.0));
        on_drag_update(&mut tool, pick_pos + Vec2::new(2.0, 3.0));

        assert!(
            tool.editable_user_dirty,
            "Drag-Update muss editable_user_dirty setzen"
        );
        let moved_pos = tool.editable.as_ref().unwrap().junctions[&first_id].world_pos;
        assert_eq!(moved_pos, pick_pos + Vec2::new(2.0, 3.0));
    }

    /// User-Drag-Position bleibt nach einem Resync der Editable-Basis erhalten,
    /// solange die [`super::editable::EditableJunctionId`] noch existiert.
    #[test]
    fn user_drag_position_persists_through_editable_resync() {
        let mut tool = drive_tool_to_editing();
        let first_id = *tool
            .editable
            .as_ref()
            .unwrap()
            .junctions
            .keys()
            .next()
            .expect("Editable braucht mindestens eine Junction");
        let user_pos = Vec2::new(42.0, 17.0);

        // User-Drag simulieren: Position setzen und Flag aktivieren.
        assert!(tool
            .editable
            .as_mut()
            .unwrap()
            .move_junction(first_id, user_pos));
        tool.editable_user_dirty = true;

        // Resync (wie er von on_matching_config_changed / on_preview_core_config_changed
        // ausgeloest wird). Das identische Sampling-Setup haelt das Skelett stabil,
        // sodass die ID-Map durchgehend aufloest.
        tool.rebuild_editing_preview_in_place(true);

        assert!(
            tool.editable_user_dirty,
            "Flag bleibt true, solange mindestens eine User-ID erhalten werden konnte"
        );
        let editable = tool.editable.as_ref().expect("Editable muss bestehen");
        assert_eq!(
            editable.junctions[&first_id].world_pos, user_pos,
            "User-Drag-Position muss durch den Resync erhalten bleiben"
        );
    }

    /// `Reset` raeumt das Dirty-Flag mit auf, damit die naechste Editing-Sitzung
    /// wieder mit frischer Editable-Basis startet.
    #[test]
    fn editable_user_dirty_cleared_by_reset() {
        use crate::app::ui_contract::ColorPathPanelAction;

        let mut tool = drive_tool_to_editing();
        tool.editable_user_dirty = true;

        let _ = tool.apply_panel_action(ColorPathPanelAction::Reset);

        assert!(
            !tool.editable_user_dirty,
            "Reset muss editable_user_dirty zurueck auf false setzen"
        );
    }

    /// Ein erneuter Compute (Sampling → Editing) verwirft alte User-Drags.
    #[test]
    fn editable_user_dirty_cleared_by_compute() {
        let mut tool = drive_tool_to_editing();
        tool.editable_user_dirty = true;

        tool.compute_to_editing();

        assert!(
            !tool.editable_user_dirty,
            "compute_to_editing muss editable_user_dirty leeren"
        );
    }

    // ---------------------------------------------------------------------
    // CP-06 — apply_panel_action: kanonische Aktionen + Legacy-Mapping.
    // ---------------------------------------------------------------------

    /// Baut ein Sampling-Tool mit echten Farbsamples auf, sodass `Compute`
    /// einen vollstaendigen Pipeline-Lauf nach `Editing` ausfuehren kann.
    fn build_sampling_tool_with_samples() -> ColorPathTool {
        let image = Arc::new(build_test_image());
        let mut tool = ColorPathTool::new();
        tool.phase = ColorPathPhase::Idle;
        tool.set_background_map_image(Some(image));

        let click_pos = pixel_to_world(0, 0, tool.map_size, 10, 10);
        let road_map = RoadMap::default();
        let _ = tool.on_click(click_pos, &road_map, false);
        assert_eq!(tool.phase, ColorPathPhase::Sampling);
        tool
    }

    #[test]
    fn cp06_start_sampling_from_idle_enters_sampling() {
        use crate::app::ui_contract::ColorPathPanelAction;

        let mut tool = ColorPathTool::new();
        tool.phase = ColorPathPhase::Idle;
        let effect = tool.apply_panel_action(ColorPathPanelAction::StartSampling);

        assert!(effect.changed);
        assert_eq!(tool.phase, ColorPathPhase::Sampling);
        assert!(effect.next_action.is_none());
    }

    #[test]
    fn cp06_start_sampling_in_sampling_is_noop() {
        use crate::app::ui_contract::ColorPathPanelAction;

        let mut tool = ColorPathTool::new();
        tool.phase = ColorPathPhase::Sampling;
        let effect = tool.apply_panel_action(ColorPathPanelAction::StartSampling);

        assert!(!effect.changed);
        assert_eq!(tool.phase, ColorPathPhase::Sampling);
    }

    #[test]
    fn cp06_compute_in_sampling_drives_to_editing() {
        use crate::app::ui_contract::{ColorPathPanelAction, RouteToolPanelFollowUp};

        let mut tool = build_sampling_tool_with_samples();
        let effect = tool.apply_panel_action(ColorPathPanelAction::Compute);

        assert!(effect.changed);
        assert_eq!(tool.phase, ColorPathPhase::Editing);
        assert_eq!(
            effect.next_action,
            Some(RouteToolPanelFollowUp::UpdatePreview)
        );
    }

    #[test]
    fn cp06_compute_without_samples_is_noop() {
        use crate::app::ui_contract::ColorPathPanelAction;

        let mut tool = ColorPathTool::new();
        tool.phase = ColorPathPhase::Sampling;
        let effect = tool.apply_panel_action(ColorPathPanelAction::Compute);

        assert!(!effect.changed);
        assert_eq!(tool.phase, ColorPathPhase::Sampling);
        assert!(effect.next_action.is_none());
    }

    #[test]
    fn cp06_compute_in_editing_is_noop() {
        use crate::app::ui_contract::ColorPathPanelAction;

        let mut tool = build_preview_tool(ExistingConnectionMode::Never);
        let effect = tool.apply_panel_action(ColorPathPanelAction::Compute);

        assert!(!effect.changed);
        assert_eq!(tool.phase, ColorPathPhase::Editing);
        assert!(effect.next_action.is_none());
    }

    #[test]
    fn cp06_accept_in_editing_emits_ready_to_execute() {
        use crate::app::ui_contract::{ColorPathPanelAction, RouteToolPanelFollowUp};

        let mut tool = build_preview_tool(ExistingConnectionMode::Never);
        let effect = tool.apply_panel_action(ColorPathPanelAction::Accept);

        assert_eq!(
            effect.next_action,
            Some(RouteToolPanelFollowUp::ReadyToExecute)
        );
        assert_eq!(tool.phase, ColorPathPhase::Editing);
    }

    #[test]
    fn cp06_accept_outside_editing_is_noop() {
        use crate::app::ui_contract::ColorPathPanelAction;

        let mut tool = ColorPathTool::new();
        tool.phase = ColorPathPhase::Sampling;
        let effect = tool.apply_panel_action(ColorPathPanelAction::Accept);

        assert!(!effect.changed);
        assert!(effect.next_action.is_none());
    }

    #[test]
    fn cp06_reset_in_editing_clears_state() {
        use crate::app::ui_contract::ColorPathPanelAction;

        let mut tool = build_preview_tool(ExistingConnectionMode::Never);
        let effect = tool.apply_panel_action(ColorPathPanelAction::Reset);

        assert!(effect.changed);
        assert_eq!(tool.phase, ColorPathPhase::Idle);
        assert!(tool.preview_data.is_none());
        assert!(tool.editable.is_none());
    }

    #[test]
    fn cp06_legacy_compute_preview_alias_maps_to_compute() {
        use crate::app::ui_contract::ColorPathPanelAction;

        let mut tool = build_sampling_tool_with_samples();
        #[allow(deprecated)]
        let effect = tool.apply_panel_action(ColorPathPanelAction::ComputePreview);

        assert!(effect.changed);
        assert_eq!(tool.phase, ColorPathPhase::Editing);
    }

    #[test]
    fn cp06_legacy_next_phase_in_sampling_maps_to_compute() {
        use crate::app::ui_contract::ColorPathPanelAction;

        let mut tool = build_sampling_tool_with_samples();
        #[allow(deprecated)]
        let effect = tool.apply_panel_action(ColorPathPanelAction::NextPhase);

        assert!(effect.changed);
        assert_eq!(tool.phase, ColorPathPhase::Editing);
    }

    #[test]
    fn cp06_legacy_next_phase_in_editing_is_noop() {
        use crate::app::ui_contract::ColorPathPanelAction;

        let mut tool = build_preview_tool(ExistingConnectionMode::Never);
        #[allow(deprecated)]
        let effect = tool.apply_panel_action(ColorPathPanelAction::NextPhase);

        assert!(!effect.changed);
        assert!(effect.next_action.is_none());
        assert_eq!(tool.phase, ColorPathPhase::Editing);
    }

    #[test]
    fn cp06_legacy_prev_phase_maps_to_reset() {
        use crate::app::ui_contract::ColorPathPanelAction;

        let mut tool = build_preview_tool(ExistingConnectionMode::Never);
        #[allow(deprecated)]
        let effect = tool.apply_panel_action(ColorPathPanelAction::PrevPhase);

        assert!(effect.changed);
        assert_eq!(tool.phase, ColorPathPhase::Idle);
        assert!(tool.preview_data.is_none());
    }

    #[test]
    fn cp06_legacy_back_to_sampling_maps_to_reset() {
        use crate::app::ui_contract::ColorPathPanelAction;

        let mut tool = build_preview_tool(ExistingConnectionMode::Never);
        #[allow(deprecated)]
        let effect = tool.apply_panel_action(ColorPathPanelAction::BackToSampling);

        assert!(effect.changed);
        assert_eq!(tool.phase, ColorPathPhase::Idle);
        assert!(tool.preview_data.is_none());
    }

    #[test]
    fn cp06_panel_state_flags_in_sampling() {
        let mut tool = build_sampling_tool_with_samples();
        let state = tool.panel_state();

        assert!(state.can_compute, "Sampling mit Samples ⇒ can_compute");
        assert!(!state.can_accept);
        #[allow(deprecated)]
        {
            assert!(!state.can_next, "CP-06: can_next ist konstant false");
            assert!(!state.can_back, "CP-06: can_back ist konstant false");
        }
        let _ = &mut tool; // tool wird nach diesem Test nicht weiter genutzt
    }

    #[test]
    fn cp06_panel_state_flags_in_editing() {
        let tool = build_preview_tool(ExistingConnectionMode::Never);
        let state = tool.panel_state();

        assert!(!state.can_compute, "Editing ⇒ can_compute=false");
        assert!(state.can_accept, "Editing mit Stage F ⇒ can_accept");
        #[allow(deprecated)]
        {
            assert!(!state.can_next);
            assert!(!state.can_back);
        }
    }

    #[test]
    fn cp06_panel_phase_emits_canonical_editing() {
        use crate::app::ui_contract::ColorPathPanelPhase;

        let tool = build_preview_tool(ExistingConnectionMode::Never);
        let state = tool.panel_state();

        assert_eq!(state.phase, ColorPathPanelPhase::Editing);
    }
}
