//! Kleiner Umbrella-Vertrag fuer Route-Tools.

use super::{
    RouteToolChainInput, RouteToolCore, RouteToolDrag, RouteToolGroupEdit, RouteToolHostSync,
    RouteToolLassoInput, RouteToolPanelBridge, RouteToolRecreate, RouteToolRotate,
    RouteToolSegmentAdjustments, RouteToolTangent,
};

/// Object-safe Umbrella ueber den Kernvertrag, Panel-Bruecke und Host-Sync.
///
/// Optionale Interaktionen werden nicht mehr direkt ueber den Kern angesprochen,
/// sondern ueber Capability-Discovery (`as_drag()`, `as_tangent()`, ...).
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

    /// Liefert die Group-Edit-Capability, falls das Tool persistierbar ist.
    fn as_group_edit(&self) -> Option<&dyn RouteToolGroupEdit> {
        None
    }

    /// Liefert die mutable Group-Edit-Capability, falls das Tool persistierbar ist.
    fn as_group_edit_mut(&mut self) -> Option<&mut dyn RouteToolGroupEdit> {
        None
    }
}
