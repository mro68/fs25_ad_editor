//! Kleiner Umbrella-Vertrag fuer Route-Tools.

use crate::app::group_registry::{GroupKind, GroupRecord};

use super::{
    RouteToolChainInput, RouteToolCore, RouteToolDrag, RouteToolHostSync, RouteToolLassoInput,
    RouteToolPanelBridge, RouteToolRecreate, RouteToolRotate, RouteToolSegmentAdjustments,
    RouteToolTangent,
};

/// Object-safe Umbrella ueber den Kernvertrag, Panel-Bruecke und Host-Sync.
///
/// Optionale Interaktionen werden nicht mehr direkt ueber den Kern angesprochen,
/// sondern ueber Capability-Discovery (`as_drag()`, `as_tangent()`, ...).
/// Die Registry-/Edit-Hooks bleiben in Commit 3 bewusst hier, damit Commit 4
/// sie spaeter isoliert entkoppeln kann.
pub trait RouteTool: RouteToolCore + RouteToolPanelBridge + RouteToolHostSync {
    /// Liefert die Recreate-Capability, falls das Tool sie unterstuetzt.
    fn as_recreate(&self) -> Option<&dyn RouteToolRecreate> {
        None
    }

    /// Liefert die mutable Recreate-Capability, falls das Tool sie unterstuetzt.
    fn as_recreate_mut(&mut self) -> Option<&mut dyn RouteToolRecreate> {
        None
    }

    /// Liefert die Drag-Capability, falls das Tool sie unterstuetzt.
    fn as_drag(&self) -> Option<&dyn RouteToolDrag> {
        None
    }

    /// Liefert die mutable Drag-Capability, falls das Tool sie unterstuetzt.
    fn as_drag_mut(&mut self) -> Option<&mut dyn RouteToolDrag> {
        None
    }

    /// Liefert die Tangent-Capability, falls das Tool sie unterstuetzt.
    fn as_tangent(&self) -> Option<&dyn RouteToolTangent> {
        None
    }

    /// Liefert die mutable Tangent-Capability, falls das Tool sie unterstuetzt.
    fn as_tangent_mut(&mut self) -> Option<&mut dyn RouteToolTangent> {
        None
    }

    /// Liefert die Rotations-Capability, falls das Tool sie unterstuetzt.
    fn as_rotate(&self) -> Option<&dyn RouteToolRotate> {
        None
    }

    /// Liefert die mutable Rotations-Capability, falls das Tool sie unterstuetzt.
    fn as_rotate_mut(&mut self) -> Option<&mut dyn RouteToolRotate> {
        None
    }

    /// Liefert die Segment-Adjustments-Capability, falls das Tool sie unterstuetzt.
    fn as_segment_adjustments(&self) -> Option<&dyn RouteToolSegmentAdjustments> {
        None
    }

    /// Liefert die mutable Segment-Adjustments-Capability, falls das Tool sie unterstuetzt.
    fn as_segment_adjustments_mut(&mut self) -> Option<&mut dyn RouteToolSegmentAdjustments> {
        None
    }

    /// Liefert die Chain-Input-Capability, falls das Tool sie unterstuetzt.
    fn as_chain_input(&self) -> Option<&dyn RouteToolChainInput> {
        None
    }

    /// Liefert die mutable Chain-Input-Capability, falls das Tool sie unterstuetzt.
    fn as_chain_input_mut(&mut self) -> Option<&mut dyn RouteToolChainInput> {
        None
    }

    /// Liefert die Lasso-Capability, falls das Tool sie unterstuetzt.
    fn as_lasso_input(&self) -> Option<&dyn RouteToolLassoInput> {
        None
    }

    /// Liefert die mutable Lasso-Capability, falls das Tool sie unterstuetzt.
    fn as_lasso_input_mut(&mut self) -> Option<&mut dyn RouteToolLassoInput> {
        None
    }

    /// Erstellt einen `GroupRecord` fuer die Registry aus dem aktuellen Tool-Zustand.
    fn make_group_record(&self, _id: u64, _node_ids: &[u64]) -> Option<GroupRecord> {
        None
    }

    /// Laedt einen gespeicherten `GroupRecord` zur nachtraeglichen Bearbeitung.
    fn load_for_edit(&mut self, _record: &GroupRecord, _kind: &GroupKind) {}
}
