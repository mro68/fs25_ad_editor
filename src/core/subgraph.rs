//! Teilgraph-Extraktion fuer Copy & Paste Funktionalitaet.

use super::{Connection, MapMarker, MapNode};
use glam::Vec2;
use serde::{Deserialize, Serialize};

/// Repraesentiert einen extrahierten Teil des Graphen.
///
/// Dieser Struct ist fuer die Serialisierung (Clipboard) optimiert
/// und enthaelt alle Daten, die fuer ein Paste-Event notwendig sind.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SubGraph {
    /// Die enthaltenen Wegpunkte
    pub nodes: Vec<MapNode>,
    /// Die Verbindungen zwischen den Wegpunkten im Subgraph
    pub connections: Vec<Connection>,
    /// Map-Marker, die auf Nodes in diesem Subgraph zeigen
    pub markers: Vec<MapMarker>,
}

impl SubGraph {
    /// Erstellt einen leeren SubGraph
    pub fn empty() -> Self {
        Self {
            nodes: Vec::new(),
            connections: Vec::new(),
            markers: Vec::new(),
        }
    }

    /// Verschiebt alle Nodes im Subgraph um einen Offset.
    pub fn translate(&mut self, offset: Vec2) {
        for node in &mut self.nodes {
            node.position += offset;
        }
        // Connection-Geometrie (midpoint/angle) muss nach dem Paste in der RoadMap
        // neu berechnet werden, da sie von den finalen Node-Positionen abhaengt.
    }
}
