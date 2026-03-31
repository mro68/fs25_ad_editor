//! RouteTool-Implementierung fuer das ColorPathTool.
//!
//! Enthaelt die Berechnen-Pipeline, Preview-Logik und Execute-Logik.

use std::sync::Arc;

use crate::app::tools::{ToolAction, ToolPreview, ToolResult};
use crate::core::{simplify_polyline, FarmlandGrid, NodeFlag, RoadMap};
use crate::shared::spline_geometry::resample_by_distance;
use glam::Vec2;

use super::sampling::world_to_pixel;
use super::state::{ColorPathPhase, ColorPathTool};

// ---------------------------------------------------------------------------
// Interne Methoden
// ---------------------------------------------------------------------------

impl ColorPathTool {
    /// Fuehrt die komplette Erkennungs-Pipeline aus.
    ///
    /// 1. Farbmaske aufbauen
    /// 2. Skelett-Pfade extrahieren (Zhang-Suen + BFS)
    /// 3. Pfad zum Lasso-Startpunkt auswaehlen, vereinfachen und neu abtasten
    /// 4. Phase wechseln
    pub(super) fn compute_pipeline(&mut self) {
        let (Some(image), Some(avg_color)) = (&self.background_image.clone(), self.avg_color)
        else {
            log::warn!("ColorPathTool: Pipeline abgebrochen — kein Bild oder keine Farbsamples");
            return;
        };

        // Lasso-Startpunkt in Pixelkoordinaten umrechnen (fuer Hint)
        let img_w = image.width();
        let img_h = image.height();
        let start_hint = self.lasso_start_world.map(|w| {
            let (px, py) = world_to_pixel(w, self.map_size, img_w, img_h);
            (px as usize, py as usize)
        });

        // Maske berechnen
        let (mut mask, width, height) = super::sampling::build_color_mask(
            image,
            avg_color,
            self.config.color_tolerance,
            self.config.detection_bounds,
            self.map_size,
        );
        self.mask_width = width;
        self.mask_height = height;

        // Pfade extrahieren
        let paths = super::skeleton::extract_paths_from_mask(
            &mut mask,
            width,
            height,
            self.config.noise_filter,
            self.map_size,
            start_hint,
        );
        // Maske nach Pipeline speichern (fuer spaetere Analyse)
        self.mask = mask;

        let path_count = paths.len();
        let longest_len = paths.first().map(|p| p.len()).unwrap_or(0);
        log::info!(
            "Pipeline complete: {} paths found, longest has {} points",
            path_count,
            longest_len
        );

        self.skeleton_paths = paths;

        if self.skeleton_paths.is_empty() {
            log::warn!("ColorPathTool: Keine Pfade gefunden — Phase bleibt Sampling");
            self.selected_path_index = None;
            self.centerline.clear();
            self.resampled_nodes.clear();
            return;
        }

        // Pfad auswaehlen: Den naechsten zum Lasso-Startpunkt, nicht den laengsten.
        // Fallback: Index 0 (laengster Pfad) wenn kein Hint vorhanden.
        let best_index = if let Some(start_w) = self.lasso_start_world {
            self.skeleton_paths
                .iter()
                .enumerate()
                .min_by(|(_, a), (_, b)| {
                    let dist_a = a
                        .iter()
                        .map(|p| p.distance_squared(start_w))
                        .fold(f32::MAX, f32::min);
                    let dist_b = b
                        .iter()
                        .map(|p| p.distance_squared(start_w))
                        .fold(f32::MAX, f32::min);
                    dist_a
                        .partial_cmp(&dist_b)
                        .unwrap_or(std::cmp::Ordering::Equal)
                })
                .map(|(i, _)| i)
                .unwrap_or(0)
        } else {
            0
        };
        self.selected_path_index = Some(best_index);
        self.apply_selected_path();
        self.phase = ColorPathPhase::Preview;
    }

    /// Wendet den aktuell ausgewaehlten Pfad an:
    /// Vereinfachung + Resampling aus `skeleton_paths[selected_path_index]`.
    pub(super) fn apply_selected_path(&mut self) {
        let Some(idx) = self.selected_path_index else {
            return;
        };
        let Some(raw) = self.skeleton_paths.get(idx) else {
            return;
        };
        let simplified = simplify_polyline(raw, self.config.simplify_tolerance);
        let resampled = resample_by_distance(&simplified, self.config.node_spacing);
        log::info!(
            "Pfad {}: {} Rohpunkte → {} vereinfacht → {} Nodes",
            idx,
            raw.len(),
            simplified.len(),
            resampled.len()
        );
        self.centerline = simplified;
        self.resampled_nodes = resampled;
    }

    /// Waehlt einen Pfad per Index aus und berechnet Mittellinie + Nodes neu.
    pub(super) fn select_path(&mut self, index: usize) {
        self.selected_path_index = Some(index);
        self.apply_selected_path();
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

        let cn = connections.len();
        ToolPreview {
            nodes,
            connections,
            connection_styles: vec![(ConnectionDirection::Dual, ConnectionPriority::Regular); cn],
            labels: vec![],
        }
    }

    /// Baut die Vorschau fuer die Preview-Phase:
    /// Ausgewaehlter Pfad als Node-Kette.
    fn build_path_preview(&self) -> ToolPreview {
        if self.resampled_nodes.is_empty() {
            return ToolPreview::default();
        }
        let n = self.resampled_nodes.len();
        let connections: Vec<(usize, usize)> =
            (0..n.saturating_sub(1)).map(|i| (i, i + 1)).collect();
        let cn = connections.len();
        ToolPreview {
            nodes: self.resampled_nodes.clone(),
            connections: connections.clone(),
            connection_styles: vec![(self.direction, self.priority); cn],
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
            ColorPathPhase::Sampling => "Berechnen fuer Mittellinie",
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
        // Mittelwert aus allen gesammelten Farben aktualisieren
        self.avg_color = Some(super::sampling::compute_average_color(&self.sampled_colors));
        log::info!(
            "Color sampling: {} new pixels, {} total, avg color: {:?}",
            new_count,
            self.sampled_colors.len(),
            self.avg_color
        );
        ToolAction::Continue
    }

    fn needs_lasso_input(&self) -> bool {
        self.phase == ColorPathPhase::Sampling
    }

    fn preview(&self, _cursor_pos: Vec2, _road_map: &RoadMap) -> ToolPreview {
        match self.phase {
            ColorPathPhase::Idle => ToolPreview::default(),
            ColorPathPhase::Sampling => self.build_sampling_preview(),
            ColorPathPhase::Preview => self.build_path_preview(),
        }
    }

    fn execute(&self, road_map: &RoadMap) -> Option<ToolResult> {
        if self.phase != ColorPathPhase::Preview || self.resampled_nodes.is_empty() {
            return None;
        }

        let n = self.resampled_nodes.len();
        let new_nodes: Vec<(Vec2, NodeFlag)> = self
            .resampled_nodes
            .iter()
            .map(|&pos| (pos, NodeFlag::Regular))
            .collect();

        // Kette: 0 → 1 → 2 → …
        let internal_connections = (0..n.saturating_sub(1))
            .map(|i| (i, i + 1, self.direction, self.priority))
            .collect();

        // Externe Verbindungen an Start und Ende (optional)
        let mut external_connections = Vec::new();
        if self.config.connect_to_existing {
            if let Some(&start_pos) = self.resampled_nodes.first() {
                if let Some(hit) = road_map.nearest_node(start_pos) {
                    external_connections.push((
                        0,
                        hit.node_id,
                        true,
                        self.direction,
                        self.priority,
                    ));
                }
            }
            if n >= 2 {
                if let Some(&end_pos) = self.resampled_nodes.last() {
                    if let Some(hit) = road_map.nearest_node(end_pos) {
                        external_connections.push((
                            n - 1,
                            hit.node_id,
                            false,
                            self.direction,
                            self.priority,
                        ));
                    }
                }
            }
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
        self.mask.clear();
        self.mask_width = 0;
        self.mask_height = 0;
        self.skeleton_paths.clear();
        self.selected_path_index = None;
        self.centerline.clear();
        self.resampled_nodes.clear();
        self.lasso_start_world = None;
    }

    fn is_ready(&self) -> bool {
        self.phase == ColorPathPhase::Preview && !self.resampled_nodes.is_empty()
    }

    fn has_pending_input(&self) -> bool {
        self.phase != ColorPathPhase::Idle
    }

    fn set_background_map_image(&mut self, image: Option<Arc<image::DynamicImage>>) {
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
