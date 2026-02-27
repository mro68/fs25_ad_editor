//! Validiertes Context-Menu-System: Command-Definitionen, Preconditions und Kataloge.
//!
//! Architektur:
//! - `CommandId`: Eindeutige Identifikation jedes Menü-Eintrags
//! - `Precondition`: Enum mit Vorbedingungen, die zur Laufzeit geprüft werden
//! - `MenuCatalog`: Statischer Katalog pro `MenuVariant` (welche Commands gehören dazu?)
//! - `validate_entries()`: Filtert nur gültige Commands für die aktuelle Situation
//!
//! Garantie: Nur Commands mit erfüllten Preconditions werden gerendert.

use crate::app::segment_registry::{
    TOOL_INDEX_CURVE_CUBIC, TOOL_INDEX_CURVE_QUAD, TOOL_INDEX_STRAIGHT,
};
use crate::app::{AppIntent, ConnectionDirection, ConnectionPriority, EditorTool, RoadMap};
use std::collections::HashSet;

// =============================================================================
// CommandId — Eindeutige Identifikation jedes Menü-Befehls
// =============================================================================

/// Eindeutige ID für jeden Context-Menu-Befehl.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum CommandId {
    // ── EmptyArea ────────────────────────────────────────────────────
    /// Auswahl-Werkzeug aktivieren
    SetToolSelect,
    /// Verbinden-Werkzeug aktivieren
    SetToolConnect,
    /// Node-Hinzufügen-Werkzeug aktivieren
    SetToolAddNode,
    /// Streckenteilung (in EmptyArea nur wenn aktiv)
    StreckenteilungEmptyArea,

    // ── SingleNode (unselected) ─────────────────────────────────────
    /// Node selektieren (exklusiv)
    SelectNode,
    /// Node zur Selektion hinzufügen
    AddToSelection,
    /// Marker erstellen
    CreateMarker,
    /// Marker bearbeiten
    EditMarker,
    /// Marker löschen
    RemoveMarker,

    // ── SingleNode (selected) ───────────────────────────────────────
    /// Node aus Selektion entfernen
    DeselectNode,
    /// Node löschen (bei Einzel-Selektion)
    DeleteSingleNode,
    /// Node duplizieren (bei Einzel-Selektion)
    DuplicateSingleNode,

    // ── MultipleNodes ───────────────────────────────────────────────
    /// Zwei Nodes verbinden (nur bei genau 2 unverbundenen)
    ConnectTwoNodes,
    /// Gerade Strecke erzeugen (2 Nodes)
    RouteStraight,
    /// Bézier Grad 2 erzeugen (2 Nodes)
    RouteQuadratic,
    /// Bézier Grad 3 erzeugen (2 Nodes)
    RouteCubic,
    /// Richtung: Regular setzen
    DirectionRegular,
    /// Richtung: Dual setzen
    DirectionDual,
    /// Richtung: Reverse setzen
    DirectionReverse,
    /// Richtung: Invertieren
    DirectionInvert,
    /// Priorität: Hauptstraße
    PriorityRegular,
    /// Priorität: Nebenstraße
    PrioritySub,
    /// Alle Verbindungen trennen
    RemoveAllConnections,
    /// Streckenteilung (bei MultipleNodes)
    StreckenteilungMulti,
    /// Selektion invertieren
    InvertSelection,
    /// Alle Nodes auswählen
    SelectAll,
    /// Selektion aufheben
    ClearSelection,
    /// Selektierte Nodes löschen
    DeleteSelected,
    /// Selektierte Nodes duplizieren
    DuplicateSelected,

    // ── RouteTool ────────────────────────────────────────────────────
    /// Route ausführen
    RouteExecute,
    /// Route neu berechnen
    RouteRecreate,
    /// Route abbrechen
    RouteCancel,
}

// =============================================================================
// Precondition — Vorbedingungen als Enum (kein dyn Trait, performant)
// =============================================================================

/// Prüfbare Vorbedingung für einen Context-Menu-Befehl.
#[derive(Debug, Clone, Copy)]
pub enum Precondition {
    /// Node existiert noch in der RoadMap
    NodeExists(u64),
    /// Node hat einen Marker
    HasMarker(u64),
    /// Node hat keinen Marker
    HasNoMarker(u64),
    /// Genau 2 Nodes selektiert
    ExactlyTwoSelected,
    /// Genau 2 Nodes selektiert UND keine Verbindung dazwischen
    TwoSelectedUnconnected,
    /// Es gibt Verbindungen zwischen selektierten Nodes
    HasConnectionsBetweenSelected,
    /// Streckenteilung ist aktiv
    StreckenteilungActive(bool),
}

/// Kontext für die Precondition-Auswertung — alle nötigen Daten aus dem aktuellen State.
pub struct PreconditionContext<'a> {
    pub road_map: &'a RoadMap,
    pub selected_node_ids: &'a HashSet<u64>,
    /// Ob die Streckenteilung gerade aktiv ist
    pub distanzen_active: bool,
}

impl Precondition {
    /// Prüft ob die Vorbedingung im gegebenen Kontext erfüllt ist.
    pub fn is_valid(&self, ctx: &PreconditionContext) -> bool {
        match self {
            Self::NodeExists(id) => ctx.road_map.nodes.contains_key(id),

            Self::HasMarker(id) => ctx.road_map.has_marker(*id),

            Self::HasNoMarker(id) => !ctx.road_map.has_marker(*id),

            Self::ExactlyTwoSelected => ctx.selected_node_ids.len() == 2,

            Self::TwoSelectedUnconnected => {
                if ctx.selected_node_ids.len() != 2 {
                    return false;
                }
                let ids: Vec<u64> = ctx.selected_node_ids.iter().copied().collect();
                let (a, b) = (ids[0], ids[1]);
                // Keine Verbindung in beide Richtungen
                !ctx.road_map.has_connection(a, b) && !ctx.road_map.has_connection(b, a)
            }

            Self::HasConnectionsBetweenSelected => {
                ctx.road_map.connections_between_ids(ctx.selected_node_ids).next().is_some()
            },

            Self::StreckenteilungActive(expected) => ctx.distanzen_active == *expected,
        }
    }
}

// =============================================================================
// MenuEntry + MenuCatalog — Statische Beschreibung der Menü-Struktur
// =============================================================================

/// Ein einzelner Eintrag im Menü-Katalog.
#[derive(Debug, Clone)]
pub enum MenuEntry {
    /// Überschrift / Label
    Label(String),
    /// Trennlinie
    Separator,
    /// Befehl mit optionalen Vorbedingungen
    Command {
        id: CommandId,
        label: String,
        preconditions: Vec<Precondition>,
    },
}

/// Katalog für eine bestimmte `MenuVariant` — definiert Reihenfolge und Preconditions.
#[derive(Debug, Clone)]
pub struct MenuCatalog {
    pub entries: Vec<MenuEntry>,
}

// =============================================================================
// Intent-Erzeugung — Wie wird aus einem CommandId ein AppIntent?
// =============================================================================

/// Kontext für die Intent-Erzeugung — enthält Node-IDs und Tool-Daten.
pub struct IntentContext {
    /// Node-ID (für SingleNode-Varianten)
    pub node_id: Option<u64>,
    /// Node-Position (für NodePick)
    pub node_position: Option<glam::Vec2>,
    /// Sortierte Zwei-Node-IDs (für RouteToolWithAnchorsRequested)
    pub two_node_ids: Option<(u64, u64)>,
}

impl CommandId {
    /// Erzeugt den passenden `AppIntent` für diesen Command.
    pub fn to_intent(&self, ctx: &IntentContext) -> AppIntent {
        match self {
            // ── EmptyArea ────────────────────────────────────────────
            Self::SetToolSelect => AppIntent::SetEditorToolRequested {
                tool: EditorTool::Select,
            },
            Self::SetToolConnect => AppIntent::SetEditorToolRequested {
                tool: EditorTool::Connect,
            },
            Self::SetToolAddNode => AppIntent::SetEditorToolRequested {
                tool: EditorTool::AddNode,
            },
            Self::StreckenteilungEmptyArea | Self::StreckenteilungMulti => {
                AppIntent::StreckenteilungAktivieren
            }

            // ── SingleNode (unselected) ──────────────────────────────
            Self::SelectNode => AppIntent::NodePickRequested {
                world_pos: ctx.node_position.unwrap_or_default(),
                additive: false,
                extend_path: false,
            },
            Self::AddToSelection => AppIntent::NodePickRequested {
                world_pos: ctx.node_position.unwrap_or_default(),
                additive: true,
                extend_path: false,
            },
            Self::CreateMarker => AppIntent::CreateMarkerRequested {
                node_id: ctx.node_id.unwrap_or(0),
            },
            Self::EditMarker => AppIntent::EditMarkerRequested {
                node_id: ctx.node_id.unwrap_or(0),
            },
            Self::RemoveMarker => AppIntent::RemoveMarkerRequested {
                node_id: ctx.node_id.unwrap_or(0),
            },

            // ── SingleNode (selected) ────────────────────────────────
            Self::DeselectNode => AppIntent::NodePickRequested {
                world_pos: ctx.node_position.unwrap_or_default(),
                additive: true,
                extend_path: false,
            },
            Self::DeleteSingleNode | Self::DeleteSelected => AppIntent::DeleteSelectedRequested,
            Self::DuplicateSingleNode | Self::DuplicateSelected => {
                AppIntent::DuplicateSelectedNodesRequested
            }

            // ── MultipleNodes ────────────────────────────────────────
            Self::ConnectTwoNodes => AppIntent::ConnectSelectedNodesRequested,
            Self::RouteStraight => {
                let (s, e) = ctx.two_node_ids.unwrap_or((0, 0));
                AppIntent::RouteToolWithAnchorsRequested {
                    index: TOOL_INDEX_STRAIGHT,
                    start_node_id: s,
                    end_node_id: e,
                }
            }
            Self::RouteQuadratic => {
                let (s, e) = ctx.two_node_ids.unwrap_or((0, 0));
                AppIntent::RouteToolWithAnchorsRequested {
                    index: TOOL_INDEX_CURVE_QUAD,
                    start_node_id: s,
                    end_node_id: e,
                }
            }
            Self::RouteCubic => {
                let (s, e) = ctx.two_node_ids.unwrap_or((0, 0));
                AppIntent::RouteToolWithAnchorsRequested {
                    index: TOOL_INDEX_CURVE_CUBIC,
                    start_node_id: s,
                    end_node_id: e,
                }
            }
            Self::DirectionRegular => {
                AppIntent::SetAllConnectionsDirectionBetweenSelectedRequested {
                    direction: ConnectionDirection::Regular,
                }
            }
            Self::DirectionDual => AppIntent::SetAllConnectionsDirectionBetweenSelectedRequested {
                direction: ConnectionDirection::Dual,
            },
            Self::DirectionReverse => {
                AppIntent::SetAllConnectionsDirectionBetweenSelectedRequested {
                    direction: ConnectionDirection::Reverse,
                }
            }
            Self::DirectionInvert => AppIntent::InvertAllConnectionsBetweenSelectedRequested,
            Self::PriorityRegular => AppIntent::SetAllConnectionsPriorityBetweenSelectedRequested {
                priority: ConnectionPriority::Regular,
            },
            Self::PrioritySub => AppIntent::SetAllConnectionsPriorityBetweenSelectedRequested {
                priority: ConnectionPriority::SubPriority,
            },
            Self::RemoveAllConnections => AppIntent::RemoveAllConnectionsBetweenSelectedRequested,
            Self::InvertSelection => AppIntent::InvertSelectionRequested,
            Self::SelectAll => AppIntent::SelectAllRequested,
            Self::ClearSelection => AppIntent::ClearSelectionRequested,

            // ── RouteTool ────────────────────────────────────────────
            Self::RouteExecute => AppIntent::RouteToolExecuteRequested,
            Self::RouteRecreate => AppIntent::RouteToolRecreateRequested,
            Self::RouteCancel => AppIntent::RouteToolCancelled,
        }
    }
}

// =============================================================================
// Katalog-Definitionen pro MenuVariant
// =============================================================================

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

// =============================================================================
// Validator — Filtert gültige Einträge
// =============================================================================

/// Prüft ob alle Preconditions eines Menu-Eintrags erfüllt sind.
pub(crate) fn all_preconditions_valid(preconditions: &[Precondition], ctx: &PreconditionContext) -> bool {
    preconditions.iter().all(|p| p.is_valid(ctx))
}

/// Ergebnis der Validierung: Sichtbare Einträge mit ihrem Intent.
#[derive(Debug)]
pub enum ValidatedEntry {
    /// Label (immer sichtbar)
    Label(String),
    /// Trennlinie (wird nur angezeigt wenn umgebende Commands sichtbar sind)
    Separator,
    /// Gültiger Befehl mit fertigem Intent
    Command {
        #[allow(dead_code)]
        id: CommandId,
        label: String,
        intent: Box<AppIntent>,
    },
}

/// Validiert einen MenuCatalog und gibt nur die sichtbaren Einträge zurück.
///
/// Separatoren werden intelligent gefiltert: Doppelte Separatoren und
/// Separatoren am Anfang/Ende werden entfernt.
pub fn validate_entries(
    catalog: &MenuCatalog,
    precondition_ctx: &PreconditionContext,
    intent_ctx: &IntentContext,
) -> Vec<ValidatedEntry> {
    let mut raw: Vec<ValidatedEntry> = Vec::new();

    for entry in &catalog.entries {
        match entry {
            MenuEntry::Label(text) => {
                raw.push(ValidatedEntry::Label(text.clone()));
            }
            MenuEntry::Separator => {
                raw.push(ValidatedEntry::Separator);
            }
            MenuEntry::Command {
                id,
                label,
                preconditions,
            } => {
                if all_preconditions_valid(preconditions, precondition_ctx) {
                    raw.push(ValidatedEntry::Command {
                        id: *id,
                        label: label.clone(),
                        intent: Box::new(id.to_intent(intent_ctx)),
                    });
                }
            }
        }
    }

    // Separatoren bereinigen: keine doppelten, keine am Anfang/Ende,
    // keine direkt nach Label ohne folgendem Command
    cleanup_separators(raw)
}

/// Entfernt überflüssige Separatoren und Labels ohne folgende Commands.
pub(crate) fn cleanup_separators(entries: Vec<ValidatedEntry>) -> Vec<ValidatedEntry> {
    let mut result: Vec<ValidatedEntry> = Vec::new();

    for entry in entries {
        match &entry {
            ValidatedEntry::Separator => {
                // Separator nur wenn vorheriger Eintrag kein Separator ist und es vorherige Einträge gibt
                if !result.is_empty() && !matches!(result.last(), Some(ValidatedEntry::Separator)) {
                    result.push(entry);
                }
            }
            _ => {
                result.push(entry);
            }
        }
    }

    // Trailing Separator entfernen
    if matches!(result.last(), Some(ValidatedEntry::Separator)) {
        result.pop();
    }

    // Labels ohne nachfolgende Commands entfernen (Sektion ohne Einträge)
    remove_orphaned_labels(result)
}

/// Entfernt Labels die nicht von mindestens einem Command gefolgt werden
/// (bevor der nächste Separator oder das Ende kommt).
pub(crate) fn remove_orphaned_labels(entries: Vec<ValidatedEntry>) -> Vec<ValidatedEntry> {
    let len = entries.len();
    // Markiere welche Indizes behalten werden
    let mut keep = vec![true; len];

    for i in 0..len {
        if matches!(&entries[i], ValidatedEntry::Label(_)) {
            // Prüfe ob nach diesem Label (bis zum nächsten Separator/Label/Ende) ein Command kommt
            let has_following_command = entries[i + 1..]
                .iter()
                .take_while(|e| !matches!(e, ValidatedEntry::Separator | ValidatedEntry::Label(_)))
                .any(|e| matches!(e, ValidatedEntry::Command { .. }));

            if !has_following_command {
                keep[i] = false;
            }
        }
    }

    entries
        .into_iter()
        .enumerate()
        .filter(|(i, _)| keep[*i])
        .map(|(_, e)| e)
        .collect()
}

#[cfg(test)]
mod tests;
