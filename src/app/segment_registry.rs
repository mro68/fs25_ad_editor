//! In-Session-Registry aller erstellten Segmente (zum nachträglichen Bearbeiten).
//!
//! Wird **nicht** in den Undo/Redo-Snapshot aufgenommen — die Registry ist
//! transient und gilt nur für die aktuelle Session. Beim Laden einer Datei
//! ist sie leer.
//!
//! Beim Bearbeiten eines Segments werden die zugehörigen Nodes aus der
//! RoadMap gelöscht und das passende Tool mit den gespeicherten Parametern
//! neu geladen.

use crate::app::tools::common::TangentSource;
use crate::app::tools::ToolAnchor;
use crate::core::{ConnectionDirection, ConnectionPriority};
use glam::Vec2;

/// Art des Segments — enthält alle tool-spezifischen Parameter.
#[derive(Debug, Clone)]
pub enum SegmentKind {
    /// Gerade Strecke
    Straight {
        /// Verbindungsrichtung
        direction: ConnectionDirection,
        /// Straßenart
        priority: ConnectionPriority,
        /// Maximaler Abstand zwischen Zwischen-Nodes
        max_segment_length: f32,
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
        /// Verbindungsrichtung
        direction: ConnectionDirection,
        /// Straßenart
        priority: ConnectionPriority,
        /// Maximaler Abstand zwischen Zwischen-Nodes
        max_segment_length: f32,
    },
    /// Quadratische Bézier-Kurve (Grad 2)
    CurveQuad {
        /// Steuerpunkt
        cp1: Vec2,
        /// Verbindungsrichtung
        direction: ConnectionDirection,
        /// Straßenart
        priority: ConnectionPriority,
        /// Maximaler Abstand zwischen Zwischen-Nodes
        max_segment_length: f32,
    },
    /// Catmull-Rom-Spline
    Spline {
        /// Anker-Punkte (alle geklickten Punkte inkl. Start/Ende)
        anchors: Vec<ToolAnchor>,
        /// Quell-Tangente am Startpunkt
        tangent_start: TangentSource,
        /// Quell-Tangente am Endpunkt
        tangent_end: TangentSource,
        /// Verbindungsrichtung
        direction: ConnectionDirection,
        /// Straßenart
        priority: ConnectionPriority,
        /// Maximaler Abstand zwischen Zwischen-Nodes
        max_segment_length: f32,
    },
}

impl SegmentKind {
    /// Gibt den Tool-Index im ToolManager für dieses Segment zurück.
    ///
    /// Muss mit der Registrierungsreihenfolge in `ToolManager::new()` übereinstimmen:
    /// 0 = StraightLineTool, 1 = CurveTool(Quadratic), 2 = CurveTool(Cubic), 3 = SplineTool
    pub fn tool_index(&self) -> usize {
        match self {
            SegmentKind::Straight { .. } => 0,
            SegmentKind::CurveQuad { .. } => 1,
            SegmentKind::CurveCubic { .. } => 2,
            SegmentKind::Spline { .. } => 3,
        }
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
}

/// In-Session-Registry aller erstellten Segmente.
///
/// Ermöglicht das nachträgliche Bearbeiten von Segmenten, indem die
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

    /// Registriert ein neues Segment und gibt die vergebene ID zurück.
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

    /// Gibt den Record mit der angegebenen ID zurück (falls vorhanden).
    pub fn get(&self, record_id: u64) -> Option<&SegmentRecord> {
        self.records.iter().find(|r| r.id == record_id)
    }

    /// Entfernt den Record mit der angegebenen ID.
    pub fn remove(&mut self, record_id: u64) {
        self.records.retain(|r| r.id != record_id);
    }

    /// Gibt alle Records zurück, die mindestens einen der angegebenen Node-IDs enthalten.
    pub fn find_by_node_ids(&self, node_ids: &[u64]) -> Vec<&SegmentRecord> {
        let id_set: std::collections::HashSet<u64> =
            node_ids.iter().copied().collect();
        self.records
            .iter()
            .filter(|r| r.node_ids.iter().any(|nid| id_set.contains(nid)))
            .collect()
    }

    /// Entfernt alle Records, die mindestens einen der angegebenen Node-IDs enthalten.
    ///
    /// Wird aufgerufen wenn Nodes manuell gelöscht werden (z.B. Delete-Taste).
    pub fn invalidate_by_node_ids(&mut self, node_ids: &[u64]) {
        let id_set: std::collections::HashSet<u64> =
            node_ids.iter().copied().collect();
        self.records
            .retain(|r| !r.node_ids.iter().any(|nid| id_set.contains(nid)));
    }

    /// Gibt die Anzahl der gespeicherten Records zurück.
    pub fn len(&self) -> usize {
        self.records.len()
    }

    /// Gibt zurück ob die Registry leer ist.
    pub fn is_empty(&self) -> bool {
        self.records.is_empty()
    }
}
