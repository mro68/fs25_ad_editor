//! State-Definitionen und Konstruktor für das Bézier-Kurven-Tool.

use super::super::common::{SegmentConfig, TangentSource, TangentState, ToolLifecycleState};
use super::super::ToolAnchor;
use super::geometry::{approx_length, compute_tangent_cp, cubic_bezier, quadratic_bezier};
use crate::core::{ConnectionDirection, ConnectionPriority};
use crate::shared::SNAP_RADIUS;
use glam::Vec2;

/// Welcher Punkt wird gerade per Drag verschoben?
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum DragTarget {
    Start,
    End,
    CP1,
    CP2,
}

/// Grad der Bézier-Kurve
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CurveDegree {
    /// Quadratisch: 1 Steuerpunkt
    Quadratic,
    /// Kubisch: 2 Steuerpunkte
    Cubic,
}

/// Phasen des Kurven-Tools
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum Phase {
    /// Startpunkt wählen
    Start,
    /// Endpunkt wählen
    End,
    /// Steuerpunkt(e) wählen / verschieben (Klick aktualisiert, Enter bestätigt)
    Control,
}

/// Bézier-Kurven-Tool (Grad 2 oder 3)
pub struct CurveTool {
    pub(crate) phase: Phase,
    pub(crate) start: Option<ToolAnchor>,
    pub(crate) end: Option<ToolAnchor>,
    /// Steuerpunkt 1 (frei positionierbar)
    pub(crate) control_point1: Option<Vec2>,
    /// Steuerpunkt 2 (nur bei kubisch)
    pub(crate) control_point2: Option<Vec2>,
    /// Gerade per Drag verschobener Punkt
    pub(crate) dragging: Option<DragTarget>,
    /// Grad der Kurve
    pub degree: CurveDegree,
    /// Segment-Konfiguration (Abstand / Node-Anzahl)
    pub(crate) seg: SegmentConfig,
    pub direction: ConnectionDirection,
    pub priority: ConnectionPriority,
    /// Gemeinsamer Lifecycle-Zustand (IDs, Endpunkt-Anker, Recreate-Flag, Snap-Radius)
    pub(crate) lifecycle: ToolLifecycleState,
    pub(crate) last_start_anchor: Option<ToolAnchor>,
    pub(crate) last_control_point1: Option<Vec2>,
    pub(crate) last_control_point2: Option<Vec2>,
    /// Tangenten-Zustand (Start/Ende, Nachbarn-Cache, Recreation-Kopien)
    pub(crate) tangents: TangentState,
}

impl CurveTool {
    /// Erstellt ein neues Kurven-Tool mit Standardparametern.
    pub fn new() -> Self {
        Self {
            phase: Phase::Start,
            start: None,
            end: None,
            control_point1: None,
            control_point2: None,
            dragging: None,
            degree: CurveDegree::Quadratic,
            seg: SegmentConfig::new(2.0),
            direction: ConnectionDirection::Dual,
            priority: ConnectionPriority::Regular,
            lifecycle: ToolLifecycleState::new(SNAP_RADIUS),
            last_start_anchor: None,
            last_control_point1: None,
            last_control_point2: None,
            tangents: TangentState::new(),
        }
    }

    /// Approximierte Kurvenlänge über Polylinien-Segmente.
    pub(crate) fn approx_length(positions_fn: impl Fn(f32) -> Vec2, samples: usize) -> f32 {
        approx_length(positions_fn, samples)
    }

    /// Kurvenlänge je nach Grad.
    pub(crate) fn curve_length(&self) -> f32 {
        let s = self.start.as_ref().map(|a| a.position());
        let e = self.end.as_ref().map(|a| a.position());
        match self.degree {
            CurveDegree::Quadratic => {
                let (Some(start), Some(end), Some(cp)) = (s, e, self.control_point1) else {
                    return 0.0;
                };
                Self::approx_length(|t| quadratic_bezier(start, cp, end, t), 64)
            }
            CurveDegree::Cubic => {
                let (Some(start), Some(end), Some(cp1), Some(cp2)) =
                    (s, e, self.control_point1, self.control_point2)
                else {
                    return 0.0;
                };
                Self::approx_length(|t| cubic_bezier(start, cp1, cp2, end, t), 64)
            }
        }
    }

    pub(crate) fn sync_derived(&mut self) {
        self.seg.sync_from_length(self.curve_length());
    }

    /// True wenn alle Steuerpunkte für den aktuellen Grad gesetzt sind.
    pub(crate) fn controls_complete(&self) -> bool {
        match self.degree {
            CurveDegree::Quadratic => self.control_point1.is_some(),
            CurveDegree::Cubic => self.control_point1.is_some() && self.control_point2.is_some(),
        }
    }

    /// Wendet die gewählten Tangenten auf die Steuerpunkte an (nur Cubic).
    pub(crate) fn apply_tangent_to_cp(&mut self) {
        if self.degree != CurveDegree::Cubic {
            return;
        }
        let (Some(start), Some(end)) = (self.start, self.end) else {
            return;
        };

        if let TangentSource::Connection { angle, .. } = self.tangents.tangent_start {
            self.control_point1 = Some(compute_tangent_cp(
                start.position(),
                angle,
                end.position(),
                true,
            ));
        }
        if let TangentSource::Connection { angle, .. } = self.tangents.tangent_end {
            self.control_point2 = Some(compute_tangent_cp(
                end.position(),
                angle,
                start.position(),
                false,
            ));
        }
    }
}

impl Default for CurveTool {
    fn default() -> Self {
        Self::new()
    }
}
