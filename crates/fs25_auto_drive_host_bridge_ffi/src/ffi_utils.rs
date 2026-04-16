//! Gemeinsame FFI-Hilfsfunktionen und Error-State fuer alle FFI-Submodule.

use anyhow::{anyhow, Context, Result};
use std::cell::RefCell;
use std::ffi::{c_char, CStr, CString};

thread_local! {
    /// Thread-lokaler letzter FFI-Fehlerstring.
    pub(crate) static LAST_ERROR: RefCell<Option<String>> = const { RefCell::new(None) };
}

/// Loescht den gespeicherten letzten FFI-Fehler dieses Threads.
pub(crate) fn clear_last_error() {
    LAST_ERROR.with(|slot| {
        *slot.borrow_mut() = None;
    });
}

/// Speichert einen FFI-Fehlerstring fuer diesen Thread.
pub(crate) fn set_last_error(error: impl Into<String>) {
    let message = error.into().replace('\0', " ");
    LAST_ERROR.with(|slot| {
        *slot.borrow_mut() = Some(message);
    });
}

/// Konvertiert einen `String` in einen heap-allozierten `*mut c_char`.
///
/// Der Aufrufer ist fuer die Freigabe via `fs25ad_host_bridge_string_free` verantwortlich.
#[allow(clippy::expect_used)]
pub(crate) fn into_c_string_ptr(value: String) -> *mut c_char {
    CString::new(value)
        .expect("sanitized string must not contain interior NUL bytes")
        .into_raw()
}

/// Serialisiert einen serializierbaren Wert als JSON-C-String.
pub(crate) fn serialize_json<T: serde::Serialize>(value: &T) -> Result<*mut c_char> {
    let payload = serde_json::to_string(value).context("JSON serialization failed")?;
    Ok(into_c_string_ptr(payload))
}

/// Liest ein JSON-Argument aus einem C-String-Zeiger und deserialisiert es.
pub(crate) fn read_json_arg<T>(value: *const c_char, type_name: &str) -> Result<T>
where
    T: serde::de::DeserializeOwned,
{
    if value.is_null() {
        return Err(anyhow!("{type_name} JSON pointer must not be null"));
    }

    // SAFETY: Aufrufer garantiert einen gueltigen, null-terminierten UTF-8-String oder null.
    let text = unsafe {
        CStr::from_ptr(value)
            .to_str()
            .context("FFI JSON must be valid UTF-8")?
    };
    serde_json::from_str(text).with_context(|| format!("failed to parse {type_name} JSON"))
}

/// Dekodiert einen FFI-Sentinel-Wert fuer eine optionale Node-ID.
///
/// `-1` wird als "kein Fokus-Node" interpretiert; positive Werte sind gueltige Node-IDs.
pub(crate) fn decode_focus_node_id(focus_node_id_or_neg1: i64) -> Result<Option<u64>> {
    if focus_node_id_or_neg1 == -1 {
        return Ok(None);
    }

    let node_id = u64::try_from(focus_node_id_or_neg1)
        .map_err(|_| anyhow!("focus_node_id must be -1 or a non-negative node id"))?;
    Ok(Some(node_id))
}
