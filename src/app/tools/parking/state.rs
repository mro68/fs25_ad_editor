//! State-Strukturen fuer das ParkingTool.

use crate::app::tools::common::ToolLifecycleState;
use crate::core::{ConnectionDirection, ConnectionPriority};
use glam::Vec2;

/// Seitenwahl fuer Ein-/Ausfahrt aus Sicht des Markers.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum RampSide {
    /// Links aus Marker-Sicht (lokal +Y / noerdlich)
    Left,
    /// Rechts aus Marker-Sicht (lokal -Y / suedlich)
    Right,
}

/// Interaktionsphasen des ParkingTool.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ParkingPhase {
    /// Vorschau folgt dem Cursor; Alt+Scroll dreht; Klick → Configuring.
    Idle,
    /// Position+Rotation fixiert; Config-Panel aktiv; Bestätigen/Abbrechen.
    Configuring,
    /// Repositionierung: Vorschau folgt Cursor wieder; Klick → zurück zu Configuring.
    Adjusting,
}

/// Konfiguration fuer ein Parkplatz-Layout.
#[derive(Debug, Clone)]
pub struct ParkingConfig {
    /// Anzahl Parkreihen (1–10).
    pub num_rows: usize,
    /// Abstand zwischen benachbarten Reihen in Metern (4–20).
    pub row_spacing: f32,
    /// Laenge jeder Reihe (Ost-West-Ausdehnung) in Metern (10–50).
    pub bay_length: f32,
    /// Position der Einfahrt entlang der Reihenlaenge (0.0 = Ost, 1.0 = West).
    pub entry_t: f32,
    /// Position der Ausfahrt entlang der Reihenlaenge (0.0 = Ost, 1.0 = West).
    pub exit_t: f32,
    /// Laenge der 45°-Rampen fuer Ein- und Ausfahrt in Metern.
    pub ramp_length: f32,
    /// Seite der Einfahrt aus Sicht des Markers.
    pub entry_side: RampSide,
    /// Seite der Ausfahrt aus Sicht des Markers.
    pub exit_side: RampSide,
    /// Marker-Gruppe fuer alle erzeugten Buchten-Marker.
    pub marker_group: String,
}

impl Default for ParkingConfig {
    fn default() -> Self {
        Self {
            num_rows: 2,
            row_spacing: 7.0,
            bay_length: 35.0,
            entry_t: 0.4,
            exit_t: 0.7,
            ramp_length: 5.0,
            entry_side: RampSide::Right,
            exit_side: RampSide::Right,
            marker_group: "Parkplatz".to_string(),
        }
    }
}

/// Parkplatz-Layout-Tool.
pub struct ParkingTool {
    pub(crate) phase: ParkingPhase,
    /// Gesetzter Ursprungspunkt (Mitte der östlichen Enden).
    pub(crate) origin: Option<Vec2>,
    /// Rotationswinkel (Radiant), gesteuert durch Alt+Scroll.
    pub(crate) angle: f32,
    /// Drehungs-Schrittweite in Grad (einstellbar im Config-Panel).
    pub(crate) rotation_step_deg: f32,
    pub(crate) config: ParkingConfig,
    pub direction: ConnectionDirection,
    pub priority: ConnectionPriority,
    pub(crate) lifecycle: ToolLifecycleState,
}

impl Default for ParkingTool {
    fn default() -> Self {
        Self::new()
    }
}

impl ParkingTool {
    /// Erstellt ein neues ParkingTool mit Standardwerten.
    pub fn new() -> Self {
        Self {
            phase: ParkingPhase::Idle,
            origin: None,
            angle: 0.0,
            rotation_step_deg: 5.0,
            config: ParkingConfig::default(),
            direction: ConnectionDirection::Dual,
            priority: ConnectionPriority::Regular,
            lifecycle: ToolLifecycleState::new(3.0),
        }
    }
}
