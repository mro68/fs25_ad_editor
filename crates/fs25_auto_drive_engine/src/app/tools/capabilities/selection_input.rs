//! Capability fuer Tools, die aktuelle Node-Selektion als Eingabe benoetigen.

use crate::core::ConnectedNeighbor;
use glam::Vec2;

/// Nachbarschafts-Snapshot eines selektierten Nodes fuer selection-getriebene Tools.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct RouteToolConnectedNeighborSeed {
    /// ID des Nachbar-Nodes.
    pub neighbor_id: u64,
    /// Position des Nachbar-Nodes im Weltkoordinatensystem.
    pub position: Vec2,
    /// atan2-Winkel der Verbindung in Richtung selektierter Node → Nachbar.
    pub angle: f32,
    /// `true`, wenn die zugrunde liegende Connection vom selektierten Node zum Nachbar laeuft.
    pub is_outgoing: bool,
}

impl RouteToolConnectedNeighborSeed {
    /// Baut einen serialisierbaren Tool-Snapshot aus Core-Nachbar- und Positionsdaten.
    pub fn new(neighbor: ConnectedNeighbor, position: Vec2) -> Self {
        Self {
            neighbor_id: neighbor.neighbor_id,
            position,
            angle: neighbor.angle,
            is_outgoing: neighbor.is_outgoing,
        }
    }
}

/// Selektion als Tool-Eingabe.
#[derive(Debug, Clone)]
pub struct RouteToolSelectionSeed {
    /// IDs der aktuell selektierten Nodes in der Reihenfolge der App-Selektion.
    pub node_ids: Vec<u64>,
    /// Positionen der selektierten Nodes parallel zu `node_ids`.
    pub positions: Vec<Vec2>,
    /// Nachbar-Snapshots pro selektiertem Node, parallel zu `node_ids`.
    pub connected_neighbors: Vec<Vec<RouteToolConnectedNeighborSeed>>,
}

/// Optionale Capability fuer selection-getriebene Tools.
pub trait RouteToolSelectionInput {
    /// Laedt die aktuelle Node-Selektion als Tool-Eingabe.
    fn load_selection(&mut self, selection: RouteToolSelectionSeed);
}
