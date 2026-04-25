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
/// Seit CP-05 ist der ColorPath-Wizard ein Single-Step-Modell mit den drei
/// kanonischen Phasen [`Idle`](Self::Idle), [`Sampling`](Self::Sampling) und
/// [`Editing`](Self::Editing). Die legacy Wizard-Phasen `Preview`,
/// `CenterlinePreview`, `JunctionEdit` und `Finalize` bleiben additiv
/// erhalten, sind aber `#[deprecated]` markiert: die Engine setzt sie nicht
/// mehr und der DTO-Layer faltet sie auf den kanonischen `"editing"`-String,
/// damit aeltere Hosts/Snapshots weiter geparst werden koennen. CP-11
/// entfernt die Legacy-Varianten endgueltig.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ColorPathPanelPhase {
    /// Warten auf Start.
    Idle,
    /// Farben werden gesammelt.
    Sampling,
    /// Single-Step-Editing: Centerlines liegen vor, Junctions koennen bewegt
    /// werden, Stage F wird live nachgezogen, das Netz ist uebernahmebereit.
    ///
    /// Kanonische Phase ab CP-05 (loest `CenterlinePreview`/`JunctionEdit`/
    /// `Finalize` ab). Der DTO-Layer serialisiert sie als `"editing"`.
    Editing,
    /// Legacy-Wizard-Phase (Stage E fertig, Stage F ausstehend).
    #[deprecated(since = "2.1.0", note = "Single-step wizard: use Editing")]
    CenterlinePreview,
    /// Legacy-Wizard-Phase (Junction-Trim-Stage).
    #[deprecated(since = "2.1.0", note = "Single-step wizard: use Editing")]
    JunctionEdit,
    /// Legacy-Wizard-Phase (Stage F angewendet, uebernahmebereit).
    #[deprecated(since = "2.1.0", note = "Single-step wizard: use Editing")]
    Finalize,
    /// Legacy-Alias fuer Hosts vor dem Wizard-Umbau.
    #[deprecated(since = "2.1.0", note = "Single-step wizard: use Editing")]
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
/// Seit CP-05 bildet der Zustand das Single-Step-Modell des ColorPath-Tools
/// ab: aus der [`ColorPathPanelPhase`] und den Compute-/Accept-Flags
/// (`can_compute`, `can_accept`) leiten Host-/UI-Schichten die drei Buttons
/// `Reset` / `Berechnen` / `Uebernehmen` ab. Die Wizard-Flags `can_next` und
/// `can_back` bleiben fuer Host-Kompat additiv erhalten, sind aber
/// `#[deprecated]`: die Engine setzt sie ab CP-06 immer auf `false`.
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
    ///
    /// Kanonisches Compute-Flag: `true` in `Sampling` mit vorhandenen Samples.
    pub can_compute: bool,
    /// Legacy-Wizard-Flag fuer den "Weiter"-Button.
    ///
    /// Engine setzt dieses Flag ab CP-06 immer auf `false`. Hosts sollen
    /// stattdessen `can_compute` lesen.
    #[deprecated(since = "2.1.0", note = "Single-step wizard: use can_compute")]
    pub can_next: bool,
    /// Legacy-Wizard-Flag fuer den "Zurueck"-Button.
    ///
    /// Engine setzt dieses Flag ab CP-06 immer auf `false`. Hosts brauchen
    /// keinen Back-Button mehr; `Reset` ersetzt den Rueckweg.
    #[deprecated(since = "2.1.0", note = "Single-step wizard: removed")]
    pub can_back: bool,
    /// Wizard-Flag: darf der "Uebernehmen"-Button gedrueckt werden?
    ///
    /// Nur in `Editing` und nur wenn ein uebernahmefaehiges Netz vorliegt.
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
/// Seit CP-05 ist die kanonische Wizard-Aktion `Compute`: sie schaltet aus
/// `Sampling` direkt nach `Editing` und baut das Netz inklusive Stage F auf.
/// Die Legacy-Aktionen `ComputePreview`, `BackToSampling`, `NextPhase` und
/// `PrevPhase` bleiben als `#[deprecated]` Aliase erhalten, damit bestehende
/// Hosts/FFI-Snapshots weiter deserialisierbar sind. CP-06 zieht die
/// Engine-Semantik vollstaendig auf `Compute`/`Accept`/`Reset` um.
#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
#[serde(tag = "kind", content = "value", rename_all = "snake_case")]
pub enum ColorPathPanelAction {
    /// Sampling starten.
    StartSampling,
    /// Single-Step-Wizard: aus `Sampling` nach `Editing` rechnen und
    /// Centerlines + Stage F aufbauen. Kanonische Compute-Aktion ab CP-05.
    Compute,
    /// Legacy-Alias fuer den Uebergang `Sampling` → `CenterlinePreview`.
    #[deprecated(since = "2.1.0", note = "Single-step wizard: use Compute")]
    ComputePreview,
    /// Legacy-Alias fuer den Rueckweg aus einer Preview-Phase in `Sampling`.
    #[deprecated(since = "2.1.0", note = "Single-step wizard: use Reset")]
    BackToSampling,
    /// Legacy-Wizard-Aktion: eine Phase nach vorn.
    #[deprecated(since = "2.1.0", note = "Single-step wizard: use Compute")]
    NextPhase,
    /// Legacy-Wizard-Aktion: eine Phase zurueck.
    #[deprecated(since = "2.1.0", note = "Single-step wizard: use Reset")]
    PrevPhase,
    /// Wizard: das fertige Netz aus `Editing` uebernehmen.
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
