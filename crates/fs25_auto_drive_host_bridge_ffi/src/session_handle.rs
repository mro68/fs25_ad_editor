//! Opaquer Session-Handle fuer serialisierten Zugriff auf die kanonische Host-Bridge-Session.

use anyhow::{anyhow, Result};
use fs25_auto_drive_host_bridge::HostBridgeSession;
use std::sync::Mutex;

/// Opaquer Session-Handle mit serialisiertem Zugriff auf die kanonische Session.
pub struct HostBridgeSessionHandle {
    session: Mutex<HostBridgeSession>,
}

impl HostBridgeSessionHandle {
    /// Erstellt einen neuen Session-Handle mit einer frischen Bridge-Session.
    pub(crate) fn new() -> Self {
        Self {
            session: Mutex::new(HostBridgeSession::new()),
        }
    }

    /// Fuhert eine Operation unter exklusivem Mutex-Lock auf der Session aus.
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

/// Validiert einen Session-Zeiger und fuhert eine Mutation unter Lock aus.
pub(crate) fn with_session_mut<T>(
    session: *mut HostBridgeSessionHandle,
    f: impl FnOnce(&mut HostBridgeSession) -> Result<T>,
) -> Result<T> {
    if session.is_null() {
        return Err(anyhow!("HostBridgeSession pointer must not be null"));
    }

    // SAFETY: Aufrufer garantiert einen gueltigen, durch session_new allozierten Zeiger.
    let session = unsafe { &*session };
    session.with_lock(f)
}
