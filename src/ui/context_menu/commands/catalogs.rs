//! Statische Menue-Kataloge pro MenuVariant.
//!
//! Definiert welche Commands in welchem Kontext erscheinen.

use super::preconditions::Precondition;
use super::{CommandId, MenuCatalog, MenuEntry};

impl MenuCatalog {
    /// Werkzeug-Submenu: Auswahl/Verbinden/Hinzufuegen — wird in allen Varianten
    /// ausser RouteToolActive verwendet.
    fn tool_submenu() -> MenuEntry {
        MenuEntry::Submenu {
            label: "🛠 Werkzeug".into(),
            entries: vec![
                MenuEntry::Command {
                    id: CommandId::SetToolSelect,
                    label: "Auswahl (1)".into(),
                    preconditions: vec![],
                },
                MenuEntry::Command {
                    id: CommandId::SetToolConnect,
                    label: "Verbinden (2)".into(),
                    preconditions: vec![],
                },
                MenuEntry::Command {
                    id: CommandId::SetToolAddNode,
                    label: "Node hinzufuegen (3)".into(),
                    preconditions: vec![],
                },
            ],
        }
    }

    /// EmptyArea: Tool-Auswahl inkl. Route-Tools, optional Streckenteilung.
    pub fn for_empty_area() -> Self {
        let entries = vec![
            Self::tool_submenu(),
            MenuEntry::Submenu {
                label: "📐 Strecke".into(),
                entries: vec![
                    MenuEntry::Command {
                        id: CommandId::SetToolRouteConstraint,
                        label: "Constraint-Route (4)".into(),
                        preconditions: vec![],
                    },
                    MenuEntry::Command {
                        id: CommandId::SetToolRouteStraight,
                        label: "Gerade Strecke (5)".into(),
                        preconditions: vec![],
                    },
                    MenuEntry::Command {
                        id: CommandId::SetToolRouteQuadratic,
                        label: "Bézier Grad 2 (6)".into(),
                        preconditions: vec![],
                    },
                    MenuEntry::Command {
                        id: CommandId::SetToolRouteCubic,
                        label: "Bézier Grad 3 (7)".into(),
                        preconditions: vec![],
                    },
                ],
            },
        ];

        MenuCatalog { entries }
    }

    /// Selektions-Befehle (≥1 Nodes selektiert, kein fokussierter Node).
    ///
    /// Wird auch als unterer Teil von `for_node_focused()` verwendet.
    fn selection_entries() -> Vec<MenuEntry> {
        vec![
            // ── Verbinden ────────────────────────────────────────
            MenuEntry::Command {
                id: CommandId::ConnectTwoNodes,
                label: "🔗 Nodes verbinden".into(),
                preconditions: vec![Precondition::TwoSelectedUnconnected],
            },
            // ── Strecke erzeugen (nur bei 2 Nodes) ───────────────
            MenuEntry::Submenu {
                label: "📐 Strecke erzeugen".into(),
                entries: vec![
                    MenuEntry::Command {
                        id: CommandId::RouteConstraint,
                        label: "Constraint-Route".into(),
                        preconditions: vec![Precondition::ExactlyTwoSelected],
                    },
                    MenuEntry::Command {
                        id: CommandId::RouteStraight,
                        label: "Gerade Strecke".into(),
                        preconditions: vec![Precondition::ExactlyTwoSelected],
                    },
                    MenuEntry::Command {
                        id: CommandId::RouteQuadratic,
                        label: "Bézier Grad 2".into(),
                        preconditions: vec![Precondition::ExactlyTwoSelected],
                    },
                    MenuEntry::Command {
                        id: CommandId::RouteCubic,
                        label: "Bézier Grad 3".into(),
                        preconditions: vec![Precondition::ExactlyTwoSelected],
                    },
                ],
            },
            // ── Verbindungs-Management ────────────────────────────
            MenuEntry::Submenu {
                label: "↔ Richtung".into(),
                entries: vec![
                    MenuEntry::Command {
                        id: CommandId::DirectionRegular,
                        label: "↦ Einbahn vorwaerts".into(),
                        preconditions: vec![Precondition::HasConnectionsBetweenSelected],
                    },
                    MenuEntry::Command {
                        id: CommandId::DirectionDual,
                        label: "⇆ Zweirichtungsverkehr".into(),
                        preconditions: vec![Precondition::HasConnectionsBetweenSelected],
                    },
                    MenuEntry::Command {
                        id: CommandId::DirectionReverse,
                        label: "↤ Einbahn rueckwaerts".into(),
                        preconditions: vec![Precondition::HasConnectionsBetweenSelected],
                    },
                    MenuEntry::Command {
                        id: CommandId::DirectionInvert,
                        label: "⇄ Invertieren".into(),
                        preconditions: vec![Precondition::HasConnectionsBetweenSelected],
                    },
                ],
            },
            MenuEntry::Submenu {
                label: "🚧 Strassenart".into(),
                entries: vec![
                    MenuEntry::Command {
                        id: CommandId::PriorityRegular,
                        label: "🛣 Hauptstrasse".into(),
                        preconditions: vec![Precondition::HasConnectionsBetweenSelected],
                    },
                    MenuEntry::Command {
                        id: CommandId::PrioritySub,
                        label: "🛤 Nebenstrasse".into(),
                        preconditions: vec![Precondition::HasConnectionsBetweenSelected],
                    },
                ],
            },
            MenuEntry::Command {
                id: CommandId::RemoveAllConnections,
                label: "✕ Alle trennen".into(),
                preconditions: vec![Precondition::HasConnectionsBetweenSelected],
            },
            // ── Selektion ────────────────────────────────────────
            MenuEntry::Separator,
            MenuEntry::Submenu {
                label: "📐 Selektion".into(),
                entries: vec![
                    MenuEntry::Command {
                        id: CommandId::InvertSelection,
                        label: "🔄 Invertieren".into(),
                        preconditions: vec![],
                    },
                    MenuEntry::Command {
                        id: CommandId::SelectAll,
                        label: "☑ Alles auswaehlen".into(),
                        preconditions: vec![],
                    },
                    MenuEntry::Command {
                        id: CommandId::ClearSelection,
                        label: "✕ Auswahl loeschen".into(),
                        preconditions: vec![],
                    },
                ],
            },
            // ── Streckenteilung ────────────────────────────────────
            MenuEntry::Separator,
            MenuEntry::Command {
                id: CommandId::StreckenteilungMulti,
                label: "📏 Streckenteilung".into(),
                preconditions: vec![
                    Precondition::IsResampleableChain,
                    Precondition::StreckenteilungActive(false),
                ],
            },
        ]
    }

    /// SelectionOnly: Befehle fuer selektierte Nodes (Rechtsklick ins Leere).
    pub fn for_selection_only() -> Self {
        let mut entries = vec![Self::tool_submenu(), MenuEntry::Separator];
        entries.extend(Self::selection_entries());
        // ── Aktionen ─────────────────────────────────────────
        entries.push(MenuEntry::Separator);
        entries.push(MenuEntry::Command {
            id: CommandId::DeleteSelected,
            label: "🗑 Loeschen".into(),
            preconditions: vec![],
        });
        MenuCatalog { entries }
    }

    /// NodeFocused: Werkzeug + Einzelnode-Befehle + Selektions-Befehle + Info.
    pub fn for_node_focused(node_id: u64) -> Self {
        let mut entries = vec![
            Self::tool_submenu(),
            MenuEntry::Separator,
            // ── Einzelnode-Befehle (oberer Bereich) ──────────────
            MenuEntry::Submenu {
                label: "🗺 Marker".into(),
                entries: vec![
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
                        label: "✕ Marker loeschen".into(),
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
            },
            MenuEntry::Command {
                id: CommandId::DeleteSelected,
                label: "🗑 Loeschen".into(),
                preconditions: vec![],
            },
        ];

        // ── Separator zwischen Einzel- und Selektions-Befehlen ───
        entries.push(MenuEntry::Separator);

        // ── Selektions-Befehle (unterer Bereich) ─────────────────
        entries.extend(Self::selection_entries());

        MenuCatalog { entries }
    }

    /// Route-Tool aktiv mit pending input.
    pub fn for_route_tool() -> Self {
        MenuCatalog {
            entries: vec![
                MenuEntry::Label("➤ Route-Tool aktiv".into()),
                MenuEntry::Separator,
                MenuEntry::Command {
                    id: CommandId::RouteExecute,
                    label: "✓ Ausfuehren".into(),
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
