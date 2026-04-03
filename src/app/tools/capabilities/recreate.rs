//! Capability fuer Recreate- und Verkettungs-Lifecycle.

use crate::core::RoadMap;

use super::super::{ToolAnchor, ToolResult};

/// Optionale Capability fuer Tools mit Recreate- oder Verkettungs-Lifecycle.
pub trait RouteToolRecreate {
    /// Uebernimmt die IDs einer gerade angewendeten Tool-Ausfuehrung.
    fn on_applied(&mut self, ids: &[u64], road_map: &RoadMap);

    /// Kompatibilitaetsalias fuer bestehende Aufrufer von `set_last_created()`.
    fn set_last_created(&mut self, ids: &[u64], road_map: &RoadMap) {
        self.on_applied(ids, road_map);
    }

    /// Gibt die IDs der zuletzt erstellten Nodes zurueck.
    fn last_created_ids(&self) -> &[u64];

    /// Gibt den letzten End-Anker fuer Verkettung zurueck.
    fn last_end_anchor(&self) -> Option<ToolAnchor>;

    /// Signalisiert, ob das Tool eine Neuberechnung benoetigt.
    fn needs_recreate(&self) -> bool;

    /// Setzt das Recreate-Flag zurueck.
    fn clear_recreate_flag(&mut self);

    /// Baut ein neues Tool-Ergebnis aus den gespeicherten Ankern auf.
    fn execute_from_anchors(&self, road_map: &RoadMap) -> Option<ToolResult>;
}
