//! Kanonische (nicht-Flutter) C-ABI-Exporte der Host-Bridge-Session.

use crate::ffi_utils::{
    clear_last_error, decode_focus_node_id, into_c_string_ptr, read_json_arg, serialize_json,
    set_last_error, LAST_ERROR,
};
use crate::session_handle::{with_session_mut, HostBridgeSessionHandle};
use crate::{ffi_guard_bool, ffi_guard_ptr};
use anyhow::Context;
use fs25_auto_drive_host_bridge::dto::{host_ui_snapshot_json, viewport_overlay_snapshot_json};
use fs25_auto_drive_host_bridge::{
    HostChromeSnapshot, HostConnectionPairSnapshot, HostContextMenuSnapshot, HostDialogRequest,
    HostDialogResult, HostDialogSnapshot, HostEditingSnapshot, HostRouteToolViewportSnapshot,
    HostSessionAction, HostSessionSnapshot, HostUiSnapshot, HostViewportGeometrySnapshot,
    ViewportOverlaySnapshot,
};
use std::ffi::c_char;

/// ABI-Version des nativen Host-Bridge-Vertrags.
pub const FS25AD_HOST_BRIDGE_ABI_VERSION: u32 = 4;

/// Liefert die ABI-Version des nativen Host-Bridge-Vertrags.
#[unsafe(no_mangle)]
pub extern "C" fn fs25ad_host_bridge_abi_version() -> u32 {
    FS25AD_HOST_BRIDGE_ABI_VERSION
}

/// Gibt die letzte Fehlernachricht dieses Threads als neu allokierten UTF-8-String zurueck.
#[unsafe(no_mangle)]
pub extern "C" fn fs25ad_host_bridge_last_error_message() -> *mut c_char {
    LAST_ERROR.with(|slot| {
        slot.borrow()
            .as_ref()
            .map_or(std::ptr::null_mut(), |message| {
                into_c_string_ptr(message.clone())
            })
    })
}

/// Gibt einen durch diese Bibliothek allozierten UTF-8-String frei.
///
/// # Safety
///
/// `value` muss ein durch diese Bibliothek allokierter Zeiger sein oder `null`.
/// Nach dem Aufruf ist der Zeiger ungueltig. Doppeltes Freigeben ist undefiniertes Verhalten.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn fs25ad_host_bridge_string_free(value: *mut c_char) {
    if value.is_null() {
        return;
    }
    // SAFETY: Aufrufer garantiert, dass `value` durch diese Bibliothek allokiert wurde.
    unsafe { drop(std::ffi::CString::from_raw(value)) };
}

/// Erstellt eine neue Bridge-Session.
#[unsafe(no_mangle)]
pub extern "C" fn fs25ad_host_bridge_session_new() -> *mut HostBridgeSessionHandle {
    clear_last_error();
    Box::into_raw(Box::new(HostBridgeSessionHandle::new()))
}

/// Gibt eine zuvor erstellte Bridge-Session frei.
///
/// # Safety
///
/// `session` muss ein durch `fs25ad_host_bridge_session_new` rueckgegebener Zeiger sein oder `null`.
/// Nach dem Aufruf ist der Zeiger ungueltig. Doppeltes Freigeben ist undefiniertes Verhalten.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn fs25ad_host_bridge_session_dispose(session: *mut HostBridgeSessionHandle) {
    clear_last_error();
    if session.is_null() {
        return;
    }
    // SAFETY: Aufrufer garantiert, dass `session` durch `session_new` allokiert wurde.
    unsafe { drop(Box::from_raw(session)) };
}

/// Serialisiert den aktuellen Session-Snapshot als UTF-8-JSON.
///
/// # Safety
///
/// `session` muss ein gueltiger, durch `fs25ad_host_bridge_session_new` erzeugter Zeiger sein.
/// Der rueckgegebene String muss mit `fs25ad_host_bridge_string_free` freigegeben werden.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn fs25ad_host_bridge_session_snapshot_json(
    session: *mut HostBridgeSessionHandle,
) -> *mut c_char {
    ffi_guard_ptr! {
        with_session_mut(session, |session| {
            let snapshot: HostSessionSnapshot = session.snapshot_owned();
            serialize_json(&snapshot)
        })
    }
}

/// Serialisiert den host-neutralen Chrome-Snapshot als UTF-8-JSON.
///
/// # Safety
///
/// `session` muss ein gueltiger, durch `fs25ad_host_bridge_session_new` erzeugter Zeiger sein.
/// Der rueckgegebene String muss mit `fs25ad_host_bridge_string_free` freigegeben werden.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn fs25ad_host_bridge_session_chrome_snapshot_json(
    session: *mut HostBridgeSessionHandle,
) -> *mut c_char {
    ffi_guard_ptr! {
        with_session_mut(session, |session| {
            let snapshot: HostChromeSnapshot = session.build_host_chrome_snapshot();
            serialize_json(&snapshot)
        })
    }
}

/// Serialisiert den aktuell inspizierten Node als UTF-8-JSON.
///
/// Gibt `null` zurueck, wenn aktuell kein inspizierter Node vorliegt oder die
/// inspizierte Node-ID in der geladenen Karte nicht existiert. Das ist kein
/// Fehlerfall und setzt keine Last-Error-Nachricht.
///
/// # Safety
///
/// `session` muss ein gueltiger, durch `fs25ad_host_bridge_session_new` erzeugter Zeiger sein.
/// Der rueckgegebene String muss mit `fs25ad_host_bridge_string_free` freigegeben werden.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn fs25ad_host_bridge_session_node_details_json(
    session: *mut HostBridgeSessionHandle,
) -> *mut c_char {
    ffi_guard_ptr! {
        with_session_mut(session, |session| {
            Ok(session
                .node_details_json()
                .map_or(std::ptr::null_mut(), into_c_string_ptr))
        })
    }
}

/// Serialisiert die komplette Marker-Liste als UTF-8-JSON.
///
/// # Safety
///
/// `session` muss ein gueltiger, durch `fs25ad_host_bridge_session_new` erzeugter Zeiger sein.
/// Der rueckgegebene String muss mit `fs25ad_host_bridge_string_free` freigegeben werden.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn fs25ad_host_bridge_session_marker_list_json(
    session: *mut HostBridgeSessionHandle,
) -> *mut c_char {
    ffi_guard_ptr! {
        with_session_mut(session, |session| Ok(into_c_string_ptr(session.marker_list_json())))
    }
}

/// Serialisiert die Verbindungsdetails zwischen zwei Nodes als UTF-8-JSON.
///
/// # Safety
///
/// `session` muss ein gueltiger, durch `fs25ad_host_bridge_session_new` erzeugter Zeiger sein.
/// Der rueckgegebene String muss mit `fs25ad_host_bridge_string_free` freigegeben werden.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn fs25ad_host_bridge_session_connection_pair_json(
    session: *mut HostBridgeSessionHandle,
    node_a: u64,
    node_b: u64,
) -> *mut c_char {
    ffi_guard_ptr! {
        with_session_mut(session, |session| {
            let snapshot: HostConnectionPairSnapshot = session.connection_pair(node_a, node_b);
            serialize_json(&snapshot)
        })
    }
}

/// Liefert den Dirty-Status der Session als Integer zurueck.
///
/// Rueckgabewerte:
/// - `1`: Session ist dirty
/// - `0`: Session ist nicht dirty
/// - `-1`: Fehler; Details koennen ueber `fs25ad_host_bridge_last_error_message()` abgefragt werden
///
/// # Safety
///
/// `session` muss ein gueltiger, durch `fs25ad_host_bridge_session_new` erzeugter Zeiger sein.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn fs25ad_host_bridge_session_is_dirty(
    session: *mut HostBridgeSessionHandle,
) -> i32 {
    clear_last_error();
    match std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
        with_session_mut(session, |session| Ok(session.is_dirty()))
    })) {
        Ok(Ok(true)) => 1,
        Ok(Ok(false)) => 0,
        Ok(Err(e)) => {
            set_last_error(e.to_string());
            -1
        }
        Err(_) => {
            set_last_error("internal panic in FFI call");
            -1
        }
    }
}

/// Serialisiert den host-neutralen UI-Snapshot als UTF-8-JSON.
///
/// # Safety
///
/// `session` muss ein gueltiger, durch `fs25ad_host_bridge_session_new` erzeugter Zeiger sein.
/// Der rueckgegebene String muss mit `fs25ad_host_bridge_string_free` freigegeben werden.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn fs25ad_host_bridge_session_ui_snapshot_json(
    session: *mut HostBridgeSessionHandle,
) -> *mut c_char {
    ffi_guard_ptr! {
        with_session_mut(session, |session| {
            let snapshot: HostUiSnapshot = session.build_host_ui_snapshot();
            let payload = host_ui_snapshot_json(&snapshot).context("JSON serialization failed")?;
            Ok(into_c_string_ptr(payload))
        })
    }
}

/// Serialisiert den host-neutralen Dialog-Snapshot als UTF-8-JSON.
///
/// # Safety
///
/// `session` muss ein gueltiger, durch `fs25ad_host_bridge_session_new` erzeugter Zeiger sein.
/// Der rueckgegebene String muss mit `fs25ad_host_bridge_string_free` freigegeben werden.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn fs25ad_host_bridge_session_dialog_snapshot_json(
    session: *mut HostBridgeSessionHandle,
) -> *mut c_char {
    ffi_guard_ptr! {
        with_session_mut(session, |session| {
            let snapshot: HostDialogSnapshot = session.dialog_snapshot();
            serialize_json(&snapshot)
        })
    }
}

/// Serialisiert den host-neutralen Editing-Snapshot als UTF-8-JSON.
///
/// # Safety
///
/// `session` muss ein gueltiger, durch `fs25ad_host_bridge_session_new` erzeugter Zeiger sein.
/// Der rueckgegebene String muss mit `fs25ad_host_bridge_string_free` freigegeben werden.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn fs25ad_host_bridge_session_editing_snapshot_json(
    session: *mut HostBridgeSessionHandle,
) -> *mut c_char {
    ffi_guard_ptr! {
        with_session_mut(session, |session| {
            let snapshot: HostEditingSnapshot = session.editing_snapshot();
            serialize_json(&snapshot)
        })
    }
}

/// Serialisiert den host-neutralen Kontextmenue-Snapshot als UTF-8-JSON.
///
/// `focus_node_id_or_neg1` nutzt `-1` als FFI-Sentinel fuer "kein Fokus-Node".
///
/// # Safety
///
/// `session` muss ein gueltiger, durch `fs25ad_host_bridge_session_new` erzeugter Zeiger sein.
/// Der rueckgegebene String muss mit `fs25ad_host_bridge_string_free` freigegeben werden.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn fs25ad_host_bridge_session_context_menu_snapshot_json(
    session: *mut HostBridgeSessionHandle,
    focus_node_id_or_neg1: i64,
) -> *mut c_char {
    ffi_guard_ptr! {{
        let focus_node_id = decode_focus_node_id(focus_node_id_or_neg1)?;
        with_session_mut(session, |session| {
            let snapshot: HostContextMenuSnapshot = session.context_menu_snapshot(focus_node_id);
            serialize_json(&snapshot)
        })
    }}
}

/// Serialisiert den Route-Tool-Viewport-Snapshot als UTF-8-JSON.
///
/// # Safety
///
/// `session` muss ein gueltiger, durch `fs25ad_host_bridge_session_new` erzeugter Zeiger sein.
/// Der rueckgegebene String muss mit `fs25ad_host_bridge_string_free` freigegeben werden.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn fs25ad_host_bridge_session_route_tool_viewport_json(
    session: *mut HostBridgeSessionHandle,
) -> *mut c_char {
    ffi_guard_ptr! {
        with_session_mut(session, |session| {
            let snapshot: HostRouteToolViewportSnapshot =
                session.build_route_tool_viewport_snapshot();
            serialize_json(&snapshot)
        })
    }
}

/// Serialisiert den host-neutralen Viewport-Overlay-Snapshot als UTF-8-JSON.
///
/// `cursor_world_x` und `cursor_world_y` beschreiben die aktuelle Cursor-Position
/// in Weltkoordinaten.
///
/// # Safety
///
/// `session` muss ein gueltiger, durch `fs25ad_host_bridge_session_new` erzeugter Zeiger sein.
/// Der rueckgegebene String muss mit `fs25ad_host_bridge_string_free` freigegeben werden.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn fs25ad_host_bridge_session_viewport_overlay_json(
    session: *mut HostBridgeSessionHandle,
    cursor_world_x: f32,
    cursor_world_y: f32,
) -> *mut c_char {
    ffi_guard_ptr! {
        with_session_mut(session, |session| {
            let snapshot: ViewportOverlaySnapshot = session
                .build_viewport_overlay_snapshot(Some([cursor_world_x, cursor_world_y].into()));
            let payload = viewport_overlay_snapshot_json(&snapshot)
                .context("JSON serialization failed")?;
            Ok(into_c_string_ptr(payload))
        })
    }
}

/// Wendet eine kanonische `HostSessionAction` an, uebergeben als UTF-8-JSON.
///
/// # Safety
///
/// `session` muss ein gueltiger, durch `fs25ad_host_bridge_session_new` erzeugter Zeiger sein.
/// `action_json` muss ein gueltiger, null-terminierter UTF-8-String oder `null` sein.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn fs25ad_host_bridge_session_apply_action_json(
    session: *mut HostBridgeSessionHandle,
    action_json: *const c_char,
) -> bool {
    ffi_guard_bool! {{
        let action: HostSessionAction = read_json_arg(action_json, "HostSessionAction")?;
        with_session_mut(session, |session| session.apply_action(action))
    }}
}

/// Entnimmt alle aktuell ausstehenden Dialog-Anforderungen als UTF-8-JSON-Array.
///
/// # Safety
///
/// `session` muss ein gueltiger, durch `fs25ad_host_bridge_session_new` erzeugter Zeiger sein.
/// Der rueckgegebene String muss mit `fs25ad_host_bridge_string_free` freigegeben werden.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn fs25ad_host_bridge_session_take_dialog_requests_json(
    session: *mut HostBridgeSessionHandle,
) -> *mut c_char {
    ffi_guard_ptr! {
        with_session_mut(session, |session| {
            let requests: Vec<HostDialogRequest> = session.take_dialog_requests();
            serialize_json(&requests)
        })
    }
}

/// Reicht ein `HostDialogResult` als UTF-8-JSON in die Session zurueck.
///
/// # Safety
///
/// `session` muss ein gueltiger, durch `fs25ad_host_bridge_session_new` erzeugter Zeiger sein.
/// `result_json` muss ein gueltiger, null-terminierter UTF-8-String oder `null` sein.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn fs25ad_host_bridge_session_submit_dialog_result_json(
    session: *mut HostBridgeSessionHandle,
    result_json: *const c_char,
) -> bool {
    ffi_guard_bool! {{
        let result: HostDialogResult = read_json_arg(result_json, "HostDialogResult")?;
        with_session_mut(session, |session| session.submit_dialog_result(result))
    }}
}

/// Baut einen minimalen Viewport-Geometry-Snapshot als UTF-8-JSON.
///
/// # Safety
///
/// `session` muss ein gueltiger, durch `fs25ad_host_bridge_session_new` erzeugter Zeiger sein.
/// Der rueckgegebene String muss mit `fs25ad_host_bridge_string_free` freigegeben werden.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn fs25ad_host_bridge_session_viewport_geometry_json(
    session: *mut HostBridgeSessionHandle,
    viewport_width: f32,
    viewport_height: f32,
) -> *mut c_char {
    ffi_guard_ptr! {
        with_session_mut(session, |session| {
            let snapshot: HostViewportGeometrySnapshot =
                session.build_viewport_geometry_snapshot([viewport_width, viewport_height]);
            serialize_json(&snapshot)
        })
    }
}
