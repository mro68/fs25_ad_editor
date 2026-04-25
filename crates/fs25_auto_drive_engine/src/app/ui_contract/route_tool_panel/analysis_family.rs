use crate::core::{ConnectionDirection, ConnectionPriority};
use crate::shared::I18nKey;
use serde::{Deserialize, Serialize};

/// Panelzustand des Strecken-Versatz-Tools.
#[derive(Debug, Clone, PartialEq)]
pub struct RouteOffsetPanelState {
    /// Gibt an, ob eine gueltige Kette geladen ist.
    pub has_chain: bool,
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
#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
#[serde(tag = "kind", content = "value", rename_all = "snake_case")]
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
    /// Zeigt Hinweis: weiterer Klick waehlt anderes Feld.
    pub show_select_hint: bool,
}

/// Panel-Aktion des Feldgrenz-Tools.
#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
#[serde(tag = "kind", content = "value", rename_all = "snake_case")]
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
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
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
    /// Titel der Seite (I18n-Schluessel).
    pub title: I18nKey,
    /// Zusammenfassender Text fuer dynamische Inhalte (Felder-Liste, Segmentanzahl).
    pub text: String,
    /// Statischer Leer-Hinweis als I18n-Schluessel (gesetzt wenn `is_empty` und kein dynamischer Text).
    pub empty_hint: Option<I18nKey>,
    /// Gibt an, ob die Seite aktuell leer ist.
    pub is_empty: bool,
}

/// Semantischer Vorschau-Status im Feldweg-Tool.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum FieldPathPreviewStatus {
    /// Keine Mittellinie gefunden; Seiten anpassen.
    NoMiddleLine,
    /// Vorschau erfolgreich generiert.
    Generated { node_count: usize },
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
    /// Semantischer Vorschau-Status, falls eine Vorschau vorliegt.
    pub preview_status: Option<FieldPathPreviewStatus>,
    /// Abstand zwischen erzeugten Nodes.
    pub node_spacing: f32,
    /// Douglas-Peucker-Toleranz.
    pub simplify_tolerance: f32,
    /// Anschluss an bestehende Nodes aktiv?
    pub connect_to_existing: bool,
}

/// Panel-Aktion des Feldweg-Tools.
#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
#[serde(tag = "kind", content = "value", rename_all = "snake_case")]
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
///
/// Seit CP-04 loest der dreistufige Wizard-Fluss
/// `CenterlinePreview` → `JunctionEdit` → `Finalize` die alte Sammel-Phase
/// `Preview` ab. Die Legacy-Variante bleibt additiv erhalten, um bestehende
/// FFI-Hosts nicht zu brechen, wird aber vom Engine-Layer nicht mehr
/// emittiert und per `#[deprecated]` zur Migration markiert. CP-11 entfernt
/// sie endgueltig, sobald die Hosts umgezogen sind.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ColorPathPanelPhase {
    /// Warten auf Start.
    Idle,
    /// Farben werden gesammelt.
    Sampling,
    /// Phase 1: Stage E ist durchgelaufen, Stage F noch nicht; Centerlines liegen vor.
    CenterlinePreview,
    /// Phase 2: Junctions koennen bewegt werden, Stage F bleibt zurueckgehalten.
    JunctionEdit,
    /// Phase 3: Stage F angewendet; das Netz ist zum Uebernehmen bereit.
    Finalize,
    /// Legacy-Alias fuer Hosts vor dem Wizard-Umbau.
    ///
    /// Wird von der Engine nicht mehr gesetzt. Fuer Einlese-/Serde-Pfade
    /// weiterhin vorhanden, damit alte FFI-Snapshots geparst werden koennen.
    #[deprecated(
        since = "2.1.0",
        note = "Wizard-Phasen verwenden: CenterlinePreview/JunctionEdit/Finalize"
    )]
    Preview,
}

/// Anschlussmodus des Farb-Pfad-Tools.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
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
///
/// Seit CP-04 bildet der Zustand den ColorPath-Wizard ab: neben der aktuellen
/// [`ColorPathPanelPhase`] liefert er die drei Wizard-Flags `can_next`,
/// `can_back` und `can_accept`, ueber die Host-/UI-Schichten die
/// Navigations-Buttons (`NextPhase`, `PrevPhase`, `Accept`) enablen. Das
/// legacy-kompatible Flag `can_compute` bleibt als Startkriterium fuer die
/// Sampling→CenterlinePreview-Transition erhalten.
#[derive(Debug, Clone, PartialEq)]
pub struct ColorPathPanelState {
    /// Aktuelle Panel-Phase.
    pub phase: ColorPathPanelPhase,
    /// Anzahl gesampelter Farben.
    pub sample_count: usize,
    /// Mittelwert der gesampelten Farbe, falls vorhanden.
    pub avg_color: Option<[u8; 3]>,
    /// Farbpalette fuer Matching und Preview.
    pub palette_colors: Vec<[u8; 3]>,
    /// Gibt an, ob die Pipeline aus dem aktuellen Sampling gestartet werden kann.
    pub can_compute: bool,
    /// Wizard-Flag: darf der "Weiter"-Button gedrueckt werden?
    ///
    /// Berechnet vom Engine-Layer; Host-/UI-Schichten lesen nur. Semantik:
    /// - `Sampling` → `can_compute` (Samples liegen vor).
    /// - `CenterlinePreview`/`JunctionEdit` → `true`.
    /// - andere Phasen → `false`.
    pub can_next: bool,
    /// Wizard-Flag: darf der "Zurueck"-Button gedrueckt werden?
    ///
    /// `true` in jeder Phase ab `CenterlinePreview`; sonst `false`.
    pub can_back: bool,
    /// Wizard-Flag: darf der "Uebernehmen"-Button gedrueckt werden?
    ///
    /// Nur in `Finalize` und nur wenn ein uebernahmefaehiges Netz vorliegt.
    pub can_accept: bool,
    /// Kennzahlen der Vorschau, falls vorhanden.
    pub preview_stats: Option<ColorPathPreviewStats>,
    /// Exaktmodus aktiv?
    pub exact_color_match: bool,
    /// Farbtoleranz fuer unscharfes Matching.
    pub color_tolerance: f32,
    /// Node-Abstand fuer die finale Punktverteilung der PreparedSegments.
    pub node_spacing: f32,
    /// Vereinfachungs-Toleranz fuer PreparedSegments.
    pub simplify_tolerance: f32,
    /// Radius in Metern fuer die Kreuzungsbegradigung beim Junction-Trim.
    pub junction_radius: f32,
    /// Rauschfilter aktiv?
    pub noise_filter: bool,
    /// Anschlussmodus an bestehende Nodes.
    pub existing_connection_mode: ExistingConnectionModeChoice,
}

/// Panel-Aktion des Farb-Pfad-Tools.
///
/// Der Wizard-Fluss kennt seit CP-04 die additiven Aktionen `NextPhase`,
/// `PrevPhase` und `Accept`. Die Legacy-Aktionen `ComputePreview` und
/// `BackToSampling` bleiben als deprecated-Alias bestehen, damit bestehende
/// Hosts weiter kompilieren; die Engine-Semantik wird erst in CP-05
/// vollstaendig auf die neuen Varianten gezogen.
#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
#[serde(tag = "kind", content = "value", rename_all = "snake_case")]
pub enum ColorPathPanelAction {
    /// Sampling starten.
    StartSampling,
    /// Legacy-Alias fuer den Uebergang `Sampling` → `CenterlinePreview`.
    #[deprecated(since = "2.1.0", note = "Wizard-Aktion `NextPhase` verwenden")]
    ComputePreview,
    /// Legacy-Alias fuer den Rueckweg aus einer Preview-Phase in `Sampling`.
    #[deprecated(since = "2.1.0", note = "Wizard-Aktion `PrevPhase` verwenden")]
    BackToSampling,
    /// Wizard: eine Phase nach vorn (Sampling→CenterlinePreview→JunctionEdit→Finalize).
    NextPhase,
    /// Wizard: eine Phase zurueck (Finalize→JunctionEdit→CenterlinePreview→Sampling).
    PrevPhase,
    /// Wizard: das fertige Netz aus `Finalize` uebernehmen.
    Accept,
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
    /// Radius fuer Junction-Trim und Kreuzungsbegradigung setzen.
    SetJunctionRadius(f32),
    /// Rauschfilter setzen.
    SetNoiseFilter(bool),
    /// Anschlussmodus setzen.
    SetExistingConnectionMode(ExistingConnectionModeChoice),
}
