use crate::app::tool_contract::{RouteToolId, TangentSource};
use crate::app::ui_contract::TangentOptionData;

use super::RouteToolConfigState;

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
