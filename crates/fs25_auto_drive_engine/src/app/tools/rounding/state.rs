//! Laufzeit- und Persistenz-State fuer das Verrundungs-Tool.

use super::geometry::{
    recompute_arc_plan, recompute_quadratic_plan, ArcPlan, ArcValidation, QuadraticPlan,
    QuadraticValidation,
};
use crate::app::tool_editing::RouteToolEditPayload;
use crate::app::tools::{OrderedNodeChain, RouteToolConnectedNeighborSeed, RouteToolSelectionSeed};
use glam::Vec2;
use std::collections::HashMap;

pub(crate) const DEFAULT_ARC_RADIUS_M: f32 = 6.0;
pub(crate) const DEFAULT_ARC_SAMPLE_SPACING_M: f32 = 3.0;

/// Interne Moduswahl des Verrundungs-Tools.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum RoundingMode {
    /// Verrundet einen einzelnen Eckpunkt ueber einen Arc-/Fillet-Solver.
    ArcOnePoint,
    /// Verrundet eine geordnete 3-Punkt-Kette ueber eine quadratische Kurve.
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
            sample_spacing_m: DEFAULT_ARC_SAMPLE_SPACING_M,
            validation: ArcValidation::NeedSingleSelection,
            plan: None,
        }
    }
}

/// Laufzeit-State fuer den 3-Punkt-Quadratic-Modus.
#[derive(Debug, Clone)]
pub struct QuadraticThreePointState {
    /// Aktuell geladene Selektions-IDs.
    pub(crate) selected_node_ids: Vec<u64>,
    /// Nachbar-Snapshots pro selektiertem Node, indexiert nach Node-ID.
    pub(crate) selected_neighbors: HashMap<u64, Vec<RouteToolConnectedNeighborSeed>>,
    /// Geordnete 3-Node-Kette als `[P1, P2, P3]`.
    pub(crate) chain_node_ids: Vec<u64>,
    /// Positionen der geordneten 3-Node-Kette parallel zu `chain_node_ids`.
    pub(crate) chain_positions: Vec<Vec2>,
    /// Maximale Segmentlaenge fuer die quadratische Preview-/Execute-Approximation.
    pub(crate) sample_spacing_m: f32,
    /// Letztes Validierungsergebnis fuer den Quadratic-Kontext.
    pub(crate) validation: QuadraticValidation,
    /// Zuletzt berechneter Quadratic-Plan fuer Preview/Execute.
    pub(crate) plan: Option<QuadraticPlan>,
}

impl Default for QuadraticThreePointState {
    fn default() -> Self {
        Self {
            selected_node_ids: Vec::new(),
            selected_neighbors: HashMap::new(),
            chain_node_ids: Vec::new(),
            chain_positions: Vec::new(),
            sample_spacing_m: DEFAULT_ARC_SAMPLE_SPACING_M,
            validation: QuadraticValidation::NeedOrderedThreeNodeChain,
            plan: None,
        }
    }
}

/// Persistenter Recreate-/Edit-Zustand des Verrundungs-Tools.
#[derive(Debug, Clone, Default)]
pub struct RoundingLifecycleState {
    /// IDs der zuletzt erzeugten Nodes fuer den Recreate-Flow.
    pub(crate) last_created_ids: Vec<u64>,
    /// Signalisiert eine noetige Neuberechnung der letzten Verrundung.
    pub(crate) recreate_needed: bool,
    /// Persistierter Edit-Payload des zuletzt erzeugten oder restaurierten Segments.
    pub(crate) edit_payload: Option<RouteToolEditPayload>,
    /// `true`, wenn das Tool aus einem Group-Edit-Payload restauriert wurde.
    pub(crate) restored_for_edit: bool,
}

/// Gemeinsamer Tool-State fuer das oeffentliche Verrundungs-Tool.
pub struct RoundingTool {
    /// Aktiver interner Modus.
    pub(crate) mode: RoundingMode,
    /// Laufzeit-State fuer den Arc-Modus.
    pub(crate) arc: ArcOnePointState,
    /// Laufzeit-State fuer den Quadratic-Modus.
    pub(crate) quadratic: QuadraticThreePointState,
    /// Persistenter Recreate-/Edit-Zustand.
    pub(crate) lifecycle: RoundingLifecycleState,
    /// Zuletzt synchronisierter Snap-Radius aus dem Host.
    pub(crate) snap_radius: f32,
}

impl RoundingTool {
    /// Erstellt das Verrundungs-Tool im standardmaessigen Arc-Modus.
    pub fn new() -> Self {
        Self {
            mode: RoundingMode::ArcOnePoint,
            arc: ArcOnePointState::default(),
            quadratic: QuadraticThreePointState::default(),
            lifecycle: RoundingLifecycleState::default(),
            snap_radius: 3.0,
        }
    }

    /// Setzt nur die laufzeitbezogenen Modus-States zurueck.
    pub(crate) fn reset_runtime_state(&mut self) {
        let arc_radius_m = self.arc.radius_m;
        let arc_sample_spacing_m = self.arc.sample_spacing_m;
        let quadratic_sample_spacing_m = self.quadratic.sample_spacing_m;
        self.arc = ArcOnePointState {
            radius_m: arc_radius_m,
            sample_spacing_m: arc_sample_spacing_m,
            ..ArcOnePointState::default()
        };
        self.quadratic = QuadraticThreePointState {
            sample_spacing_m: quadratic_sample_spacing_m,
            ..QuadraticThreePointState::default()
        };
    }

    pub(crate) fn clear_persisted_edit_state(&mut self) {
        self.lifecycle.last_created_ids.clear();
        self.lifecycle.recreate_needed = false;
        self.lifecycle.edit_payload = None;
        self.lifecycle.restored_for_edit = false;
    }

    pub(crate) fn mode_locked(&self) -> bool {
        self.lifecycle.edit_payload.is_some()
            && (self.lifecycle.restored_for_edit || !self.lifecycle.last_created_ids.is_empty())
    }

    pub(crate) fn is_adjusting(&self) -> bool {
        self.lifecycle.restored_for_edit || !self.lifecycle.last_created_ids.is_empty()
    }

    pub(crate) fn has_restored_payload_for_active_mode(&self) -> bool {
        matches!(
            (self.mode, self.lifecycle.edit_payload.as_ref()),
            (
                RoundingMode::ArcOnePoint,
                Some(RouteToolEditPayload::RoundingArc { .. })
            ) | (
                RoundingMode::QuadraticThreePoint,
                Some(RouteToolEditPayload::RoundingQuadratic { .. })
            )
        )
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
        if self.mode_locked() {
            return false;
        }
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

    pub(crate) fn refresh_quadratic_state(&mut self) {
        let (validation, plan) = recompute_quadratic_plan(&self.quadratic);
        self.quadratic.validation = validation;
        self.quadratic.plan = plan;
    }

    /// Laedt die aktuelle geordnete Kette in den Quadratic-Modus.
    pub(crate) fn load_chain_seed(&mut self, chain: OrderedNodeChain) {
        if !chain.positions.is_empty() {
            self.clear_persisted_edit_state();
        }
        self.quadratic.chain_positions = chain.positions;
        self.quadratic.chain_node_ids.clear();
        self.quadratic.chain_node_ids.push(chain.start_id);
        self.quadratic.chain_node_ids.extend(chain.inner_ids);
        self.quadratic.chain_node_ids.push(chain.end_id);
        self.refresh_quadratic_state();
    }

    /// Laedt die aktuelle Node-Selektion in den Arc-Modus.
    pub(crate) fn load_selection_seed(&mut self, selection: RouteToolSelectionSeed) {
        let RouteToolSelectionSeed {
            node_ids,
            positions,
            connected_neighbors,
        } = selection;

        if !node_ids.is_empty() {
            self.clear_persisted_edit_state();
        }

        let arc_neighbors = match connected_neighbors.as_slice() {
            [neighbors] if node_ids.len() == 1 => neighbors.clone(),
            _ => Vec::new(),
        };

        self.arc.selected_node_ids = node_ids.clone();
        self.arc.selected_positions = positions;
        self.arc.selected_neighbors = Vec::new();
        self.quadratic.selected_node_ids = node_ids.clone();
        self.quadratic.selected_neighbors =
            node_ids.iter().copied().zip(connected_neighbors).collect();

        if let [node_id] = self.arc.selected_node_ids.as_slice() {
            self.arc.corner_node_id = Some(*node_id);
            self.arc.corner_position = self.arc.selected_positions.first().copied();
            self.arc.selected_neighbors = arc_neighbors;
        } else {
            self.arc.corner_node_id = None;
            self.arc.corner_position = None;
        }

        self.refresh_arc_state();
        self.refresh_quadratic_state();
    }
}

impl Default for RoundingTool {
    fn default() -> Self {
        Self::new()
    }
}
