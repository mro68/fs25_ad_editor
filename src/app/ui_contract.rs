//! App-weite Read-DTOs und schmale UI-Adapter fuer Route-Tool-Daten.

use crate::app::tools::RouteTool;
use crate::app::tool_contract::TangentSource;
use glam::Vec2;

/// Eine waehlbare Tangenten-Option mit bereits aufbereitetem UI-Label.
#[derive(Debug, Clone, PartialEq)]
pub struct TangentOptionData {
    /// Semantische Quelle der Tangente.
    pub source: TangentSource,
    /// Fertig formatierter UI-Text fuer Menues und Listen.
    pub label: String,
}

/// Reine Menue-Daten fuer die Tangenten-Auswahl eines Route-Tools.
///
/// Enthalten sind nur read-only DTOs und primitive Werte, damit die UI keine
/// Tool-Interna oder `egui`-nahe Zustandsobjekte kennen muss.
#[derive(Debug, Clone, PartialEq)]
pub struct TangentMenuData {
    /// Aufbereitete Optionen fuer die Start-Tangente.
    pub start_options: Vec<TangentOptionData>,
    /// Aufbereitete Optionen fuer die End-Tangente.
    pub end_options: Vec<TangentOptionData>,
    /// Aktuell gewaehlte Start-Tangente.
    pub current_start: TangentSource,
    /// Aktuell gewaehlte End-Tangente.
    pub current_end: TangentSource,
}

/// Read-DTO fuer das schwebende Route-Tool-Panel.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct RouteToolPanelData {
    /// Statustext des aktiven Tools, falls vorhanden.
    pub status_text: Option<String>,
    /// Gibt an, ob das Tool bereits Eingaben gesammelt hat.
    pub has_pending_input: bool,
}

/// Ergebnis des tool-spezifischen Config-Renderings im Route-Tool-Panel.
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub struct RouteToolConfigRenderData {
    /// Mindestens ein Konfigurationswert wurde geaendert.
    pub changed: bool,
    /// Die Aenderung erfordert eine Neuberechnung bestehender Tool-Geometrie.
    pub needs_recreate: bool,
}

/// Schmale App-Fassade fuer das schwebende Route-Tool-Panel.
///
/// Die UI erhaelt nur die benoetigten Read-Daten und eine eng gefasste
/// `render_config`-Operation statt direkten Zugriff auf den gesamten `ToolManager`.
pub struct RouteToolPanelAdapter<'a> {
    tool: Option<&'a mut dyn RouteTool>,
}

impl<'a> RouteToolPanelAdapter<'a> {
    pub(crate) fn new(tool: Option<&'a mut dyn RouteTool>) -> Self {
        Self { tool }
    }

    /// Liefert den read-only Panelzustand des aktiven Route-Tools.
    pub fn data(&self) -> RouteToolPanelData {
        RouteToolPanelData {
            status_text: self.tool.as_ref().map(|tool| tool.status_text().to_owned()),
            has_pending_input: self
                .tool
                .as_ref()
                .is_some_and(|tool| tool.has_pending_input()),
        }
    }

    /// Rendert die tool-spezifische Konfiguration und meldet Recreate-Bedarf zurueck.
    pub fn render_config(
        &mut self,
        ui: &mut egui::Ui,
        distance_wheel_step_m: f32,
    ) -> RouteToolConfigRenderData {
        let Some(tool) = self.tool.as_deref_mut() else {
            return RouteToolConfigRenderData::default();
        };

        let changed = tool.render_config(ui, distance_wheel_step_m);
        RouteToolConfigRenderData {
            changed,
            needs_recreate: changed && tool.needs_recreate(),
        }
    }
}

/// Read-DTO fuer Route-Tool-spezifische Viewport-Eingaben.
#[derive(Debug, Clone, Default, PartialEq)]
pub struct RouteToolViewportData {
    /// Drag-Ziele des aktiven Tools fuer Hit-Tests im Viewport.
    pub drag_targets: Vec<Vec2>,
    /// Gibt an, ob das Tool bereits angefangene Eingaben besitzt.
    pub has_pending_input: bool,
    /// Optional vorbereitete Tangenten-Daten fuer das Kontextmenue.
    pub tangent_menu_data: Option<TangentMenuData>,
    /// Gibt an, ob Alt+Drag als Tool-Lasso statt als Selektion geroutet werden muss.
    pub needs_lasso_input: bool,
}

impl RouteToolViewportData {
    pub(crate) fn from_active_tool(tool: Option<&dyn RouteTool>) -> Self {
        let Some(tool) = tool else {
            return Self::default();
        };

        Self {
            drag_targets: tool.drag_targets(),
            has_pending_input: tool.has_pending_input(),
            tangent_menu_data: tool.tangent_menu_data(),
            needs_lasso_input: tool.needs_lasso_input(),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::app::tools::{RouteTool, ToolAction, ToolPreview, ToolResult};
    use crate::core::RoadMap;

    struct FakeRouteTool {
        drag_targets: Vec<Vec2>,
        has_pending_input: bool,
        tangent_menu_data: Option<TangentMenuData>,
        needs_lasso_input: bool,
    }

    impl RouteTool for FakeRouteTool {
        fn name(&self) -> &str {
            "Fake Route Tool"
        }

        fn description(&self) -> &str {
            "Test-Double fuer Read-DTO-Tests"
        }

        fn status_text(&self) -> &str {
            ""
        }

        fn on_click(&mut self, _pos: Vec2, _road_map: &RoadMap, _ctrl: bool) -> ToolAction {
            ToolAction::Continue
        }

        fn preview(&self, _cursor_pos: Vec2, _road_map: &RoadMap) -> ToolPreview {
            ToolPreview::default()
        }

        fn render_config(&mut self, _ui: &mut egui::Ui, _distance_wheel_step_m: f32) -> bool {
            false
        }

        fn execute(&self, _road_map: &RoadMap) -> Option<ToolResult> {
            None
        }

        fn reset(&mut self) {}

        fn is_ready(&self) -> bool {
            false
        }

        fn has_pending_input(&self) -> bool {
            self.has_pending_input
        }

        fn drag_targets(&self) -> Vec<Vec2> {
            self.drag_targets.clone()
        }

        fn tangent_menu_data(&self) -> Option<TangentMenuData> {
            self.tangent_menu_data.clone()
        }

        fn needs_lasso_input(&self) -> bool {
            self.needs_lasso_input
        }
    }

    #[test]
    fn viewport_data_from_active_tool_copies_route_tool_read_state() {
        let tangent_menu_data = TangentMenuData {
            start_options: vec![TangentOptionData {
                source: TangentSource::Connection {
                    neighbor_id: 42,
                    angle: 0.25,
                },
                label: "Start -> Node #42".to_owned(),
            }],
            end_options: vec![TangentOptionData {
                source: TangentSource::None,
                label: "Manuell".to_owned(),
            }],
            current_start: TangentSource::Connection {
                neighbor_id: 42,
                angle: 0.25,
            },
            current_end: TangentSource::None,
        };
        let tool = FakeRouteTool {
            drag_targets: vec![Vec2::new(1.0, 2.0), Vec2::new(3.0, 4.0)],
            has_pending_input: true,
            tangent_menu_data: Some(tangent_menu_data.clone()),
            needs_lasso_input: true,
        };

        assert_eq!(
            RouteToolViewportData::from_active_tool(None),
            RouteToolViewportData::default()
        );

        let data = RouteToolViewportData::from_active_tool(Some(&tool));

        assert_eq!(data.drag_targets, vec![Vec2::new(1.0, 2.0), Vec2::new(3.0, 4.0)]);
        assert!(data.has_pending_input);
        assert_eq!(data.tangent_menu_data, Some(tangent_menu_data));
        assert!(data.needs_lasso_input);
    }
}