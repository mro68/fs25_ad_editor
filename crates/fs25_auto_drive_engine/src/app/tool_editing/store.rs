//! Separater Store fuer tool-spezifische Edit-Payloads.

use std::collections::HashMap;

use crate::app::tool_contract::RouteToolId;

use super::RouteToolEditPayload;

/// Persistenter Edit-Eintrag fuer eine gruppenbasierte Tool-Ausfuehrung.
#[derive(Debug, Clone)]
pub struct ToolEditRecord {
    /// Zugehoerige Gruppen-ID in der Registry.
    pub group_id: u64,
    /// Stabile Tool-ID fuer die Rehydrierung.
    pub tool_id: RouteToolId,
    /// Tool-spezifischer Edit-Snapshot.
    pub payload: RouteToolEditPayload,
}

/// Session-Store fuer tool-editierbare Gruppen.
#[derive(Debug, Clone, Default)]
pub struct ToolEditStore {
    records: HashMap<u64, ToolEditRecord>,
}

impl ToolEditStore {
    /// Erstellt einen leeren ToolEditStore.
    pub fn new() -> Self {
        Self::default()
    }

    /// Registriert oder ersetzt einen Tool-Edit-Eintrag.
    pub fn insert(&mut self, record: ToolEditRecord) {
        self.records.insert(record.group_id, record);
    }

    /// Liefert den Tool-Edit-Eintrag einer Gruppe.
    pub fn get(&self, group_id: u64) -> Option<&ToolEditRecord> {
        self.records.get(&group_id)
    }

    /// Entfernt und liefert den Tool-Edit-Eintrag einer Gruppe.
    pub fn remove(&mut self, group_id: u64) -> Option<ToolEditRecord> {
        self.records.remove(&group_id)
    }

    /// Entfernt mehrere Tool-Edit-Eintraege in einem Schritt.
    pub fn remove_many<I>(&mut self, group_ids: I)
    where
        I: IntoIterator<Item = u64>,
    {
        for group_id in group_ids {
            self.records.remove(&group_id);
        }
    }

    /// Gibt `true` zurueck wenn fuer die Gruppe ein Tool-Edit-Eintrag existiert.
    pub fn contains(&self, group_id: u64) -> bool {
        self.records.contains_key(&group_id)
    }

    /// Liefert die Tool-ID einer gruppenbasierten Payload.
    pub fn tool_id_for(&self, group_id: u64) -> Option<RouteToolId> {
        self.get(group_id).map(|record| record.tool_id)
    }
}
