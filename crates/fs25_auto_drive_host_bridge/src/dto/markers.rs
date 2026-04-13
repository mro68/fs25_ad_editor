//! Marker-Management-DTOs fuer die Flutter-Bridge.

use serde::{Deserialize, Serialize};

/// Vollstaendige Marker-Information fuer Listen- und Detailansichten.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct HostMarkerInfo {
    /// Node-ID, an der der Marker sitzt.
    pub node_id: u64,
    /// Name des Markers.
    pub name: String,
    /// Gruppe des Markers (z. B. `All`, `Feldarbeit`).
    pub group: String,
    /// Marker-Index aus dem XML (`markerIndex`).
    pub marker_index: u32,
    /// Ob es sich um einen Debug-Marker handelt.
    pub is_debug: bool,
    /// Weltposition des zugehoerigen Nodes als `[x, z]`.
    pub position: [f32; 2],
}

/// Snapshot aller Marker fuer das Flutter-Marker-Panel.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct HostMarkerListSnapshot {
    /// Alle Marker, sortiert nach `marker_index`.
    pub markers: Vec<HostMarkerInfo>,
    /// Distincte Gruppennamen fuer ComboBoxen oder Filter.
    pub groups: Vec<String>,
}

#[cfg(test)]
mod tests {
    use serde_json::json;

    use super::{HostMarkerInfo, HostMarkerListSnapshot};
    use crate::dto::HostSessionAction;

    #[test]
    fn host_marker_list_snapshot_roundtrip_json() {
        let snapshot = HostMarkerListSnapshot {
            markers: vec![
                HostMarkerInfo {
                    node_id: 11,
                    name: "Hof".to_string(),
                    group: "All".to_string(),
                    marker_index: 1,
                    is_debug: false,
                    position: [100.0, 200.0],
                },
                HostMarkerInfo {
                    node_id: 13,
                    name: "Feld".to_string(),
                    group: "Feldarbeit".to_string(),
                    marker_index: 2,
                    is_debug: true,
                    position: [125.0, 225.0],
                },
            ],
            groups: vec!["All".to_string(), "Feldarbeit".to_string()],
        };

        let payload = serde_json::to_value(&snapshot)
            .expect("HostMarkerListSnapshot muss als JSON serialisierbar sein");
        let parsed: HostMarkerListSnapshot = serde_json::from_value(payload)
            .expect("HostMarkerListSnapshot muss aus JSON zuruecklesbar sein");

        assert_eq!(parsed, snapshot);
    }

    #[test]
    fn host_session_action_create_marker_roundtrip_json() {
        let action = HostSessionAction::CreateMarker {
            node_id: 9,
            name: "Abladestelle".to_string(),
            group: "All".to_string(),
        };

        let payload =
            serde_json::to_value(&action).expect("CreateMarker muss als JSON serialisierbar sein");
        assert_eq!(
            payload,
            json!({
                "kind": "create_marker",
                "node_id": 9,
                "name": "Abladestelle",
                "group": "All"
            })
        );

        let parsed: HostSessionAction =
            serde_json::from_value(payload).expect("CreateMarker muss aus JSON zuruecklesbar sein");
        assert_eq!(parsed, action);
    }
}
