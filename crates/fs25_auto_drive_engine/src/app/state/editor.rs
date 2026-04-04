use crate::app::tool_contract::RouteToolId;
use crate::app::tools::{RouteToolGroup, ToolManager, ToolPreview};
use crate::app::ui_contract::{RouteToolPanelState, RouteToolViewportData};
use crate::core::{ConnectionDirection, ConnectionPriority, RoadMap};
use glam::Vec2;

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

    /// Liefert den egui-freien Panelzustand des aktiven Route-Tools.
    ///
    /// Gibt `Some(...)` nur im Route-Modus zurueck. Ohne aktives Tool bleibt der
    /// DTO leer genug, damit die UI generische Controls ohne Tool-Interna
    /// rendern kann.
    pub fn route_tool_panel_state(&self) -> Option<RouteToolPanelState> {
        if self.active_tool != EditorTool::Route {
            return None;
        }

        let active_tool_id = self.tool_manager.active_id();
        let tool = self.tool_manager.active_tool();

        Some(RouteToolPanelState {
            active_tool_id,
            status_text: tool.map(|tool| tool.status_text().to_owned()),
            has_pending_input: tool.is_some_and(|tool| tool.has_pending_input()),
            can_execute: tool.is_some_and(|tool| tool.is_ready()),
            config_state: tool.map(|tool| tool.panel_state()),
        })
    }

    /// Liefert die fuer den Viewport benoetigten Route-Tool-Daten als Read-DTO.
    pub fn route_tool_viewport_data(&self) -> RouteToolViewportData {
        if self.active_tool != EditorTool::Route {
            return RouteToolViewportData::default();
        }

        if let Some(tool) = self.tool_manager.active_tool() {
            let has_pending_input = tool.has_pending_input();

            RouteToolViewportData {
                drag_targets: self
                    .tool_manager
                    .active_drag()
                    .map(|tool| tool.drag_targets())
                    .unwrap_or_default(),
                has_pending_input,
                segment_shortcuts_active: has_pending_input
                    && self.tool_manager.active_segment_adjustments().is_some(),
                tangent_menu_data: self
                    .tool_manager
                    .active_tangent()
                    .and_then(|tool| tool.tangent_menu_data()),
                needs_lasso_input: self
                    .tool_manager
                    .active_lasso_input()
                    .is_some_and(|tool| tool.is_lasso_input_active()),
            }
        } else {
            RouteToolViewportData::default()
        }
    }

    /// Berechnet die aktuelle Preview-Geometrie des aktiven Route-Tools.
    pub fn route_tool_preview(
        &self,
        cursor_world: Vec2,
        road_map: &RoadMap,
    ) -> Option<ToolPreview> {
        if self.active_tool != EditorTool::Route {
            return None;
        }

        self.tool_manager
            .active_tool()
            .map(|tool| tool.preview(cursor_world, road_map))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::app::tool_contract::RouteToolId;
    use crate::app::tools::ToolAction;

    #[test]
    fn route_facades_only_expose_active_tool_in_route_mode() {
        let mut state = EditorToolState::new();
        let road_map = RoadMap::default();
        let cursor_world = Vec2::new(12.0, 0.0);

        state.active_tool = EditorTool::Route;
        state.tool_manager.set_active_by_id(RouteToolId::Straight);

        let action = state
            .tool_manager
            .active_tool_mut()
            .expect("Gerade Strecke muss fuer den F4b-Test aktiv sein")
            .on_click(Vec2::ZERO, &road_map, false);
        assert_eq!(action, ToolAction::Continue);

        let panel_data = state
            .route_tool_panel_state()
            .expect("Im Route-Modus muss ein Panelzustand vorhanden sein");
        assert_eq!(panel_data.status_text.as_deref(), Some("Endpunkt klicken"));
        assert!(panel_data.has_pending_input);

        let preview = state
            .route_tool_preview(cursor_world, &road_map)
            .expect("Im Route-Modus muss die Preview weitergereicht werden");
        assert_eq!(
            preview.nodes,
            vec![
                Vec2::new(0.0, 0.0),
                Vec2::new(6.0, 0.0),
                Vec2::new(12.0, 0.0)
            ]
        );
        assert_eq!(preview.connections, vec![(0, 1), (1, 2)]);

        state.active_tool = EditorTool::Select;

        assert!(state.route_tool_panel_state().is_none());
        assert_eq!(
            state.route_tool_viewport_data(),
            RouteToolViewportData::default()
        );
        assert!(state.route_tool_preview(cursor_world, &road_map).is_none());
    }

    #[test]
    fn route_viewport_data_only_activates_segment_shortcuts_for_matching_capability() {
        let road_map = RoadMap::default();
        let mut state = EditorToolState::new();

        state.active_tool = EditorTool::Route;
        state.tool_manager.set_active_by_id(RouteToolId::Parking);

        let action = state
            .tool_manager
            .active_tool_mut()
            .expect("Parking-Tool muss fuer den Shortcut-Test aktiv sein")
            .on_click(Vec2::new(5.0, 5.0), &road_map, false);
        assert_eq!(action, ToolAction::Continue);

        let parking_view = state.route_tool_viewport_data();
        assert!(parking_view.has_pending_input);
        assert!(!parking_view.segment_shortcuts_active);

        state.tool_manager.set_active_by_id(RouteToolId::Straight);

        let action = state
            .tool_manager
            .active_tool_mut()
            .expect("Straight-Tool muss fuer den Shortcut-Test aktiv sein")
            .on_click(Vec2::ZERO, &road_map, false);
        assert_eq!(action, ToolAction::Continue);

        let straight_view = state.route_tool_viewport_data();
        assert!(straight_view.has_pending_input);
        assert!(straight_view.segment_shortcuts_active);
    }
}
