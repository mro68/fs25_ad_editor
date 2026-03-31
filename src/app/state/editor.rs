use crate::app::tools::{RouteToolGroup, RouteToolId, ToolManager};
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

/// Merkt das zuletzt gewaehlte Route-Tool pro Katalog-Gruppe.
#[derive(Debug, Clone, Copy)]
pub struct RouteToolSelectionMemory {
    /// Zuletzt gewaehlt in der Gruppe „Basics".
    pub basics: RouteToolId,
    /// Zuletzt gewaehlt in der Gruppe „Section".
    pub section: RouteToolId,
    /// Zuletzt gewaehlt in der Gruppe „Analysis".
    pub analysis: RouteToolId,
}

impl Default for RouteToolSelectionMemory {
    fn default() -> Self {
        Self {
            basics: RouteToolId::Straight,
            section: RouteToolId::Bypass,
            analysis: RouteToolId::FieldBoundary,
        }
    }
}

impl RouteToolSelectionMemory {
    /// Liefert das zuletzt gewaehlte Tool fuer eine Gruppe.
    pub fn selected_for(self, group: RouteToolGroup) -> RouteToolId {
        match group {
            RouteToolGroup::Basics => self.basics,
            RouteToolGroup::Section => self.section,
            RouteToolGroup::Analysis => self.analysis,
        }
    }

    /// Merkt das zuletzt verwendete Tool fuer dessen Gruppe.
    pub fn remember(&mut self, group: RouteToolGroup, tool_id: RouteToolId) {
        match group {
            RouteToolGroup::Basics => self.basics = tool_id,
            RouteToolGroup::Section => self.section = tool_id,
            RouteToolGroup::Analysis => self.analysis = tool_id,
        }
    }
}

/// Aktueller Zustand des Editor-Werkzeugs inklusive Route-Tool-Memory.
pub struct EditorToolState {
    /// Aktives Werkzeug
    pub active_tool: EditorTool,
    /// Quell-Node fuer Connect-Tool (wartet auf Ziel)
    pub connect_source_node: Option<u64>,
    /// Standard-Richtung fuer neue Verbindungen
    pub default_direction: ConnectionDirection,
    /// Standard-Strassenart fuer neue Verbindungen
    pub default_priority: ConnectionPriority,
    /// Zuletzt gewaehlte Route-Tools pro Gruppe.
    pub route_tool_memory: RouteToolSelectionMemory,
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
            route_tool_memory: RouteToolSelectionMemory::default(),
            tool_manager: ToolManager::new(),
        }
    }

    /// Merkt die Selektion eines Route-Tools fuer dessen Gruppe.
    pub fn remember_route_tool(&mut self, group: RouteToolGroup, tool_id: RouteToolId) {
        self.route_tool_memory.remember(group, tool_id);
    }
}
