//! Preview- und Execute-Aufbereitung fuer das ColorPathTool.

use glam::Vec2;

use crate::app::tools::{ToolAnchor, ToolPreview, ToolResult};
use crate::core::{ConnectionDirection, ConnectionPriority, NodeFlag, RoadMap};

use super::skeleton::SkeletonGraphNodeKind;
use super::state::{ColorPathPhase, ColorPathTool, ExistingConnectionMode};

impl ColorPathTool {
    /// Kennzahlen fuer die Sidebar-Vorschau.
    pub(super) fn preview_stats(&self) -> (usize, usize, usize) {
        let Some(preview_data) = &self.preview_data else {
            return (0, 0, 0);
        };
        (
            preview_data.network.junction_count(),
            preview_data.network.open_end_count(),
            preview_data.prepared_segments.len(),
        )
    }

    /// Anzahl sichtbarer Preview-Nodes inklusive Segment-Zwischenpunkte.
    pub(super) fn preview_node_count(&self) -> usize {
        let Some(preview_data) = &self.preview_data else {
            return 0;
        };

        let intermediate_count: usize = preview_data
            .prepared_segments
            .iter()
            .map(|segment| segment.resampled_nodes.len().saturating_sub(2))
            .sum();
        preview_data.network.nodes.len() + intermediate_count
    }

    /// Baut die Vorschau fuer die Sampling-Phase.
    pub(super) fn build_sampling_preview(&self) -> ToolPreview {
        let mut nodes: Vec<Vec2> = Vec::new();
        let mut connections: Vec<(usize, usize)> = Vec::new();

        for polygon in &self.sampling.lasso_regions {
            if polygon.len() < 2 {
                continue;
            }
            let start = nodes.len();
            nodes.extend_from_slice(polygon);
            let n = polygon.len();
            for i in 0..n {
                connections.push((start + i, start + (i + 1) % n));
            }
        }

        if let Some(sampling_preview) = &self.sampling_preview {
            for &(start, end) in &sampling_preview.boundary_segments {
                let base = nodes.len();
                nodes.push(start);
                nodes.push(end);
                connections.push((base, base + 1));
            }
        }

        let cn = connections.len();
        ToolPreview {
            nodes,
            connections,
            connection_styles: vec![(ConnectionDirection::Dual, ConnectionPriority::Regular); cn],
            labels: vec![],
        }
    }

    /// Baut die Vorschau fuer die Preview-Phase als Netz aus PreparedSegments.
    pub(super) fn build_network_preview(&self) -> ToolPreview {
        let Some(preview_data) = &self.preview_data else {
            return ToolPreview::default();
        };
        if preview_data.prepared_segments.is_empty() {
            return ToolPreview::default();
        }

        let mut nodes: Vec<Vec2> = preview_data
            .network
            .nodes
            .iter()
            .map(|node| node.world_position)
            .collect();
        let mut connections = Vec::new();
        let mut connection_styles = Vec::new();

        for segment in &preview_data.prepared_segments {
            if segment.resampled_nodes.len() < 2 {
                continue;
            }

            let mut chain = Vec::with_capacity(segment.resampled_nodes.len());
            chain.push(segment.start_node);

            for &pos in segment
                .resampled_nodes
                .iter()
                .skip(1)
                .take(segment.resampled_nodes.len().saturating_sub(2))
            {
                nodes.push(pos);
                chain.push(nodes.len() - 1);
            }

            chain.push(segment.end_node);
            if chain.len() < 2 || (chain.len() == 2 && chain[0] == chain[1]) {
                continue;
            }

            for edge in chain.windows(2) {
                connections.push((edge[0], edge[1]));
                connection_styles.push((self.direction, self.priority));
            }
        }

        ToolPreview {
            nodes,
            connections,
            connection_styles,
            labels: self.build_junction_labels(),
        }
    }

    /// Erzeugt Hinweis-Labels fuer die Junction-Knoten im Preview.
    ///
    /// Wird nur in [`ColorPathPhase::JunctionEdit`] befuellt und markiert echte
    /// Junction-Knoten (kein `OpenEnd`/`LoopAnchor`) mit einem Raute-Symbol,
    /// damit der User sie als per Drag verschiebbare Griffe wahrnimmt (CP-08).
    fn build_junction_labels(&self) -> Vec<(usize, String)> {
        if self.phase != ColorPathPhase::JunctionEdit {
            return Vec::new();
        }
        let Some(preview_data) = &self.preview_data else {
            return Vec::new();
        };
        preview_data
            .network
            .nodes
            .iter()
            .enumerate()
            .filter(|(_, node)| node.kind == SkeletonGraphNodeKind::Junction)
            .map(|(idx, _)| (idx, "◆".to_string()))
            .collect()
    }

    /// Konvertiert Stage F und Bestands-Snaps in ein ausfuehrbares `ToolResult`.
    pub(super) fn execute_result(&self, road_map: &RoadMap) -> Option<ToolResult> {
        let preview_data = self.preview_data.as_ref()?;
        if preview_data.prepared_segments.is_empty() {
            return None;
        }

        let mut new_nodes: Vec<(Vec2, NodeFlag)> = preview_data
            .network
            .nodes
            .iter()
            .map(|node| (node.world_position, NodeFlag::Regular))
            .collect();

        let mut internal_connections = Vec::new();
        for segment in &preview_data.prepared_segments {
            if segment.resampled_nodes.len() < 2 {
                continue;
            }

            let mut chain = Vec::with_capacity(segment.resampled_nodes.len());
            chain.push(segment.start_node);
            for &pos in segment
                .resampled_nodes
                .iter()
                .skip(1)
                .take(segment.resampled_nodes.len().saturating_sub(2))
            {
                let idx = new_nodes.len();
                new_nodes.push((pos, NodeFlag::Regular));
                chain.push(idx);
            }
            chain.push(segment.end_node);

            if chain.len() < 2 || (chain.len() == 2 && chain[0] == chain[1]) {
                continue;
            }

            for edge in chain.windows(2) {
                internal_connections.push((edge[0], edge[1], self.direction, self.priority));
            }
        }

        if internal_connections.is_empty() {
            return None;
        }

        let mut external_connections = Vec::new();
        for (node_index, node) in preview_data.network.nodes.iter().enumerate() {
            if !self.should_connect_node(node.kind) {
                continue;
            }

            let ToolAnchor::ExistingNode(existing_id, _) =
                self.lifecycle.snap_at(node.world_position, road_map)
            else {
                continue;
            };

            let (existing_to_new, direction) = self.external_connection_spec(node_index);
            external_connections.push((
                node_index,
                existing_id,
                existing_to_new,
                direction,
                self.priority,
            ));
        }

        Some(ToolResult {
            new_nodes,
            internal_connections,
            external_connections,
            markers: Vec::new(),
            nodes_to_remove: Vec::new(),
        })
    }

    /// Prueft ob ein Netz-Knoten nach aktuellem Modus an Bestand angeschlossen werden darf.
    fn should_connect_node(&self, kind: SkeletonGraphNodeKind) -> bool {
        match self.config.existing_connection_mode {
            ExistingConnectionMode::Never => false,
            ExistingConnectionMode::OpenEnds => kind == SkeletonGraphNodeKind::OpenEnd,
            ExistingConnectionMode::OpenEndsAndJunctions => {
                matches!(
                    kind,
                    SkeletonGraphNodeKind::OpenEnd | SkeletonGraphNodeKind::Junction
                )
            }
        }
    }

    /// Bestimmt die Anschlussrichtung eines externen Bestands-Snaps.
    fn external_connection_spec(&self, node_index: usize) -> (bool, ConnectionDirection) {
        let Some(preview_data) = &self.preview_data else {
            return (true, self.direction);
        };

        let mut has_outgoing = false;
        let mut has_incoming = false;

        for segment in &preview_data.prepared_segments {
            if segment.start_node == node_index {
                has_outgoing = true;
            }
            if segment.end_node == node_index {
                has_incoming = true;
            }
        }

        let existing_to_new = match (has_outgoing, has_incoming) {
            (true, false) => true,
            (false, true) => false,
            _ => true,
        };
        let direction = if has_outgoing && has_incoming {
            ConnectionDirection::Dual
        } else {
            self.direction
        };

        (existing_to_new, direction)
    }
}
