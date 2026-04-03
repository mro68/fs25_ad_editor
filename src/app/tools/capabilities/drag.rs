//! Capability fuer Viewport-Drag auf Tool-Punkten.

use crate::core::RoadMap;
use glam::Vec2;

/// Optionale Capability fuer draggbare Route-Tool-Punkte.
pub trait RouteToolDrag {
    /// Liefert die Weltpositionen aller draggbaren Punkte.
    fn drag_targets(&self) -> Vec<Vec2>;

    /// Startet einen Drag nahe `pos`.
    fn on_drag_start(&mut self, pos: Vec2, road_map: &RoadMap, pick_radius: f32) -> bool;

    /// Aktualisiert die Position waehrend des Drags.
    fn on_drag_update(&mut self, pos: Vec2);

    /// Beendet den Drag.
    fn on_drag_end(&mut self, road_map: &RoadMap);
}
