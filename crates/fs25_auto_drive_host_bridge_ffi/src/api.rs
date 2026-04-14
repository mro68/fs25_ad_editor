//! Dedizierte flutter_rust_bridge-API-Einstiegspunkte fuer die Flutter-Control-Plane.
//!
//! Dieses Modul bleibt bewusst duenn und delegiert an `flutter_api`, damit die
//! bestehende Control-Plane unveraendert bleibt und `flutter_rust_bridge_codegen`
//! die Default-Datei `src/api.rs` scannen kann.

#![allow(dead_code)]

use anyhow::Result;

pub use crate::flutter_api::FlutterSessionHandle;

/// Erzeugt eine neue Flutter-Session fuer die FRB-Control-Plane.
pub fn flutter_session_new() -> FlutterSessionHandle {
    crate::flutter_api::flutter_session_new()
}

/// Gibt eine zuvor erzeugte Flutter-Session frei.
pub fn flutter_session_dispose(handle: FlutterSessionHandle) {
    crate::flutter_api::flutter_session_dispose(handle)
}

/// Wendet eine serialisierte Session-Action an.
pub fn flutter_session_apply_action(
    handle: &FlutterSessionHandle,
    action_json: String,
) -> Result<()> {
    crate::flutter_api::flutter_session_apply_action(handle, action_json)
}

/// Entnimmt alle aktuell ausstehenden Dialog-Anforderungen als JSON-Array.
pub fn flutter_session_take_dialog_requests_json(handle: &FlutterSessionHandle) -> Result<String> {
    crate::flutter_api::flutter_session_take_dialog_requests_json(handle)
}

/// Reicht ein serialisiertes Dialog-Ergebnis an die Session weiter.
pub fn flutter_session_submit_dialog_result_json(
    handle: &FlutterSessionHandle,
    result_json: String,
) -> Result<()> {
    crate::flutter_api::flutter_session_submit_dialog_result_json(handle, result_json)
}

/// Gibt den aktuellen Session-Snapshot als JSON-String zurueck.
pub fn flutter_session_snapshot_json(handle: &FlutterSessionHandle) -> Result<String> {
    crate::flutter_api::flutter_session_snapshot_json(handle)
}

/// Gibt den aktuell inspizierten Node als JSON-String zurueck.
pub fn flutter_session_node_details_json(handle: &FlutterSessionHandle) -> Option<String> {
    crate::flutter_api::flutter_session_node_details_json(handle)
}

/// Gibt die aktuelle Marker-Liste als JSON-String zurueck.
pub fn flutter_session_marker_list_json(handle: &FlutterSessionHandle) -> String {
    crate::flutter_api::flutter_session_marker_list_json(handle)
}

/// Gibt den host-neutralen Route-Tool-Viewport-Snapshot als JSON-String zurueck.
pub fn flutter_session_route_tool_viewport_json(handle: &FlutterSessionHandle) -> Result<String> {
    crate::flutter_api::flutter_session_route_tool_viewport_json(handle)
}

/// Gibt die Verbindungsdetails zwischen zwei Nodes als JSON-String zurueck.
pub fn flutter_session_connection_pair_json(
    handle: &FlutterSessionHandle,
    node_a: u64,
    node_b: u64,
) -> Result<String> {
    crate::flutter_api::flutter_session_connection_pair_json(handle, node_a, node_b)
}

/// Gibt zurueck, ob die geladene Karte seit dem letzten Load/Save veraendert wurde.
pub fn flutter_session_is_dirty(handle: &FlutterSessionHandle) -> Result<bool> {
    crate::flutter_api::flutter_session_is_dirty(handle)
}

/// Gibt den aktuellen host-neutralen UI-Snapshot als JSON-String zurueck.
pub fn flutter_session_ui_snapshot_json(handle: &FlutterSessionHandle) -> Result<String> {
    crate::flutter_api::flutter_session_ui_snapshot_json(handle)
}

/// Gibt den aktuellen host-neutralen Chrome-Snapshot als JSON-String zurueck.
pub fn flutter_session_chrome_snapshot_json(handle: &FlutterSessionHandle) -> Result<String> {
    crate::flutter_api::flutter_session_chrome_snapshot_json(handle)
}

/// Gibt den aktuellen host-neutralen Dialog-Snapshot als JSON-String zurueck.
pub fn flutter_session_dialog_snapshot_json(handle: &FlutterSessionHandle) -> Result<String> {
    crate::flutter_api::flutter_session_dialog_snapshot_json(handle)
}

/// Gibt den aktuellen host-neutralen Editing-Snapshot als JSON-String zurueck.
pub fn flutter_session_editing_snapshot_json(handle: &FlutterSessionHandle) -> Result<String> {
    crate::flutter_api::flutter_session_editing_snapshot_json(handle)
}

/// Gibt den aktuellen host-neutralen Kontextmenue-Snapshot als JSON-String zurueck.
pub fn flutter_session_context_menu_snapshot_json(
    handle: &FlutterSessionHandle,
    focus_node_id_or_neg1: i64,
) -> Result<String> {
    crate::flutter_api::flutter_session_context_menu_snapshot_json(handle, focus_node_id_or_neg1)
}

/// Gibt den aktuellen host-neutralen Viewport-Overlay-Snapshot als JSON-String zurueck.
pub fn flutter_session_viewport_overlay_json(
    handle: &FlutterSessionHandle,
    cursor_world_x: f32,
    cursor_world_y: f32,
) -> Result<String> {
    crate::flutter_api::flutter_session_viewport_overlay_json(
        handle,
        cursor_world_x,
        cursor_world_y,
    )
}

/// Liefert den Viewport-Geometrie-Snapshot als JSON-String.
pub fn flutter_session_viewport_geometry_json(
    handle: &FlutterSessionHandle,
    viewport_width: f32,
    viewport_height: f32,
) -> Result<String> {
    crate::flutter_api::flutter_session_viewport_geometry_json(
        handle,
        viewport_width,
        viewport_height,
    )
}

/// Klont den Arc der Session-Instanz fuer den GPU-Hot-Path.
///
/// Gibt einen rohen Zeiger als i64 zurueck. Der Aufrufer muss ihn exakt einmal konsumieren:
/// Entweder per `fs25ad_gpu_runtime_new_with_shared_session_arc` (konsumiert ihn)
/// oder per `flutter_session_release_shared_arc_raw` (gibt ihn frei).
pub fn flutter_session_acquire_shared_arc_raw(handle: &FlutterSessionHandle) -> i64 {
    crate::flutter_api::flutter_session_acquire_shared_arc_raw(handle)
}

/// Gibt den geklonten Arc-Zeiger frei.
///
/// Nur aufrufen wenn der Wert NICHT an `fs25ad_gpu_runtime_new_with_shared_session_arc`
/// uebergeben wurde.
pub fn flutter_session_release_shared_arc_raw(raw: i64) {
    crate::flutter_api::flutter_session_release_shared_arc_raw(raw)
}
