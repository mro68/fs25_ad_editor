//! RouteTool-Implementierung fuer das ParkingTool.

use crate::app::group_registry::{GroupBase, GroupKind, GroupRecord};
use crate::app::tools::{RouteToolId, ToolAction, ToolAnchor, ToolPreview, ToolResult};
use crate::core::RoadMap;
use glam::Vec2;

use super::geometry;
use super::state::{ParkingPhase, ParkingTool};

impl crate::app::tools::RouteTool for ParkingTool {
    fn name(&self) -> &str {
        "Parkplatz"
    }

    fn icon(&self) -> &str {
        "\u{1f17f}" // 🅿
    }

    fn description(&self) -> &str {
        "Erzeugt ein Parkplatz-Layout mit Wendekreis"
    }

    fn status_text(&self) -> &str {
        match self.phase {
            ParkingPhase::Idle => "Klicken zum Platzieren — Alt+Mausrad zum Drehen",
            ParkingPhase::Configuring => "Layout konfigurieren — Bestätigen oder Abbrechen",
            ParkingPhase::Adjusting => "Klicken zum Fixieren — Alt+Mausrad zum Drehen",
        }
    }

    fn on_click(&mut self, pos: Vec2, _road_map: &RoadMap, _ctrl: bool) -> ToolAction {
        match self.phase {
            ParkingPhase::Idle => {
                self.origin = Some(pos);
                self.phase = ParkingPhase::Configuring;
                ToolAction::Continue
            }
            ParkingPhase::Configuring => {
                // Viewport-Klick waehrend Config → Repositionierung starten
                self.phase = ParkingPhase::Adjusting;
                ToolAction::Continue
            }
            ParkingPhase::Adjusting => {
                // Erneuter Klick → Position fixieren, zurueck zu Configuring
                self.origin = Some(pos);
                self.phase = ParkingPhase::Configuring;
                ToolAction::Continue
            }
        }
    }

    fn on_scroll_rotate(&mut self, delta: f32) {
        // Nur in Idle oder Adjusting rotierbar (in Configuring ist alles fixiert)
        if matches!(self.phase, ParkingPhase::Idle | ParkingPhase::Adjusting) {
            let step = self.rotation_step_deg.to_radians();
            if delta > 0.0 {
                self.angle += step;
            } else {
                self.angle -= step;
            }
        }
    }

    fn preview(&self, cursor_pos: Vec2, _road_map: &RoadMap) -> ToolPreview {
        let (origin, angle) = match self.phase {
            ParkingPhase::Idle => (cursor_pos, self.angle),
            ParkingPhase::Configuring => {
                let origin = self.origin.unwrap_or(cursor_pos);
                (origin, self.angle)
            }
            ParkingPhase::Adjusting => (cursor_pos, self.angle),
        };

        let layout = geometry::generate_parking_layout(
            origin,
            angle,
            &self.config,
            self.direction,
            self.priority,
        );
        geometry::build_preview(&layout)
    }

    fn render_config(&mut self, ui: &mut egui::Ui, distance_wheel_step_m: f32) -> bool {
        self.render_config_view(ui, distance_wheel_step_m)
    }

    fn execute(&self, _road_map: &RoadMap) -> Option<ToolResult> {
        // Nur in Configuring-Phase ausfuehrbar (verhindert Execute mit veralteter Position)
        if self.phase != ParkingPhase::Configuring {
            return None;
        }
        let origin = self.origin?;
        let layout = geometry::generate_parking_layout(
            origin,
            self.angle,
            &self.config,
            self.direction,
            self.priority,
        );
        Some(geometry::build_parking_result(layout))
    }

    fn reset(&mut self) {
        self.phase = ParkingPhase::Idle;
        self.origin = None;
        self.angle = 0.0;
    }

    fn is_ready(&self) -> bool {
        self.phase == ParkingPhase::Configuring && self.origin.is_some()
    }

    fn has_pending_input(&self) -> bool {
        matches!(
            self.phase,
            ParkingPhase::Configuring | ParkingPhase::Adjusting
        )
    }

    // ── Lifecycle-Delegation ─────────────────────────────────────────────────

    crate::impl_lifecycle_delegation_no_seg!();

    /// Erstellt einen `GroupRecord` fuer die Registry aus dem aktuellen Tool-Zustand.
    fn make_group_record(&self, id: u64, node_ids: &[u64]) -> Option<GroupRecord> {
        let origin = self.origin?;
        let angle = self.angle;
        // n7 (Index 6) ist die Einfahrt, n8 (letzter Index) die Ausfahrt
        let entry_node_id = node_ids.get(6).copied();
        let exit_node_id = node_ids.last().copied();
        Some(GroupRecord {
            id,
            tool_id: Some(RouteToolId::Parking),
            node_ids: node_ids.to_vec(),
            start_anchor: ToolAnchor::NewPosition(origin),
            end_anchor: ToolAnchor::NewPosition(origin),
            original_positions: Vec::new(), // wird im Handler befuellt
            marker_node_ids: Vec::new(),    // wird im Handler befuellt
            locked: true,
            entry_node_id,
            exit_node_id,
            kind: GroupKind::Parking {
                origin,
                angle,
                config: self.config.clone(),
                base: GroupBase {
                    direction: self.direction,
                    priority: self.priority,
                    max_segment_length: 0.0,
                },
            },
        })
    }

    /// Laedt einen gespeicherten `GroupRecord` zur nachtraeglichen Bearbeitung.
    fn load_for_edit(&mut self, _record: &GroupRecord, kind: &GroupKind) {
        let GroupKind::Parking {
            origin,
            angle,
            config,
            base,
        } = kind
        else {
            return;
        };
        self.origin = Some(*origin);
        self.angle = *angle;
        self.phase = ParkingPhase::Configuring;
        self.config = config.clone();
        self.direction = base.direction;
        self.priority = base.priority;
    }
}
