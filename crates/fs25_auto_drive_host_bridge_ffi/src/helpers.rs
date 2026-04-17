//! Interne Hilfsfunktionen und -typen fuer den FFI-Transport.
//!
//! Dieses Modul fasst alle nicht-exportierten Hilfsstrukturen und Hilfsfunktionen zusammen,
//! die von den `#[no_mangle]`-Exporten in `lib.rs` gemeinsam genutzt werden.

use anyhow::{anyhow, Context, Result};
use fs25_auto_drive_host_bridge::HostBridgeSession;
use std::cell::RefCell;
use std::ffi::{c_char, CStr, CString};
use std::sync::Mutex;

#[cfg(feature = "flutter")]
use crate::flutter_api::FlutterSessionHandle;

thread_local! {
    pub(crate) static LAST_ERROR: RefCell<Option<String>> = const { RefCell::new(None) };
}

pub(crate) fn clear_last_error() {
    LAST_ERROR.with(|slot| {
        *slot.borrow_mut() = None;
    });
}

pub(crate) fn set_last_error(error: impl Into<String>) {
    let message = error.into().replace('\0', " ");
    LAST_ERROR.with(|slot| {
        *slot.borrow_mut() = Some(message);
    });
}

/// Opaquer Session-Handle mit serialisiertem Zugriff auf die kanonische Session.
pub struct HostBridgeSessionHandle {
    session: Mutex<HostBridgeSession>,
}

impl HostBridgeSessionHandle {
    pub(crate) fn new() -> Self {
        Self {
            session: Mutex::new(HostBridgeSession::new()),
        }
    }

    pub(crate) fn with_lock<T>(
        &self,
        f: impl FnOnce(&mut HostBridgeSession) -> Result<T>,
    ) -> Result<T> {
        let mut guard = self
            .session
            .lock()
            .map_err(|_| anyhow!("HostBridgeSession lock poisoned"))?;
        f(&mut guard)
    }
}

pub(crate) fn into_c_string_ptr(value: String) -> *mut c_char {
    CString::new(value)
        .expect("sanitized string must not contain interior NUL bytes")
        .into_raw()
}

pub(crate) fn serialize_json<T: serde::Serialize>(value: &T) -> Result<*mut c_char> {
    let payload = serde_json::to_string(value).context("JSON serialization failed")?;
    Ok(into_c_string_ptr(payload))
}

pub(crate) fn read_json_arg<T>(value: *const c_char, type_name: &str) -> Result<T>
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

pub(crate) fn decode_focus_node_id(focus_node_id_or_neg1: i64) -> Result<Option<u64>> {
    if focus_node_id_or_neg1 == -1 {
        return Ok(None);
    }

    let node_id = u64::try_from(focus_node_id_or_neg1)
        .map_err(|_| anyhow!("focus_node_id must be -1 or a non-negative node id"))?;
    Ok(Some(node_id))
}

pub(crate) fn with_session_mut<T>(
    session: *mut HostBridgeSessionHandle,
    f: impl FnOnce(&mut HostBridgeSession) -> Result<T>,
) -> Result<T> {
    if session.is_null() {
        return Err(anyhow!("HostBridgeSession pointer must not be null"));
    }

    let session = unsafe { &*session };
    session.with_lock(f)
}

#[cfg(feature = "flutter")]
pub(crate) fn with_flutter_session_fallible<T>(
    session: *const FlutterSessionHandle,
    f: impl FnOnce(&mut HostBridgeSession) -> Result<T>,
) -> Result<T> {
    if session.is_null() {
        return Err(anyhow!("FlutterSessionHandle pointer must not be null"));
    }

    let handle = unsafe { &*session };
    handle
        .with_session(|session| f(session))
        .and_then(|result| result)
}
