//! Chrome-Snapshot-DTOs fuer host-neutrale Menues und Panels.

use fs25_auto_drive_engine::shared::EditorOptions;
use serde::{Deserialize, Serialize};

use super::actions::HostActiveTool;
use super::route_tool::{
    HostDefaultConnectionDirection, HostDefaultConnectionPriority, HostRouteToolEntrySnapshot,
    HostRouteToolId, HostRouteToolSelectionSnapshot,
};

/// Host-neutraler Read-Snapshot fuer Chrome-nahe Menues und Panels.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HostChromeSnapshot {
    /// Letzte Statusmeldung der Session.
    pub status_message: Option<String>,
    /// Ob die Command-Palette sichtbar ist.
    pub show_command_palette: bool,
    /// Ob der Optionen-Dialog sichtbar ist.
    pub show_options_dialog: bool,
    /// Ob aktuell eine Karte geladen ist.
    pub has_map: bool,
    /// Ob aktuell mindestens ein Node selektiert ist.
    pub has_selection: bool,
    /// Ob die Zwischenablage Node-Daten enthaelt.
    pub has_clipboard: bool,
    /// Gibt an, ob ein Undo-Schritt verfuegbar ist.
    pub can_undo: bool,
    /// Gibt an, ob ein Redo-Schritt verfuegbar ist.
    pub can_redo: bool,
    /// Aktives Editor-Tool als stabiler Identifier.
    pub active_tool: HostActiveTool,
    /// Aktives Route-Tool im Route-Modus.
    pub active_route_tool: Option<HostRouteToolId>,
    /// Aktuelle Verbindungs-Default-Richtung.
    pub default_direction: HostDefaultConnectionDirection,
    /// Aktuelle Verbindungs-Default-Prioritaet.
    pub default_priority: HostDefaultConnectionPriority,
    /// Zuletzt gewaehlte Route-Tools je Gruppe.
    pub route_tool_memory: HostRouteToolSelectionSnapshot,
    /// Vollstaendige Laufzeitoptionen fuer host-neutrale Panels.
    pub options: EditorOptions,
    /// Aufgeloeste Route-Tool-Eintraege fuer Menues und Panels.
    pub route_tool_entries: Vec<HostRouteToolEntrySnapshot>,
    /// Anzahl der Nodes in der geladenen Karte (0 wenn keine Karte).
    pub node_count: usize,
    /// Anzahl der Verbindungen in der geladenen Karte (0 wenn keine Karte).
    pub connection_count: usize,
    /// Anzahl der Marker in der geladenen Karte (0 wenn keine Karte).
    pub marker_count: usize,
    /// Name der geladenen Karte (None wenn keine Karte oder kein Name).
    pub map_name: Option<String>,
    /// Aktueller Kamera-Zoom-Faktor.
    pub camera_zoom: f32,
    /// Aktuelle Kamera-Position in Weltkoordinaten.
    pub camera_position: [f32; 2],
    /// Pfad zur geladenen Heightmap (None wenn keine geladen).
    pub heightmap_path: Option<String>,
    /// Anzahl der selektierten Nodes.
    pub selection_count: usize,
    /// Beispiel-ID eines selektierten Nodes (None wenn keine Selektion).
    pub selection_example_id: Option<u64>,
}
