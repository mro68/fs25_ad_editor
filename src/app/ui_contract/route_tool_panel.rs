//! Egui-freier Panel-Vertrag fuer das Floating-Route-Tool-Panel.

use crate::app::tool_contract::{RouteToolId, TangentSource};
use crate::core::{ConnectionDirection, ConnectionPriority};
use glam::Vec2;
use std::ops::RangeInclusive;

use super::TangentOptionData;

/// Gemeinsame Eingabegrenzen fuer Gleitkomma-Felder im Route-Tool-Panel.
#[derive(Debug, Clone, Copy, PartialEq)]
pub(crate) struct FloatInputLimits {
    min: f32,
    max: f32,
}

impl FloatInputLimits {
    /// Erstellt einen neuen Gleitkomma-Grenzwertbereich.
    pub(crate) const fn new(min: f32, max: f32) -> Self {
        Self { min, max }
    }

    /// Klemmt einen Wert in den gueltigen Bereich.
    pub(crate) fn clamp(self, value: f32) -> f32 {
        value.clamp(self.min, self.max)
    }

    /// Liefert den Bereich fuer egui-Widgets.
    pub(crate) fn range(self) -> RangeInclusive<f32> {
        self.min..=self.max
    }
}

/// Gemeinsame Eingabegrenzen fuer Ganzzahl-Felder im Route-Tool-Panel.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) struct UsizeInputLimits {
    min: usize,
    max: usize,
}

impl UsizeInputLimits {
    /// Erstellt einen neuen Ganzzahl-Grenzwertbereich.
    pub(crate) const fn new(min: usize, max: usize) -> Self {
        Self { min, max }
    }

    /// Klemmt einen Wert in den gueltigen Bereich.
    pub(crate) fn clamp(self, value: usize) -> usize {
        value.clamp(self.min, self.max)
    }

    /// Liefert den Bereich fuer egui-Widgets.
    pub(crate) fn range(self) -> RangeInclusive<usize> {
        self.min..=self.max
    }
}

pub(crate) const BYPASS_OFFSET_LIMITS: FloatInputLimits = FloatInputLimits::new(-200.0, 200.0);
pub(crate) const BYPASS_BASE_SPACING_LIMITS: FloatInputLimits = FloatInputLimits::new(1.0, 50.0);
pub(crate) const ROUTE_OFFSET_DISTANCE_LIMITS: FloatInputLimits = FloatInputLimits::new(0.5, 200.0);
pub(crate) const ROUTE_OFFSET_BASE_SPACING_LIMITS: FloatInputLimits =
    FloatInputLimits::new(1.0, 50.0);
pub(crate) const SMOOTH_CURVE_MAX_ANGLE_LIMITS: FloatInputLimits =
    FloatInputLimits::new(5.0, 135.0);
pub(crate) const SMOOTH_CURVE_MIN_DISTANCE_LIMITS: FloatInputLimits =
    FloatInputLimits::new(0.5, 20.0);
pub(crate) const PARKING_NUM_ROWS_LIMITS: UsizeInputLimits = UsizeInputLimits::new(1, 10);
pub(crate) const PARKING_ROW_SPACING_LIMITS: FloatInputLimits = FloatInputLimits::new(4.0, 20.0);
pub(crate) const PARKING_BAY_LENGTH_LIMITS: FloatInputLimits = FloatInputLimits::new(10.0, 100.0);
pub(crate) const PARKING_MAX_NODE_DISTANCE_LIMITS: FloatInputLimits =
    FloatInputLimits::new(2.0, 20.0);
pub(crate) const PARKING_ENTRY_EXIT_T_LIMITS: FloatInputLimits = FloatInputLimits::new(0.0, 1.0);
pub(crate) const PARKING_RAMP_LENGTH_LIMITS: FloatInputLimits = FloatInputLimits::new(2.0, 20.0);
pub(crate) const PARKING_ROTATION_STEP_LIMITS: FloatInputLimits = FloatInputLimits::new(0.5, 45.0);

/// Read-Zustand des schwebenden Route-Tool-Panels.
#[derive(Debug, Clone, Default, PartialEq)]
pub struct RouteToolPanelState {
    /// Stabile ID des aktiven Tools.
    pub active_tool_id: Option<RouteToolId>,
    /// Statustext des aktiven Tools.
    pub status_text: Option<String>,
    /// Gibt an, ob das Tool bereits Eingaben gesammelt hat.
    pub has_pending_input: bool,
    /// Gibt an, ob das Tool aktuell ausgefuehrt werden kann.
    pub can_execute: bool,
    /// Tool-spezifischer Panelzustand.
    pub config_state: Option<RouteToolConfigState>,
}

/// Tool-spezifischer Read-Zustand fuer das Panel.
#[derive(Debug, Clone, PartialEq)]
pub enum RouteToolConfigState {
    /// Panelzustand fuer das Gerade-Strecke-Tool.
    Straight(StraightPanelState),
    /// Panelzustand fuer das Bézier-Kurven-Tool.
    Curve(CurvePanelState),
    /// Panelzustand fuer das Catmull-Rom-Spline-Tool.
    Spline(SplinePanelState),
    /// Panelzustand fuer das Geglaettete-Kurve-Tool.
    SmoothCurve(SmoothCurvePanelState),
    /// Panelzustand fuer das Ausweichstrecken-Tool.
    Bypass(BypassPanelState),
    /// Panelzustand fuer das Parkplatz-Tool.
    Parking(ParkingPanelState),
    /// Panelzustand fuer das Feldgrenz-Tool.
    FieldBoundary(FieldBoundaryPanelState),
    /// Panelzustand fuer das Feldweg-Tool.
    FieldPath(FieldPathPanelState),
    /// Panelzustand fuer das Strecken-Versatz-Tool.
    RouteOffset(RouteOffsetPanelState),
    /// Panelzustand fuer das Farb-Pfad-Tool.
    ColorPath(ColorPathPanelState),
}

/// Semantische Panel-Aktion fuer das aktive Route-Tool.
#[derive(Debug, Clone, PartialEq)]
pub enum RouteToolPanelAction {
    /// Panel-Aktion fuer das Gerade-Strecke-Tool.
    Straight(StraightPanelAction),
    /// Panel-Aktion fuer das Bézier-Kurven-Tool.
    Curve(CurvePanelAction),
    /// Panel-Aktion fuer das Catmull-Rom-Spline-Tool.
    Spline(SplinePanelAction),
    /// Panel-Aktion fuer das Geglaettete-Kurve-Tool.
    SmoothCurve(SmoothCurvePanelAction),
    /// Panel-Aktion fuer das Ausweichstrecken-Tool.
    Bypass(BypassPanelAction),
    /// Panel-Aktion fuer das Parkplatz-Tool.
    Parking(ParkingPanelAction),
    /// Panel-Aktion fuer das Feldgrenz-Tool.
    FieldBoundary(FieldBoundaryPanelAction),
    /// Panel-Aktion fuer das Feldweg-Tool.
    FieldPath(FieldPathPanelAction),
    /// Panel-Aktion fuer das Strecken-Versatz-Tool.
    RouteOffset(RouteOffsetPanelAction),
    /// Panel-Aktion fuer das Farb-Pfad-Tool.
    ColorPath(ColorPathPanelAction),
}

/// Rueckgabe der App nach einer semantischen Panel-Aktion.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct RouteToolPanelEffect {
    /// Mindestens ein Fachwert wurde geaendert.
    pub changed: bool,
    /// Die Aenderung erfordert eine Neuberechnung bestehender Geometrie.
    pub needs_recreate: bool,
    /// Optionaler Folgefluss fuer die bestehende Tool-Handler-Semantik.
    pub next_action: Option<RouteToolPanelFollowUp>,
}

/// Optionale Folgeaktion nach einer Panel-Mutation.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum RouteToolPanelFollowUp {
    /// Weitere Interaktion ohne Spezialbehandlung.
    Continue,
    /// Preview neu auswerten, aber nicht ausfuehren.
    UpdatePreview,
    /// Tool ist direkt zur Ausfuehrung bereit.
    ReadyToExecute,
}

/// Darstellungsmodus der Segment-Konfiguration.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SegmentPanelMode {
    /// Tool ist noch nicht bereit; nur Distanz ist editierbar.
    Default,
    /// Tool ist bereit; Distanz und Node-Anzahl steuern die Live-Vorschau.
    Ready,
    /// Nachbearbeitung vorhandener Geometrie; Aenderungen erfordern Recreate.
    Adjusting,
}

/// Read-Zustand fuer das gemeinsame Segment-Panel mehrerer Tools.
#[derive(Debug, Clone, PartialEq)]
pub struct SegmentConfigPanelState {
    /// Aktueller Anzeigemodus.
    pub mode: SegmentPanelMode,
    /// Label fuer die aktuelle Streckenlaenge.
    pub length_label: String,
    /// Aktuelle Laenge in Metern; `None` im Default-Modus.
    pub length_m: Option<f32>,
    /// Aktueller minimaler Segment-Abstand.
    pub max_segment_length: f32,
    /// Untere Grenze fuer den Segment-Abstand.
    pub max_segment_length_min: f32,
    /// Obere Grenze fuer den Segment-Abstand.
    pub max_segment_length_max: f32,
    /// Aktuelle Node-Anzahl; `None`, wenn das Tool nur Distanz anbietet.
    pub node_count: Option<usize>,
    /// Untere Grenze fuer die Node-Anzahl.
    pub node_count_min: Option<usize>,
    /// Obere Grenze fuer die Node-Anzahl.
    pub node_count_max: Option<usize>,
}

/// Semantische Aktion fuer die gemeinsame Segment-Konfiguration.
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum SegmentConfigPanelAction {
    /// Minimalen Segment-Abstand setzen.
    SetMaxSegmentLength(f32),
    /// Gewuenschte Node-Anzahl setzen.
    SetNodeCount(usize),
}

/// Auswahlzustand fuer eine einzelne Tangenten-Auswahl.
#[derive(Debug, Clone, PartialEq)]
pub struct TangentSelectionState {
    /// Anzeigename der Auswahl.
    pub label: String,
    /// Label fuer die manuelle/fehlende Auswahl.
    pub none_label: String,
    /// Aktuell gewaehlte Tangente.
    pub current: TangentSource,
    /// Verfuegbare Verbindungsoptionen ohne die `None`-Variante.
    pub options: Vec<TangentOptionData>,
    /// Gibt an, ob die Auswahl aktuell editiert werden darf.
    pub enabled: bool,
}

/// Auswahlliste fuer den Kurvengrad.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CurveDegreeChoice {
    /// Quadratische Bézier-Kurve.
    Quadratic,
    /// Kubische Bézier-Kurve.
    Cubic,
}

/// Panelzustand des Gerade-Strecke-Tools.
#[derive(Debug, Clone, PartialEq)]
pub struct StraightPanelState {
    /// Gemeinsame Segment-Konfiguration.
    pub segment: SegmentConfigPanelState,
}

/// Panel-Aktion des Gerade-Strecke-Tools.
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum StraightPanelAction {
    /// Gemeinsame Segment-Konfiguration aendern.
    Segment(SegmentConfigPanelAction),
}

/// Panelzustand der Tangenten-Sektion im Kurven-Tool.
#[derive(Debug, Clone, PartialEq)]
pub struct CurveTangentsPanelState {
    /// Optionaler Hinweistext fuer die Sektion.
    pub help_text: Option<String>,
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
#[derive(Debug, Clone, Copy, PartialEq)]
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
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum SplinePanelAction {
    /// Start-Tangente setzen.
    SetTangentStart(TangentSource),
    /// End-Tangente setzen.
    SetTangentEnd(TangentSource),
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
#[derive(Debug, Clone, Copy, PartialEq)]
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
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
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
#[derive(Debug, Clone, PartialEq)]
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
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum BypassPanelAction {
    /// Versatz setzen.
    SetOffset(f32),
    /// Grundabstand setzen.
    SetBaseSpacing(f32),
}

/// Panelzustand des Strecken-Versatz-Tools.
#[derive(Debug, Clone, PartialEq)]
pub struct RouteOffsetPanelState {
    /// Gibt an, ob eine gueltige Kette geladen ist.
    pub has_chain: bool,
    /// Meldung fuer den Fall ohne geladene Kette.
    pub empty_message: Option<String>,
    /// Links-Versatz aktiv?
    pub left_enabled: bool,
    /// Distanz des Links-Versatzes.
    pub left_distance: f32,
    /// Rechts-Versatz aktiv?
    pub right_enabled: bool,
    /// Distanz des Rechts-Versatzes.
    pub right_distance: f32,
    /// Maximaler Abstand zwischen neuen Nodes.
    pub base_spacing: f32,
    /// Original-Kette beibehalten?
    pub keep_original: bool,
    /// Anzahl Nodes der geladenen Kette.
    pub chain_node_count: usize,
}

/// Panel-Aktion des Strecken-Versatz-Tools.
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum RouteOffsetPanelAction {
    /// Links-Versatz aktivieren/deaktivieren.
    SetLeftEnabled(bool),
    /// Distanz des Links-Versatzes setzen.
    SetLeftDistance(f32),
    /// Rechts-Versatz aktivieren/deaktivieren.
    SetRightEnabled(bool),
    /// Distanz des Rechts-Versatzes setzen.
    SetRightDistance(f32),
    /// Maximalen Node-Abstand setzen.
    SetBaseSpacing(f32),
    /// Original-Kette beibehalten setzen.
    SetKeepOriginal(bool),
}

/// Panelzustand des Feldgrenz-Tools.
#[derive(Debug, Clone, PartialEq)]
pub struct FieldBoundaryPanelState {
    /// ID des aktuell gewaehlten Feldes.
    pub selected_field_id: Option<u32>,
    /// Meldung fuer den Fall ohne Auswahl.
    pub empty_selection_text: Option<String>,
    /// Abstand zwischen erzeugten Nodes.
    pub node_spacing: f32,
    /// Versatz in Metern.
    pub offset: f32,
    /// Toleranz fuer Douglas-Peucker.
    pub straighten_tolerance: f32,
    /// Ecken-Erkennung aktiv?
    pub corner_detection_enabled: bool,
    /// Winkel-Schwellwert fuer Ecken-Erkennung.
    pub corner_angle_threshold_deg: f32,
    /// Eckenverrundung aktiv?
    pub corner_rounding_enabled: bool,
    /// Radius der Eckenverrundung.
    pub corner_rounding_radius: f32,
    /// Maximale Winkelabweichung beim Verrunden.
    pub corner_rounding_max_angle_deg: f32,
    /// Verbindungsrichtung der erzeugten Route.
    pub direction: ConnectionDirection,
    /// Strassenart der erzeugten Route.
    pub priority: ConnectionPriority,
    /// Optionaler Bedienhinweis.
    pub hint_text: Option<String>,
}

/// Panel-Aktion des Feldgrenz-Tools.
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum FieldBoundaryPanelAction {
    /// Node-Abstand setzen.
    SetNodeSpacing(f32),
    /// Versatz setzen.
    SetOffset(f32),
    /// Begradigungs-Toleranz setzen.
    SetStraightenTolerance(f32),
    /// Ecken-Erkennung aktivieren/deaktivieren.
    SetCornerDetectionEnabled(bool),
    /// Winkel-Schwellwert setzen.
    SetCornerAngleThresholdDeg(f32),
    /// Eckenverrundung aktivieren/deaktivieren.
    SetCornerRoundingEnabled(bool),
    /// Verrundungsradius setzen.
    SetCornerRoundingRadius(f32),
    /// Maximale Winkelabweichung beim Verrunden setzen.
    SetCornerRoundingMaxAngleDeg(f32),
    /// Verbindungsrichtung setzen.
    SetDirection(ConnectionDirection),
    /// Strassenart setzen.
    SetPriority(ConnectionPriority),
}

/// Auswahlmodus des Feldweg-Panels.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum FieldPathModeChoice {
    /// Ganze Felder pro Seite waehlen.
    Fields,
    /// Feldgrenz-Segmente pro Seite waehlen.
    Boundaries,
}

/// Phase des Feldweg-Panels.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum FieldPathPanelPhase {
    /// Startzustand.
    Idle,
    /// Seite 1 wird gesammelt.
    SelectingSide1,
    /// Seite 2 wird gesammelt.
    SelectingSide2,
    /// Berechnete Vorschau liegt vor.
    Preview,
}

/// Zusammenfassung einer Feldweg-Seite im Panel.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct FieldPathSelectionSummary {
    /// Titel der Seite.
    pub title: String,
    /// Zusammenfassender Text fuer Auswahl oder Leerzustand.
    pub text: String,
    /// Gibt an, ob die Seite aktuell leer ist.
    pub is_empty: bool,
}

/// Panelzustand des Feldweg-Tools.
#[derive(Debug, Clone, PartialEq)]
pub struct FieldPathPanelState {
    /// Aktueller Auswahlmodus.
    pub mode: FieldPathModeChoice,
    /// Aktuelle Panel-Phase.
    pub phase: FieldPathPanelPhase,
    /// Zusammenfassung fuer Seite 1.
    pub side1: FieldPathSelectionSummary,
    /// Zusammenfassung fuer Seite 2, sobald diese relevant ist.
    pub side2: Option<FieldPathSelectionSummary>,
    /// Gibt an, ob der Wechsel zu Seite 2 moeglich ist.
    pub can_advance_to_side2: bool,
    /// Gibt an, ob eine Berechnung moeglich ist.
    pub can_compute: bool,
    /// Optionaler Preview-Status.
    pub preview_message: Option<String>,
    /// Abstand zwischen erzeugten Nodes.
    pub node_spacing: f32,
    /// Douglas-Peucker-Toleranz.
    pub simplify_tolerance: f32,
    /// Anschluss an bestehende Nodes aktiv?
    pub connect_to_existing: bool,
}

/// Panel-Aktion des Feldweg-Tools.
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum FieldPathPanelAction {
    /// Modus setzen.
    SetMode(FieldPathModeChoice),
    /// Selektion von Seite 1 starten.
    Start,
    /// Zu Seite 2 wechseln.
    AdvanceToSide2,
    /// Mittellinie berechnen.
    Compute,
    /// Zurueck zu Seite 1 wechseln.
    BackToSide1,
    /// Zurueck zu Seite 2 wechseln.
    BackToSide2,
    /// Tool zurücksetzen.
    Reset,
    /// Node-Abstand setzen.
    SetNodeSpacing(f32),
    /// Vereinfachungs-Toleranz setzen.
    SetSimplifyTolerance(f32),
    /// Anschluss an bestehende Nodes setzen.
    SetConnectToExisting(bool),
}

/// Panel-Phase des Farb-Pfad-Tools.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ColorPathPanelPhase {
    /// Warten auf Start.
    Idle,
    /// Farben werden gesammelt.
    Sampling,
    /// Extrahiertes Wegenetz liegt als Vorschau vor.
    Preview,
}

/// Anschlussmodus des Farb-Pfad-Tools.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ExistingConnectionModeChoice {
    /// Nie anschliessen.
    Never,
    /// Nur offene Enden anschliessen.
    OpenEnds,
    /// Offene Enden und Junctions anschliessen.
    OpenEndsAndJunctions,
}

/// Kennzahlen der ColorPath-Vorschau.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct ColorPathPreviewStats {
    /// Anzahl Junctions im Netz.
    pub junction_count: usize,
    /// Anzahl offener Enden im Netz.
    pub open_end_count: usize,
    /// Anzahl Segmente im Netz.
    pub segment_count: usize,
    /// Anzahl Vorschau-Nodes ueber alle PreparedSegments.
    pub node_count: usize,
    /// Gibt an, ob eine Uebernahme moeglich ist.
    pub can_accept: bool,
}

/// Panelzustand des Farb-Pfad-Tools.
#[derive(Debug, Clone, PartialEq)]
pub struct ColorPathPanelState {
    /// Aktuelle Panel-Phase.
    pub phase: ColorPathPanelPhase,
    /// Anzahl gesampelter Farben.
    pub sample_count: usize,
    /// Mittelwert der gesampelten Farbe, falls vorhanden.
    pub avg_color: Option<[u8; 3]>,
    /// Label fuer die aktuell aktive Farbmenge.
    pub palette_label: String,
    /// Farbpalette fuer Matching und Preview.
    pub palette_colors: Vec<[u8; 3]>,
    /// Gibt an, ob die Pipeline aus dem aktuellen Sampling gestartet werden kann.
    pub can_compute: bool,
    /// Kennzahlen der Vorschau, falls vorhanden.
    pub preview_stats: Option<ColorPathPreviewStats>,
    /// Exaktmodus aktiv?
    pub exact_color_match: bool,
    /// Farbtoleranz fuer unscharfes Matching.
    pub color_tolerance: f32,
    /// Node-Abstand fuer PreparedSegments.
    pub node_spacing: f32,
    /// Vereinfachungs-Toleranz fuer PreparedSegments.
    pub simplify_tolerance: f32,
    /// Rauschfilter aktiv?
    pub noise_filter: bool,
    /// Anschlussmodus an bestehende Nodes.
    pub existing_connection_mode: ExistingConnectionModeChoice,
}

/// Panel-Aktion des Farb-Pfad-Tools.
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum ColorPathPanelAction {
    /// Sampling starten.
    StartSampling,
    /// Pipeline aus dem aktuellen Sampling berechnen.
    ComputePreview,
    /// Von der Preview zurueck in das Sampling wechseln.
    BackToSampling,
    /// Tool zurücksetzen.
    Reset,
    /// Exaktmodus setzen.
    SetExactColorMatch(bool),
    /// Farbtoleranz setzen.
    SetColorTolerance(f32),
    /// Node-Abstand setzen.
    SetNodeSpacing(f32),
    /// Vereinfachungs-Toleranz setzen.
    SetSimplifyTolerance(f32),
    /// Rauschfilter setzen.
    SetNoiseFilter(bool),
    /// Anschlussmodus setzen.
    SetExistingConnectionMode(ExistingConnectionModeChoice),
}
