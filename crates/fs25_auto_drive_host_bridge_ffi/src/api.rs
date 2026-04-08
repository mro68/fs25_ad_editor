//! Dedizierte flutter_rust_bridge-API-Einstiegspunkte fuer die Flutter-Control-Plane.
//!
//! Dieses Modul bleibt bewusst duenn und delegiert an `flutter_api`, damit die
//! bestehende Control-Plane unveraendert bleibt und `flutter_rust_bridge_codegen`
//! die Default-Datei `src/api.rs` scannen kann.

#![allow(dead_code)]

use anyhow::Result;

pub use crate::flutter_api::FlutterSessionHandle;

/// Erzeugt eine neue Flutter-Session fuer die FRB-Control-Plane.
pub fn flutter_session_new() -> Box<FlutterSessionHandle> {
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

/// Gibt den aktuellen Session-Snapshot als JSON-String zurueck.
pub fn flutter_session_snapshot_json(handle: &FlutterSessionHandle) -> Result<String> {
    crate::flutter_api::flutter_session_snapshot_json(handle)
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
