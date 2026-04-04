//! Tool-neutrale Typen und Konstanten fuer die Segment-Registry.

use glam::Vec2;

/// Richtung einer Gruppen-Grenz-Verbindung (Ein-/Ausfahrtstyp).
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum BoundaryDirection {
    /// Nur eingehende externe Verbindungen an diesem Node.
    Entry,
    /// Nur ausgehende externe Verbindungen an diesem Node.
    Exit,
    /// Ein- und ausgehende externe Verbindungen an diesem Node.
    Bidirectional,
}

/// Gecachte Information ueber einen Gruppen-Grenz-Node.
#[derive(Debug, Clone)]
pub struct BoundaryInfo {
    /// ID des Nodes an der Gruppengrenze.
    pub node_id: u64,
    /// true = mindestens eine Verbindung fuehrt zu einem Node ausserhalb JEDER registrierten Gruppe.
    pub has_external_connection: bool,
    /// Richtung der externen Verbindungen an diesem Node.
    pub direction: BoundaryDirection,
    /// Maximale Winkelabweichung zwischen interner Fahrtrichtung und externer Verbindung (Radiant, 0..PI).
    /// `None` wenn keine internen Verbindungen vorhanden (Winkelvergleich nicht moeglich).
    pub max_external_angle_deviation: Option<f32>,
}

/// Tool-neutraler Session-Record einer Gruppe.
#[derive(Debug, Clone)]
pub struct GroupRecord {
    /// Eindeutige Registry-ID (nicht identisch mit Node-IDs).
    pub id: u64,
    /// IDs aller zur Gruppe gehoerenden Nodes.
    pub node_ids: Vec<u64>,
    /// Original-Positionen der Nodes zum Zeitpunkt der letzten Persistierung.
    /// Index-Reihenfolge entspricht `node_ids`.
    pub original_positions: Vec<Vec2>,
    /// IDs der Nodes mit Map-Markern fuer Tool-Edit-Cleanup.
    pub marker_node_ids: Vec<u64>,
    /// Ob die Gruppe gesperrt ist (true = alle Nodes bewegen sich gemeinsam).
    pub locked: bool,
    /// Explizit gesetzte Einfahrt-Node-ID (None = kein Einfahrts-Icon).
    pub entry_node_id: Option<u64>,
    /// Explizit gesetzte Ausfahrt-Node-ID (None = kein Ausfahrts-Icon).
    pub exit_node_id: Option<u64>,
}
