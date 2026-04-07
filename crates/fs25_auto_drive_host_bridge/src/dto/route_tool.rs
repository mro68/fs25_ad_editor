//! Route-Tool-Katalog- und Snapshot-DTOs fuer host-neutrale Panels und Menues.

use serde::{Deserialize, Serialize};

use super::actions::HostTangentSource;

/// Host-neutrale Richtung fuer Verbindungs-Defaults im Chrome-Snapshot.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum HostDefaultConnectionDirection {
    /// Standardrichtung Start -> Ende.
    Regular,
    /// Bidirektionaler Standard.
    Dual,
    /// Umgekehrte Standardrichtung Ende -> Start.
    Reverse,
}

/// Host-neutrale Prioritaet fuer Verbindungs-Defaults im Chrome-Snapshot.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum HostDefaultConnectionPriority {
    /// Normale Verbindung.
    Regular,
    /// Subpriorisierte Verbindung.
    SubPriority,
}

/// Stabile Route-Tool-ID fuer host-neutrale Chrome-Snapshots.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum HostRouteToolId {
    /// Gerade Strecke.
    Straight,
    /// Quadratische Bézier-Kurve.
    CurveQuad,
    /// Kubische Bézier-Kurve.
    CurveCubic,
    /// Catmull-Rom-Spline.
    Spline,
    /// Ausweichstrecke.
    Bypass,
    /// Geglaettete Kurve.
    SmoothCurve,
    /// Parkplatz-Generator.
    Parking,
    /// Feldgrenzen-Analyse.
    FieldBoundary,
    /// Feldweg-Analyse.
    FieldPath,
    /// Strecken-Versatz.
    RouteOffset,
    /// Farb-Pfad-Analyse.
    ColorPath,
}

/// Stabile Route-Tool-Gruppe fuer host-neutrale Chrome-Snapshots.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum HostRouteToolGroup {
    /// Grundlegende Streckenwerkzeuge.
    Basics,
    /// Abschnitts- und Generator-Werkzeuge.
    Section,
    /// Analyse-Werkzeuge.
    Analysis,
}

/// Stabile Route-Tool-Surface fuer host-neutrale Chrome-Snapshots.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum HostRouteToolSurface {
    /// Schwebendes Floating-Menue.
    FloatingMenu,
    /// Defaults-Panel in der Sidebar.
    DefaultsPanel,
    /// Hauptmenue.
    MainMenu,
    /// Command Palette.
    CommandPalette,
}

/// Stabile Icon-Klassifikation fuer Route-Tool-Eintraege.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum HostRouteToolIconKey {
    /// Icon fuer Gerade Strecke.
    Straight,
    /// Icon fuer Bézier Grad 2.
    CurveQuad,
    /// Icon fuer Bézier Grad 3.
    CurveCubic,
    /// Icon fuer Spline.
    Spline,
    /// Icon fuer Ausweichstrecke.
    Bypass,
    /// Icon fuer Geglaettete Kurve.
    SmoothCurve,
    /// Icon fuer Parkplatz.
    Parking,
    /// Icon fuer Feldgrenze.
    FieldBoundary,
    /// Icon fuer Feldweg.
    FieldPath,
    /// Icon fuer Streckenversatz.
    RouteOffset,
    /// Icon fuer Farbpfad.
    ColorPath,
}

/// Stabile Deaktivierungsgruende fuer Route-Tool-Eintraege.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum HostRouteToolDisabledReason {
    /// Farmland-Daten fehlen.
    MissingFarmland,
    /// Hintergrundbild fehlt.
    MissingBackground,
    /// Geordnete Ketten-Selektion fehlt.
    MissingOrderedChain,
}

/// Host-neutraler Route-Tool-Eintrag fuer Menues und Panels.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct HostRouteToolEntrySnapshot {
    /// Surface, fuer die der Eintrag aufgeloest wurde.
    pub surface: HostRouteToolSurface,
    /// Anzeigegruppe des Eintrags.
    pub group: HostRouteToolGroup,
    /// Stabile Tool-ID.
    pub tool: HostRouteToolId,
    /// Kanonischer Slot des Tools im Katalog.
    pub slot: usize,
    /// Stabile Icon-Klassifikation des Eintrags.
    pub icon_key: HostRouteToolIconKey,
    /// Gibt an, ob der Eintrag aktuell aktivierbar ist.
    pub enabled: bool,
    /// Optionaler Deaktivierungsgrund.
    pub disabled_reason: Option<HostRouteToolDisabledReason>,
}

/// Zuletzt gewaehlte Route-Tools je Gruppe.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub struct HostRouteToolSelectionSnapshot {
    /// Zuletzt gewaehltes Tool in der Gruppe `Basics`.
    pub basics: HostRouteToolId,
    /// Zuletzt gewaehltes Tool in der Gruppe `Section`.
    pub section: HostRouteToolId,
    /// Zuletzt gewaehltes Tool in der Gruppe `Analysis`.
    pub analysis: HostRouteToolId,
}

/// Eine Tangentenoption fuer host-neutrale Route-Tool-Snapshots.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct HostTangentOptionSnapshot {
    /// Semantische Tangentenquelle.
    pub source: HostTangentSource,
    /// Fertig aufbereitetes Label fuer Menues/Listen.
    pub label: String,
}

/// Host-neutraler Tangenten-Menuezustand fuer Route-Tool-Kontextmenues.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct HostTangentMenuSnapshot {
    /// Verfuegbare Start-Tangenten.
    pub start_options: Vec<HostTangentOptionSnapshot>,
    /// Verfuegbare End-Tangenten.
    pub end_options: Vec<HostTangentOptionSnapshot>,
    /// Aktuell gewaehlte Start-Tangente.
    pub current_start: HostTangentSource,
    /// Aktuell gewaehlte End-Tangente.
    pub current_end: HostTangentSource,
}

/// Host-neutraler Read-Snapshot fuer Route-Tool-Viewport-Eingaben.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct HostRouteToolViewportSnapshot {
    /// Drag-Ziele des aktiven Route-Tools in Weltkoordinaten.
    pub drag_targets: Vec<[f32; 2]>,
    /// Gibt an, ob das Tool bereits Eingaben gesammelt hat.
    pub has_pending_input: bool,
    /// Gibt an, ob Segment-Shortcuts aktiv sind.
    pub segment_shortcuts_active: bool,
    /// Optionale Tangenten-Menue-Daten fuer Kontextmenues.
    pub tangent_menu_data: Option<HostTangentMenuSnapshot>,
    /// Gibt an, ob Alt+Drag als Tool-Lasso geroutet werden muss.
    pub needs_lasso_input: bool,
}
