use crate::app::segment_registry::{
    TOOL_INDEX_BYPASS, TOOL_INDEX_CURVE_QUAD, TOOL_INDEX_SMOOTH_CURVE, TOOL_INDEX_STRAIGHT,
};
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
    /// Quell-Node fuer Connect-Tool (wartet auf Ziel)
    pub connect_source_node: Option<u64>,
    /// Standard-Richtung fuer neue Verbindungen
    pub default_direction: ConnectionDirection,
    /// Standard-Strassenart fuer neue Verbindungen
    pub default_priority: ConnectionPriority,
    /// Zuletzt gewaehlter Tool-Index in der Gruppe "Geraden".
    pub last_straight_index: usize,
    /// Zuletzt gewaehlter Tool-Index in der Gruppe "Kurven".
    pub last_curve_index: usize,
    /// Zuletzt gewaehlter Tool-Index in der Gruppe "Kurven" (inkl. geglättete Kurve).
    pub last_smooth_curve_index: usize,
    /// Zuletzt gewaehlter Tool-Index in der Gruppe "Abschnittswerkzeuge".
    pub last_section_tool_index: usize,
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
            last_straight_index: TOOL_INDEX_STRAIGHT,
            last_curve_index: TOOL_INDEX_CURVE_QUAD,
            last_smooth_curve_index: TOOL_INDEX_SMOOTH_CURVE,
            last_section_tool_index: TOOL_INDEX_BYPASS,
            tool_manager: ToolManager::new(),
        }
    }
}
