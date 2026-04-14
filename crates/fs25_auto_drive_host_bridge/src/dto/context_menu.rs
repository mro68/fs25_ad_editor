//! Kontextmenue-DTOs fuer die Host-Bridge.

use serde::{Deserialize, Serialize};

/// Host-neutrale Variante des aktuell aufgebauten Kontextmenues.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum HostContextMenuVariant {
    /// Rechtsklick auf leeren Bereich ohne aktive Selektion.
    EmptyArea,
    /// Rechtsklick bei vorhandener Selektion ohne fokussierten Node.
    SelectionOnly,
    /// Rechtsklick auf einen spezifischen Node.
    NodeFocused,
    /// Route-Tool mit pending Input hat Prioritaet vor dem normalen Menue.
    RouteToolActive,
}

/// Ein serialisierbarer Kontextmenue-Eintrag fuer den aktuellen Host-Zustand.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct HostContextMenuAction {
    /// Stabile, host-neutrale Aktions-ID.
    pub id: String,
    /// Bereits lokalisierte Beschriftung fuer die aktuelle Host-Sprache.
    pub label: String,
    /// Gibt an, ob die Aktion im aktuellen Zustand ausfuehrbar ist.
    pub enabled: bool,
    /// Optionale stabile Kategorie-ID fuer Gruppierung im Host.
    pub group: Option<String>,
}

/// Host-neutraler Snapshot des aktuell relevanten Kontextmenues.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct HostContextMenuSnapshot {
    /// Variante des aktuell aktiven Kontextmenues.
    pub variant: HostContextMenuVariant,
    /// Optional fokussierter Node fuer node-bezogene Menues.
    pub focus_node_id: Option<u64>,
    /// Alle fuer die aktuelle Menue-Variante relevanten Aktionen inklusive Enablement.
    pub available_actions: Vec<HostContextMenuAction>,
}

#[cfg(test)]
mod tests {
    use super::{HostContextMenuAction, HostContextMenuSnapshot, HostContextMenuVariant};

    #[test]
    fn context_menu_snapshot_roundtrips_via_serde_json() {
        let snapshot = HostContextMenuSnapshot {
            variant: HostContextMenuVariant::NodeFocused,
            focus_node_id: Some(42),
            available_actions: vec![
                HostContextMenuAction {
                    id: "edit_marker".to_string(),
                    label: "Marker bearbeiten".to_string(),
                    enabled: true,
                    group: Some("marker".to_string()),
                },
                HostContextMenuAction {
                    id: "paste_here".to_string(),
                    label: "Einfuegen".to_string(),
                    enabled: false,
                    group: Some("clipboard".to_string()),
                },
            ],
        };

        let json = serde_json::to_string(&snapshot)
            .expect("Kontextmenue-Snapshot muss serialisierbar sein");
        let parsed: HostContextMenuSnapshot =
            serde_json::from_str(&json).expect("Kontextmenue-Snapshot muss parsebar bleiben");

        assert_eq!(parsed, snapshot);
        assert!(json.contains("available_actions"));
        assert!(json.contains("edit_marker"));
    }
}
