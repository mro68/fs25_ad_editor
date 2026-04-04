//! Statische Menue-Kataloge pro MenuVariant.
//!
//! Definiert welche Commands in welchem Kontext erscheinen.

use super::preconditions::Precondition;
use super::{CommandId, MenuCatalog, MenuEntry};
use crate::shared::{t, I18nKey, Language};

impl MenuCatalog {
    /// Werkzeug-Submenu: Auswahl/Verbinden/Hinzufuegen — wird in allen Varianten
    /// ausser RouteToolActive verwendet.
    fn tool_submenu(lang: Language) -> MenuEntry {
        MenuEntry::Submenu {
            label: t(lang, I18nKey::CtxToolSubmenu).into(),
            entries: vec![
                MenuEntry::Command {
                    id: CommandId::SetToolSelect,
                    label: t(lang, I18nKey::CtxToolSelect).into(),
                    preconditions: vec![],
                },
                MenuEntry::Command {
                    id: CommandId::SetToolConnect,
                    label: t(lang, I18nKey::CtxToolConnect).into(),
                    preconditions: vec![],
                },
                MenuEntry::Command {
                    id: CommandId::SetToolAddNode,
                    label: t(lang, I18nKey::CtxToolAddNode).into(),
                    preconditions: vec![],
                },
            ],
        }
    }

    /// Zoom-Submenu: Auf gesamte Map oder auf Selektion zoomen.
    fn zoom_submenu(lang: Language) -> MenuEntry {
        MenuEntry::Submenu {
            label: t(lang, I18nKey::CtxZoomSubmenu).into(),
            entries: vec![
                MenuEntry::Command {
                    id: CommandId::ZoomToFit,
                    label: t(lang, I18nKey::CtxZoomFullMap).into(),
                    preconditions: vec![],
                },
                MenuEntry::Command {
                    id: CommandId::ZoomToSelection,
                    label: t(lang, I18nKey::CtxZoomSelection).into(),
                    preconditions: vec![Precondition::AtLeastTwoSelected],
                },
            ],
        }
    }

    /// EmptyArea: Tool-Auswahl inkl. Route-Tools, optional Streckenteilung.
    pub fn for_empty_area(lang: Language) -> Self {
        let entries = vec![
            Self::tool_submenu(lang),
            Self::zoom_submenu(lang),
            MenuEntry::Submenu {
                label: t(lang, I18nKey::CtxRouteSubmenu).into(),
                entries: vec![
                    MenuEntry::Command {
                        id: CommandId::SetToolRouteSmoothCurve,
                        label: t(lang, I18nKey::CtxRouteSmoothCurve).into(),
                        preconditions: vec![],
                    },
                    MenuEntry::Command {
                        id: CommandId::SetToolRouteStraight,
                        label: t(lang, I18nKey::CtxRouteStraight).into(),
                        preconditions: vec![],
                    },
                    MenuEntry::Command {
                        id: CommandId::SetToolRouteQuadratic,
                        label: t(lang, I18nKey::CtxRouteQuadratic).into(),
                        preconditions: vec![],
                    },
                    MenuEntry::Command {
                        id: CommandId::SetToolRouteCubic,
                        label: t(lang, I18nKey::CtxRouteCubic).into(),
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
    fn selection_entries(lang: Language) -> Vec<MenuEntry> {
        vec![
            // ── Segment bearbeiten ────────────────────────────────────────────────
            MenuEntry::Command {
                id: CommandId::EditGroup,
                label: t(lang, I18nKey::CtxEditGroup).into(),
                preconditions: vec![Precondition::SelectionIsValidSegment],
            },
            MenuEntry::Command {
                id: CommandId::GroupSelectionAsGroup,
                label: t(lang, I18nKey::CtxGroupAsSegment).into(),
                preconditions: vec![
                    Precondition::IsConnectedSubgraph,
                    Precondition::NoGroupEditActive,
                ],
            },
            MenuEntry::Command {
                id: CommandId::RemoveFromGroup,
                label: t(lang, I18nKey::CtxRemoveFromGroup).into(),
                preconditions: vec![Precondition::SelectionHasGroupMember],
            },
            MenuEntry::Command {
                id: CommandId::DissolveGroup,
                label: t(lang, I18nKey::CtxDissolveGroup).into(),
                preconditions: vec![Precondition::SelectionHasGroupMember],
            },
            MenuEntry::Separator,
            // ── Verbinden ────────────────────────────────────────
            MenuEntry::Command {
                id: CommandId::ConnectTwoNodes,
                label: t(lang, I18nKey::CtxConnectNodes).into(),
                preconditions: vec![Precondition::TwoSelectedUnconnected],
            },
            // ── Strecke erzeugen (nur bei 2 Nodes) ───────────────
            MenuEntry::Submenu {
                label: t(lang, I18nKey::CtxCreateRoute).into(),
                entries: vec![
                    MenuEntry::Command {
                        id: CommandId::RouteSmoothCurve,
                        label: t(lang, I18nKey::CtxRouteSmoothCurve).into(),
                        preconditions: vec![Precondition::ExactlyTwoSelected],
                    },
                    MenuEntry::Command {
                        id: CommandId::RouteStraight,
                        label: t(lang, I18nKey::CtxRouteStraight).into(),
                        preconditions: vec![Precondition::ExactlyTwoSelected],
                    },
                    MenuEntry::Command {
                        id: CommandId::RouteQuadratic,
                        label: t(lang, I18nKey::CtxRouteQuadratic).into(),
                        preconditions: vec![Precondition::ExactlyTwoSelected],
                    },
                    MenuEntry::Command {
                        id: CommandId::RouteCubic,
                        label: t(lang, I18nKey::CtxRouteCubic).into(),
                        preconditions: vec![Precondition::ExactlyTwoSelected],
                    },
                ],
            },
            // ── Verbindungs-Management ────────────────────────────
            MenuEntry::Submenu {
                label: t(lang, I18nKey::CtxDirectionSubmenu).into(),
                entries: vec![
                    MenuEntry::Command {
                        id: CommandId::DirectionRegular,
                        label: t(lang, I18nKey::CtxDirectionRegular).into(),
                        preconditions: vec![Precondition::HasConnectionsBetweenSelected],
                    },
                    MenuEntry::Command {
                        id: CommandId::DirectionDual,
                        label: t(lang, I18nKey::CtxDirectionDual).into(),
                        preconditions: vec![Precondition::HasConnectionsBetweenSelected],
                    },
                    MenuEntry::Command {
                        id: CommandId::DirectionReverse,
                        label: t(lang, I18nKey::CtxDirectionReverse).into(),
                        preconditions: vec![Precondition::HasConnectionsBetweenSelected],
                    },
                    MenuEntry::Command {
                        id: CommandId::DirectionInvert,
                        label: t(lang, I18nKey::CtxDirectionInvert).into(),
                        preconditions: vec![Precondition::HasConnectionsBetweenSelected],
                    },
                ],
            },
            MenuEntry::Submenu {
                label: t(lang, I18nKey::CtxPrioritySubmenu).into(),
                entries: vec![
                    MenuEntry::Command {
                        id: CommandId::PriorityRegular,
                        label: t(lang, I18nKey::CtxPriorityMain).into(),
                        preconditions: vec![Precondition::HasConnectionsBetweenSelected],
                    },
                    MenuEntry::Command {
                        id: CommandId::PrioritySub,
                        label: t(lang, I18nKey::CtxPrioritySub).into(),
                        preconditions: vec![Precondition::HasConnectionsBetweenSelected],
                    },
                ],
            },
            MenuEntry::Command {
                id: CommandId::RemoveAllConnections,
                label: t(lang, I18nKey::CtxRemoveAllConnections).into(),
                preconditions: vec![Precondition::HasConnectionsBetweenSelected],
            },
            // ── Selektion ────────────────────────────────────────
            MenuEntry::Separator,
            MenuEntry::Submenu {
                label: t(lang, I18nKey::CtxSelectionSubmenu).into(),
                entries: vec![
                    MenuEntry::Command {
                        id: CommandId::InvertSelection,
                        label: t(lang, I18nKey::CtxSelectionInvert).into(),
                        preconditions: vec![],
                    },
                    MenuEntry::Command {
                        id: CommandId::SelectAll,
                        label: t(lang, I18nKey::CtxSelectAll).into(),
                        preconditions: vec![],
                    },
                    MenuEntry::Command {
                        id: CommandId::ClearSelection,
                        label: t(lang, I18nKey::CtxClearSelection).into(),
                        preconditions: vec![],
                    },
                ],
            },
            // ── Streckenteilung ────────────────────────────────────
            MenuEntry::Separator,
            MenuEntry::Command {
                id: CommandId::StreckenteilungMulti,
                label: t(lang, I18nKey::CtxStreckenteilung).into(),
                preconditions: vec![
                    Precondition::IsResampleableChain,
                    Precondition::StreckenteilungActive(false),
                ],
            },
        ]
    }

    /// SelectionOnly: Befehle fuer selektierte Nodes (Rechtsklick ins Leere).
    pub fn for_selection_only(lang: Language) -> Self {
        let mut entries = vec![
            Self::tool_submenu(lang),
            Self::zoom_submenu(lang),
            MenuEntry::Separator,
        ];
        entries.extend(Self::selection_entries(lang));
        // ── Aktionen ─────────────────────────────────────────
        entries.push(MenuEntry::Separator);
        entries.push(MenuEntry::Command {
            id: CommandId::DeleteSelected,
            label: t(lang, I18nKey::CtxDeleteSelected).into(),
            preconditions: vec![],
        });
        // ── Copy/Paste ────────────────────────────────────────
        entries.push(MenuEntry::Separator);
        entries.push(MenuEntry::Command {
            id: CommandId::CopySelection,
            label: t(lang, I18nKey::CtxCopy).into(),
            preconditions: vec![Precondition::HasSelection],
        });
        entries.push(MenuEntry::Command {
            id: CommandId::PasteHere,
            label: t(lang, I18nKey::CtxPaste).into(),
            preconditions: vec![Precondition::ClipboardHasData],
        });
        MenuCatalog { entries }
    }

    /// NodeFocused: Werkzeug + Einzelnode-Befehle + Selektions-Befehle + Info.
    pub fn for_node_focused(node_id: u64, lang: Language) -> Self {
        let mut entries = vec![
            Self::tool_submenu(lang),
            Self::zoom_submenu(lang),
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
                label: t(lang, I18nKey::CtxDeleteSelected).into(),
                preconditions: vec![],
            },
        ];

        // ── Separator zwischen Einzel- und Selektions-Befehlen ───
        entries.push(MenuEntry::Separator);

        // ── Selektions-Befehle (unterer Bereich) ─────────────────
        entries.extend(Self::selection_entries(lang));

        // ── Copy/Paste ────────────────────────────────────────────
        entries.push(MenuEntry::Separator);
        entries.push(MenuEntry::Command {
            id: CommandId::CopySelection,
            label: t(lang, I18nKey::CtxCopy).into(),
            preconditions: vec![Precondition::HasSelection],
        });
        entries.push(MenuEntry::Command {
            id: CommandId::PasteHere,
            label: t(lang, I18nKey::CtxPaste).into(),
            preconditions: vec![Precondition::ClipboardHasData],
        });

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
