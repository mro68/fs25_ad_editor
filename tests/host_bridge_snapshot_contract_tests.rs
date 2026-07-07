//! Snapshot-Konsistenztest ueber die Host-Bridge-Session-Seams.
//!
//! Prueft, dass die bridge-owned, serialisierbaren Snapshot-DTOs einen
//! stabilen JSON-Rundlauf (Serialize → Deserialize) ueberstehen. Das ist der
//! Vertrag, auf den sich Flutter-/FFI-Hosts verlassen: Ein einmal erzeugter
//! Snapshot muss verlustfrei ueber JSON transportierbar sein.

use fs25_auto_drive_host_bridge::{
    viewport_overlay_snapshot_json, HostBridgeSession, HostChromeSnapshot, HostContextMenuSnapshot,
    HostDialogSnapshot, HostEditingSnapshot, HostRouteToolViewportSnapshot, HostSessionSnapshot,
};

fn roundtrip<T>(value: &T)
where
    T: serde::Serialize + serde::de::DeserializeOwned + PartialEq + std::fmt::Debug,
{
    let json = serde_json::to_string(value).expect("Snapshot muss als JSON serialisierbar sein");
    let restored: T =
        serde_json::from_str(&json).expect("Snapshot-JSON muss wieder deserialisierbar sein");
    assert_eq!(
        value, &restored,
        "Snapshot muss nach JSON-Rundlauf unveraendert bleiben"
    );
}

/// Variante fuer Snapshot-Typen ohne `PartialEq` (z. B. `HostChromeSnapshot`):
/// prueft nur, dass Serialisierung und Deserialisierung erfolgreich sind.
fn roundtrip_deserializes<T>(value: &T)
where
    T: serde::Serialize + serde::de::DeserializeOwned + std::fmt::Debug,
{
    let json = serde_json::to_string(value).expect("Snapshot muss als JSON serialisierbar sein");
    let _restored: T =
        serde_json::from_str(&json).expect("Snapshot-JSON muss wieder deserialisierbar sein");
}

#[test]
fn host_session_snapshot_survives_json_roundtrip() {
    let mut session = HostBridgeSession::new();
    let snapshot: HostSessionSnapshot = session.snapshot().clone();
    roundtrip(&snapshot);
}

#[test]
fn host_dialog_snapshot_survives_json_roundtrip() {
    let session = HostBridgeSession::new();
    let snapshot: HostDialogSnapshot = session.dialog_snapshot();
    roundtrip(&snapshot);
}

#[test]
fn host_editing_snapshot_survives_json_roundtrip() {
    let session = HostBridgeSession::new();
    let snapshot: HostEditingSnapshot = session.editing_snapshot();
    roundtrip(&snapshot);
}

#[test]
fn host_context_menu_snapshot_survives_json_roundtrip() {
    let session = HostBridgeSession::new();
    let snapshot: HostContextMenuSnapshot = session.context_menu_snapshot(None);
    roundtrip(&snapshot);
}

#[test]
fn host_chrome_snapshot_survives_json_roundtrip() {
    let session = HostBridgeSession::new();
    let snapshot: HostChromeSnapshot = session.build_host_chrome_snapshot();
    roundtrip_deserializes(&snapshot);
}

#[test]
fn host_route_tool_viewport_snapshot_survives_json_roundtrip() {
    let session = HostBridgeSession::new();
    let snapshot: HostRouteToolViewportSnapshot = session.build_route_tool_viewport_snapshot();
    roundtrip(&snapshot);
}

/// `ViewportOverlaySnapshot` (engine-seitig, nicht bridge-owned) hat bewusst
/// keine `Deserialize`-Implementierung — die Bridge stellt dafuer den
/// expliziten Einweg-JSON-Helfer `viewport_overlay_snapshot_json(...)`
/// bereit. Dieser Test stellt sicher, dass dieser Helfer weiterhin gueltiges,
/// parsebares JSON liefert (struktureller Kontrakt statt Rundlauf-Gleichheit).
#[test]
fn viewport_overlay_snapshot_json_helper_produces_valid_json() {
    let mut session = HostBridgeSession::new();
    let snapshot = session.build_viewport_overlay_snapshot(None);

    let json =
        viewport_overlay_snapshot_json(&snapshot).expect("Overlay-Snapshot muss JSON liefern");
    let parsed: serde_json::Value =
        serde_json::from_str(&json).expect("Overlay-Snapshot-JSON muss gueltiges JSON sein");
    assert!(
        parsed.is_object(),
        "Overlay-Snapshot-JSON muss ein Objekt sein"
    );
}
