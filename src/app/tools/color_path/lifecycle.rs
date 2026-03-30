//! RouteTool-Implementierung (Geruest) fuer das ColorPathTool.

use std::sync::Arc;

use crate::app::tools::{ToolAction, ToolPreview, ToolResult};
use crate::core::{FarmlandGrid, RoadMap};
use glam::Vec2;

use super::state::{ColorPathPhase, ColorPathTool};

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
        let new_colors =
            super::sampling::sample_colors_in_polygon(&polygon, image, self.map_size);
        let new_count = new_colors.len();
        self.sampled_colors.extend(new_colors);
        self.lasso_regions.push(polygon);
        // Mittelwert aus allen gesammelten Farben aktualisieren
        self.avg_color =
            Some(super::sampling::compute_average_color(&self.sampled_colors));
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
        ToolPreview::default()
    }

    fn execute(&self, _road_map: &RoadMap) -> Option<ToolResult> {
        None
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
