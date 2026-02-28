//! Statische Menü-Kataloge pro MenuVariant.
//!
//! Definiert welche Commands in welchem Kontext erscheinen.

use super::preconditions::Precondition;
use super::{CommandId, MenuCatalog, MenuEntry};

impl MenuCatalog {
    /// EmptyArea: Tool-Auswahl, optional Streckenteilung.
    pub fn for_empty_area(distanzen_active: bool) -> Self {
        let mut entries = vec![
            MenuEntry::Label("🛠 Werkzeug".into()),
            MenuEntry::Separator,
            MenuEntry::Command {
                id: CommandId::SetToolSelect,
                label: "⭘ Auswahl (1)".into(),
                preconditions: vec![],
            },
            MenuEntry::Command {
                id: CommandId::SetToolConnect,
                label: "⚡ Verbinden (2)".into(),
                preconditions: vec![],
            },
            MenuEntry::Command {
                id: CommandId::SetToolAddNode,
                label: "➕ Node hinzufügen (3)".into(),
                preconditions: vec![],
            },
        ];

        // Streckenteilung nur anzeigen, wenn sie gerade aktiv ist
        if distanzen_active {
            entries.push(MenuEntry::Separator);
            entries.push(MenuEntry::Command {
                id: CommandId::StreckenteilungEmptyArea,
                label: "✂ Streckenteilung".into(),
                preconditions: vec![Precondition::StreckenteilungActive(true)],
            });
        }

        MenuCatalog { entries }
    }

    /// Einzelner Node (noch nicht selektiert).
    pub fn for_single_node_unselected(node_id: u64) -> Self {
        MenuCatalog {
            entries: vec![
                // Info-Header wird separat gerendert (nicht als Command)
                MenuEntry::Separator,
                MenuEntry::Command {
                    id: CommandId::SelectNode,
                    label: "✓ Selektieren".into(),
                    preconditions: vec![Precondition::NodeExists(node_id)],
                },
                MenuEntry::Command {
                    id: CommandId::AddToSelection,
                    label: "⬚ Zur Selektion hinzufügen".into(),
                    preconditions: vec![Precondition::NodeExists(node_id)],
                },
                MenuEntry::Separator,
                MenuEntry::Label("🗺 Marker".into()),
                MenuEntry::Command {
                    id: CommandId::EditMarker,
                    label: "✏ Bearbeiten...".into(),
                    preconditions: vec![
                        Precondition::NodeExists(node_id),
                        Precondition::HasMarker(node_id),
                    ],
                },
                MenuEntry::Command {
                    id: CommandId::RemoveMarker,
                    label: "✕ Marker löschen".into(),
                    preconditions: vec![
                        Precondition::NodeExists(node_id),
                        Precondition::HasMarker(node_id),
                    ],
                },
                MenuEntry::Command {
                    id: CommandId::CreateMarker,
                    label: "🗺 Erstellen...".into(),
                    preconditions: vec![
                        Precondition::NodeExists(node_id),
                        Precondition::HasNoMarker(node_id),
                    ],
                },
            ],
        }
    }

    /// Einzelner Node (bereits selektiert).
    pub fn for_single_node_selected(node_id: u64) -> Self {
        MenuCatalog {
            entries: vec![
                // Info-Header separat (nicht als Command)
                MenuEntry::Separator,
                MenuEntry::Command {
                    id: CommandId::DeselectNode,
                    label: "⬚ Abwählen".into(),
                    preconditions: vec![Precondition::NodeExists(node_id)],
                },
                MenuEntry::Separator,
                MenuEntry::Label("🗺 Marker".into()),
                MenuEntry::Command {
                    id: CommandId::EditMarker,
                    label: "✏ Bearbeiten...".into(),
                    preconditions: vec![
                        Precondition::NodeExists(node_id),
                        Precondition::HasMarker(node_id),
                    ],
                },
                MenuEntry::Command {
                    id: CommandId::RemoveMarker,
                    label: "✕ Löschen".into(),
                    preconditions: vec![
                        Precondition::NodeExists(node_id),
                        Precondition::HasMarker(node_id),
                    ],
                },
                MenuEntry::Command {
                    id: CommandId::CreateMarker,
                    label: "🗺 Erstellen...".into(),
                    preconditions: vec![
                        Precondition::NodeExists(node_id),
                        Precondition::HasNoMarker(node_id),
                    ],
                },
                MenuEntry::Separator,
                MenuEntry::Command {
                    id: CommandId::DeleteSingleNode,
                    label: "✂ Löschen".into(),
                    preconditions: vec![Precondition::NodeExists(node_id)],
                },
                MenuEntry::Command {
                    id: CommandId::DuplicateSingleNode,
                    label: "⧉ Duplizieren".into(),
                    preconditions: vec![Precondition::NodeExists(node_id)],
                },
            ],
        }
    }

    /// Mehrere Nodes selektiert (≥2).
    pub fn for_multiple_nodes_selected() -> Self {
        MenuCatalog {
            entries: vec![
                // Info-Header separat
                // ── Verbinden ────────────────────────────────────────
                MenuEntry::Separator,
                MenuEntry::Command {
                    id: CommandId::ConnectTwoNodes,
                    label: "🔗 Nodes verbinden".into(),
                    preconditions: vec![Precondition::TwoSelectedUnconnected],
                },
                // ── Strecke erzeugen (nur bei 2 Nodes) ───────────────
                MenuEntry::Separator,
                MenuEntry::Label("📐 Strecke erzeugen".into()),
                MenuEntry::Command {
                    id: CommandId::RouteStraight,
                    label: "━ Gerade Strecke".into(),
                    preconditions: vec![Precondition::ExactlyTwoSelected],
                },
                MenuEntry::Command {
                    id: CommandId::RouteQuadratic,
                    label: "⌒ Bézier Grad 2".into(),
                    preconditions: vec![Precondition::ExactlyTwoSelected],
                },
                MenuEntry::Command {
                    id: CommandId::RouteCubic,
                    label: "〜 Bézier Grad 3".into(),
                    preconditions: vec![Precondition::ExactlyTwoSelected],
                },
                // ── Verbindungs-Management ────────────────────────────
                MenuEntry::Separator,
                MenuEntry::Label("Richtung:".into()),
                MenuEntry::Command {
                    id: CommandId::DirectionRegular,
                    label: "↦ Regular (Einbahn)".into(),
                    preconditions: vec![Precondition::HasConnectionsBetweenSelected],
                },
                MenuEntry::Command {
                    id: CommandId::DirectionDual,
                    label: "⇆ Dual (beidseitig)".into(),
                    preconditions: vec![Precondition::HasConnectionsBetweenSelected],
                },
                MenuEntry::Command {
                    id: CommandId::DirectionReverse,
                    label: "↤ Reverse (rückwärts)".into(),
                    preconditions: vec![Precondition::HasConnectionsBetweenSelected],
                },
                MenuEntry::Command {
                    id: CommandId::DirectionInvert,
                    label: "⇄ Invertieren".into(),
                    preconditions: vec![Precondition::HasConnectionsBetweenSelected],
                },
                MenuEntry::Separator,
                MenuEntry::Label("Straßenart:".into()),
                MenuEntry::Command {
                    id: CommandId::PriorityRegular,
                    label: "🛣 Hauptstraße".into(),
                    preconditions: vec![Precondition::HasConnectionsBetweenSelected],
                },
                MenuEntry::Command {
                    id: CommandId::PrioritySub,
                    label: "🛤 Nebenstraße".into(),
                    preconditions: vec![Precondition::HasConnectionsBetweenSelected],
                },
                MenuEntry::Separator,
                MenuEntry::Command {
                    id: CommandId::RemoveAllConnections,
                    label: "✕ Alle trennen".into(),
                    preconditions: vec![Precondition::HasConnectionsBetweenSelected],
                },
                // ── Streckenteilung ──────────────────────────────────
                MenuEntry::Separator,
                MenuEntry::Command {
                    id: CommandId::StreckenteilungMulti,
                    label: "✂ Streckenteilung".into(),
                    preconditions: vec![],
                },
                // ── Selektion ────────────────────────────────────────
                MenuEntry::Separator,
                MenuEntry::Label("📐 Selektion".into()),
                MenuEntry::Command {
                    id: CommandId::InvertSelection,
                    label: "🔄 Invertieren".into(),
                    preconditions: vec![],
                },
                MenuEntry::Command {
                    id: CommandId::SelectAll,
                    label: "Alles auswählen".into(),
                    preconditions: vec![],
                },
                MenuEntry::Command {
                    id: CommandId::ClearSelection,
                    label: "✕ Auswahl löschen".into(),
                    preconditions: vec![],
                },
                // ── Aktionen ─────────────────────────────────────────
                MenuEntry::Separator,
                MenuEntry::Command {
                    id: CommandId::DeleteSelected,
                    label: "✂ Löschen".into(),
                    preconditions: vec![],
                },
                MenuEntry::Command {
                    id: CommandId::DuplicateSelected,
                    label: "⧉ Duplizieren".into(),
                    preconditions: vec![],
                },
            ],
        }
    }

    /// Route-Tool aktiv mit pending input.
    pub fn for_route_tool() -> Self {
        MenuCatalog {
            entries: vec![
                MenuEntry::Label("➤ Route-Tool aktiv".into()),
                MenuEntry::Separator,
                MenuEntry::Command {
                    id: CommandId::RouteExecute,
                    label: "✓ Ausführen".into(),
                    preconditions: vec![],
                },
                MenuEntry::Command {
                    id: CommandId::RouteRecreate,
                    label: "🔄 Neu berechnen".into(),
                    preconditions: vec![],
                },
                MenuEntry::Command {
                    id: CommandId::RouteCancel,
                    label: "✕ Abbrechen".into(),
                    preconditions: vec![],
                },
                // Tangenten werden separat gerendert (dynamisch, nicht als Command)
            ],
        }
    }
}
