//! Laufzeit- und Persistenz-State fuer das Arc-only-Verrundungs-Tool mit `max_angle_deg`-basierter Arc-Segmentierung.

use super::geometry::{recompute_arc_plan, ArcPlan, ArcValidation};
use crate::app::tool_editing::RouteToolEditPayload;
use crate::app::tools::{RouteToolLinearStretchSeed, RouteToolSelectionSeed};
use glam::Vec2;

pub(crate) const DEFAULT_ARC_RADIUS_M: f32 = 6.0;
pub(crate) const DEFAULT_ARC_MAX_ANGLE_DEG: f32 = 22.5;
pub(crate) const MIN_ARC_MAX_ANGLE_DEG: f32 = 1.0;
pub(crate) const MAX_ARC_MAX_ANGLE_DEG: f32 = 45.0;

/// Begrenzt den maximalen Segmentwinkel des Arc-Samplers auf einen stabilen Bereich.
pub(crate) fn clamp_arc_max_angle_deg(value: f32) -> f32 {
    value.clamp(MIN_ARC_MAX_ANGLE_DEG, MAX_ARC_MAX_ANGLE_DEG)
}

/// Laufzeit-State fuer den 1-Punkt-Arc-Modus.
#[derive(Debug, Clone)]
pub struct ArcOnePointState {
    /// Aktuell geladene Selektions-IDs.
    pub(crate) selected_node_ids: Vec<u64>,
    /// Positionen der geladenen Selektion parallel zu `selected_node_ids`.
    pub(crate) selected_positions: Vec<Vec2>,
    /// Linear aufgeloeste Anschlussstrecken des selektierten Corner-Nodes.
    pub(crate) selected_stretches: Vec<RouteToolLinearStretchSeed>,
    /// Eindeutig selektierter Corner-Node fuer den Arc-Modus.
    pub(crate) corner_node_id: Option<u64>,
    /// Position des selektierten Corner-Nodes.
    pub(crate) corner_position: Option<Vec2>,
    /// Fester Verrundungsradius in Metern.
    pub(crate) radius_m: f32,
    /// Maximaler Winkel zwischen zwei Arc-Segmenten in Grad.
    pub(crate) max_angle_deg: f32,
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
            selected_stretches: Vec::new(),
            corner_node_id: None,
            corner_position: None,
            radius_m: DEFAULT_ARC_RADIUS_M,
            max_angle_deg: DEFAULT_ARC_MAX_ANGLE_DEG,
            validation: ArcValidation::NeedSingleSelection,
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

/// Gemeinsamer Tool-State fuer das oeffentliche Arc-only-Verrundungs-Tool.
pub struct RoundingTool {
    /// Laufzeit-State fuer den Arc-only-Modus.
    pub(crate) arc: ArcOnePointState,
    /// Persistenter Recreate-/Edit-Zustand.
    pub(crate) lifecycle: RoundingLifecycleState,
    /// Zuletzt synchronisierter Snap-Radius aus dem Host.
    pub(crate) snap_radius: f32,
}

impl RoundingTool {
    /// Erstellt das Arc-only-Verrundungs-Tool mit Standardwerten.
    pub fn new() -> Self {
        Self {
            arc: ArcOnePointState::default(),
            lifecycle: RoundingLifecycleState::default(),
            snap_radius: 3.0,
        }
    }

    /// Setzt nur die laufzeitbezogenen Modus-States zurueck.
    pub(crate) fn reset_runtime_state(&mut self) {
        let arc_radius_m = self.arc.radius_m;
        let arc_max_angle_deg = self.arc.max_angle_deg;
        self.arc = ArcOnePointState {
            radius_m: arc_radius_m,
            max_angle_deg: clamp_arc_max_angle_deg(arc_max_angle_deg),
            ..ArcOnePointState::default()
        };
    }

    pub(crate) fn clear_persisted_edit_state(&mut self) {
        self.lifecycle.last_created_ids.clear();
        self.lifecycle.recreate_needed = false;
        self.lifecycle.edit_payload = None;
        self.lifecycle.restored_for_edit = false;
    }

    pub(crate) fn is_adjusting(&self) -> bool {
        self.lifecycle.restored_for_edit || !self.lifecycle.last_created_ids.is_empty()
    }

    pub(crate) fn has_restored_payload_for_active_mode(&self) -> bool {
        matches!(
            self.lifecycle.edit_payload.as_ref(),
            Some(RouteToolEditPayload::RoundingArc { .. })
        )
    }

    pub(crate) fn refresh_arc_state(&mut self) {
        let (validation, plan) = recompute_arc_plan(&self.arc);
        self.arc.validation = validation;
        self.arc.plan = plan;
    }

    /// Laedt die aktuelle Node-Selektion in den Arc-only-Kontext.
    pub(crate) fn load_selection_seed(&mut self, selection: RouteToolSelectionSeed) {
        let RouteToolSelectionSeed {
            node_ids,
            positions,
            linear_stretches,
            connected_neighbors: _,
            anchor_paths: _,
        } = selection;

        if !node_ids.is_empty() {
            self.clear_persisted_edit_state();
        }

        let arc_stretches = match linear_stretches.as_slice() {
            [stretches] if node_ids.len() == 1 => stretches.clone(),
            _ => Vec::new(),
        };

        self.arc.selected_node_ids = node_ids.clone();
        self.arc.selected_positions = positions.clone();
        self.arc.selected_stretches = Vec::new();

        if let [node_id] = self.arc.selected_node_ids.as_slice() {
            self.arc.corner_node_id = Some(*node_id);
            self.arc.corner_position = self.arc.selected_positions.first().copied();
            self.arc.selected_stretches = arc_stretches;
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
