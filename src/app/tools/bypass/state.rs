//! State-Definitionen und Konstruktor für das Ausweichstrecken-Tool.

use crate::core::{ConnectionDirection, ConnectionPriority};
use glam::Vec2;

/// Ausweichstrecken-Tool — generiert eine parallele Ausweichstrecke
/// zur selektierten Kette mit S-förmigen An-/Abfahrten.
pub struct BypassTool {
    /// Geordnete Positionen der Quell-Kette (aus Selektion gesetzt)
    pub(crate) chain_positions: Vec<Vec2>,
    /// ID des ersten Ketten-Nodes (existenter Start-Anker)
    pub(crate) chain_start_id: u64,
    /// ID des letzten Ketten-Nodes (existenter End-Anker)
    pub(crate) chain_end_id: u64,
    /// Seitlicher Versatz in Welteinheiten (positiv = links, negativ = rechts)
    pub offset: f32,
    /// Abstand zwischen Nodes auf der Hauptstrecke
    pub base_spacing: f32,
    /// Verbindungsrichtung für die erzeugten Verbindungen
    pub direction: ConnectionDirection,
    /// Priorität für die erzeugten Verbindungen
    pub priority: ConnectionPriority,
    /// Gecachte Bypass-Positionen (None = Cache ungültig)
    pub(crate) cached_positions: Option<Vec<Vec2>>,
    /// Letzter verwendeter d_blend-Wert (für Info-Anzeige)
    pub(crate) d_blend: f32,
}

impl BypassTool {
    /// Erstellt ein neues Bypass-Tool mit Standardwerten.
    pub fn new() -> Self {
        Self {
            chain_positions: Vec::new(),
            chain_start_id: 0,
            chain_end_id: 0,
            offset: 8.0,
            base_spacing: 6.0,
            direction: ConnectionDirection::Dual,
            priority: ConnectionPriority::Regular,
            cached_positions: None,
            d_blend: 0.0,
        }
    }

    /// Gibt `true` zurück wenn eine gültige Kette geladen ist (mind. 2 Punkte).
    pub fn has_chain(&self) -> bool {
        self.chain_positions.len() >= 2
    }
}

impl Default for BypassTool {
    fn default() -> Self {
        Self::new()
    }
}
