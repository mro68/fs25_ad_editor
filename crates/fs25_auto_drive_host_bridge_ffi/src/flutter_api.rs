//! Flutter Control-Plane API — Dart-seitig erreichbare Rust-Funktionen.
//!
//! Dieses Modul definiert die High-Level-Control-Plane fuer die Flutter-Integration.
//! Alle Funktionen sind sicher (kein unsafe), typsicher und sollen von
//! `flutter_rust_bridge`-Codegen zu Dart-Bindings verarbeitet werden.
//!
//! # Verwendung
//! Funktionen werden via `flutter_rust_bridge`-Codegen als Dart-Futures exportiert.
//! Das Codegen wird durch `build.rs` unter dem `flutter`-Feature ausgeloest.
//!
//! # TODO(flutter-codegen)
//! `#[flutter_rust_bridge::frb]`-Annotationen hinzufuegen sobald das Dart-Codegen
//! in den Build-Prozess integriert ist.

use anyhow::Result;
use fs25_auto_drive_host_bridge::{HostBridgeSession, HostSessionAction};
use std::sync::{Arc, Mutex};

/// Opaquer Session-Handle fuer die Flutter Control-Plane.
///
/// Der Handle kapselt eine `HostBridgeSession` hinter `Arc<Mutex<...>>`, damit
/// Flutter-Control-Plane und weitere Runtime-Adapter denselben Session-Besitz
/// thread-sicher teilen koennen. Dart-seitig wird dieser Handle als opaker
/// Zeiger (via flutter_rust_bridge `RustOpaque`) verwaltet.
pub struct FlutterSessionHandle {
    session: Arc<Mutex<HostBridgeSession>>,
}

impl FlutterSessionHandle {
    fn with_session<T>(&self, f: impl FnOnce(&mut HostBridgeSession) -> T) -> Result<T> {
        let mut guard = self
            .session
            .lock()
            .map_err(|_| anyhow::anyhow!("flutter session lock poisoned"))?;
        Ok(f(&mut guard))
    }

    #[allow(dead_code)]
    pub(crate) fn session_arc(&self) -> Arc<Mutex<HostBridgeSession>> {
        Arc::clone(&self.session)
    }
}

/// Erzeugt eine neue Flutter-Session.
///
/// Gibt einen opaques Handle-Pointer zurueck der fuer alle weiteren API-Aufrufe
/// benoetigt wird. Dart ist verantwortlich `flutter_session_dispose` aufzurufen.
pub fn flutter_session_new() -> Box<FlutterSessionHandle> {
    Box::new(FlutterSessionHandle {
        session: Arc::new(Mutex::new(HostBridgeSession::new())),
    })
}

/// Gibt eine Flutter-Session frei.
///
/// Nach diesem Aufruf darf `handle` nicht mehr verwendet werden.
pub fn flutter_session_dispose(_handle: Box<FlutterSessionHandle>) {
    // Handle wird durch Drop freigegeben
}

/// Wendet eine serialisierte Session-Action an.
///
/// `action_json` muss ein gueltiger JSON-String sein der eine [`HostSessionAction`] repraesentiert.
///
/// # Fehler
/// Gibt einen Fehler-String zurueck wenn JSON ungueltig oder die Aktion fehlschlaegt.
pub fn flutter_session_apply_action(
    handle: &FlutterSessionHandle,
    action_json: String,
) -> Result<()> {
    let action: HostSessionAction = serde_json::from_str(&action_json)
        .map_err(|e| anyhow::anyhow!("flutter_session_apply_action: JSON-Fehler: {e}"))?;
    handle.with_session(|s| s.apply_action(action))?
}

/// Gibt den aktuellen Session-Snapshot als JSON-String zurueck.
///
/// Der Snapshot enthaelt Node-Count, Selections, Status-Message etc.
///
/// # Fehler
/// Gibt einen Fehler-String zurueck wenn die JSON-Serialisierung fehlschlaegt.
pub fn flutter_session_snapshot_json(handle: &FlutterSessionHandle) -> Result<String> {
    let snapshot = handle.with_session(|s| s.snapshot().clone())?;
    serde_json::to_string(&snapshot)
        .map_err(|e| anyhow::anyhow!("flutter_session_snapshot_json: Serialisierungsfehler: {e}"))
}

/// Liefert den Viewport-Geometrie-Snapshot als JSON-String.
///
/// Enthaelt alle sichtbaren Nodes, Connections und Markers mit Positionen.
///
/// # Fehler
/// Gibt einen Fehler-String zurueck wenn die JSON-Serialisierung fehlschlaegt.
pub fn flutter_session_viewport_geometry_json(
    handle: &FlutterSessionHandle,
    viewport_width: f32,
    viewport_height: f32,
) -> Result<String> {
    let snapshot = handle
        .with_session(|s| s.build_viewport_geometry_snapshot([viewport_width, viewport_height]))?;
    serde_json::to_string(&snapshot).map_err(|e| {
        anyhow::anyhow!("flutter_session_viewport_geometry_json: Serialisierungsfehler: {e}")
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    /// Prueft, dass FlutterSessionHandle erzeugt und freigegeben werden kann.
    #[test]
    fn test_flutter_session_lifecycle() {
        let handle = flutter_session_new();
        flutter_session_dispose(handle);
    }

    /// Prueft, dass ungueliges JSON einen Fehler zurueckgibt (kein Panic).
    #[test]
    fn test_flutter_session_apply_action_rejects_invalid_json() {
        let handle = flutter_session_new();
        let result = flutter_session_apply_action(&handle, "not json".to_string());
        assert!(result.is_err(), "Ungueltiges JSON muss Err zurueckgeben");
        flutter_session_dispose(handle);
    }

    /// Prueft, dass snapshot_json einen validen JSON-String liefert.
    #[test]
    fn test_flutter_session_snapshot_json_roundtrip() {
        let handle = flutter_session_new();
        let json =
            flutter_session_snapshot_json(&handle).expect("Snapshot-Serialisierung muss gelingen");
        assert!(!json.is_empty());
        assert!(json.starts_with('{'), "Snapshot muss JSON-Objekt sein");
        flutter_session_dispose(handle);
    }
}
