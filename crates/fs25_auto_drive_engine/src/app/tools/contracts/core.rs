//! Schlanker Kernvertrag fuer Route-Tools.

use crate::core::RoadMap;
use glam::Vec2;

use super::super::{ToolAction, ToolPreview, ToolResult};

/// Universeller Kernvertrag fuer alle Route-Tools.
///
/// Der Kern enthaelt nur Klick-Lifecycle, Preview, Execute und Reset.
/// Optionale Interaktionen wie Drag, Tangenten, Lasso oder Recreate
/// leben in separaten Capabilities.
pub trait RouteToolCore {
    /// Verarbeitet einen Viewport-Klick fuer das aktive Tool.
    fn on_click(&mut self, pos: Vec2, road_map: &RoadMap, ctrl: bool) -> ToolAction;

    /// Berechnet die aktuelle Preview-Geometrie.
    fn preview(&self, cursor_pos: Vec2, road_map: &RoadMap) -> ToolPreview;

    /// Erzeugt das Tool-Ergebnis als reine Daten.
    fn execute(&self, road_map: &RoadMap) -> Option<ToolResult>;

    /// Setzt den internen Eingabezustand des Tools zurueck.
    fn reset(&mut self);

    /// Gibt an, ob das Tool aktuell ausfuehrbar ist.
    fn is_ready(&self) -> bool;

    /// Gibt an, ob das Tool bereits angefangene Eingaben haelt.
    fn has_pending_input(&self) -> bool;
}
