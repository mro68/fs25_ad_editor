//! `RouteTool`-Implementierung fuer das Ausweichstrecken-Tool.

use std::borrow::Cow;

use super::geometry::compute_bypass_positions;
use super::state::BypassTool;
use crate::app::group_registry::{GroupBase, GroupKind, GroupRecord};
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
        "Generiert eine parallele Ausweichstrecke zur selektierten Kette mit S-foermigen Uebergaengen"
    }

    fn status_text(&self) -> &str {
        if self.has_chain() {
            "Bereit — Enter zum Ausfuehren, Escape zum Abbrechen"
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

        // Cache-Hit: Referenz ausleihen (kein Clone) — Cow::Borrowed vermeidet
        // die per-Frame Vec-Allokation bei unveraendertem Cache.
        let positions: Cow<'_, [Vec2]> = if let Some(cached) = &self.cached_positions {
            Cow::Borrowed(cached.as_slice())
        } else {
            let Some((new_pts, _d_blend)) =
                compute_bypass_positions(&self.chain_positions, self.offset, self.base_spacing)
            else {
                return ToolPreview::default();
            };
            Cow::Owned(new_pts)
        };

        // Vollstaendige Preview-Sequenz: chain_start + bypass_nodes + chain_end
        let chain_start = *self.chain_positions.first().unwrap();
        let chain_end = *self.chain_positions.last().unwrap();

        let mut nodes = Vec::with_capacity(positions.len() + 2);
        nodes.push(chain_start);
        nodes.extend_from_slice(&positions);
        nodes.push(chain_end);

        let connections: Vec<(usize, usize)> = if let Some(cached) = &self.cached_connections {
            cached.clone()
        } else {
            (0..nodes.len().saturating_sub(1))
                .map(|i| (i, i + 1))
                .collect()
        };
        let styles = vec![(self.direction, self.priority); connections.len()];

        ToolPreview {
            nodes,
            connections,
            connection_styles: styles,
            labels: vec![],
        }
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

        // Vollstaendige Positions-Sequenz: start + neue Nodes + end
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

    /// Loescht die geladene Kette, den Cache und den Lifecycle-Zustand.
    fn reset(&mut self) {
        self.chain_positions.clear();
        self.cached_positions = None;
        self.cached_connections = None;
        self.d_blend = 0.0;
        let snap_radius = self.lifecycle.snap_radius;
        self.lifecycle = crate::app::tools::common::ToolLifecycleState::new(snap_radius);
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

    /// Laedt die geordnete Kette aus der Selektion.
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

    // ── Lifecycle-Delegation (manuell, da kein SegmentConfig) ────

    fn set_snap_radius(&mut self, radius: f32) {
        self.lifecycle.snap_radius = radius;
    }

    fn last_created_ids(&self) -> &[u64] {
        &self.lifecycle.last_created_ids
    }

    fn last_end_anchor(&self) -> Option<crate::app::tools::ToolAnchor> {
        self.lifecycle.last_end_anchor
    }

    fn needs_recreate(&self) -> bool {
        self.lifecycle.recreate_needed
    }

    fn clear_recreate_flag(&mut self) {
        self.lifecycle.recreate_needed = false;
    }

    fn set_last_created(&mut self, ids: &[u64], _road_map: &RoadMap) {
        self.lifecycle.save_created_ids(ids);
    }

    /// Erstellt einen `GroupRecord` fuer die Registry aus dem aktuellen Tool-Zustand.
    fn make_group_record(&self, id: u64, node_ids: &[u64]) -> Option<GroupRecord> {
        if !self.has_chain() {
            return None;
        }
        let start_pos = *self.chain_positions.first()?;
        let end_pos = *self.chain_positions.last()?;
        Some(GroupRecord {
            id,
            node_ids: node_ids.to_vec(),
            start_anchor: ToolAnchor::ExistingNode(self.chain_start_id, start_pos),
            end_anchor: ToolAnchor::ExistingNode(self.chain_end_id, end_pos),
            original_positions: Vec::new(), // wird im Handler befuellt
            marker_node_ids: Vec::new(),
            locked: true,
            kind: GroupKind::Bypass {
                chain_positions: self.chain_positions.clone(),
                chain_start_id: self.chain_start_id,
                chain_end_id: self.chain_end_id,
                offset: self.offset,
                base_spacing: self.base_spacing,
                base: GroupBase {
                    direction: self.direction,
                    priority: self.priority,
                    max_segment_length: self.base_spacing,
                },
            },
        })
    }

    /// Laedt einen gespeicherten `GroupRecord` zur nachtraeglichen Bearbeitung.
    fn load_for_edit(&mut self, _record: &GroupRecord, kind: &GroupKind) {
        let GroupKind::Bypass {
            chain_positions,
            chain_start_id,
            chain_end_id,
            offset,
            base_spacing,
            base,
        } = kind
        else {
            return;
        };
        self.load_chain(chain_positions.clone(), *chain_start_id, *chain_end_id);
        self.offset = *offset;
        self.base_spacing = *base_spacing;
        self.direction = base.direction;
        self.priority = base.priority;
    }
}
