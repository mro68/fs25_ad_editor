use super::{
    HostBridgeSession, HostConnectionPairSnapshot, HostMarkerListSnapshot, HostNodeDetails,
};
use crate::dto::{
    HostConnectionPairEntry, HostMarkerInfo, HostNodeFlag, HostNodeMarkerInfo, HostNodeNeighbor,
};
use std::collections::BTreeSet;

impl HostBridgeSession {
    pub(super) fn build_node_details_for(&self, node_id: u64) -> Option<HostNodeDetails> {
        let road_map = self.state.road_map.as_deref()?;
        let node = road_map.node(node_id)?;

        Some(HostNodeDetails {
            id: node.id,
            position: [node.position.x, node.position.y],
            flag: HostNodeFlag::from(&node.flag),
            neighbors: road_map
                .connected_neighbors(node_id)
                .into_iter()
                .map(|neighbor| HostNodeNeighbor {
                    neighbor_id: neighbor.neighbor_id,
                    angle: neighbor.angle,
                    is_outgoing: neighbor.is_outgoing,
                })
                .collect(),
            marker: road_map
                .find_marker_by_node_id(node_id)
                .map(|marker| HostNodeMarkerInfo {
                    name: marker.name.clone(),
                    group: marker.group.clone(),
                }),
        })
    }

    pub(super) fn build_marker_list_snapshot(&self) -> HostMarkerListSnapshot {
        let Some(road_map) = self.state.road_map.as_deref() else {
            return HostMarkerListSnapshot {
                markers: Vec::new(),
                groups: Vec::new(),
            };
        };

        let mut groups = BTreeSet::new();
        let mut markers: Vec<HostMarkerInfo> = road_map
            .map_markers()
            .iter()
            .filter_map(|marker| {
                let node = road_map.node(marker.id)?;
                groups.insert(marker.group.clone());

                Some(HostMarkerInfo {
                    node_id: marker.id,
                    name: marker.name.clone(),
                    group: marker.group.clone(),
                    marker_index: marker.marker_index,
                    is_debug: marker.is_debug,
                    position: [node.position.x, node.position.y],
                })
            })
            .collect();
        markers.sort_by_key(|marker| marker.marker_index);

        HostMarkerListSnapshot {
            markers,
            groups: groups.into_iter().collect(),
        }
    }

    pub(super) fn build_connection_pair_snapshot(
        &self,
        node_a: u64,
        node_b: u64,
    ) -> HostConnectionPairSnapshot {
        let connections = self
            .state
            .road_map
            .as_deref()
            .map(|road_map| {
                road_map
                    .find_connections_between(node_a, node_b)
                    .into_iter()
                    .map(|connection| HostConnectionPairEntry {
                        start_id: connection.start_id,
                        end_id: connection.end_id,
                        direction: super::map_connection_direction(connection.direction),
                        priority: super::map_connection_priority(connection.priority),
                    })
                    .collect()
            })
            .unwrap_or_default();

        HostConnectionPairSnapshot {
            node_a,
            node_b,
            connections,
        }
    }
}
