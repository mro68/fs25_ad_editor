//! C-ABI-Transport ueber der kanonischen Host-Bridge-Session.

#[cfg(feature = "flutter")]
mod api;
#[cfg(feature = "flutter")]
pub mod flutter_api;
mod frb_generated;
mod shared_texture_v2;
mod texture_registration_v4;

/// Hilfsmakro: Wraps einen bool-FFI-Aufruf mit Panic-Isolation und Last-Error-Behandlung.
macro_rules! ffi_guard_bool {
    ($body:expr) => {{
        clear_last_error();
        match std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| $body)) {
            Ok(Ok(())) => true,
            Ok(Err(e)) => {
                set_last_error(e.to_string());
                false
            }
            Err(_) => {
                set_last_error("internal panic in FFI call");
                false
            }
        }
    }};
}

/// Hilfsmakro: Wraps einen ptr-rueckgebenden FFI-Aufruf mit Panic-Isolation und Last-Error-Behandlung.
macro_rules! ffi_guard_ptr {
    ($body:expr) => {{
        clear_last_error();
        match std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| $body)) {
            Ok(Ok(ptr)) => ptr,
            Ok(Err(e)) => {
                set_last_error(e.to_string());
                std::ptr::null_mut()
            }
            Err(_) => {
                set_last_error("internal panic in FFI call");
                std::ptr::null_mut()
            }
        }
    }};
}

#[cfg(all(feature = "flutter-linux", target_os = "linux"))]
pub mod flutter_gpu;

use anyhow::{anyhow, Context, Result};
use fs25_auto_drive_host_bridge::{
    HostBridgeSession, HostChromeSnapshot, HostDialogRequest, HostDialogResult,
    HostRouteToolViewportSnapshot, HostSessionAction, HostSessionSnapshot,
    HostViewportGeometrySnapshot,
};
use std::cell::RefCell;
use std::ffi::{c_char, CStr, CString};
use std::sync::Mutex;

thread_local! {
    static LAST_ERROR: RefCell<Option<String>> = const { RefCell::new(None) };
}

const FS25AD_HOST_BRIDGE_ABI_VERSION: u32 = 4;

/// Opaquer Session-Handle mit serialisiertem Zugriff auf die kanonische Session.
pub struct HostBridgeSessionHandle {
    session: Mutex<HostBridgeSession>,
}

impl HostBridgeSessionHandle {
    fn new() -> Self {
        Self {
            session: Mutex::new(HostBridgeSession::new()),
        }
    }

    fn with_lock<T>(&self, f: impl FnOnce(&mut HostBridgeSession) -> Result<T>) -> Result<T> {
        let mut guard = self
            .session
            .lock()
            .map_err(|_| anyhow!("HostBridgeSession lock poisoned"))?;
        f(&mut guard)
    }
}

fn clear_last_error() {
    LAST_ERROR.with(|slot| {
        *slot.borrow_mut() = None;
    });
}

fn set_last_error(error: impl Into<String>) {
    let message = error.into().replace('\0', " ");
    LAST_ERROR.with(|slot| {
        *slot.borrow_mut() = Some(message);
    });
}

fn into_c_string_ptr(value: String) -> *mut c_char {
    CString::new(value)
        .expect("sanitized string must not contain interior NUL bytes")
        .into_raw()
}

fn serialize_json<T: serde::Serialize>(value: &T) -> Result<*mut c_char> {
    let payload = serde_json::to_string(value).context("JSON serialization failed")?;
    Ok(into_c_string_ptr(payload))
}

fn read_json_arg<T>(value: *const c_char, type_name: &str) -> Result<T>
where
    T: serde::de::DeserializeOwned,
{
    if value.is_null() {
        return Err(anyhow!("{type_name} JSON pointer must not be null"));
    }

    let text = unsafe {
        CStr::from_ptr(value)
            .to_str()
            .context("FFI JSON must be valid UTF-8")?
    };
    serde_json::from_str(text).with_context(|| format!("failed to parse {type_name} JSON"))
}

fn with_session_mut<T>(
    session: *mut HostBridgeSessionHandle,
    f: impl FnOnce(&mut HostBridgeSession) -> Result<T>,
) -> Result<T> {
    if session.is_null() {
        return Err(anyhow!("HostBridgeSession pointer must not be null"));
    }

    let session = unsafe { &*session };
    session.with_lock(f)
}

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
    unsafe { drop(CString::from_raw(value)) };
}

/// Erstellt eine neue Bridge-Session.
#[unsafe(no_mangle)]
pub extern "C" fn fs25ad_host_bridge_session_new() -> *mut HostBridgeSessionHandle {
    clear_last_error();
    Box::into_raw(Box::new(HostBridgeSessionHandle::new()))
}

/// Erstellt eine neue Flutter-Session als C-ABI-Handle.
#[cfg(feature = "flutter")]
#[unsafe(no_mangle)]
pub extern "C" fn fs25ad_flutter_session_new() -> *mut flutter_api::FlutterSessionHandle {
    ffi_guard_ptr! {{
        Result::<*mut flutter_api::FlutterSessionHandle>::Ok(Box::into_raw(
            flutter_api::flutter_session_new(),
        ))
    }}
}

/// Gibt einen zuvor erstellten Flutter-Session-Handle frei.
///
/// # Safety
///
/// `session` muss ein durch `fs25ad_flutter_session_new` erzeugter Zeiger sein oder `null`.
/// Nach dem Aufruf ist der Zeiger ungueltig. Doppeltes Freigeben ist undefiniertes Verhalten.
#[cfg(feature = "flutter")]
#[unsafe(no_mangle)]
pub unsafe extern "C" fn fs25ad_flutter_session_dispose(
    session: *mut flutter_api::FlutterSessionHandle,
) {
    clear_last_error();
    if session.is_null() {
        return;
    }

    // SAFETY: Aufrufer garantiert, dass `session` durch `fs25ad_flutter_session_new`
    // alloziert wurde und hier exklusiv freigegeben werden darf.
    unsafe { flutter_api::flutter_session_dispose(*Box::from_raw(session)) };
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

#[cfg(test)]
mod tests {
    use super::{
        fs25ad_host_bridge_abi_version, fs25ad_host_bridge_last_error_message,
        fs25ad_host_bridge_session_apply_action_json,
        fs25ad_host_bridge_session_chrome_snapshot_json, fs25ad_host_bridge_session_dispose,
        fs25ad_host_bridge_session_new, fs25ad_host_bridge_session_route_tool_viewport_json,
        fs25ad_host_bridge_session_snapshot_json,
        fs25ad_host_bridge_session_submit_dialog_result_json,
        fs25ad_host_bridge_session_take_dialog_requests_json,
        fs25ad_host_bridge_session_viewport_geometry_json, fs25ad_host_bridge_string_free,
        FS25AD_HOST_BRIDGE_ABI_VERSION,
    };
    use fs25_auto_drive_host_bridge::{
        HostActiveTool, HostDialogRequest, HostDialogRequestKind, HostDialogResult,
        HostInputModifiers, HostPointerButton, HostRouteToolAction, HostRouteToolId,
        HostRouteToolViewportSnapshot, HostSessionAction, HostSessionSnapshot, HostTangentSource,
        HostTapKind, HostViewportGeometrySnapshot, HostViewportInputBatch, HostViewportInputEvent,
    };
    use std::ffi::{CStr, CString};

    // Sicherheits-Wrapper fuer unsafe FFI-Funktionen im Testkontext.
    fn string_free(ptr: *mut std::ffi::c_char) {
        unsafe { fs25ad_host_bridge_string_free(ptr) }
    }
    fn session_dispose(s: *mut super::HostBridgeSessionHandle) {
        unsafe { fs25ad_host_bridge_session_dispose(s) }
    }
    fn session_snapshot_json(s: *mut super::HostBridgeSessionHandle) -> *mut std::ffi::c_char {
        unsafe { fs25ad_host_bridge_session_snapshot_json(s) }
    }
    fn session_chrome_snapshot_json(
        s: *mut super::HostBridgeSessionHandle,
    ) -> *mut std::ffi::c_char {
        unsafe { fs25ad_host_bridge_session_chrome_snapshot_json(s) }
    }
    fn session_route_tool_viewport_json(
        s: *mut super::HostBridgeSessionHandle,
    ) -> *mut std::ffi::c_char {
        unsafe { fs25ad_host_bridge_session_route_tool_viewport_json(s) }
    }
    fn session_apply_action_json(
        s: *mut super::HostBridgeSessionHandle,
        j: *const std::ffi::c_char,
    ) -> bool {
        unsafe { fs25ad_host_bridge_session_apply_action_json(s, j) }
    }
    fn session_take_dialog_requests_json(
        s: *mut super::HostBridgeSessionHandle,
    ) -> *mut std::ffi::c_char {
        unsafe { fs25ad_host_bridge_session_take_dialog_requests_json(s) }
    }
    fn session_submit_dialog_result_json(
        s: *mut super::HostBridgeSessionHandle,
        j: *const std::ffi::c_char,
    ) -> bool {
        unsafe { fs25ad_host_bridge_session_submit_dialog_result_json(s, j) }
    }
    fn session_viewport_geometry_json(
        s: *mut super::HostBridgeSessionHandle,
        w: f32,
        h: f32,
    ) -> *mut std::ffi::c_char {
        unsafe { fs25ad_host_bridge_session_viewport_geometry_json(s, w, h) }
    }

    #[cfg(feature = "flutter")]
    fn flutter_session_dispose(session: *mut super::flutter_api::FlutterSessionHandle) {
        unsafe { super::fs25ad_flutter_session_dispose(session) }
    }

    fn read_and_free_string(ptr: *mut std::ffi::c_char) -> String {
        assert!(!ptr.is_null());
        let value = unsafe { CStr::from_ptr(ptr) }
            .to_str()
            .expect("FFI string must be valid UTF-8")
            .to_string();
        string_free(ptr);
        value
    }

    fn apply_action_json(session: *mut super::HostBridgeSessionHandle, action: HostSessionAction) {
        let action_json =
            CString::new(serde_json::to_string(&action).expect("action JSON must serialize"))
                .expect("CString must build");

        if session_apply_action_json(session, action_json.as_ptr()) {
            return;
        }

        let error = read_and_free_string(fs25ad_host_bridge_last_error_message());
        panic!("HostSessionAction failed unexpectedly: {error}");
    }

    #[test]
    fn ffi_transport_reports_stable_abi_version() {
        assert_eq!(
            fs25ad_host_bridge_abi_version(),
            FS25AD_HOST_BRIDGE_ABI_VERSION
        );
        assert_eq!(fs25ad_host_bridge_abi_version(), 4);
    }

    #[cfg(feature = "flutter")]
    #[test]
    fn ffi_flutter_session_lifecycle_roundtrip() {
        let session = super::fs25ad_flutter_session_new();
        assert!(
            !session.is_null(),
            "Flutter-Session-FFI muss einen gueltigen Handle liefern"
        );
        flutter_session_dispose(session);
    }

    #[test]
    fn ffi_transport_roundtrips_session_actions_dialogs_and_snapshots() {
        let session = fs25ad_host_bridge_session_new();
        assert!(!session.is_null());

        let action_json = CString::new(
            serde_json::to_string(&HostSessionAction::ToggleCommandPalette)
                .expect("action JSON must serialize"),
        )
        .expect("CString must build");
        assert!(session_apply_action_json(session, action_json.as_ptr()));

        let snapshot_json = read_and_free_string(session_snapshot_json(session));
        let snapshot: HostSessionSnapshot =
            serde_json::from_str(&snapshot_json).expect("snapshot JSON must parse");
        assert!(snapshot.show_command_palette);

        let chrome_snapshot_json = read_and_free_string(session_chrome_snapshot_json(session));
        let chrome_snapshot: fs25_auto_drive_host_bridge::HostChromeSnapshot =
            serde_json::from_str(&chrome_snapshot_json).expect("chrome snapshot JSON must parse");
        assert!(chrome_snapshot.show_command_palette);
        assert_eq!(chrome_snapshot.status_message, None);

        let route_tool_viewport_json =
            read_and_free_string(session_route_tool_viewport_json(session));
        let route_tool_viewport: HostRouteToolViewportSnapshot =
            serde_json::from_str(&route_tool_viewport_json)
                .expect("route tool viewport JSON must parse");
        assert!(!route_tool_viewport.has_pending_input);
        assert!(route_tool_viewport.drag_targets.is_empty());

        let request_action_json = CString::new(
            serde_json::to_string(&HostSessionAction::RequestHeightmapSelection)
                .expect("dialog action JSON must serialize"),
        )
        .expect("CString must build");
        assert!(session_apply_action_json(
            session,
            request_action_json.as_ptr()
        ));

        let requests_json = read_and_free_string(session_take_dialog_requests_json(session));
        let requests: Vec<HostDialogRequest> =
            serde_json::from_str(&requests_json).expect("dialog request JSON must parse");
        assert_eq!(requests.len(), 1);
        assert_eq!(requests[0].kind, HostDialogRequestKind::Heightmap);

        let result_json = CString::new(
            serde_json::to_string(&HostDialogResult::PathSelected {
                kind: HostDialogRequestKind::Heightmap,
                path: "/tmp/test_heightmap.png".to_string(),
            })
            .expect("dialog result JSON must serialize"),
        )
        .expect("CString must build");
        assert!(session_submit_dialog_result_json(
            session,
            result_json.as_ptr()
        ));

        let geometry_json =
            read_and_free_string(session_viewport_geometry_json(session, 800.0, 600.0));
        let geometry: HostViewportGeometrySnapshot =
            serde_json::from_str(&geometry_json).expect("geometry JSON must parse");
        assert_eq!(geometry.viewport_size, [800.0, 600.0]);

        let viewport_input_json = CString::new(
            serde_json::to_string(&HostSessionAction::SubmitViewportInput {
                batch: HostViewportInputBatch {
                    events: vec![
                        HostViewportInputEvent::Resize {
                            size_px: [1024.0, 768.0],
                        },
                        HostViewportInputEvent::Scroll {
                            screen_pos: Some([512.0, 384.0]),
                            smooth_delta_y: 1.0,
                            raw_delta_y: 0.0,
                            modifiers: HostInputModifiers::default(),
                        },
                        HostViewportInputEvent::Tap {
                            button: HostPointerButton::Primary,
                            tap_kind: HostTapKind::Double,
                            screen_pos: [512.0, 384.0],
                            modifiers: HostInputModifiers::default(),
                        },
                    ],
                },
            })
            .expect("viewport input JSON must serialize"),
        )
        .expect("CString must build");
        assert!(session_apply_action_json(
            session,
            viewport_input_json.as_ptr()
        ));

        let viewport_snapshot_json = read_and_free_string(session_snapshot_json(session));
        let viewport_snapshot: HostSessionSnapshot = serde_json::from_str(&viewport_snapshot_json)
            .expect("snapshot JSON after viewport input must parse");
        assert!(viewport_snapshot.viewport.zoom > 1.0);

        let updated_geometry_json =
            read_and_free_string(session_viewport_geometry_json(session, 1024.0, 768.0));
        let updated_geometry: HostViewportGeometrySnapshot =
            serde_json::from_str(&updated_geometry_json).expect("updated geometry JSON must parse");
        assert_eq!(updated_geometry.viewport_size, [1024.0, 768.0]);
        assert!(updated_geometry.zoom > 1.0);

        session_dispose(session);
    }

    #[test]
    fn ffi_transport_reports_errors_for_invalid_json() {
        let session = fs25ad_host_bridge_session_new();
        assert!(!session.is_null());

        let invalid_json = CString::new("{not valid json}").expect("CString must build");
        assert!(!session_apply_action_json(session, invalid_json.as_ptr()));

        let error = read_and_free_string(fs25ad_host_bridge_last_error_message());
        assert!(error.contains("HostSessionAction"));

        session_dispose(session);
    }

    #[test]
    fn ffi_transport_rejects_null_session_and_payload_pointers() {
        assert!(session_snapshot_json(std::ptr::null_mut()).is_null());
        assert!(
            read_and_free_string(fs25ad_host_bridge_last_error_message())
                .contains("HostBridgeSession pointer")
        );

        assert!(session_chrome_snapshot_json(std::ptr::null_mut()).is_null());
        assert!(
            read_and_free_string(fs25ad_host_bridge_last_error_message())
                .contains("HostBridgeSession pointer")
        );

        assert!(session_route_tool_viewport_json(std::ptr::null_mut()).is_null());
        assert!(
            read_and_free_string(fs25ad_host_bridge_last_error_message())
                .contains("HostBridgeSession pointer")
        );

        assert!(session_take_dialog_requests_json(std::ptr::null_mut()).is_null());
        assert!(
            read_and_free_string(fs25ad_host_bridge_last_error_message())
                .contains("HostBridgeSession pointer")
        );

        assert!(session_viewport_geometry_json(std::ptr::null_mut(), 320.0, 200.0).is_null());
        assert!(
            read_and_free_string(fs25ad_host_bridge_last_error_message())
                .contains("HostBridgeSession pointer")
        );

        let session = fs25ad_host_bridge_session_new();
        assert!(!session.is_null());

        assert!(!session_apply_action_json(session, std::ptr::null()));
        assert!(
            read_and_free_string(fs25ad_host_bridge_last_error_message())
                .contains("HostSessionAction JSON pointer must not be null")
        );

        assert!(!session_submit_dialog_result_json(
            session,
            std::ptr::null()
        ));
        assert!(
            read_and_free_string(fs25ad_host_bridge_last_error_message())
                .contains("HostDialogResult JSON pointer must not be null")
        );

        session_dispose(session);
    }

    #[test]
    fn ffi_route_tool_write_actions_cover_anchors_click_drag_lasso_and_related_json_paths() {
        let session = fs25ad_host_bridge_session_new();
        assert!(!session.is_null());

        apply_action_json(session, HostSessionAction::OpenFile);

        let requests_json = read_and_free_string(session_take_dialog_requests_json(session));
        let requests: Vec<HostDialogRequest> =
            serde_json::from_str(&requests_json).expect("dialog request JSON must parse");
        assert!(!requests.is_empty());
        assert_eq!(requests[0].kind, HostDialogRequestKind::OpenFile);

        let test_map_path = format!(
            "{}/../../ad_sample_data/AutoDrive_config-test.xml",
            env!("CARGO_MANIFEST_DIR")
        );
        let open_result_json = CString::new(
            serde_json::to_string(&HostDialogResult::PathSelected {
                kind: HostDialogRequestKind::OpenFile,
                path: test_map_path,
            })
            .expect("dialog result JSON must serialize"),
        )
        .expect("CString must build");
        assert!(session_submit_dialog_result_json(
            session,
            open_result_json.as_ptr()
        ));

        let geometry_json =
            read_and_free_string(session_viewport_geometry_json(session, 1024.0, 768.0));
        let geometry: HostViewportGeometrySnapshot =
            serde_json::from_str(&geometry_json).expect("geometry JSON must parse");
        assert!(geometry.nodes.len() >= 2);

        let start_id = geometry.nodes[0].id;
        let end_id = geometry.nodes[1].id;
        let start_pos = geometry.nodes[0].position;
        let end_pos = geometry.nodes[1].position;

        apply_action_json(
            session,
            HostSessionAction::SetEditorTool {
                tool: HostActiveTool::Route,
            },
        );

        apply_action_json(
            session,
            HostSessionAction::RouteTool {
                action: HostRouteToolAction::SelectToolWithAnchors {
                    tool: HostRouteToolId::CurveCubic,
                    start_node_id: start_id,
                    end_node_id: end_id,
                },
            },
        );

        apply_action_json(
            session,
            HostSessionAction::RouteTool {
                action: HostRouteToolAction::Click {
                    world_pos: start_pos,
                    ctrl: false,
                },
            },
        );
        apply_action_json(
            session,
            HostSessionAction::RouteTool {
                action: HostRouteToolAction::DragStart {
                    world_pos: start_pos,
                },
            },
        );
        apply_action_json(
            session,
            HostSessionAction::RouteTool {
                action: HostRouteToolAction::DragUpdate {
                    world_pos: [
                        (start_pos[0] + end_pos[0]) * 0.5,
                        (start_pos[1] + end_pos[1]) * 0.5,
                    ],
                },
            },
        );
        apply_action_json(
            session,
            HostSessionAction::RouteTool {
                action: HostRouteToolAction::DragEnd,
            },
        );

        apply_action_json(
            session,
            HostSessionAction::RouteTool {
                action: HostRouteToolAction::LassoCompleted {
                    polygon: vec![
                        start_pos,
                        [end_pos[0], start_pos[1]],
                        end_pos,
                        [start_pos[0], end_pos[1]],
                    ],
                },
            },
        );

        apply_action_json(
            session,
            HostSessionAction::RouteTool {
                action: HostRouteToolAction::ApplyTangent {
                    start: HostTangentSource::None,
                    end: HostTangentSource::None,
                },
            },
        );
        apply_action_json(
            session,
            HostSessionAction::RouteTool {
                action: HostRouteToolAction::ScrollRotate { delta: 1.0 },
            },
        );
        apply_action_json(
            session,
            HostSessionAction::RouteTool {
                action: HostRouteToolAction::IncreaseNodeCount,
            },
        );
        apply_action_json(
            session,
            HostSessionAction::RouteTool {
                action: HostRouteToolAction::DecreaseSegmentLength,
            },
        );

        let viewport_json = read_and_free_string(session_route_tool_viewport_json(session));
        let viewport_snapshot: HostRouteToolViewportSnapshot =
            serde_json::from_str(&viewport_json).expect("route tool viewport JSON must parse");
        assert!(viewport_snapshot.has_pending_input);

        session_dispose(session);
    }
}
