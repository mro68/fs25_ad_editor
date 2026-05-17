//! Laufzeit-State fuer die CP-01-Shell des Verrundungs-Tools.

use crate::app::tools::{
    RouteTool, RouteToolCore, RouteToolHostSync, RouteToolPanelBridge, ToolAction, ToolHostContext,
    ToolPreview, ToolResult,
};
use crate::app::ui_contract::{
    RoundingModeChoice, RoundingPanelAction, RoundingPanelState, RouteToolConfigState,
    RouteToolPanelAction, RouteToolPanelEffect, ROUNDING_ARC_RADIUS_LIMITS,
    ROUNDING_SAMPLE_SPACING_LIMITS,
};
use crate::core::RoadMap;
use glam::Vec2;

const DEFAULT_ARC_RADIUS_M: f32 = 6.0;
const DEFAULT_SAMPLE_SPACING_M: f32 = 3.0;
const DEFAULT_SNAP_RADIUS_M: f32 = 3.0;

/// Interne Moduswahl des Verrundungs-Tools.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum RoundingMode {
    /// Verrundet einen einzelnen Eckpunkt ueber einen spaeteren Arc-/Fillet-Solver.
    ArcOnePoint,
    /// Verrundet eine geordnete 3-Punkt-Kette ueber eine spaetere quadratische Kurve.
    QuadraticThreePoint,
}

impl RoundingMode {
    fn panel_choice(self) -> RoundingModeChoice {
        match self {
            Self::ArcOnePoint => RoundingModeChoice::ArcOnePoint,
            Self::QuadraticThreePoint => RoundingModeChoice::QuadraticThreePoint,
        }
    }

    fn from_panel_choice(choice: RoundingModeChoice) -> Self {
        match choice {
            RoundingModeChoice::ArcOnePoint => Self::ArcOnePoint,
            RoundingModeChoice::QuadraticThreePoint => Self::QuadraticThreePoint,
        }
    }
}

/// Minimaler Shell-State fuer das oeffentliche Verrundungs-Tool in CP-01.
pub struct RoundingTool {
    mode: RoundingMode,
    arc_radius_m: f32,
    arc_sample_spacing_m: f32,
    quadratic_sample_spacing_m: f32,
    snap_radius: f32,
}

impl RoundingTool {
    /// Erstellt das Verrundungs-Tool im standardmaessigen Arc-Modus.
    pub fn new() -> Self {
        Self {
            mode: RoundingMode::ArcOnePoint,
            arc_radius_m: DEFAULT_ARC_RADIUS_M,
            arc_sample_spacing_m: DEFAULT_SAMPLE_SPACING_M,
            quadratic_sample_spacing_m: DEFAULT_SAMPLE_SPACING_M,
            snap_radius: DEFAULT_SNAP_RADIUS_M,
        }
    }
}

impl Default for RoundingTool {
    fn default() -> Self {
        Self::new()
    }
}

impl RouteToolCore for RoundingTool {
    fn on_click(&mut self, _pos: Vec2, _road_map: &RoadMap, _ctrl: bool) -> ToolAction {
        ToolAction::Continue
    }

    fn preview(&self, _cursor_pos: Vec2, _road_map: &RoadMap) -> ToolPreview {
        ToolPreview::default()
    }

    fn execute(&self, _road_map: &RoadMap) -> Option<ToolResult> {
        None
    }

    fn reset(&mut self) {}

    fn is_ready(&self) -> bool {
        false
    }

    fn has_pending_input(&self) -> bool {
        false
    }
}

impl RouteToolPanelBridge for RoundingTool {
    fn status_text(&self) -> &str {
        let _ = self.snap_radius;
        "Verrundungs-Tool vorbereitet; Arc- und Quadratic-Logik folgen in den naechsten Commits."
    }

    fn panel_state(&self) -> RouteToolConfigState {
        RouteToolConfigState::Rounding(RoundingPanelState {
            mode: self.mode.panel_choice(),
            mode_locked: false,
            arc_radius_m: self.arc_radius_m,
            arc_sample_spacing_m: self.arc_sample_spacing_m,
            quadratic_sample_spacing_m: self.quadratic_sample_spacing_m,
            selected_node_count: 0,
            chain_node_count: 0,
            preview_node_count: None,
            is_adjusting: false,
        })
    }

    fn apply_panel_action(&mut self, action: RouteToolPanelAction) -> RouteToolPanelEffect {
        let RouteToolPanelAction::Rounding(action) = action else {
            return RouteToolPanelEffect::default();
        };

        let changed = match action {
            RoundingPanelAction::SetMode(choice) => {
                let next = RoundingMode::from_panel_choice(choice);
                if self.mode == next {
                    false
                } else {
                    self.mode = next;
                    true
                }
            }
            RoundingPanelAction::SetArcRadius(value) => {
                let next = ROUNDING_ARC_RADIUS_LIMITS.clamp(value);
                if (self.arc_radius_m - next).abs() < f32::EPSILON {
                    false
                } else {
                    self.arc_radius_m = next;
                    true
                }
            }
            RoundingPanelAction::SetArcSampleSpacing(value) => {
                let next = ROUNDING_SAMPLE_SPACING_LIMITS.clamp(value);
                if (self.arc_sample_spacing_m - next).abs() < f32::EPSILON {
                    false
                } else {
                    self.arc_sample_spacing_m = next;
                    true
                }
            }
            RoundingPanelAction::SetQuadraticSampleSpacing(value) => {
                let next = ROUNDING_SAMPLE_SPACING_LIMITS.clamp(value);
                if (self.quadratic_sample_spacing_m - next).abs() < f32::EPSILON {
                    false
                } else {
                    self.quadratic_sample_spacing_m = next;
                    true
                }
            }
        };

        RouteToolPanelEffect {
            changed,
            needs_recreate: false,
            next_action: None,
        }
    }
}

impl RouteToolHostSync for RoundingTool {
    fn sync_host(&mut self, context: &ToolHostContext) {
        self.snap_radius = context.snap_radius;
    }
}

impl RouteTool for RoundingTool {}
