//! Capability fuer Tangentenmenues und Tangentenanwendung.

use crate::app::tool_contract::TangentSource;
use crate::app::ui_contract::TangentMenuData;

/// Optionale Capability fuer Tools mit Tangenten-Auswahl.
pub trait RouteToolTangent {
    /// Liefert die Daten fuer das Tangenten-Kontextmenue.
    fn tangent_menu_data(&self) -> Option<TangentMenuData>;

    /// Wendet die gewaehlten Tangenten an.
    fn apply_tangent_selection(&mut self, start: TangentSource, end: TangentSource);
}
