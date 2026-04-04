//! Capability fuer persistierbare Tool-Edit-Payloads.

use crate::app::tool_editing::RouteToolEditPayload;

/// Optionale Capability fuer group-backed Tools mit separatem Edit-Snapshot.
pub trait RouteToolGroupEdit {
    /// Baut einen tool-spezifischen Edit-Snapshot aus dem aktuellen Zustand.
    fn build_edit_payload(&self) -> Option<RouteToolEditPayload>;

    /// Stellt einen zuvor gespeicherten Edit-Snapshot wieder her.
    fn restore_edit_payload(&mut self, payload: &RouteToolEditPayload);
}
