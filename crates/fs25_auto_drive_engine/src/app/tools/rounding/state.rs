//! Laufzeit-State fuer den CP-02-Arc-Pfad des Verrundungs-Tools.

use super::geometry::{recompute_arc_plan, ArcPlan, ArcValidation};
use crate::app::tools::{RouteToolConnectedNeighborSeed, RouteToolSelectionSeed};
use glam::Vec2;

pub(crate) const DEFAULT_ARC_RADIUS_M: f32 = 6.0;
pub(crate) const DEFAULT_SAMPLE_SPACING_M: f32 = 3.0;

/// Interne Moduswahl des Verrundungs-Tools.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum RoundingMode {
    /// Verrundet einen einzelnen Eckpunkt ueber einen Arc-/Fillet-Solver.
    ArcOnePoint,
    /// Platzhalter fuer die spaetere 3-Punkt-Quadratic-Verrundung.
    QuadraticThreePoint,
}

/// Laufzeit-State fuer den 1-Punkt-Arc-Modus.
#[derive(Debug, Clone)]
pub struct ArcOnePointState {
    /// Aktuell geladene Selektions-IDs.
    pub(crate) selected_node_ids: Vec<u64>,
    /// Positionen der geladenen Selektion parallel zu `selected_node_ids`.
    pub(crate) selected_positions: Vec<Vec2>,
    /// Nachbar-Snapshots des selektierten Corner-Nodes.
    pub(crate) selected_neighbors: Vec<RouteToolConnectedNeighborSeed>,
    /// Eindeutig selektierter Corner-Node fuer den Arc-Modus.
    pub(crate) corner_node_id: Option<u64>,
    /// Position des selektierten Corner-Nodes.
    pub(crate) corner_position: Option<Vec2>,
    /// Fester Verrundungsradius in Metern.
    pub(crate) radius_m: f32,
    /// Maximale Segmentlaenge fuer die Arc-Approximation.
    pub(crate) sample_spacing_m: f32,
    /// Letztes Validierungsergebnis fuer den Arc-Kontext.
    pub(crate) validation: ArcValidation,
    /// Zuletzt berechneter Arc-Plan fuer Preview/Execute.
    pub(crate) plan: Option<ArcPlan>,
}

impl Default for ArcOnePointState {
    fn default() -> Self {
        Self {
            selected_node_ids: Vec::new(),
            selected_positions: Vec::new(),
            selected_neighbors: Vec::new(),
            corner_node_id: None,
            corner_position: None,
            radius_m: DEFAULT_ARC_RADIUS_M,
            sample_spacing_m: DEFAULT_SAMPLE_SPACING_M,
            validation: ArcValidation::NeedSingleSelection,
            plan: None,
        }
    }
}

/// Gemeinsamer Tool-State fuer das oeffentliche Verrundungs-Tool in CP-02.
pub struct RoundingTool {
    /// Aktiver interner Modus.
    pub(crate) mode: RoundingMode,
    /// Laufzeit-State fuer den Arc-Modus.
    pub(crate) arc: ArcOnePointState,
    /// Panelwert fuer den spaeteren Quadratic-Modus.
    pub(crate) quadratic_sample_spacing_m: f32,
    /// Anzahl aktuell geladener selektierter Nodes.
    pub(crate) selected_node_count: usize,
    /// Zuletzt synchronisierter Snap-Radius aus dem Host.
    pub(crate) snap_radius: f32,
}

impl RoundingTool {
    /// Erstellt das Verrundungs-Tool im standardmaessigen Arc-Modus.
    pub fn new() -> Self {
        Self {
            mode: RoundingMode::ArcOnePoint,
            arc: ArcOnePointState::default(),
            quadratic_sample_spacing_m: DEFAULT_SAMPLE_SPACING_M,
            selected_node_count: 0,
            snap_radius: 3.0,
        }
    }

    pub(crate) fn reset_runtime_state(&mut self) {
        let radius_m = self.arc.radius_m;
        let sample_spacing_m = self.arc.sample_spacing_m;
        let quadratic_sample_spacing_m = self.quadratic_sample_spacing_m;
        self.arc = ArcOnePointState {
            radius_m,
            sample_spacing_m,
            ..ArcOnePointState::default()
        };
        self.quadratic_sample_spacing_m = quadratic_sample_spacing_m;
        self.selected_node_count = 0;
    }

    pub(crate) fn panel_mode(&self) -> crate::app::ui_contract::RoundingModeChoice {
        match self.mode {
            RoundingMode::ArcOnePoint => crate::app::ui_contract::RoundingModeChoice::ArcOnePoint,
            RoundingMode::QuadraticThreePoint => {
                crate::app::ui_contract::RoundingModeChoice::QuadraticThreePoint
            }
        }
    }

    pub(crate) fn set_panel_mode(
        &mut self,
        mode: crate::app::ui_contract::RoundingModeChoice,
    ) -> bool {
        let next_mode = match mode {
            crate::app::ui_contract::RoundingModeChoice::ArcOnePoint => RoundingMode::ArcOnePoint,
            crate::app::ui_contract::RoundingModeChoice::QuadraticThreePoint => {
                RoundingMode::QuadraticThreePoint
            }
        };
        if self.mode == next_mode {
            false
        } else {
            self.mode = next_mode;
            true
        }
    }

    pub(crate) fn refresh_arc_state(&mut self) {
        let (validation, plan) = recompute_arc_plan(&self.arc);
        self.arc.validation = validation;
        self.arc.plan = plan;
    }

    /// Laedt die aktuelle Node-Selektion in den Arc-Modus.
    pub(crate) fn load_selection_seed(&mut self, selection: RouteToolSelectionSeed) {
        let RouteToolSelectionSeed {
            node_ids,
            positions,
            connected_neighbors,
        } = selection;

        self.selected_node_count = node_ids.len();
        self.arc.selected_node_ids = node_ids.clone();
        self.arc.selected_positions = positions;
        self.arc.selected_neighbors = Vec::new();

        let arc_neighbors = match connected_neighbors.as_slice() {
            [neighbors] if node_ids.len() == 1 => neighbors.clone(),
            _ => Vec::new(),
        };

        if let [node_id] = self.arc.selected_node_ids.as_slice() {
            self.arc.corner_node_id = Some(*node_id);
            self.arc.corner_position = self.arc.selected_positions.first().copied();
            self.arc.selected_neighbors = arc_neighbors;
        } else {
            self.arc.corner_node_id = None;
            self.arc.corner_position = None;
        }

        self.refresh_arc_state();
    }
}

impl Default for RoundingTool {
    fn default() -> Self {
        Self::new()
    }
}
