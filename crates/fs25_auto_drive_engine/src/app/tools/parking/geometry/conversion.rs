//! Konvertierung von ParkingLayout zu ToolResult und ToolPreview.

use crate::app::tools::common::ToolResultBuilder;
use crate::app::tools::{ToolPreview, ToolResult};
use crate::core::NodeFlag;

use super::ParkingLayout;

/// Konvertiert ein ParkingLayout in ein ToolResult.
pub fn build_parking_result(layout: ParkingLayout) -> ToolResult {
    let ParkingLayout {
        nodes,
        connections,
        markers,
    } = layout;

    ToolResultBuilder::new(
        nodes
            .into_iter()
            .map(|pos| (pos, NodeFlag::Regular))
            .collect(),
        connections,
    )
    .with_markers(markers)
    .build()
}

/// Konvertiert ein ParkingLayout in eine ToolPreview.
pub fn build_preview(layout: &ParkingLayout) -> ToolPreview {
    let row_count = layout.nodes.len() / 8;
    let mut labels = Vec::with_capacity(row_count * 8);
    for i in 0..row_count {
        let base = i * 8;
        labels.push((base, "n1".to_string()));
        labels.push((base + 1, "n2".to_string()));
        labels.push((base + 2, "n3".to_string()));
        labels.push((base + 3, "n4".to_string()));
        labels.push((base + 4, "n5".to_string()));
        labels.push((base + 5, "n6".to_string()));
        labels.push((base + 6, "n7".to_string()));
        labels.push((base + 7, "n8".to_string()));
    }

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
        labels,
    }
}
