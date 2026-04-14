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
use fs25_auto_drive_host_bridge::{HostBridgeSession, HostDialogResult, HostSessionAction};
use std::sync::{Arc, Mutex};

fn decode_focus_node_id(focus_node_id_or_neg1: i64) -> Result<Option<u64>> {
    if focus_node_id_or_neg1 == -1 {
        return Ok(None);
    }

    let node_id = u64::try_from(focus_node_id_or_neg1).map_err(|_| {
        anyhow::anyhow!(
            "flutter_session_context_menu_snapshot_json: ungueltige focus_node_id {}",
            focus_node_id_or_neg1
        )
    })?;
    Ok(Some(node_id))
}

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

/// Entnimmt alle aktuell ausstehenden Dialog-Anforderungen als JSON-Array.
///
/// Der Aufruf drainet die interne Queue der Session und liefert ein JSON-Array
/// aus [`fs25_auto_drive_host_bridge::HostDialogRequest`].
///
/// # Fehler
/// Gibt einen Fehler-String zurueck wenn die Session nicht gelockt werden kann
/// oder die JSON-Serialisierung fehlschlaegt.
pub fn flutter_session_take_dialog_requests_json(handle: &FlutterSessionHandle) -> Result<String> {
    let requests = handle.with_session(|s| s.take_dialog_requests())?;
    serde_json::to_string(&requests).map_err(|e| {
        anyhow::anyhow!("flutter_session_take_dialog_requests_json: Serialisierungsfehler: {e}")
    })
}

/// Reicht ein serialisiertes Dialog-Ergebnis an die Session weiter.
///
/// `result_json` muss ein gueltiger JSON-String sein der ein
/// [`HostDialogResult`] repraesentiert.
///
/// # Fehler
/// Gibt einen Fehler-String zurueck wenn JSON ungueltig oder die Session-
/// Mutation fehlgeschlagen ist.
pub fn flutter_session_submit_dialog_result_json(
    handle: &FlutterSessionHandle,
    result_json: String,
) -> Result<()> {
    let result: HostDialogResult = serde_json::from_str(&result_json).map_err(|e| {
        anyhow::anyhow!("flutter_session_submit_dialog_result_json: JSON-Fehler: {e}")
    })?;
    handle.with_session(|s| s.submit_dialog_result(result))?
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

/// Gibt den aktuell inspizierten Node als JSON-String zurueck.
pub fn flutter_session_node_details_json(handle: &FlutterSessionHandle) -> Option<String> {
    handle
        .with_session(|s| s.node_details_json())
        .ok()
        .flatten()
}

/// Gibt die aktuelle Marker-Liste als JSON-String zurueck.
pub fn flutter_session_marker_list_json(handle: &FlutterSessionHandle) -> String {
    handle
        .with_session(|s| s.marker_list_json())
        .unwrap_or_else(|_| "{\"markers\":[],\"groups\":[]}".to_string())
}

/// Gibt den host-neutralen Route-Tool-Viewport-Snapshot als JSON-String zurueck.
///
/// # Fehler
/// Gibt einen Fehler-String zurueck wenn die JSON-Serialisierung fehlschlaegt.
pub fn flutter_session_route_tool_viewport_json(handle: &FlutterSessionHandle) -> Result<String> {
    let snapshot = handle.with_session(|s| s.build_route_tool_viewport_snapshot())?;
    serde_json::to_string(&snapshot).map_err(|e| {
        anyhow::anyhow!("flutter_session_route_tool_viewport_json: Serialisierungsfehler: {e}")
    })
}

/// Gibt die Verbindungsdetails zwischen zwei Nodes als JSON-String zurueck.
///
/// # Fehler
/// Gibt einen Fehler-String zurueck wenn die JSON-Serialisierung fehlschlaegt.
pub fn flutter_session_connection_pair_json(
    handle: &FlutterSessionHandle,
    node_a: u64,
    node_b: u64,
) -> Result<String> {
    let snapshot = handle.with_session(|s| s.connection_pair(node_a, node_b))?;
    serde_json::to_string(&snapshot).map_err(|e| {
        anyhow::anyhow!("flutter_session_connection_pair_json: Serialisierungsfehler: {e}")
    })
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

/// Gibt den aktuellen host-neutralen Dialog-Snapshot als JSON-String zurueck.
pub fn flutter_session_dialog_snapshot_json(handle: &FlutterSessionHandle) -> Result<String> {
    let snapshot = handle.with_session(|s| s.dialog_snapshot())?;
    serde_json::to_string(&snapshot).map_err(|e| {
        anyhow::anyhow!("flutter_session_dialog_snapshot_json: Serialisierungsfehler: {e}")
    })
}

/// Gibt den aktuellen host-neutralen Editing-Snapshot als JSON-String zurueck.
pub fn flutter_session_editing_snapshot_json(handle: &FlutterSessionHandle) -> Result<String> {
    let snapshot = handle.with_session(|s| s.editing_snapshot())?;
    serde_json::to_string(&snapshot).map_err(|e| {
        anyhow::anyhow!("flutter_session_editing_snapshot_json: Serialisierungsfehler: {e}")
    })
}

/// Gibt den aktuellen host-neutralen Kontextmenue-Snapshot als JSON-String zurueck.
///
/// `focus_node_id_or_neg1` nutzt `-1` als FFI-Sentinel fuer "kein Fokus-Node".
pub fn flutter_session_context_menu_snapshot_json(
    handle: &FlutterSessionHandle,
    focus_node_id_or_neg1: i64,
) -> Result<String> {
    let focus_node_id = decode_focus_node_id(focus_node_id_or_neg1)?;
    let snapshot = handle.with_session(|s| s.context_menu_snapshot(focus_node_id))?;
    serde_json::to_string(&snapshot).map_err(|e| {
        anyhow::anyhow!("flutter_session_context_menu_snapshot_json: Serialisierungsfehler: {e}")
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
    use fs25_auto_drive_host_bridge::{
        HostConnectionPairSnapshot, HostContextMenuSnapshot, HostDialogRequest,
        HostDialogRequestKind, HostDialogResult, HostDialogSnapshot, HostEditingSnapshot,
        HostMarkerListSnapshot, HostRouteToolViewportSnapshot, HostSessionAction,
    };

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

    /// Prueft, dass dialog_snapshot_json ein parsebares JSON-Objekt liefert.
    #[test]
    fn test_flutter_session_dialog_snapshot_json_roundtrip() {
        let handle = flutter_session_new();
        handle
            .with_session(|session| {
                let dialog_state = session.dialog_ui_state_mut();
                dialog_state.ui.show_heightmap_warning = true;
                dialog_state.ui.confirm_dissolve_group_id = Some(21);
            })
            .expect("Lokale Dialog-Mutation fuer den Snapshot-Test muss gelingen");

        let json = flutter_session_dialog_snapshot_json(&handle)
            .expect("Dialog-Snapshot-Serialisierung muss gelingen");
        let snapshot: HostDialogSnapshot =
            serde_json::from_str(&json).expect("Dialog-Snapshot muss parsebares JSON sein");

        assert!(snapshot.heightmap_warning.visible);
        assert_eq!(snapshot.confirm_dissolve_group.segment_id, Some(21));

        flutter_session_dispose(handle);
    }

    /// Prueft, dass editing_snapshot_json ein parsebares JSON-Objekt liefert.
    #[test]
    fn test_flutter_session_editing_snapshot_json_roundtrip() {
        let handle = flutter_session_new();

        let json = flutter_session_editing_snapshot_json(&handle)
            .expect("Editing-Snapshot-Serialisierung muss gelingen");
        let snapshot: HostEditingSnapshot =
            serde_json::from_str(&json).expect("Editing-Snapshot muss parsebares JSON sein");

        assert!(snapshot.editable_groups.is_empty());
        assert!(!snapshot.resample.active);

        flutter_session_dispose(handle);
    }

    /// Prueft, dass context_menu_snapshot_json ein parsebares JSON-Objekt liefert.
    #[test]
    fn test_flutter_session_context_menu_snapshot_json_roundtrip() {
        let handle = flutter_session_new();

        let json = flutter_session_context_menu_snapshot_json(&handle, -1)
            .expect("Kontextmenue-Snapshot-Serialisierung muss gelingen");
        let snapshot: HostContextMenuSnapshot =
            serde_json::from_str(&json).expect("Kontextmenue-Snapshot muss parsebares JSON sein");

        assert_eq!(
            snapshot.variant,
            fs25_auto_drive_host_bridge::HostContextMenuVariant::EmptyArea
        );
        assert!(snapshot.available_actions.is_empty());

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

    /// Prueft, dass QueryNodeDetails ueber die Flutter-Action-Surface gesetzt werden kann.
    #[test]
    fn test_flutter_session_node_details_json_returns_none_without_matching_node() {
        let handle = flutter_session_new();
        let action = serde_json::json!({
            "kind": "query_node_details",
            "node_id": 99
        })
        .to_string();

        flutter_session_apply_action(&handle, action)
            .expect("QueryNodeDetails muss ueber die Flutter-Surface akzeptiert werden");
        assert!(flutter_session_node_details_json(&handle).is_none());

        flutter_session_dispose(handle);
    }

    /// Prueft, dass marker_list_json fuer leere Sessions ein parsebares Snapshot-Objekt liefert.
    #[test]
    fn test_flutter_session_marker_list_json_returns_empty_snapshot() {
        let handle = flutter_session_new();
        let json = flutter_session_marker_list_json(&handle);
        let snapshot: HostMarkerListSnapshot =
            serde_json::from_str(&json).expect("Marker-Liste muss als Snapshot-JSON parsebar sein");

        assert!(snapshot.markers.is_empty());
        assert!(snapshot.groups.is_empty());

        flutter_session_dispose(handle);
    }

    /// Prueft, dass take_dialog_requests_json Dialog-Requests serialisiert und drainet.
    #[test]
    fn test_flutter_session_take_dialog_requests_json_roundtrip() {
        let handle = flutter_session_new();
        let action_json = serde_json::to_string(&HostSessionAction::RequestHeightmapSelection)
            .expect("HostSessionAction muss serialisierbar sein");

        flutter_session_apply_action(&handle, action_json)
            .expect("Heightmap-Dialog-Anforderung muss ueber Flutter funktionieren");

        let json = flutter_session_take_dialog_requests_json(&handle)
            .expect("Dialog-Request-JSON muss serialisiert werden koennen");
        let requests: Vec<HostDialogRequest> =
            serde_json::from_str(&json).expect("Dialog-Request-JSON muss parsebar sein");

        assert_eq!(requests.len(), 1);
        assert_eq!(requests[0].kind, HostDialogRequestKind::Heightmap);

        let drained_json = flutter_session_take_dialog_requests_json(&handle)
            .expect("Gedrainte Queue muss als leeres JSON-Array serialisierbar sein");
        let drained: Vec<HostDialogRequest> =
            serde_json::from_str(&drained_json).expect("Leeres Dialog-Array muss parsebar sein");
        assert!(drained.is_empty());

        flutter_session_dispose(handle);
    }

    /// Prueft, dass submit_dialog_result_json gueltiges JSON akzeptiert.
    #[test]
    fn test_flutter_session_submit_dialog_result_json_accepts_serialized_result() {
        let handle = flutter_session_new();
        let action_json = serde_json::to_string(&HostSessionAction::RequestHeightmapSelection)
            .expect("HostSessionAction muss serialisierbar sein");

        flutter_session_apply_action(&handle, action_json)
            .expect("Heightmap-Dialog-Anforderung muss ueber Flutter funktionieren");
        let _ = flutter_session_take_dialog_requests_json(&handle)
            .expect("Dialog-Request-Drain muss vor dem Submit funktionieren");

        let result_json = serde_json::to_string(&HostDialogResult::PathSelected {
            kind: HostDialogRequestKind::Heightmap,
            path: "/tmp/test_heightmap.png".to_string(),
        })
        .expect("HostDialogResult muss serialisierbar sein");

        flutter_session_submit_dialog_result_json(&handle, result_json)
            .expect("Dialog-Ergebnis muss ueber Flutter akzeptiert werden");

        let snapshot_json = flutter_session_snapshot_json(&handle)
            .expect("Session-Snapshot muss nach Dialog-Submit weiter verfuegbar sein");
        assert!(snapshot_json.starts_with('{'));

        flutter_session_dispose(handle);
    }

    /// Prueft, dass route_tool_viewport_json ein parsebares Snapshot-JSON liefert.
    #[test]
    fn test_flutter_session_route_tool_viewport_json_roundtrip() {
        let handle = flutter_session_new();
        let json = flutter_session_route_tool_viewport_json(&handle)
            .expect("Route-Tool-Snapshot-Serialisierung muss gelingen");
        let snapshot: HostRouteToolViewportSnapshot =
            serde_json::from_str(&json).expect("Route-Tool-Snapshot muss parsebares JSON sein");

        assert!(!snapshot.has_pending_input);
        assert!(snapshot.drag_targets.is_empty());

        flutter_session_dispose(handle);
    }

    /// Prueft, dass connection_pair_json auch fuer leere Sessions stabil serialisiert.
    #[test]
    fn test_flutter_session_connection_pair_json_roundtrip() {
        let handle = flutter_session_new();
        let json = flutter_session_connection_pair_json(&handle, 7, 9)
            .expect("Connection-Pair-Snapshot-Serialisierung muss gelingen");
        let snapshot: HostConnectionPairSnapshot = serde_json::from_str(&json)
            .expect("Connection-Pair-Snapshot muss parsebares JSON sein");

        assert_eq!(snapshot.node_a, 7);
        assert_eq!(snapshot.node_b, 9);
        assert!(snapshot.connections.is_empty());

        flutter_session_dispose(handle);
    }
}
