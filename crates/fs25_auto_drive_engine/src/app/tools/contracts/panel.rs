//! UI-freie Panel-Bruecke fuer Route-Tools.

use crate::app::ui_contract::{RouteToolConfigState, RouteToolPanelAction, RouteToolPanelEffect};

/// Bruecke zwischen Route-Tool-Zustand und semantischen Panel-Aktionen.
pub trait RouteToolPanelBridge {
    /// Liefert den Statustext fuer das Floating-Panel.
    fn status_text(&self) -> &str;

    /// Liefert den egui-freien Konfigurationszustand des Tools.
    fn panel_state(&self) -> RouteToolConfigState;

    /// Wendet eine semantische Panel-Aktion auf das Tool an.
    fn apply_panel_action(&mut self, action: RouteToolPanelAction) -> RouteToolPanelEffect;
}
