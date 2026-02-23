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
    /// Virtueller Scheitelpunkt B(0.5) — nur bei Cubic
    Apex,
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
    /// Statischer Anzeigename (gesetzt beim Erstellen)
    pub(crate) tool_name: &'static str,
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
    /// Virtueller Scheitelpunkt: B(0.5) der kubischen Kurve — draggbares Handle
    pub(crate) virtual_apex: Option<Vec2>,
}

impl CurveTool {
    /// Erstellt ein neues Kurven-Tool (Grad 2, quadratisch).
    pub fn new() -> Self {
        Self {
            tool_name: "Bézier Grad 2",
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
            virtual_apex: None,
        }
    }

    /// Erstellt ein neues Kurven-Tool (Grad 3, kubisch).
    pub fn new_cubic() -> Self {
        Self {
            tool_name: "Bézier Grad 3",
            degree: CurveDegree::Cubic,
            ..Self::new()
        }
    }

    /// Approximierte Kurvenlänge über Polylinien-Segmente.
    pub(crate) fn approx_length(positions_fn: impl Fn(f32) -> Vec2, samples: usize) -> f32 {
        approx_length(positions_fn, samples)
    }

    /// Berechnet und speichert den virtuellen Scheitelpunkt B(0.5).
    ///
    /// Fehlende Steuerpunkte werden mit Sehnen-Dritteln als Fallback gesetzt.
    pub(crate) fn init_apex(&mut self) {
        if self.degree != CurveDegree::Cubic {
            return;
        }
        let (Some(start), Some(end)) = (self.start, self.end) else {
            return;
        };
        let p0 = start.position();
        let p3 = end.position();
        let chord = p3 - p0;
        let cp1 = self.control_point1.unwrap_or(p0 + chord / 3.0);
        let cp2 = self.control_point2.unwrap_or(p0 + chord * 2.0 / 3.0);
        self.virtual_apex = Some(cubic_bezier(p0, cp1, cp2, p3, 0.5));
    }

    /// Setzt CP2 auf den Standard-Wert (Sehnen-Drittel von P3),
    /// wenn CP1 bereits gesetzt ist aber CP2 noch fehlt.
    pub(crate) fn set_default_cp2_if_missing(&mut self) {
        if self.degree != CurveDegree::Cubic || self.control_point2.is_some() {
            return;
        }
        let (Some(start), Some(end)) = (self.start, self.end) else {
            return;
        };
        let chord = end.position() - start.position();
        let chord_len = chord.length();
        if chord_len < f32::EPSILON {
            return;
        }
        let chord_dir = chord / chord_len;
        self.control_point2 = Some(end.position() - chord_dir * (chord_len / 3.0));
    }

    /// Wählt automatisch die beste Start-Tangente aus `start_neighbors`.
    ///
    /// Bevorzugt eingehende Verbindungen (`is_outgoing = false`). Wählt den
    /// Nachbarn, dessen Fortsetzungsrichtung am stärksten in Richtung Endpunkt zeigt.
    /// Nichts wird gesetzt wenn kein Nachbar einen positiven Dot-Produktwert hat.
    pub(crate) fn auto_suggest_start_tangent(&mut self) {
        if self.degree != CurveDegree::Cubic {
            return;
        }
        let (Some(start), Some(end)) = (self.start, self.end) else {
            return;
        };
        let chord = end.position() - start.position();
        let chord_len = chord.length();
        if chord_len < f32::EPSILON {
            return;
        }
        if let Some(t) =
            Self::auto_suggest_tangent(&self.tangents.start_neighbors, chord / chord_len, true)
        {
            self.tangents.tangent_start = t;
        }
    }

    /// Wählt automatisch die beste End-Tangente aus `end_neighbors`.
    ///
    /// Bevorzugt ausgehende Verbindungen (`is_outgoing = true`). Wählt den
    /// Nachbarn, dessen Richtung am stärksten vom Startpunkt weg zeigt
    /// (spiegelverkehrt zu `auto_suggest_start_tangent`).
    pub(crate) fn auto_suggest_end_tangent(&mut self) {
        if self.degree != CurveDegree::Cubic {
            return;
        }
        let (Some(start), Some(end)) = (self.start, self.end) else {
            return;
        };
        let chord = end.position() - start.position();
        let chord_len = chord.length();
        if chord_len < f32::EPSILON {
            return;
        }
        if let Some(t) =
            Self::auto_suggest_tangent(&self.tangents.end_neighbors, chord / chord_len, false)
        {
            self.tangents.tangent_end = t;
        }
    }

    /// Parametrisierte Auto-Tangenten-Auswahl (gemeinsam für Start und Ende).
    ///
    /// - `neighbors`: Verfügbare Nachbarn am betreffenden Endpunkt
    /// - `chord_dir`: Normalisierte Sehnenrichtung Start→Ende (immer gleich für beide)
    /// - `is_start`: true = Start-Tangente (bevorzugt incoming, vergleicht angle+PI),
    ///   false = End-Tangente (bevorzugt outgoing, vergleicht angle direkt)
    fn auto_suggest_tangent(
        neighbors: &[crate::core::ConnectedNeighbor],
        chord_dir: Vec2,
        is_start: bool,
    ) -> Option<TangentSource> {
        use std::f32::consts::PI;
        if neighbors.is_empty() {
            return None;
        }

        // Start: eingehende bevorzugen; Ende: ausgehende bevorzugen
        let prefer_outgoing = !is_start;
        let preferred: Vec<_> = neighbors
            .iter()
            .filter(|n| n.is_outgoing == prefer_outgoing)
            .collect();
        let candidates = if preferred.is_empty() {
            neighbors.iter().collect::<Vec<_>>()
        } else {
            preferred
        };

        // Start: Fortsetzungsrichtung (angle + PI) mit Sehnenrichtung vergleichen
        // Ende: Richtung direkt (angle) mit Sehnenrichtung vergleichen
        let angle_offset = if is_start { PI } else { 0.0 };

        let best = candidates.iter().max_by(|a, b| {
            let da = Vec2::from_angle(a.angle + angle_offset).dot(chord_dir);
            let db = Vec2::from_angle(b.angle + angle_offset).dot(chord_dir);
            da.partial_cmp(&db).unwrap_or(std::cmp::Ordering::Equal)
        })?;
        let dot = Vec2::from_angle(best.angle + angle_offset).dot(chord_dir);
        if dot > 0.0 {
            Some(TangentSource::Connection {
                neighbor_id: best.neighbor_id,
                angle: best.angle,
            })
        } else {
            None
        }
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
