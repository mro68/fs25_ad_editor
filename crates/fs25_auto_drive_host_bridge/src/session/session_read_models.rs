//! Read-Model-/JSON-Gruppe der `HostBridgeSession`: getypte und JSON-
//! serialisierte Node-/Marker-/Connection-Reads sowie der Inspected-Node-State.
//! Reine interne Aufteilung — die oeffentliche Session-Surface bleibt unveraendert.

use super::HostBridgeSession;
use crate::dto::{HostConnectionPairSnapshot, HostMarkerListSnapshot, HostNodeDetails};

impl HostBridgeSession {
    /// Liefert die Details eines Nodes als getypten Rust-Struct.
    ///
    /// Die Methode ist ein reiner Read ohne JSON-Serialisierung und ohne
    /// Seiteneffekt auf `inspected_node_id`.
    pub fn node_details(&self, node_id: u64) -> Option<HostNodeDetails> {
        self.build_node_details_for(node_id)
    }

    /// Liefert die komplette Markerliste als getypten Rust-Struct.
    pub fn marker_list(&self) -> HostMarkerListSnapshot {
        self.build_marker_list_snapshot()
    }

    /// Liefert die Verbindungsdetails zwischen zwei Nodes.
    pub fn connection_pair(&self, node_a: u64, node_b: u64) -> HostConnectionPairSnapshot {
        self.build_connection_pair_snapshot(node_a, node_b)
    }

    /// Serialisiert den aktuell inspizierten Node als JSON fuer Flutter.
    pub fn node_details_json(&self) -> Option<String> {
        let snapshot = self
            .inspected_node_id
            .and_then(|node_id| self.node_details(node_id))?;
        serde_json::to_string(&snapshot).ok()
    }

    /// Serialisiert die aktuelle Marker-Liste als JSON fuer Flutter.
    pub fn marker_list_json(&self) -> String {
        serde_json::to_string(&self.marker_list())
            .unwrap_or_else(|_| "{\"markers\":[],\"groups\":[]}".to_string())
    }

    /// Setzt die aktuell fuer das Properties-Panel inspizierte Node-ID.
    pub fn set_inspected_node_id(&mut self, id: Option<u64>) {
        self.inspected_node_id = id;
    }

    /// Liefert die aktuell fuer das Properties-Panel inspizierte Node-ID.
    pub fn inspected_node_id(&self) -> Option<u64> {
        self.inspected_node_id
    }
}
