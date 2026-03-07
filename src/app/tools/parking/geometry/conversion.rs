//! Konvertierung von ParkingLayout zu ToolResult und ToolPreview.

use crate::app::tools::{ToolPreview, ToolResult};
use crate::core::NodeFlag;

use super::ParkingLayout;

/// Konvertiert ein ParkingLayout in ein ToolResult.
pub fn build_parking_result(layout: ParkingLayout) -> ToolResult {
    ToolResult {
        new_nodes: layout
            .nodes
            .into_iter()
            .map(|pos| (pos, NodeFlag::Regular))
            .collect(),
        internal_connections: layout.connections,
        external_connections: vec![],
        markers: layout.markers,
        nodes_to_remove: Vec::new(),
    }
}

/// Konvertiert ein ParkingLayout in eine ToolPreview.
pub fn build_preview(layout: &ParkingLayout) -> ToolPreview {
    ToolPreview {
        nodes: layout.nodes.clone(),
        connections: layout
            .connections
            .iter()
            .map(|&(a, b, _dir, _prio)| (a, b))
            .collect(),
        connection_styles: layout
            .connections
            .iter()
            .map(|&(_a, _b, dir, prio)| (dir, prio))
            .collect(),
    }
}
