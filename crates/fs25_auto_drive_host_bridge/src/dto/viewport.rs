//! Viewport-Geometry und Session-Snapshot-DTOs fuer host-neutrale Transport-Adapter.

use serde::{Deserialize, Serialize};

use super::actions::HostActiveTool;

/// Serialisierbarer Snapshot der aktuellen Auswahl.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct HostSelectionSnapshot {
    /// Aktuell selektierte Node-IDs in stabiler Reihenfolge.
    pub selected_node_ids: Vec<u64>,
}

/// Serialisierbarer Snapshot des aktuellen Viewports.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct HostViewportSnapshot {
    /// Kameraposition in Weltkoordinaten.
    pub camera_position: [f32; 2],
    /// Zoom-Faktor des aktuellen Frames.
    pub zoom: f32,
}

/// Stabile Render-Klassifikation eines Nodes fuer host-neutrale Geometry-Snapshots.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum HostViewportNodeKind {
    /// Standard-Node ohne besondere Warn- oder Subprio-Faerbung.
    Regular,
    /// Subpriorisierter Node.
    SubPrio,
    /// Warn-Node.
    Warning,
}

/// Stabile Richtungsklassifikation einer Verbindung fuer Geometry-Snapshots.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum HostViewportConnectionDirection {
    /// Pfeil in Start-zu-Ende-Richtung.
    Regular,
    /// Bidirektionale Verbindung ohne Pfeil.
    Dual,
    /// Pfeil entgegengesetzt zur Start-zu-Ende-Geometrie.
    Reverse,
}

/// Stabile Prioritaetsklassifikation einer Verbindung fuer Geometry-Snapshots.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum HostViewportConnectionPriority {
    /// Normale Verbindung.
    Regular,
    /// Subpriorisierte Verbindung.
    SubPriority,
}

/// Host-neutraler Node-Eintrag fuer einen vollstaendigen Viewport-Geometry-Snapshot.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct HostViewportNodeSnapshot {
    /// Stabile Node-ID.
    pub id: u64,
    /// Weltposition des Nodes.
    pub position: [f32; 2],
    /// Render-Klassifikation fuer die host-seitige Darstellung.
    pub kind: HostViewportNodeKind,
    /// Gibt an, ob der Node auch bei Decimation sichtbar bleiben soll.
    pub preserve_when_decimating: bool,
    /// Ob der Node aktuell selektiert ist.
    pub selected: bool,
    /// Ob der Node aktuell ausgeblendet ist.
    pub hidden: bool,
    /// Ob der Node aktuell gedimmt ist.
    pub dimmed: bool,
}

/// Host-neutrale Verbindung fuer einen vollstaendigen Viewport-Geometry-Snapshot.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct HostViewportConnectionSnapshot {
    /// Start-Node-ID.
    pub start_id: u64,
    /// End-Node-ID.
    pub end_id: u64,
    /// Weltposition des Startpunkts.
    pub start_position: [f32; 2],
    /// Weltposition des Endpunkts.
    pub end_position: [f32; 2],
    /// Richtungsklassifikation der Verbindung.
    pub direction: HostViewportConnectionDirection,
    /// Prioritaetsklassifikation der Verbindung.
    pub priority: HostViewportConnectionPriority,
    /// Ob die Verbindung ueber Hidden-Nodes aktuell ausgeblendet ist.
    pub hidden: bool,
    /// Ob die Verbindung ueber gedimmte Nodes aktuell gedimmt ist.
    pub dimmed: bool,
}

/// Host-neutraler Marker-Eintrag fuer einen vollstaendigen Viewport-Geometry-Snapshot.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct HostViewportMarkerSnapshot {
    /// Weltposition des Markers.
    pub position: [f32; 2],
}

/// Vollstaendiger, serialisierbarer Viewport-Geometry-Snapshot fuer Transport-Adapter.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct HostViewportGeometrySnapshot {
    /// Ob aktuell eine Karte im Render-Snapshot vorhanden ist.
    pub has_map: bool,
    /// Viewport-Groesse in Pixeln [width, height].
    pub viewport_size: [f32; 2],
    /// Kameraposition in Weltkoordinaten.
    pub camera_position: [f32; 2],
    /// Zoom-Faktor des Frames.
    pub zoom: f32,
    /// Welt-Einheiten pro Pixel im aktuellen Frame.
    pub world_per_pixel: f32,
    /// Gibt an, ob fuer den Frame ein Hintergrund-Asset vorhanden ist.
    pub has_background: bool,
    /// Gibt an, ob der Hintergrund in diesem Frame sichtbar ist.
    pub background_visible: bool,
    /// Read-only Node-Snapshot fuer den aktuellen Frame.
    pub nodes: Vec<HostViewportNodeSnapshot>,
    /// Read-only Verbindungs-Snapshot fuer den aktuellen Frame.
    pub connections: Vec<HostViewportConnectionSnapshot>,
    /// Read-only Marker-Snapshot fuer den aktuellen Frame.
    pub markers: Vec<HostViewportMarkerSnapshot>,
}

/// Kleine, serialisierbare Session-Zusammenfassung fuer Host-Frontends.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct HostSessionSnapshot {
    /// Ob aktuell eine Karte geladen ist.
    pub has_map: bool,
    /// Ob die geladene Karte seit dem letzten erfolgreichen Load/Save veraendert wurde.
    pub is_dirty: bool,
    /// Anzahl der Nodes der geladenen Karte.
    pub node_count: usize,
    /// Anzahl der Verbindungen der geladenen Karte.
    pub connection_count: usize,
    /// Aktives Editor-Tool als stabiler, expliziter Identifier.
    pub active_tool: HostActiveTool,
    /// Letzte Statusmeldung der Session.
    pub status_message: Option<String>,
    /// Ob die Command-Palette sichtbar ist.
    pub show_command_palette: bool,
    /// Ob der Options-Dialog sichtbar ist.
    pub show_options_dialog: bool,
    /// Gibt an, ob ein Undo-Schritt verfuegbar ist.
    pub can_undo: bool,
    /// Gibt an, ob ein Redo-Schritt verfuegbar ist.
    pub can_redo: bool,
    /// Anzahl aktuell ausstehender Dialog-Anforderungen.
    pub pending_dialog_request_count: usize,
    /// Read-only Snapshot der aktuellen Auswahl.
    pub selection: HostSelectionSnapshot,
    /// Read-only Snapshot des aktuellen Viewports.
    pub viewport: HostViewportSnapshot,
}
