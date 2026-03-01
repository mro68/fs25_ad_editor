//! Statische Menü-Kataloge pro MenuVariant.
//!
//! Definiert welche Commands in welchem Kontext erscheinen.

use super::preconditions::Precondition;
use super::{CommandId, MenuCatalog, MenuEntry};

impl MenuCatalog {
    /// EmptyArea: Tool-Auswahl inkl. Route-Tools, optional Streckenteilung.
    pub fn for_empty_area() -> Self {
        let entries = vec![
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
                        label: "Node hinzufügen (3)".into(),
                        preconditions: vec![],
                    },
                ],
            },
            MenuEntry::Submenu {
                label: "📐 Strecke".into(),
                entries: vec![
                    MenuEntry::Command {
                        id: CommandId::SetToolRouteStraight,
                        label: "Gerade Strecke (4)".into(),
                        preconditions: vec![],
                    },
                    MenuEntry::Command {
                        id: CommandId::SetToolRouteQuadratic,
                        label: "Bézier Grad 2 (5)".into(),
                        preconditions: vec![],
                    },
                    MenuEntry::Command {
                        id: CommandId::SetToolRouteCubic,
                        label: "Bézier Grad 3 (6)".into(),
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
                ],
            },
            MenuEntry::Submenu {
                label: "🚧 Straßenart".into(),
                entries: vec![
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
                        label: "☑ Alles auswählen".into(),
                        preconditions: vec![],
                    },
                    MenuEntry::Command {
                        id: CommandId::ClearSelection,
                        label: "✕ Auswahl löschen".into(),
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
            // ── Route-Tools aus Kette ────────────────────────────
            MenuEntry::Submenu {
                label: "📐 Strecke ersetzen".into(),
                entries: vec![
                    MenuEntry::Command {
                        id: CommandId::ChainRouteStraight,
                        label: "Gerade Strecke".into(),
                        preconditions: vec![
                            Precondition::IsResampleableChain,
                            Precondition::StreckenteilungActive(false),
                        ],
                    },
                    MenuEntry::Command {
                        id: CommandId::ChainRouteQuadratic,
                        label: "Bézier Grad 2".into(),
                        preconditions: vec![
                            Precondition::IsResampleableChain,
                            Precondition::StreckenteilungActive(false),
                        ],
                    },
                    MenuEntry::Command {
                        id: CommandId::ChainRouteCubic,
                        label: "Bézier Grad 3".into(),
                        preconditions: vec![
                            Precondition::IsResampleableChain,
                            Precondition::StreckenteilungActive(false),
                        ],
                    },
                ],
            },
            // ── Aktionen ─────────────────────────────────────────
            MenuEntry::Separator,
            MenuEntry::Command {
                id: CommandId::DeleteSelected,
                label: "🗑 Löschen".into(),
                preconditions: vec![],
            },
            MenuEntry::Command {
                id: CommandId::DuplicateSelected,
                label: "⧉ Duplizieren".into(),
                preconditions: vec![],
            },
        ]
    }

    /// SelectionOnly: Befehle für selektierte Nodes (Rechtsklick ins Leere).
    pub fn for_selection_only() -> Self {
        MenuCatalog {
            entries: Self::selection_entries(),
        }
    }

    /// NodeFocused: Einzelnode-Befehle oben + Selektions-Befehle unten.
    pub fn for_node_focused(node_id: u64) -> Self {
        let mut entries = vec![
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
            },
            MenuEntry::Command {
                id: CommandId::DeleteSingleNode,
                label: "🗑 Node löschen".into(),
                preconditions: vec![Precondition::NodeExists(node_id)],
            },
            MenuEntry::Command {
                id: CommandId::DuplicateSingleNode,
                label: "⧉ Node duplizieren".into(),
                preconditions: vec![Precondition::NodeExists(node_id)],
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
