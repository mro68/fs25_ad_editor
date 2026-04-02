//! Kanonische ToolResult-Bausteine fuer einfache Tool-Topologien.

use crate::app::tools::ToolResult;
use crate::core::{ConnectionDirection, ConnectionPriority, NodeFlag};
use glam::Vec2;

type ToolNodes = Vec<(Vec2, NodeFlag)>;
type ToolInternalConnections = Vec<(usize, usize, ConnectionDirection, ConnectionPriority)>;
type ToolExternalConnections = Vec<(usize, u64, bool, ConnectionDirection, ConnectionPriority)>;
type ToolMarkers = Vec<(usize, String, String)>;

/// Baut ein `ToolResult` mit kanonischen leeren Defaults fuer optionale Sammlungen.
///
/// Geeignet fuer die haeufigen Faelle, in denen ein Tool nur neue Nodes und
/// interne Verbindungen erzeugt. Zusaetzliche Sammlungen werden nur dort
/// explizit gesetzt, wo ein Tool sie fachlich wirklich nutzt.
#[must_use]
pub(crate) struct ToolResultBuilder {
    result: ToolResult,
}

impl ToolResultBuilder {
    /// Initialisiert ein `ToolResult` mit kanonisch leeren optionalen Feldern.
    pub(crate) fn new(new_nodes: ToolNodes, internal_connections: ToolInternalConnections) -> Self {
        Self {
            result: ToolResult {
                new_nodes,
                internal_connections,
                external_connections: Vec::new(),
                markers: Vec::new(),
                nodes_to_remove: Vec::new(),
            },
        }
    }

    /// Setzt die Verbindungen zu existierenden Nodes.
    pub(crate) fn with_external_connections(
        mut self,
        external_connections: ToolExternalConnections,
    ) -> Self {
        self.result.external_connections = external_connections;
        self
    }

    /// Setzt die Marker-Ausgabe fuer Tools mit Marker-Semantik.
    pub(crate) fn with_markers(mut self, markers: ToolMarkers) -> Self {
        self.result.markers = markers;
        self
    }

    /// Setzt die zu entfernenden Nodes fuer Tools mit Ersetzungs-Semantik.
    pub(crate) fn with_nodes_to_remove(mut self, nodes_to_remove: Vec<u64>) -> Self {
        self.result.nodes_to_remove = nodes_to_remove;
        self
    }

    /// Schliesst den Builder ab und gibt das fertige `ToolResult` zurueck.
    pub(crate) fn build(self) -> ToolResult {
        self.result
    }
}

#[cfg(test)]
mod tests {
    use super::ToolResultBuilder;
    use crate::core::{ConnectionDirection, ConnectionPriority, NodeFlag};
    use glam::Vec2;

    #[test]
    fn builder_initialisiert_optionale_tool_result_felder_kononisch_leer() {
        let result = ToolResultBuilder::new(
            vec![(Vec2::ZERO, NodeFlag::Regular)],
            vec![(0, 0, ConnectionDirection::Dual, ConnectionPriority::Regular)],
        )
        .build();

        assert!(result.external_connections.is_empty());
        assert!(result.markers.is_empty());
        assert!(result.nodes_to_remove.is_empty());
    }

    #[test]
    fn builder_setzt_nur_explizit_angeforderte_seitenkanaele() {
        let result = ToolResultBuilder::new(vec![], vec![])
            .with_external_connections(vec![(
                0,
                42,
                true,
                ConnectionDirection::Regular,
                ConnectionPriority::SubPriority,
            )])
            .with_markers(vec![(0, "P1".to_string(), "Parking".to_string())])
            .with_nodes_to_remove(vec![7, 8])
            .build();

        assert_eq!(result.external_connections.len(), 1);
        assert_eq!(result.markers.len(), 1);
        assert_eq!(result.nodes_to_remove, vec![7, 8]);
    }
}
