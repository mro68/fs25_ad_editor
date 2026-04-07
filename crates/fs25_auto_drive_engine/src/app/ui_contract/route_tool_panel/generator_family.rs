use glam::Vec2;
use serde::{Deserialize, Serialize};

use super::{SegmentConfigPanelAction, SegmentConfigPanelState};

/// Panelzustand des Gerade-Strecke-Tools.
#[derive(Debug, Clone, PartialEq)]
pub struct StraightPanelState {
    /// Gemeinsame Segment-Konfiguration.
    pub segment: SegmentConfigPanelState,
}

/// Panel-Aktion des Gerade-Strecke-Tools.
#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
#[serde(tag = "kind", content = "value", rename_all = "snake_case")]
pub enum StraightPanelAction {
    /// Gemeinsame Segment-Konfiguration aendern.
    Segment(SegmentConfigPanelAction),
}

/// Read-Zustand eines automatisch berechneten Steuerpunkts im SmoothCurve-Tool.
#[derive(Debug, Clone, PartialEq)]
pub struct SmoothCurveSteererState {
    /// Position des Steuerpunkts im Weltkoordinatensystem.
    pub position: Vec2,
    /// Gibt an, ob der Punkt manuell verschoben wurde.
    pub is_manual: bool,
}

/// Panelzustand des Geglaettete-Kurve-Tools.
#[derive(Debug, Clone, PartialEq)]
pub struct SmoothCurvePanelState {
    /// Maximale Richtungsaenderung pro Segment in Grad.
    pub max_angle_deg: f32,
    /// Segment-Konfiguration (nur Distanz).
    pub segment: SegmentConfigPanelState,
    /// Minimaler Abstand zwischen erzeugten Nodes.
    pub min_distance: f32,
    /// Optionaler Approach-Steuerpunkt.
    pub approach_steerer: Option<SmoothCurveSteererState>,
    /// Optionaler Departure-Steuerpunkt.
    pub departure_steerer: Option<SmoothCurveSteererState>,
    /// Manuell gesetzte Zwischen-Kontrollpunkte.
    pub control_nodes: Vec<Vec2>,
    /// Anzahl Vorschau-Wegpunkte.
    pub preview_node_count: Option<usize>,
}

/// Panel-Aktion des Geglaettete-Kurve-Tools.
#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
#[serde(tag = "kind", content = "value", rename_all = "snake_case")]
pub enum SmoothCurvePanelAction {
    /// Maximalen Winkel setzen.
    SetMaxAngleDeg(f32),
    /// Segment-Abstand setzen.
    SetMaxSegmentLength(f32),
    /// Minimaldistanz setzen.
    SetMinDistance(f32),
    /// Automatischen Approach-Steuerpunkt wiederherstellen.
    ResetApproachSteerer,
    /// Automatischen Departure-Steuerpunkt wiederherstellen.
    ResetDepartureSteerer,
    /// Manuell gesetzten Kontrollpunkt entfernen.
    RemoveControlNode { index: usize },
}

/// Seitenwahl fuer Ein-/Ausfahrt des Parkplatz-Tools.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ParkingRampSideChoice {
    /// Linke Seite aus Marker-Sicht.
    Left,
    /// Rechte Seite aus Marker-Sicht.
    Right,
}

/// Panelzustand des Parkplatz-Tools.
#[derive(Debug, Clone, PartialEq)]
pub struct ParkingPanelState {
    /// Anzahl der Parkreihen.
    pub num_rows: usize,
    /// Abstand zwischen Reihen.
    pub row_spacing: f32,
    /// Laenge einer Parkreihe.
    pub bay_length: f32,
    /// Maximale Node-Distanz innerhalb einer Bucht.
    pub max_node_distance: f32,
    /// Einfahrts-Position entlang der Laenge.
    pub entry_t: f32,
    /// Ausfahrts-Position entlang der Laenge.
    pub exit_t: f32,
    /// Rampenlaenge.
    pub ramp_length: f32,
    /// Seite der Einfahrt.
    pub entry_side: ParkingRampSideChoice,
    /// Seite der Ausfahrt.
    pub exit_side: ParkingRampSideChoice,
    /// Marker-Gruppe fuer neu erzeugte Parkplaetze.
    pub marker_group: String,
    /// Drehschritt fuer Alt+Scroll.
    pub rotation_step_deg: f32,
    /// Aktueller Rotationswinkel in Grad.
    pub angle_deg: Option<f32>,
    /// Optionaler Bedienhinweis.
    pub hint_text: Option<String>,
}

/// Panel-Aktion des Parkplatz-Tools.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(tag = "kind", content = "value", rename_all = "snake_case")]
pub enum ParkingPanelAction {
    /// Anzahl der Reihen setzen.
    SetNumRows(usize),
    /// Reihenabstand setzen.
    SetRowSpacing(f32),
    /// Reihenlaenge setzen.
    SetBayLength(f32),
    /// Maximalen Node-Abstand setzen.
    SetMaxNodeDistance(f32),
    /// Einfahrts-Position setzen.
    SetEntryT(f32),
    /// Ausfahrts-Position setzen.
    SetExitT(f32),
    /// Rampenlaenge setzen.
    SetRampLength(f32),
    /// Seite der Einfahrt setzen.
    SetEntrySide(ParkingRampSideChoice),
    /// Seite der Ausfahrt setzen.
    SetExitSide(ParkingRampSideChoice),
    /// Marker-Gruppe setzen.
    SetMarkerGroup(String),
    /// Drehschritt setzen.
    SetRotationStepDeg(f32),
}

/// Panelzustand des Ausweichstrecken-Tools.
#[derive(Debug, Clone, PartialEq)]
pub struct BypassPanelState {
    /// Gibt an, ob eine gueltige Kette geladen ist.
    pub has_chain: bool,
    /// Meldung fuer den Fall ohne geladene Kette.
    pub empty_message: Option<String>,
    /// Seitlicher Versatz.
    pub offset: f32,
    /// Grundabstand zwischen Nodes.
    pub base_spacing: f32,
    /// Textliche Seitenbeschreibung des aktuellen Offsets.
    pub side_label: String,
    /// Anzahl neu erzeugter Zwischen-Nodes in der Preview.
    pub new_node_count: Option<usize>,
    /// Anzahl Nodes der geladenen Kette.
    pub chain_node_count: usize,
    /// Uebergangslaenge der S-Kurven, falls vorhanden.
    pub transition_length_m: Option<f32>,
}

/// Panel-Aktion des Ausweichstrecken-Tools.
#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
#[serde(tag = "kind", content = "value", rename_all = "snake_case")]
pub enum BypassPanelAction {
    /// Versatz setzen.
    SetOffset(f32),
    /// Grundabstand setzen.
    SetBaseSpacing(f32),
}
