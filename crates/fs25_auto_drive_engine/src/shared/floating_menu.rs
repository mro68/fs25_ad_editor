//! Schwebender Kontext-Menue-Zustand (UI-Chrome, egui-frei).

use crate::shared::RouteToolGroup;
use glam::Vec2;

/// Zustand eines schwebenden Kontextmenues.
#[derive(Debug, Clone, Copy)]
pub struct FloatingMenuState {
    /// Art des aktuell geoeffneten Menues.
    pub kind: FloatingMenuKind,
    /// Bildschirmposition des Menues in Host-Pixelkoordinaten.
    pub pos: Vec2,
}

/// Typ des schwebenden Kontextmenues.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum FloatingMenuKind {
    /// Werkzeug-Menue (Select/Connect/AddNode).
    Tools,
    /// Route-Tool-Menue fuer eine kanonische Tool-Gruppe.
    RouteTools(RouteToolGroup),
    /// Richtungs- und Strassenart-Menue (R).
    DirectionPriority,
    /// Zoom-Funktionen (Z).
    Zoom,
}
