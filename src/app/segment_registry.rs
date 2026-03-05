//! In-Session-Registry aller erstellten Segmente (zum nachtraeglichen Bearbeiten).
//!
//! Wird **nicht** in den Undo/Redo-Snapshot aufgenommen — die Registry ist
//! transient und gilt nur fuer die aktuelle Session. Beim Laden einer Datei
//! ist sie leer.
//!
//! Beim Bearbeiten eines Segments werden die zugehoerigen Nodes aus der
//! RoadMap geloescht und das passende Tool mit den gespeicherten Parametern
//! neu geladen.

use crate::app::tools::common::TangentSource;
use crate::app::tools::parking::ParkingConfig;
use crate::app::tools::ToolAnchor;
use crate::core::{ConnectionDirection, ConnectionPriority, RoadMap};
use glam::Vec2;

/// Gemeinsame Segment-Parameter aller Route-Tools.
#[derive(Debug, Clone)]
pub struct SegmentBase {
    /// Verbindungsrichtung
    pub direction: ConnectionDirection,
    /// Strassenart
    pub priority: ConnectionPriority,
    /// Maximaler Abstand zwischen Zwischen-Nodes
    pub max_segment_length: f32,
}

/// Art des Segments — enthaelt alle tool-spezifischen Parameter.
#[derive(Debug, Clone)]
pub enum SegmentKind {
    /// Gerade Strecke
    Straight {
        /// Gemeinsame Basis-Parameter
        base: SegmentBase,
    },
    /// Kubische Bézier-Kurve (Grad 3)
    CurveCubic {
        /// Erster Steuerpunkt
        cp1: Vec2,
        /// Zweiter Steuerpunkt
        cp2: Vec2,
        /// Quell-Tangente am Startpunkt
        tangent_start: TangentSource,
        /// Quell-Tangente am Endpunkt
        tangent_end: TangentSource,
        /// Gemeinsame Basis-Parameter
        base: SegmentBase,
    },
    /// Quadratische Bézier-Kurve (Grad 2)
    CurveQuad {
        /// Steuerpunkt
        cp1: Vec2,
        /// Gemeinsame Basis-Parameter
        base: SegmentBase,
    },
    /// Catmull-Rom-Spline
    Spline {
        /// Anker-Punkte (alle geklickten Punkte inkl. Start/Ende)
        anchors: Vec<ToolAnchor>,
        /// Quell-Tangente am Startpunkt
        tangent_start: TangentSource,
        /// Quell-Tangente am Endpunkt
        tangent_end: TangentSource,
        /// Gemeinsame Basis-Parameter
        base: SegmentBase,
    },
    /// Constraint-Route (winkelgeglaettet mit automatischen Tangenten)
    ConstraintRoute {
        /// Zwischen-Kontrollpunkte
        control_nodes: Vec<Vec2>,
        /// Maximale Richtungsaenderung pro Segment (Grad)
        max_angle_deg: f32,
        /// Minimaldistanz-Filter (Meter)
        min_distance: f32,
        /// Gemeinsame Basis-Parameter
        base: SegmentBase,
    },
    /// Ausweichstrecke zur selektierten Kette
    Bypass {
        /// Geordnete Positionen der Quell-Kette
        chain_positions: Vec<Vec2>,
        /// ID des ersten Ketten-Nodes
        chain_start_id: u64,
        /// ID des letzten Ketten-Nodes
        chain_end_id: u64,
        /// Seitlicher Versatz
        offset: f32,
        /// Node-Abstand auf der Bypass-Strecke
        base_spacing: f32,
        /// Gemeinsame Basis-Parameter
        base: SegmentBase,
    },
    /// Parkplatz-Layout (Wendekreis + Parkreihen)
    Parking {
        /// Ursprungspunkt des Layouts
        origin: Vec2,
        /// Rotationswinkel (Radiant)
        angle: f32,
        /// Parkplatz-Konfiguration
        config: ParkingConfig,
        /// Gemeinsame Basis-Parameter
        base: SegmentBase,
    },
}

/// Tool-Index fuer `StraightLineTool` im `ToolManager` (Registrierungs-Slot 0).
///
/// Muss mit der Reihenfolge in `ToolManager::new()` uebereinstimmen.
pub const TOOL_INDEX_STRAIGHT: usize = 0;
/// Tool-Index fuer `CurveTool(Grad 2)` im `ToolManager` (Registrierungs-Slot 1).
pub const TOOL_INDEX_CURVE_QUAD: usize = 1;
/// Tool-Index fuer `CurveTool(Grad 3)` im `ToolManager` (Registrierungs-Slot 2).
pub const TOOL_INDEX_CURVE_CUBIC: usize = 2;
/// Tool-Index fuer `SplineTool` im `ToolManager` (Registrierungs-Slot 3).
pub const TOOL_INDEX_SPLINE: usize = 3;
/// Tool-Index fuer `BypassTool` im `ToolManager` (Registrierungs-Slot 4).
pub const TOOL_INDEX_BYPASS: usize = 4;
/// Tool-Index fuer `ConstraintRouteTool` im `ToolManager` (Registrierungs-Slot 5).
pub const TOOL_INDEX_CONSTRAINT_ROUTE: usize = 5;
/// Tool-Index fuer `ParkingTool` im `ToolManager` (Registrierungs-Slot 6).
pub const TOOL_INDEX_PARKING: usize = 6;

impl SegmentKind {
    /// Gibt den Tool-Index im ToolManager fuer dieses Segment zurueck.
    ///
    /// Muss mit der Registrierungsreihenfolge in `ToolManager::new()` uebereinstimmen —
    /// abgesichert durch den Unit-Test `tool_index_stimmt_mit_tool_manager_reihenfolge_ueberein`.
    pub fn tool_index(&self) -> usize {
        match self {
            SegmentKind::Straight { .. } => TOOL_INDEX_STRAIGHT,
            SegmentKind::CurveQuad { .. } => TOOL_INDEX_CURVE_QUAD,
            SegmentKind::CurveCubic { .. } => TOOL_INDEX_CURVE_CUBIC,
            SegmentKind::Spline { .. } => TOOL_INDEX_SPLINE,
            SegmentKind::ConstraintRoute { .. } => TOOL_INDEX_CONSTRAINT_ROUTE,
            SegmentKind::Bypass { .. } => TOOL_INDEX_BYPASS,
            SegmentKind::Parking { .. } => TOOL_INDEX_PARKING,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::app::tools::ToolManager;

    /// Stellt sicher, dass `tool_index()` mit der Registrierungsreihenfolge
    /// in `ToolManager::new()` uebereinstimmt. Bricht sofort beim Umbenennen
    /// oder Umsortieren der Tools.
    #[test]
    fn tool_index_stimmt_mit_tool_manager_reihenfolge_ueberein() {
        let manager = ToolManager::new();
        let names: Vec<&str> = manager.tool_names().into_iter().map(|(_, n)| n).collect();
        assert_eq!(
            names[TOOL_INDEX_STRAIGHT], "Gerade Strecke",
            "TOOL_INDEX_STRAIGHT zeigt nicht auf StraightLineTool"
        );
        assert_eq!(
            names[TOOL_INDEX_CURVE_QUAD], "Bézier Grad 2",
            "TOOL_INDEX_CURVE_QUAD zeigt nicht auf CurveTool(Grad 2)"
        );
        assert_eq!(
            names[TOOL_INDEX_CURVE_CUBIC], "Bézier Grad 3",
            "TOOL_INDEX_CURVE_CUBIC zeigt nicht auf CurveTool(Grad 3)"
        );
        assert_eq!(
            names[TOOL_INDEX_SPLINE], "Spline",
            "TOOL_INDEX_SPLINE zeigt nicht auf SplineTool"
        );
        assert_eq!(
            names[TOOL_INDEX_CONSTRAINT_ROUTE], "Constraint-Route",
            "TOOL_INDEX_CONSTRAINT_ROUTE zeigt nicht auf ConstraintRouteTool"
        );
        assert_eq!(
            names[TOOL_INDEX_BYPASS], "Ausweichstrecke",
            "TOOL_INDEX_BYPASS zeigt nicht auf BypassTool"
        );
        assert_eq!(
            names[TOOL_INDEX_PARKING], "Parkplatz",
            "TOOL_INDEX_PARKING zeigt nicht auf ParkingTool"
        );
    }
}

/// Ein gespeichertes Segment (fertig erstellte Line).
#[derive(Debug, Clone)]
pub struct SegmentRecord {
    /// Eindeutige Registry-ID (nicht identisch mit Node-IDs)
    pub id: u64,
    /// IDs aller neu erstellten Nodes dieses Segments
    pub node_ids: Vec<u64>,
    /// Start-Anker (ExistingNode oder NewPosition)
    pub start_anchor: ToolAnchor,
    /// End-Anker (ExistingNode oder NewPosition)
    pub end_anchor: ToolAnchor,
    /// Tool-spezifische Parameter
    pub kind: SegmentKind,
    /// Original-Positionen der Nodes zum Zeitpunkt der Erstellung.
    /// Index-Reihenfolge entspricht `node_ids`; wird fuer Validitaetsprüfung genutzt.
    pub original_positions: Vec<Vec2>,
    /// IDs der Nodes mit Map-Markern (fuer Cleanup bei Edit).
    /// Leer bei Tools ohne Marker.
    pub marker_node_ids: Vec<u64>,
}

/// In-Session-Registry aller erstellten Segmente.
///
/// Ermoeglicht das nachtraegliche Bearbeiten von Segmenten, indem die
/// Tool-Parameter beim Erstellen gespeichert und beim Bearbeiten
/// wiederhergestellt werden.
#[derive(Debug, Clone, Default)]
pub struct SegmentRegistry {
    records: Vec<SegmentRecord>,
    next_id: u64,
}

impl SegmentRegistry {
    /// Erstellt eine leere Registry.
    pub fn new() -> Self {
        Self::default()
    }

    /// Registriert ein neues Segment und gibt die vergebene ID zurueck.
    pub fn register(&mut self, record: SegmentRecord) -> u64 {
        let id = record.id;
        self.records.push(record);
        id
    }

    /// Erstellt eine neue Record-ID (auto-increment).
    pub fn next_id(&mut self) -> u64 {
        let id = self.next_id;
        self.next_id += 1;
        id
    }

    /// Gibt den Record mit der angegebenen ID zurueck (falls vorhanden).
    pub fn get(&self, record_id: u64) -> Option<&SegmentRecord> {
        self.records.iter().find(|r| r.id == record_id)
    }

    /// Entfernt den Record mit der angegebenen ID.
    pub fn remove(&mut self, record_id: u64) {
        self.records.retain(|r| r.id != record_id);
    }

    /// Gibt alle Records zurueck, die mindestens einen der angegebenen Node-IDs enthalten.
    pub fn find_by_node_ids(&self, node_ids: &indexmap::IndexSet<u64>) -> Vec<&SegmentRecord> {
        self.records
            .iter()
            .filter(|r| r.node_ids.iter().any(|nid| node_ids.contains(nid)))
            .collect()
    }

    /// Entfernt alle Records, die mindestens einen der angegebenen Node-IDs enthalten.
    ///
    /// Wird aufgerufen wenn Nodes manuell geloescht werden (z.B. Delete-Taste).
    pub fn invalidate_by_node_ids(&mut self, node_ids: &[u64]) {
        let id_set: std::collections::HashSet<u64> = node_ids.iter().copied().collect();
        self.records
            .retain(|r| !r.node_ids.iter().any(|nid| id_set.contains(nid)));
    }

    /// Findet den ersten Record, der den angegebenen Node enthaelt.
    pub fn find_first_by_node_id(&self, node_id: u64) -> Option<&SegmentRecord> {
        self.records.iter().find(|r| r.node_ids.contains(&node_id))
    }

    /// Prueft ob ein Segment noch gueltig ist (Nodes existieren und Positionen unveraendert).
    pub fn is_segment_valid(&self, record: &SegmentRecord, road_map: &RoadMap) -> bool {
        if record.original_positions.len() != record.node_ids.len() {
            return false;
        }
        record
            .node_ids
            .iter()
            .zip(record.original_positions.iter())
            .all(|(id, orig_pos)| {
                road_map
                    .nodes
                    .get(id)
                    .map(|node| node.position.distance(*orig_pos) < 0.01)
                    .unwrap_or(false)
            })
    }

    /// Gibt die Anzahl der gespeicherten Records zurueck.
    pub fn len(&self) -> usize {
        self.records.len()
    }

    /// Gibt zurueck ob die Registry leer ist.
    pub fn is_empty(&self) -> bool {
        self.records.is_empty()
    }
}
