//! Capability fuer Alt+Drag-Lasso als Tool-Eingabe.

use glam::Vec2;

use super::super::ToolAction;

/// Optionale Capability fuer Tools mit Lasso-Eingabe.
pub trait RouteToolLassoInput {
    /// Gibt an, ob das Tool aktuell Lasso-Eingaben empfaengt.
    fn is_lasso_input_active(&self) -> bool;

    /// Verarbeitet ein abgeschlossenes Lasso-Polygon in Weltkoordinaten.
    fn on_lasso_completed(&mut self, polygon: Vec<Vec2>) -> ToolAction;
}
