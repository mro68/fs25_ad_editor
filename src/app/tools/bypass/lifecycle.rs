//! `RouteTool`-Implementierung für das Ausweichstrecken-Tool.

use super::geometry::compute_bypass_positions;
use super::state::BypassTool;
use crate::app::tools::common::assemble_tool_result;
use crate::app::tools::{RouteTool, ToolAction, ToolAnchor, ToolPreview, ToolResult};
use crate::core::{ConnectionDirection, ConnectionPriority, RoadMap};
use glam::Vec2;

impl RouteTool for BypassTool {
    fn name(&self) -> &str {
        "Ausweichstrecke"
    }

    fn icon(&self) -> &str {
        "⤴"
    }

    fn description(&self) -> &str {
        "Generiert eine parallele Ausweichstrecke zur selektierten Kette mit S-förmigen Übergängen"
    }

    fn status_text(&self) -> &str {
        if self.has_chain() {
            "Bereit — Enter zum Ausführen, Escape zum Abbrechen"
        } else {
            "Kette selektieren, dann Route-Tool neu aktivieren"
        }
    }

    /// Klicks im Viewport werden ignoriert — die Kette kommt aus der Selektion.
    fn on_click(&mut self, _pos: Vec2, _road_map: &RoadMap, _ctrl: bool) -> ToolAction {
        ToolAction::Continue
    }

    fn preview(&self, _cursor_pos: Vec2, _road_map: &RoadMap) -> ToolPreview {
        if !self.has_chain() {
            return ToolPreview::default();
        }

        // Cache nutzen falls vorhanden
        let positions = if let Some(cached) = &self.cached_positions {
            cached.clone()
        } else {
            let Some((new_pts, _d_blend)) =
                compute_bypass_positions(&self.chain_positions, self.offset, self.base_spacing)
            else {
                return ToolPreview::default();
            };
            new_pts
        };

        // Vollständige Preview-Sequenz: chain_start + bypass_nodes + chain_end
        let chain_start = *self.chain_positions.first().unwrap();
        let chain_end = *self.chain_positions.last().unwrap();

        let mut nodes = Vec::with_capacity(positions.len() + 2);
        nodes.push(chain_start);
        nodes.extend_from_slice(&positions);
        nodes.push(chain_end);

        let connections = if let Some(cached) = &self.cached_connections {
            cached.clone()
        } else {
            (0..nodes.len().saturating_sub(1))
                .map(|i| (i, i + 1))
                .collect()
        };

        ToolPreview { nodes, connections }
    }

    fn render_config(&mut self, ui: &mut egui::Ui, distance_wheel_step_m: f32) -> bool {
        self.render_config_view(ui, distance_wheel_step_m)
    }

    fn execute(&self, road_map: &RoadMap) -> Option<ToolResult> {
        if !self.has_chain() {
            return None;
        }

        let (new_positions, _d_blend) =
            compute_bypass_positions(&self.chain_positions, self.offset, self.base_spacing)?;

        let chain_start_pos = *self.chain_positions.first().unwrap();
        let chain_end_pos = *self.chain_positions.last().unwrap();

        // Vollständige Positions-Sequenz: start + neue Nodes + end
        let mut all_positions = Vec::with_capacity(new_positions.len() + 2);
        all_positions.push(chain_start_pos);
        all_positions.extend_from_slice(&new_positions);
        all_positions.push(chain_end_pos);

        Some(assemble_tool_result(
            &all_positions,
            &ToolAnchor::ExistingNode(self.chain_start_id, chain_start_pos),
            &ToolAnchor::ExistingNode(self.chain_end_id, chain_end_pos),
            self.direction,
            self.priority,
            road_map,
        ))
    }

    /// Löscht die geladene Kette und den Cache.
    fn reset(&mut self) {
        self.chain_positions.clear();
        self.cached_positions = None;
        self.cached_connections = None;
        self.d_blend = 0.0;
    }

    fn is_ready(&self) -> bool {
        self.has_chain()
    }

    fn has_pending_input(&self) -> bool {
        self.has_chain()
    }

    // ── Chain-Input ──────────────────────────────────────────────────────────

    fn needs_chain_input(&self) -> bool {
        true
    }

    /// Lädt die geordnete Kette aus der Selektion.
    fn load_chain(&mut self, positions: Vec<Vec2>, start_id: u64, end_id: u64) {
        self.chain_positions = positions;
        self.chain_start_id = start_id;
        self.chain_end_id = end_id;
        self.cached_positions = None;
        self.cached_connections = None;
        self.d_blend = 0.0;
    }

    // ── Editor-Defaults ──────────────────────────────────────────────────────

    fn set_direction(&mut self, dir: ConnectionDirection) {
        self.direction = dir;
    }

    fn set_priority(&mut self, prio: ConnectionPriority) {
        self.priority = prio;
    }
}
