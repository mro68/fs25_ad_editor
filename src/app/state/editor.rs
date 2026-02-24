use crate::app::tools::ToolManager;
use crate::core::{ConnectionDirection, ConnectionPriority};

/// Aktives Editor-Werkzeug
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum EditorTool {
    /// Standard: Nodes selektieren und verschieben
    #[default]
    Select,
    /// Verbindungen zwischen Nodes erstellen
    Connect,
    /// Neue Nodes auf der Karte platzieren
    AddNode,
    /// Route-Tools (Linie, Parkplatz, Kurve, …)
    Route,
}

/// Zustand des aktuellen Editor-Werkzeugs
pub struct EditorToolState {
    /// Aktives Werkzeug
    pub active_tool: EditorTool,
    /// Quell-Node für Connect-Tool (wartet auf Ziel)
    pub connect_source_node: Option<u64>,
    /// Standard-Richtung für neue Verbindungen
    pub default_direction: ConnectionDirection,
    /// Standard-Straßenart für neue Verbindungen
    pub default_priority: ConnectionPriority,
    /// Route-Tool-Manager (Linie, Parkplatz, Kurve, …)
    pub tool_manager: ToolManager,
}

impl Default for EditorToolState {
    fn default() -> Self {
        Self::new()
    }
}

impl EditorToolState {
    /// Erstellt den Standard-Werkzeugzustand (Select-Tool aktiv).
    pub fn new() -> Self {
        Self {
            active_tool: EditorTool::Select,
            connect_source_node: None,
            default_direction: ConnectionDirection::Regular,
            default_priority: ConnectionPriority::Regular,
            tool_manager: ToolManager::new(),
        }
    }
}
