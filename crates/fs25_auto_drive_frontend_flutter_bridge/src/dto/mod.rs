use serde::{Deserialize, Serialize};

/// Serialisierbarer Snapshot der aktuellen Auswahl.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct EngineSelectionSnapshot {
    /// Aktuell selektierte Node-IDs in stabiler Reihenfolge.
    pub selected_node_ids: Vec<u64>,
}

/// Serialisierbarer Snapshot des aktuellen Viewports.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct EngineViewportSnapshot {
    /// Kameraposition in Weltkoordinaten.
    pub camera_position: [f32; 2],
    /// Zoom-Faktor des aktuellen Frames.
    pub zoom: f32,
}

/// Kleine, serialisierbare Session-Zusammenfassung fuer Host-Frontends.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct EngineSessionSnapshot {
    /// Ob aktuell eine Karte geladen ist.
    pub has_map: bool,
    /// Anzahl der Nodes der geladenen Karte.
    pub node_count: usize,
    /// Anzahl der Verbindungen der geladenen Karte.
    pub connection_count: usize,
    /// Name des aktiven Editor-Tools als stabile Textdarstellung.
    pub active_tool: String,
    /// Letzte Statusmeldung der Session.
    pub status_message: Option<String>,
    /// Ob die Command-Palette sichtbar ist.
    pub show_command_palette: bool,
    /// Ob der Options-Dialog sichtbar ist.
    pub show_options_dialog: bool,
    /// Read-only Snapshot der aktuellen Auswahl.
    pub selection: EngineSelectionSnapshot,
    /// Read-only Snapshot des aktuellen Viewports.
    pub viewport: EngineViewportSnapshot,
}
