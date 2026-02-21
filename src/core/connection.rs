//! Repräsentiert eine Verbindung zwischen zwei Wegpunkten.

use glam::Vec2;

/// Richtung der Verbindung
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum ConnectionDirection {
    /// Einfache Einbahnstraße
    #[default]
    Regular,
    /// Zweispurige Verbindung (beide Richtungen)
    Dual,
    /// Rückwärts-Verbindung
    Reverse,
}

/// Priorität der Verbindung
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum ConnectionPriority {
    /// Normale Verbindung
    #[default]
    Regular,
    /// Subpriorisierte Verbindung
    SubPriority,
}

/// Eine Verbindung zwischen zwei Wegpunkten
#[derive(Debug, Clone)]
pub struct Connection {
    /// Start-Node-ID
    pub start_id: u64,
    /// End-Node-ID
    pub end_id: u64,
    /// Richtung der Verbindung
    pub direction: ConnectionDirection,
    /// Priorität der Verbindung
    pub priority: ConnectionPriority,
    /// Mittelpunkt der Verbindung (2D)
    pub midpoint: Vec2,
    /// Winkel der Verbindung (Radiant)
    pub angle: f32,
}

impl Connection {
    /// Erstellt eine neue Verbindung
    pub fn new(
        start_id: u64,
        end_id: u64,
        direction: ConnectionDirection,
        priority: ConnectionPriority,
        start_pos: Vec2,
        end_pos: Vec2,
    ) -> Self {
        let (midpoint, angle) = Self::calculate_geometry(start_pos, end_pos);

        Self {
            start_id,
            end_id,
            direction,
            priority,
            midpoint,
            angle,
        }
    }

    /// Aktualisiert die Geometrie auf Basis der Node-Positionen
    pub fn update_geometry(&mut self, start_pos: Vec2, end_pos: Vec2) {
        let (midpoint, angle) = Self::calculate_geometry(start_pos, end_pos);
        self.midpoint = midpoint;
        self.angle = angle;
    }

    fn calculate_geometry(start_pos: Vec2, end_pos: Vec2) -> (Vec2, f32) {
        let midpoint = (start_pos + end_pos) * 0.5;
        let delta = end_pos - start_pos;
        let angle = delta.y.atan2(delta.x);

        (midpoint, angle)
    }
}
