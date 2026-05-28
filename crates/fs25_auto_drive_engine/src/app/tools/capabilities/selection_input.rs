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

/// Linear aufgeloeste Anschlussstrecke eines selektierten Nodes fuer lokale Replace-Pfade.
#[derive(Debug, Clone, PartialEq)]
pub struct RouteToolLinearStretchSeed {
    /// IDs der Stretch-Nodes vom selektierten Node nach aussen.
    pub node_ids: Vec<u64>,
    /// Positionen der Stretch-Nodes parallel zu `node_ids`.
    pub positions: Vec<Vec2>,
    /// atan2-Winkel der ersten Stretch-Kante selektierter Node → erster Stretch-Node.
    pub angle: f32,
    /// `true`, wenn die zugrunde liegende Strecke in den selektierten Node hinein fuehrt.
    pub has_incoming: bool,
    /// `true`, wenn die zugrunde liegende Strecke aus dem selektierten Node heraus fuehrt.
    pub has_outgoing: bool,
}

/// Eindeutiger Anchor-Pfad zwischen zwei selektierten Nodes fuer lokale Replace-Pfade.
#[derive(Debug, Clone, PartialEq)]
pub struct RouteToolAnchorPathSeed {
    /// IDs des eindeutigen Pfads inklusive beider selektierter Anchor-Nodes.
    pub node_ids: Vec<u64>,
    /// `true`, wenn der Pfad in gespeicherter Reihenfolge vollstaendig gerichtet ist.
    pub has_forward_path: bool,
    /// `true`, wenn der Pfad in umgekehrter Reihenfolge vollstaendig gerichtet ist.
    pub has_reverse_path: bool,
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
    /// Linear aufgeloeste Anschlussstrecken pro selektiertem Node, parallel zu `node_ids`.
    pub linear_stretches: Vec<Vec<RouteToolLinearStretchSeed>>,
    /// Eindeutig aufgeloeste Anchor-Pfade zwischen selektierten Nodes.
    pub anchor_paths: Vec<RouteToolAnchorPathSeed>,
}

/// Optionale Capability fuer selection-getriebene Tools.
pub trait RouteToolSelectionInput {
    /// Laedt die aktuelle Node-Selektion als Tool-Eingabe.
    fn load_selection(&mut self, selection: RouteToolSelectionSeed);
}
