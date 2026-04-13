//! DTOs fuer Read-Snapshots einzelner Verbindungen zwischen genau zwei Nodes.

use serde::{Deserialize, Serialize};

use super::{HostDefaultConnectionDirection, HostDefaultConnectionPriority};

/// Einzelne Verbindung zwischen zwei Nodes.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct HostConnectionPairEntry {
    /// Start-Node-ID der Verbindung.
    pub start_id: u64,
    /// End-Node-ID der Verbindung.
    pub end_id: u64,
    /// Richtung der Verbindung.
    pub direction: HostDefaultConnectionDirection,
    /// Prioritaet der Verbindung.
    pub priority: HostDefaultConnectionPriority,
}

/// Verbindungsliste zwischen genau zwei Nodes.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct HostConnectionPairSnapshot {
    /// Erste Node-ID des abgefragten Paares.
    pub node_a: u64,
    /// Zweite Node-ID des abgefragten Paares.
    pub node_b: u64,
    /// Alle vorhandenen Verbindungen zwischen den beiden Nodes.
    pub connections: Vec<HostConnectionPairEntry>,
}

#[cfg(test)]
mod tests {
    use serde_json::json;

    use super::{HostConnectionPairEntry, HostConnectionPairSnapshot};
    use crate::dto::{HostDefaultConnectionDirection, HostDefaultConnectionPriority};

    #[test]
    fn host_connection_pair_snapshot_roundtrips_json() {
        let snapshot = HostConnectionPairSnapshot {
            node_a: 10,
            node_b: 20,
            connections: vec![
                HostConnectionPairEntry {
                    start_id: 10,
                    end_id: 20,
                    direction: HostDefaultConnectionDirection::Dual,
                    priority: HostDefaultConnectionPriority::Regular,
                },
                HostConnectionPairEntry {
                    start_id: 20,
                    end_id: 10,
                    direction: HostDefaultConnectionDirection::Reverse,
                    priority: HostDefaultConnectionPriority::SubPriority,
                },
            ],
        };

        let payload = serde_json::to_value(&snapshot)
            .expect("Connection-Pair-Snapshot muss als JSON serialisierbar sein");
        assert_eq!(
            payload,
            json!({
                "node_a": 10,
                "node_b": 20,
                "connections": [
                    {
                        "start_id": 10,
                        "end_id": 20,
                        "direction": "dual",
                        "priority": "regular"
                    },
                    {
                        "start_id": 20,
                        "end_id": 10,
                        "direction": "reverse",
                        "priority": "sub_priority"
                    }
                ]
            })
        );

        let parsed: HostConnectionPairSnapshot = serde_json::from_value(payload)
            .expect("Connection-Pair-Snapshot muss aus JSON zuruecklesbar sein");
        assert_eq!(parsed, snapshot);
    }
}
