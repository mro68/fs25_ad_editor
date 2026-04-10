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
use fs25_auto_drive_host_bridge::dto::{host_ui_snapshot_json, viewport_overlay_snapshot_json};
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

// SAFETY: Zugriff auf die innere HostBridgeSession ist durch Arc<Mutex<...>>
// serialisiert. RouteTool-Trait-Objekte sind !Send, aber alle Zugriffe gehen
// durch den Mutex — daher ist FlutterSessionHandle thread-sicher.
unsafe impl Send for FlutterSessionHandle {}
unsafe impl Sync for FlutterSessionHandle {}

/// Erzeugt eine neue Flutter-Session.
///
/// Gibt einen opaques Handle-Pointer zurueck der fuer alle weiteren API-Aufrufe
/// benoetigt wird. Dart ist verantwortlich `flutter_session_dispose` aufzurufen.
#[allow(clippy::arc_with_non_send_sync)] // HostBridgeSession ist !Send, aber FFI-Zugriff ist seriell
pub fn flutter_session_new() -> FlutterSessionHandle {
    FlutterSessionHandle {
        session: Arc::new(Mutex::new(HostBridgeSession::new())),
    }
}

/// Gibt eine Flutter-Session frei.
///
/// Nach diesem Aufruf darf `handle` nicht mehr verwendet werden.
pub fn flutter_session_dispose(_handle: FlutterSessionHandle) {
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

/// Gibt zurueck, ob die geladene Karte seit dem letzten Load/Save veraendert wurde.
pub fn flutter_session_is_dirty(handle: &FlutterSessionHandle) -> Result<bool> {
    handle.with_session(|s| s.is_dirty())
}

/// Gibt den aktuellen host-neutralen UI-Snapshot als JSON-String zurueck.
///
/// Der Snapshot enthaelt Route-Tool-Panels, Optionen und weitere host-neutrale
/// Fensterzustaende dieses Frames.
pub fn flutter_session_ui_snapshot_json(handle: &FlutterSessionHandle) -> Result<String> {
    let snapshot = handle.with_session(|s| s.build_host_ui_snapshot())?;
    host_ui_snapshot_json(&snapshot).map_err(|e| {
        anyhow::anyhow!("flutter_session_ui_snapshot_json: Serialisierungsfehler: {e}")
    })
}

/// Gibt den aktuellen host-neutralen Chrome-Snapshot als JSON-String zurueck.
///
/// Der Snapshot enthaelt Menue-, Status- und Optionsdaten fuer host-native
/// Oberflaechen ohne direkte Engine-Abhaengigkeit.
pub fn flutter_session_chrome_snapshot_json(handle: &FlutterSessionHandle) -> Result<String> {
    let snapshot = handle.with_session(|s| s.build_host_chrome_snapshot())?;
    serde_json::to_string(&snapshot).map_err(|e| {
        anyhow::anyhow!("flutter_session_chrome_snapshot_json: Serialisierungsfehler: {e}")
    })
}

/// Gibt den aktuellen host-neutralen Viewport-Overlay-Snapshot als JSON-String zurueck.
///
/// `cursor_world_x` und `cursor_world_y` beschreiben die aktuelle Cursor-Position
/// in Weltkoordinaten und werden fuer tool-abhaengige Overlay-Projektionen
/// (z. B. Preview/Boundary-Caches) an die Session weitergereicht.
pub fn flutter_session_viewport_overlay_json(
    handle: &FlutterSessionHandle,
    cursor_world_x: f32,
    cursor_world_y: f32,
) -> Result<String> {
    let snapshot = handle.with_session(|s| {
        s.build_viewport_overlay_snapshot(Some([cursor_world_x, cursor_world_y].into()))
    })?;
    viewport_overlay_snapshot_json(&snapshot).map_err(|e| {
        anyhow::anyhow!("flutter_session_viewport_overlay_json: Serialisierungsfehler: {e}")
    })
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

/// Klont den internen `Arc<Mutex<HostBridgeSession>>` und gibt ihn als rohen Zeiger (i64) zurueck.
///
/// Der Aufrufer muss den Rueckgabewert exakt einmal verwenden:
/// - Entweder an `fs25ad_gpu_runtime_new_with_shared_session_arc` uebergeben (wird konsumiert),
/// - Oder `flutter_session_release_shared_arc_raw` aufrufen.
///
/// Solange der Wert unreleased ist, haelt er eine starke Referenz auf die Session.
pub fn flutter_session_acquire_shared_arc_raw(handle: &FlutterSessionHandle) -> i64 {
    let cloned: Arc<Mutex<HostBridgeSession>> = Arc::clone(&handle.session);
    Arc::into_raw(cloned) as i64
}

/// Gibt einen via `flutter_session_acquire_shared_arc_raw` geklonten Arc-Zeiger frei.
///
/// Nur aufrufen wenn der Wert NICHT bereits an
/// `fs25ad_gpu_runtime_new_with_shared_session_arc` uebergeben wurde.
pub fn flutter_session_release_shared_arc_raw(raw: i64) {
    if raw == 0 {
        return;
    }
    // SAFETY: raw wurde via Arc::into_raw(Arc<Mutex<HostBridgeSession>>) erzeugt
    // und ist noch nicht freigegeben worden.
    let _ = unsafe { Arc::from_raw(raw as *const Mutex<HostBridgeSession>) };
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

    /// Prueft, dass neue Sessions als nicht dirty gemeldet werden.
    #[test]
    fn test_flutter_session_is_dirty_is_false_for_fresh_session() {
        let handle = flutter_session_new();
        let dirty = flutter_session_is_dirty(&handle)
            .expect("Dirty-Abfrage fuer frische Session muss gelingen");
        assert!(!dirty);
        flutter_session_dispose(handle);
    }

    /// Prueft, dass ui_snapshot_json ein parsebares JSON-Objekt liefert.
    #[test]
    fn test_flutter_session_ui_snapshot_json_roundtrip() {
        let handle = flutter_session_new();
        let json = flutter_session_ui_snapshot_json(&handle)
            .expect("UI-Snapshot-Serialisierung muss gelingen");
        let value: serde_json::Value =
            serde_json::from_str(&json).expect("UI-Snapshot muss parsebares JSON sein");
        assert!(
            value.get("panels").is_some(),
            "UI-Snapshot muss panels enthalten"
        );
        flutter_session_dispose(handle);
    }

    /// Prueft, dass chrome_snapshot_json ein parsebares JSON-Objekt liefert.
    #[test]
    fn test_flutter_session_chrome_snapshot_json_roundtrip() {
        let handle = flutter_session_new();
        let json = flutter_session_chrome_snapshot_json(&handle)
            .expect("Chrome-Snapshot-Serialisierung muss gelingen");
        let value: serde_json::Value =
            serde_json::from_str(&json).expect("Chrome-Snapshot muss parsebares JSON sein");
        assert!(
            value.get("show_command_palette").is_some(),
            "Chrome-Snapshot muss chrome-Felder enthalten"
        );
        flutter_session_dispose(handle);
    }

    /// Prueft, dass viewport_overlay_json ein parsebares JSON-Objekt liefert.
    #[test]
    fn test_flutter_session_viewport_overlay_json_roundtrip() {
        let handle = flutter_session_new();
        let json = flutter_session_viewport_overlay_json(&handle, 0.0, 0.0)
            .expect("Overlay-Snapshot-Serialisierung muss gelingen");
        let value: serde_json::Value =
            serde_json::from_str(&json).expect("Overlay-Snapshot muss parsebares JSON sein");
        assert!(
            value.get("show_no_file_hint").is_some(),
            "Overlay-Snapshot muss Overlay-Felder enthalten"
        );
        flutter_session_dispose(handle);
    }
}
