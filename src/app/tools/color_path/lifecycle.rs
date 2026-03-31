//! RouteTool-Implementierung fuer das ColorPathTool.
//!
//! Enthaelt die Berechnen-Pipeline, Preview-Logik und Execute-Logik.

use image::GenericImageView;
use std::sync::Arc;

use crate::app::tools::{ToolAction, ToolAnchor, ToolPreview, ToolResult};
use crate::core::{simplify_polyline, ConnectionDirection, FarmlandGrid, NodeFlag, RoadMap};
use crate::shared::spline_geometry::resample_by_distance;
use glam::Vec2;

use super::sampling::world_to_pixel;
use super::skeleton::SkeletonGraphNodeKind;
use super::state::{ColorPathPhase, ColorPathTool, ExistingConnectionMode, PreparedSegment};

// ---------------------------------------------------------------------------
// Interne Methoden
// ---------------------------------------------------------------------------

impl ColorPathTool {
    /// Fuehrt die komplette Erkennungs-Pipeline aus.
    ///
    /// 1. Farbmaske aufbauen
    /// 2. Skelett-Netz extrahieren (Zhang-Suen + Graph-Tracing)
    /// 3. Segmente vereinfachen und neu abtasten
    /// 4. Phase wechseln
    pub(super) fn compute_pipeline(&mut self) {
        let Some(image) = self.background_image.as_ref() else {
            log::warn!("ColorPathTool: Pipeline abgebrochen — kein Hintergrundbild");
            return;
        };
        if self.color_palette.is_empty() {
            log::warn!("ColorPathTool: Pipeline abgebrochen — keine Farbsamples");
            return;
        }

        // Lasso-Startpunkt in Pixelkoordinaten umrechnen (fuer Hint und Flood-Fill)
        let img_w = image.width();
        let img_h = image.height();
        let start_px = self
            .lasso_start_world
            .map(|w| world_to_pixel(w, self.map_size, img_w, img_h))
            .unwrap_or((img_w / 2, img_h / 2));
        let start_hint = Some((start_px.0 as usize, start_px.1 as usize));

        // Maske per Flood-Fill ab Lasso-Startpunkt berechnen
        let (mut mask, width, height) = super::sampling::flood_fill_color_mask(
            image,
            &self.color_palette,
            self.config.color_tolerance,
            start_px,
        );
        self.mask_width = width;
        self.mask_height = height;

        // Netz extrahieren
        let network = super::skeleton::extract_network_from_mask(
            &mut mask,
            width,
            height,
            self.config.noise_filter,
            self.map_size,
            start_hint,
        );
        // Maske nach Pipeline speichern (fuer spaetere Analyse)
        self.mask = mask;

        let segment_count = network.segments.len();
        let junction_count = network.junction_count();
        let open_end_count = network.open_end_count();
        log::info!(
            "Pipeline complete: {} junctions, {} open ends, {} segments",
            junction_count,
            open_end_count,
            segment_count
        );

        if network.is_empty() {
            log::warn!("ColorPathTool: Kein exportierbares Netz gefunden — Phase bleibt Sampling");
            self.skeleton_network = None;
            self.prepared_segments.clear();
            return;
        }

        self.skeleton_network = Some(network);
        self.rebuild_preview_segments();
        if self.prepared_segments.is_empty() {
            log::warn!(
                "ColorPathTool: Netz extrahiert, aber keine gueltigen Preview-Segmente erzeugt"
            );
            self.skeleton_network = None;
            return;
        }
        self.phase = ColorPathPhase::Preview;
    }

    /// Vereinfachung + Resampling fuer alle extrahierten Segmente neu berechnen.
    pub(super) fn rebuild_preview_segments(&mut self) {
        let Some(network) = &self.skeleton_network else {
            self.prepared_segments.clear();
            return;
        };

        let mut prepared_segments = Vec::with_capacity(network.segments.len());
        for segment in &network.segments {
            let simplified = simplify_polyline(&segment.polyline, self.config.simplify_tolerance);
            let resampled_nodes = resample_by_distance(&simplified, self.config.node_spacing);
            if resampled_nodes.len() < 2 {
                continue;
            }

            prepared_segments.push(PreparedSegment {
                start_node: segment.start_node,
                end_node: segment.end_node,
                resampled_nodes,
            });
        }

        log::info!(
            "Netz-Vorschau: {} Knoten, {} Segmente, {} Preview-Segmente",
            network.nodes.len(),
            network.segments.len(),
            prepared_segments.len()
        );

        self.prepared_segments = prepared_segments;
    }

    /// Kennzahlen fuer die Sidebar-Vorschau.
    pub(super) fn preview_stats(&self) -> (usize, usize, usize) {
        let Some(network) = &self.skeleton_network else {
            return (0, 0, 0);
        };
        (
            network.junction_count(),
            network.open_end_count(),
            self.prepared_segments.len(),
        )
    }

    /// Anzahl sichtbarer Preview-Nodes inklusive Segment-Zwischenpunkte.
    pub(super) fn preview_node_count(&self) -> usize {
        let Some(network) = &self.skeleton_network else {
            return 0;
        };

        let intermediate_count: usize = self
            .prepared_segments
            .iter()
            .map(|segment| segment.resampled_nodes.len().saturating_sub(2))
            .sum();
        network.nodes.len() + intermediate_count
    }

    /// Prueft ob ein Netz-Knoten nach aktuellem Modus an Bestand angeschlossen werden darf.
    fn should_connect_node(&self, kind: SkeletonGraphNodeKind) -> bool {
        match self.config.existing_connection_mode {
            ExistingConnectionMode::Never => false,
            ExistingConnectionMode::OpenEnds => kind == SkeletonGraphNodeKind::OpenEnd,
            ExistingConnectionMode::OpenEndsAndJunctions => {
                matches!(
                    kind,
                    SkeletonGraphNodeKind::OpenEnd | SkeletonGraphNodeKind::Junction
                )
            }
        }
    }

    /// Bestimmt die Anschlussrichtung eines externen Bestands-Snaps.
    fn external_connection_spec(&self, node_index: usize) -> (bool, ConnectionDirection) {
        let mut has_outgoing = false;
        let mut has_incoming = false;

        for segment in &self.prepared_segments {
            if segment.start_node == node_index {
                has_outgoing = true;
            }
            if segment.end_node == node_index {
                has_incoming = true;
            }
        }

        let existing_to_new = match (has_outgoing, has_incoming) {
            (true, false) => true,
            (false, true) => false,
            _ => true,
        };
        let direction = if has_outgoing && has_incoming {
            ConnectionDirection::Dual
        } else {
            self.direction
        };

        (existing_to_new, direction)
    }

    /// Baut die Vorschau fuer die Sampling-Phase:
    /// Alle bisher gezeichneten Lasso-Regionen als Linien-Polygone anzeigen.
    fn build_sampling_preview(&self) -> ToolPreview {
        use crate::core::{ConnectionDirection, ConnectionPriority};
        let mut nodes: Vec<Vec2> = Vec::new();
        let mut connections: Vec<(usize, usize)> = Vec::new();

        for polygon in &self.lasso_regions {
            if polygon.len() < 2 {
                continue;
            }
            let start = nodes.len();
            nodes.extend_from_slice(polygon);
            let n = polygon.len();
            // Polygon schliessen
            for i in 0..n {
                connections.push((start + i, start + (i + 1) % n));
            }
        }

        // Flood-Fill-Kontur als geschlossenes Polygon hinzufuegen
        if !self.flood_fill_contour.is_empty() {
            let base = nodes.len();
            nodes.extend_from_slice(&self.flood_fill_contour);
            let contour_len = self.flood_fill_contour.len();
            for i in 0..contour_len {
                let next = (i + 1) % contour_len;
                connections.push((base + i, base + next));
            }
        }

        let cn = connections.len();
        ToolPreview {
            nodes,
            connections,
            connection_styles: vec![(ConnectionDirection::Dual, ConnectionPriority::Regular); cn],
            labels: vec![],
        }
    }

    /// Baut die Vorschau fuer die Preview-Phase als echtes Netz.
    fn build_network_preview(&self) -> ToolPreview {
        let Some(network) = &self.skeleton_network else {
            return ToolPreview::default();
        };
        if self.prepared_segments.is_empty() {
            return ToolPreview::default();
        }

        let mut nodes: Vec<Vec2> = network
            .nodes
            .iter()
            .map(|node| node.world_position)
            .collect();
        let mut connections = Vec::new();
        let mut connection_styles = Vec::new();

        for segment in &self.prepared_segments {
            if segment.resampled_nodes.len() < 2 {
                continue;
            }

            let mut chain = Vec::with_capacity(segment.resampled_nodes.len());
            chain.push(segment.start_node);

            for &pos in segment
                .resampled_nodes
                .iter()
                .skip(1)
                .take(segment.resampled_nodes.len().saturating_sub(2))
            {
                nodes.push(pos);
                chain.push(nodes.len() - 1);
            }

            chain.push(segment.end_node);
            if chain.len() < 2 || (chain.len() == 2 && chain[0] == chain[1]) {
                continue;
            }

            for edge in chain.windows(2) {
                connections.push((edge[0], edge[1]));
                connection_styles.push((self.direction, self.priority));
            }
        }

        ToolPreview {
            nodes,
            connections,
            connection_styles,
            labels: vec![],
        }
    }
}

impl crate::app::tools::RouteTool for ColorPathTool {
    fn name(&self) -> &str {
        "Farb-Pfad"
    }

    fn icon(&self) -> &str {
        "🎨"
    }

    fn description(&self) -> &str {
        "Wege anhand der Farbe im Hintergrundbild erkennen"
    }

    fn status_text(&self) -> &str {
        match self.phase {
            ColorPathPhase::Idle => "Alt+Lasso fuer Farbsample",
            ColorPathPhase::Sampling => "Berechnen fuer Wegenetz",
            ColorPathPhase::Preview => "Enter zum Einfuegen, Reset zum Zuruecksetzen",
        }
    }

    fn on_click(&mut self, _pos: Vec2, _road_map: &RoadMap, _ctrl: bool) -> ToolAction {
        match self.phase {
            ColorPathPhase::Idle => {
                self.phase = ColorPathPhase::Sampling;
                ToolAction::Continue
            }
            ColorPathPhase::Sampling | ColorPathPhase::Preview => ToolAction::Continue,
        }
    }

    fn on_lasso_completed(&mut self, polygon: Vec<Vec2>) -> ToolAction {
        if self.phase != ColorPathPhase::Sampling {
            return ToolAction::Continue;
        }
        let Some(image) = &self.background_image else {
            log::warn!("ColorPathTool: Kein Hintergrundbild vorhanden — Lasso wird ignoriert");
            return ToolAction::Continue;
        };
        // Farben innerhalb des Lasso-Polygons samplen
        let new_colors = super::sampling::sample_colors_in_polygon(&polygon, image, self.map_size);
        let new_count = new_colors.len();
        // Ersten Lasso-Startpunkt merken (fuer spaetere Pfad-Auswahl)
        if self.lasso_regions.is_empty() {
            self.lasso_start_world = polygon.first().copied();
        }
        self.sampled_colors.extend(new_colors);
        self.lasso_regions.push(polygon);
        // Mittelwert (Anzeigewert) und quantisierte Palette aktualisieren
        self.avg_color = Some(super::sampling::compute_average_color(&self.sampled_colors));
        self.color_palette = super::sampling::build_color_palette(&self.sampled_colors, 8);
        log::info!(
            "Color sampling: {} new pixels, {} total, palette size: {}, avg color: {:?}",
            new_count,
            self.sampled_colors.len(),
            self.color_palette.len(),
            self.avg_color
        );
        // Quick-Flood-Fill fuer Vorschau der Bereichs-Umrisse
        if let Some(lasso_start) = self.lasso_start_world {
            let img_w = image.width();
            let img_h = image.height();
            let start_px = world_to_pixel(lasso_start, self.map_size, img_w, img_h);
            let (mask, w, h) = super::sampling::flood_fill_color_mask(
                image,
                &self.color_palette,
                self.config.color_tolerance,
                start_px,
            );
            self.flood_fill_contour =
                super::sampling::extract_contour_from_mask(&mask, w, h, self.map_size);
            log::info!(
                "Flood-Fill Vorschau: {} Kontur-Punkte",
                self.flood_fill_contour.len()
            );
        }
        ToolAction::Continue
    }

    fn needs_lasso_input(&self) -> bool {
        self.phase == ColorPathPhase::Sampling
    }

    fn preview(&self, _cursor_pos: Vec2, _road_map: &RoadMap) -> ToolPreview {
        match self.phase {
            ColorPathPhase::Idle => ToolPreview::default(),
            ColorPathPhase::Sampling => self.build_sampling_preview(),
            ColorPathPhase::Preview => self.build_network_preview(),
        }
    }

    fn execute(&self, road_map: &RoadMap) -> Option<ToolResult> {
        if self.phase != ColorPathPhase::Preview || self.prepared_segments.is_empty() {
            return None;
        }
        let network = self.skeleton_network.as_ref()?;

        let new_nodes: Vec<(Vec2, NodeFlag)> = self
            .skeleton_network
            .as_ref()?
            .nodes
            .iter()
            .map(|node| (node.world_position, NodeFlag::Regular))
            .collect();
        let mut new_nodes = new_nodes;

        let mut internal_connections = Vec::new();
        for segment in &self.prepared_segments {
            if segment.resampled_nodes.len() < 2 {
                continue;
            }

            let mut chain = Vec::with_capacity(segment.resampled_nodes.len());
            chain.push(segment.start_node);
            for &pos in segment
                .resampled_nodes
                .iter()
                .skip(1)
                .take(segment.resampled_nodes.len().saturating_sub(2))
            {
                let idx = new_nodes.len();
                new_nodes.push((pos, NodeFlag::Regular));
                chain.push(idx);
            }
            chain.push(segment.end_node);

            if chain.len() < 2 || (chain.len() == 2 && chain[0] == chain[1]) {
                continue;
            }

            for edge in chain.windows(2) {
                internal_connections.push((edge[0], edge[1], self.direction, self.priority));
            }
        }

        let mut external_connections = Vec::new();
        for (node_index, node) in network.nodes.iter().enumerate() {
            if !self.should_connect_node(node.kind) {
                continue;
            }

            let ToolAnchor::ExistingNode(existing_id, _) =
                self.lifecycle.snap_at(node.world_position, road_map)
            else {
                continue;
            };

            let (existing_to_new, direction) = self.external_connection_spec(node_index);
            external_connections.push((
                node_index,
                existing_id,
                existing_to_new,
                direction,
                self.priority,
            ));
        }

        if internal_connections.is_empty() {
            return None;
        }

        Some(ToolResult {
            new_nodes,
            internal_connections,
            external_connections,
            markers: Vec::new(),
            nodes_to_remove: Vec::new(),
        })
    }

    fn render_config(&mut self, ui: &mut egui::Ui, distance_wheel_step_m: f32) -> bool {
        super::config_ui::render_config_view(self, ui, distance_wheel_step_m)
    }

    fn reset(&mut self) {
        self.phase = ColorPathPhase::Idle;
        self.lasso_regions.clear();
        self.sampled_colors.clear();
        self.avg_color = None;
        self.color_palette.clear();
        self.mask.clear();
        self.mask_width = 0;
        self.mask_height = 0;
        self.skeleton_network = None;
        self.prepared_segments.clear();
        self.lasso_start_world = None;
        self.flood_fill_contour.clear();
    }

    fn is_ready(&self) -> bool {
        self.phase == ColorPathPhase::Preview && !self.prepared_segments.is_empty()
    }

    fn has_pending_input(&self) -> bool {
        self.phase != ColorPathPhase::Idle
    }

    fn set_background_map_image(&mut self, image: Option<Arc<image::DynamicImage>>) {
        if let Some(ref img) = image {
            // map_size aus Bilddimensionen ableiten (Fallback wenn kein FarmlandGrid)
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
    }

    fn set_farmland_grid(&mut self, grid: Option<Arc<FarmlandGrid>>) {
        if let Some(g) = &grid {
            self.map_size = g.map_size;
        }
        // Grid selbst wird nicht gecacht — map_size genuegt fuer Pixel<->Welt-Umrechnung
    }

    crate::impl_lifecycle_delegation_no_seg!();
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::app::tools::color_path::skeleton::{
        SkeletonGraphNode, SkeletonGraphNodeKind, SkeletonNetwork,
    };
    use crate::app::tools::RouteTool;
    use crate::core::{ConnectionPriority, MapNode};

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
        tool.skeleton_network = Some(sample_network());
        tool.prepared_segments = sample_prepared_segments();
        tool.lifecycle.snap_radius = 1.0;
        tool
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
        assert!(
            result_with_junctions.external_connections.iter().any(
                |&(idx, existing_id, _, direction, _)| {
                    idx == 0 && existing_id == 200 && direction == ConnectionDirection::Dual
                }
            ),
            "Gemischt gerichtete Junction-Anschluesse muessen als Dual exportiert werden"
        );
    }
}
