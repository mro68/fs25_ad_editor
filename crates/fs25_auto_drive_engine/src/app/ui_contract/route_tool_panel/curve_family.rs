use crate::app::tool_contract::TangentSource;
use serde::{Deserialize, Serialize};

use super::{SegmentConfigPanelAction, SegmentConfigPanelState, TangentSelectionState};

/// Auswahlliste fuer den Kurvengrad.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum CurveDegreeChoice {
    /// Quadratische Bézier-Kurve.
    Quadratic,
    /// Kubische Bézier-Kurve.
    Cubic,
}

/// Semantischer Hinweis fuer die Tangenten-Sektion im Kurven-Tool.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TangentHelpHint {
    /// Start- und Endpunkt noch nicht gesetzt; Tangenten koennen noch nicht gewaehlt werden.
    SetStartEnd,
}

/// Panelzustand der Tangenten-Sektion im Kurven-Tool.
#[derive(Debug, Clone, PartialEq)]
pub struct CurveTangentsPanelState {
    /// Optionaler semantischer Hinweis fuer die Sektion.
    pub help_hint: Option<TangentHelpHint>,
    /// Auswahl fuer die Start-Tangente.
    pub start: TangentSelectionState,
    /// Auswahl fuer die End-Tangente.
    pub end: TangentSelectionState,
}

/// Panelzustand des Bézier-Kurven-Tools.
#[derive(Debug, Clone, PartialEq)]
pub struct CurvePanelState {
    /// Aktuell gewaehlter Kurvengrad.
    pub degree: CurveDegreeChoice,
    /// Optionale Tangenten-Sektion fuer kubische Kurven.
    pub tangents: Option<CurveTangentsPanelState>,
    /// Gemeinsame Segment-Konfiguration.
    pub segment: SegmentConfigPanelState,
}

/// Panel-Aktion des Bézier-Kurven-Tools.
#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
#[serde(tag = "kind", content = "value", rename_all = "snake_case")]
pub enum CurvePanelAction {
    /// Kurvengrad setzen.
    SetDegree(CurveDegreeChoice),
    /// Start-Tangente setzen.
    SetTangentStart(TangentSource),
    /// End-Tangente setzen.
    SetTangentEnd(TangentSource),
    /// Gemeinsame Segment-Konfiguration aendern.
    Segment(SegmentConfigPanelAction),
}

/// Panelzustand des Catmull-Rom-Spline-Tools.
#[derive(Debug, Clone, PartialEq)]
pub struct SplinePanelState {
    /// Anzahl bestaetigter Kontrollpunkte im Live-Modus.
    pub control_point_count: Option<usize>,
    /// Optionale Start-Tangenten-Auswahl im Adjusting-Modus.
    pub start_tangent: Option<TangentSelectionState>,
    /// Optionale End-Tangenten-Auswahl im Adjusting-Modus.
    pub end_tangent: Option<TangentSelectionState>,
    /// Gemeinsame Segment-Konfiguration.
    pub segment: SegmentConfigPanelState,
}

/// Panel-Aktion des Catmull-Rom-Spline-Tools.
#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
#[serde(tag = "kind", content = "value", rename_all = "snake_case")]
pub enum SplinePanelAction {
    /// Start-Tangente setzen.
    SetTangentStart(TangentSource),
    /// End-Tangente setzen.
    SetTangentEnd(TangentSource),
    /// Gemeinsame Segment-Konfiguration aendern.
    Segment(SegmentConfigPanelAction),
}
