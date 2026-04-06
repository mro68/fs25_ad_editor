//! C-ABI-Transport ueber der kanonischen Host-Bridge-Session.

mod shared_texture_v2;
mod texture_registration_v4;

use anyhow::{anyhow, Context, Result};
use fs25_auto_drive_host_bridge::{
    HostBridgeSession, HostChromeSnapshot, HostDialogRequest, HostDialogResult, HostSessionAction,
    HostSessionSnapshot, HostViewportGeometrySnapshot,
};
use std::cell::RefCell;
use std::ffi::{c_char, CStr, CString};
use std::sync::Mutex;

thread_local! {
    static LAST_ERROR: RefCell<Option<String>> = const { RefCell::new(None) };
}

const FS25AD_HOST_BRIDGE_ABI_VERSION: u32 = 3;

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
#[allow(clippy::not_unsafe_ptr_arg_deref)]
#[unsafe(no_mangle)]
pub extern "C" fn fs25ad_host_bridge_string_free(value: *mut c_char) {
    if value.is_null() {
        return;
    }

    unsafe {
        drop(CString::from_raw(value));
    }
}

/// Erstellt eine neue Bridge-Session.
#[unsafe(no_mangle)]
pub extern "C" fn fs25ad_host_bridge_session_new() -> *mut HostBridgeSessionHandle {
    clear_last_error();
    Box::into_raw(Box::new(HostBridgeSessionHandle::new()))
}

/// Gibt eine zuvor erstellte Bridge-Session frei.
#[allow(clippy::not_unsafe_ptr_arg_deref)]
#[unsafe(no_mangle)]
pub extern "C" fn fs25ad_host_bridge_session_dispose(session: *mut HostBridgeSessionHandle) {
    clear_last_error();
    if session.is_null() {
        return;
    }

    unsafe {
        drop(Box::from_raw(session));
    }
}

/// Serialisiert den aktuellen Session-Snapshot als UTF-8-JSON.
#[unsafe(no_mangle)]
pub extern "C" fn fs25ad_host_bridge_session_snapshot_json(
    session: *mut HostBridgeSessionHandle,
) -> *mut c_char {
    clear_last_error();

    match with_session_mut(session, |session| {
        let snapshot: HostSessionSnapshot = session.snapshot_owned();
        serialize_json(&snapshot)
    }) {
        Ok(payload) => payload,
        Err(error) => {
            set_last_error(error.to_string());
            std::ptr::null_mut()
        }
    }
}

/// Serialisiert den host-neutralen Chrome-Snapshot als UTF-8-JSON.
#[unsafe(no_mangle)]
pub extern "C" fn fs25ad_host_bridge_session_chrome_snapshot_json(
    session: *mut HostBridgeSessionHandle,
) -> *mut c_char {
    clear_last_error();

    match with_session_mut(session, |session| {
        let snapshot: HostChromeSnapshot = session.build_host_chrome_snapshot();
        serialize_json(&snapshot)
    }) {
        Ok(payload) => payload,
        Err(error) => {
            set_last_error(error.to_string());
            std::ptr::null_mut()
        }
    }
}

/// Wendet eine kanonische `HostSessionAction` an, uebergeben als UTF-8-JSON.
#[unsafe(no_mangle)]
pub extern "C" fn fs25ad_host_bridge_session_apply_action_json(
    session: *mut HostBridgeSessionHandle,
    action_json: *const c_char,
) -> bool {
    clear_last_error();

    match (|| {
        let action: HostSessionAction = read_json_arg(action_json, "HostSessionAction")?;
        with_session_mut(session, |session| session.apply_action(action))
    })() {
        Ok(()) => true,
        Err(error) => {
            set_last_error(error.to_string());
            false
        }
    }
}

/// Entnimmt alle aktuell ausstehenden Dialog-Anforderungen als UTF-8-JSON-Array.
#[unsafe(no_mangle)]
pub extern "C" fn fs25ad_host_bridge_session_take_dialog_requests_json(
    session: *mut HostBridgeSessionHandle,
) -> *mut c_char {
    clear_last_error();

    match with_session_mut(session, |session| {
        let requests: Vec<HostDialogRequest> = session.take_dialog_requests();
        serialize_json(&requests)
    }) {
        Ok(payload) => payload,
        Err(error) => {
            set_last_error(error.to_string());
            std::ptr::null_mut()
        }
    }
}

/// Reicht ein `HostDialogResult` als UTF-8-JSON in die Session zurueck.
#[unsafe(no_mangle)]
pub extern "C" fn fs25ad_host_bridge_session_submit_dialog_result_json(
    session: *mut HostBridgeSessionHandle,
    result_json: *const c_char,
) -> bool {
    clear_last_error();

    match (|| {
        let result: HostDialogResult = read_json_arg(result_json, "HostDialogResult")?;
        with_session_mut(session, |session| session.submit_dialog_result(result))
    })() {
        Ok(()) => true,
        Err(error) => {
            set_last_error(error.to_string());
            false
        }
    }
}

/// Baut einen minimalen Viewport-Geometry-Snapshot als UTF-8-JSON.
#[unsafe(no_mangle)]
pub extern "C" fn fs25ad_host_bridge_session_viewport_geometry_json(
    session: *mut HostBridgeSessionHandle,
    viewport_width: f32,
    viewport_height: f32,
) -> *mut c_char {
    clear_last_error();

    match with_session_mut(session, |session| {
        let snapshot: HostViewportGeometrySnapshot =
            session.build_viewport_geometry_snapshot([viewport_width, viewport_height]);
        serialize_json(&snapshot)
    }) {
        Ok(payload) => payload,
        Err(error) => {
            set_last_error(error.to_string());
            std::ptr::null_mut()
        }
    }
}

#[cfg(test)]
mod tests {
    use super::{
        fs25ad_host_bridge_abi_version, fs25ad_host_bridge_last_error_message,
        fs25ad_host_bridge_session_apply_action_json,
        fs25ad_host_bridge_session_chrome_snapshot_json, fs25ad_host_bridge_session_dispose,
        fs25ad_host_bridge_session_new, fs25ad_host_bridge_session_snapshot_json,
        fs25ad_host_bridge_session_submit_dialog_result_json,
        fs25ad_host_bridge_session_take_dialog_requests_json,
        fs25ad_host_bridge_session_viewport_geometry_json, fs25ad_host_bridge_string_free,
        FS25AD_HOST_BRIDGE_ABI_VERSION,
    };
    use fs25_auto_drive_host_bridge::{
        HostDialogRequest, HostDialogRequestKind, HostDialogResult, HostInputModifiers,
        HostPointerButton, HostSessionAction, HostSessionSnapshot, HostTapKind,
        HostViewportGeometrySnapshot, HostViewportInputBatch, HostViewportInputEvent,
    };
    use std::ffi::{CStr, CString};

    fn read_and_free_string(ptr: *mut std::ffi::c_char) -> String {
        assert!(!ptr.is_null());
        let value = unsafe { CStr::from_ptr(ptr) }
            .to_str()
            .expect("FFI string must be valid UTF-8")
            .to_string();
        fs25ad_host_bridge_string_free(ptr);
        value
    }

    #[test]
    fn ffi_transport_reports_stable_abi_version() {
        assert_eq!(
            fs25ad_host_bridge_abi_version(),
            FS25AD_HOST_BRIDGE_ABI_VERSION
        );
        assert_eq!(fs25ad_host_bridge_abi_version(), 3);
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
        assert!(fs25ad_host_bridge_session_apply_action_json(
            session,
            action_json.as_ptr()
        ));

        let snapshot_json = read_and_free_string(fs25ad_host_bridge_session_snapshot_json(session));
        let snapshot: HostSessionSnapshot =
            serde_json::from_str(&snapshot_json).expect("snapshot JSON must parse");
        assert!(snapshot.show_command_palette);

        let chrome_snapshot_json =
            read_and_free_string(fs25ad_host_bridge_session_chrome_snapshot_json(session));
        let chrome_snapshot: fs25_auto_drive_host_bridge::HostChromeSnapshot =
            serde_json::from_str(&chrome_snapshot_json).expect("chrome snapshot JSON must parse");
        assert!(chrome_snapshot.show_command_palette);
        assert_eq!(chrome_snapshot.status_message, None);

        let request_action_json = CString::new(
            serde_json::to_string(&HostSessionAction::RequestHeightmapSelection)
                .expect("dialog action JSON must serialize"),
        )
        .expect("CString must build");
        assert!(fs25ad_host_bridge_session_apply_action_json(
            session,
            request_action_json.as_ptr()
        ));

        let requests_json = read_and_free_string(
            fs25ad_host_bridge_session_take_dialog_requests_json(session),
        );
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
        assert!(fs25ad_host_bridge_session_submit_dialog_result_json(
            session,
            result_json.as_ptr()
        ));

        let geometry_json = read_and_free_string(
            fs25ad_host_bridge_session_viewport_geometry_json(session, 800.0, 600.0),
        );
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
        assert!(fs25ad_host_bridge_session_apply_action_json(
            session,
            viewport_input_json.as_ptr()
        ));

        let viewport_snapshot_json =
            read_and_free_string(fs25ad_host_bridge_session_snapshot_json(session));
        let viewport_snapshot: HostSessionSnapshot = serde_json::from_str(&viewport_snapshot_json)
            .expect("snapshot JSON after viewport input must parse");
        assert!(viewport_snapshot.viewport.zoom > 1.0);

        let updated_geometry_json = read_and_free_string(
            fs25ad_host_bridge_session_viewport_geometry_json(session, 1024.0, 768.0),
        );
        let updated_geometry: HostViewportGeometrySnapshot =
            serde_json::from_str(&updated_geometry_json).expect("updated geometry JSON must parse");
        assert_eq!(updated_geometry.viewport_size, [1024.0, 768.0]);
        assert!(updated_geometry.zoom > 1.0);

        fs25ad_host_bridge_session_dispose(session);
    }

    #[test]
    fn ffi_transport_reports_errors_for_invalid_json() {
        let session = fs25ad_host_bridge_session_new();
        assert!(!session.is_null());

        let invalid_json = CString::new("{not valid json}").expect("CString must build");
        assert!(!fs25ad_host_bridge_session_apply_action_json(
            session,
            invalid_json.as_ptr()
        ));

        let error = read_and_free_string(fs25ad_host_bridge_last_error_message());
        assert!(error.contains("HostSessionAction"));

        fs25ad_host_bridge_session_dispose(session);
    }
}
