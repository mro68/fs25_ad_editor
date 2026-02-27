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

            Self::HasConnectionsBetweenSelected => ctx.road_map.connections_iter().any(|c| {
                ctx.selected_node_ids.contains(&c.start_id)
                    && ctx.selected_node_ids.contains(&c.end_id)
            }),

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
fn all_preconditions_valid(preconditions: &[Precondition], ctx: &PreconditionContext) -> bool {
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
        intent: AppIntent,
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
                        intent: id.to_intent(intent_ctx),
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
fn cleanup_separators(entries: Vec<ValidatedEntry>) -> Vec<ValidatedEntry> {
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
fn remove_orphaned_labels(entries: Vec<ValidatedEntry>) -> Vec<ValidatedEntry> {
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
mod tests {
    use super::*;
    use crate::core::{
        Connection, ConnectionDirection, ConnectionPriority, MapMarker, MapNode, NodeFlag,
    };
    use glam::Vec2;

    /// Erstellt eine RoadMap mit gegebenen Nodes (IDs und Positionen).
    fn make_road_map(nodes: &[(u64, f32, f32)]) -> RoadMap {
        let mut map = RoadMap::new(3);
        for &(id, x, y) in nodes {
            map.add_node(MapNode::new(id, Vec2::new(x, y), NodeFlag::Regular));
        }
        map
    }

    /// Erstellt eine RoadMap mit 2 Nodes und einer Verbindung dazwischen.
    fn make_connected_map(id_a: u64, id_b: u64) -> RoadMap {
        let mut map = make_road_map(&[(id_a, 0.0, 0.0), (id_b, 10.0, 10.0)]);
        let pos_a = map.nodes[&id_a].position;
        let pos_b = map.nodes[&id_b].position;
        map.add_connection(Connection::new(
            id_a,
            id_b,
            ConnectionDirection::Regular,
            ConnectionPriority::Regular,
            pos_a,
            pos_b,
        ));
        map
    }

    /// Zählt Commands in der validierten Entry-Liste.
    fn count_commands(entries: &[ValidatedEntry]) -> usize {
        entries
            .iter()
            .filter(|e| matches!(e, ValidatedEntry::Command { .. }))
            .count()
    }

    /// Prüft ob ein bestimmter CommandId in den Entries enthalten ist.
    fn has_command(entries: &[ValidatedEntry], target: CommandId) -> bool {
        entries
            .iter()
            .any(|e| matches!(e, ValidatedEntry::Command { id, .. } if *id == target))
    }

    // ═══════════════════════════════════════════════════════════════════
    // Precondition-Tests
    // ═══════════════════════════════════════════════════════════════════

    #[test]
    fn precondition_node_exists() {
        let map = make_road_map(&[(1, 0.0, 0.0)]);
        let selected = HashSet::new();
        let ctx = PreconditionContext {
            road_map: &map,
            selected_node_ids: &selected,
            distanzen_active: false,
        };

        assert!(Precondition::NodeExists(1).is_valid(&ctx));
        assert!(!Precondition::NodeExists(999).is_valid(&ctx));
    }

    #[test]
    fn precondition_has_marker() {
        let mut map = make_road_map(&[(1, 0.0, 0.0), (2, 10.0, 0.0)]);
        map.add_map_marker(MapMarker::new(1, "Test".into(), "Default".into(), 1, false));
        let selected = HashSet::new();
        let ctx = PreconditionContext {
            road_map: &map,
            selected_node_ids: &selected,
            distanzen_active: false,
        };

        assert!(Precondition::HasMarker(1).is_valid(&ctx));
        assert!(!Precondition::HasMarker(2).is_valid(&ctx));
        assert!(!Precondition::HasNoMarker(1).is_valid(&ctx));
        assert!(Precondition::HasNoMarker(2).is_valid(&ctx));
    }

    #[test]
    fn precondition_exactly_two_selected() {
        let map = make_road_map(&[(1, 0.0, 0.0), (2, 10.0, 0.0), (3, 20.0, 0.0)]);
        let two: HashSet<u64> = [1, 2].into();
        let three: HashSet<u64> = [1, 2, 3].into();
        let one: HashSet<u64> = [1].into();

        let ctx2 = PreconditionContext {
            road_map: &map,
            selected_node_ids: &two,
            distanzen_active: false,
        };
        let ctx3 = PreconditionContext {
            road_map: &map,
            selected_node_ids: &three,
            distanzen_active: false,
        };
        let ctx1 = PreconditionContext {
            road_map: &map,
            selected_node_ids: &one,
            distanzen_active: false,
        };

        assert!(Precondition::ExactlyTwoSelected.is_valid(&ctx2));
        assert!(!Precondition::ExactlyTwoSelected.is_valid(&ctx3));
        assert!(!Precondition::ExactlyTwoSelected.is_valid(&ctx1));
    }

    #[test]
    fn precondition_two_selected_unconnected() {
        let map_unconnected = make_road_map(&[(1, 0.0, 0.0), (2, 10.0, 0.0)]);
        let map_connected = make_connected_map(1, 2);
        let selected: HashSet<u64> = [1, 2].into();

        let ctx_unconnected = PreconditionContext {
            road_map: &map_unconnected,
            selected_node_ids: &selected,
            distanzen_active: false,
        };
        let ctx_connected = PreconditionContext {
            road_map: &map_connected,
            selected_node_ids: &selected,
            distanzen_active: false,
        };

        assert!(Precondition::TwoSelectedUnconnected.is_valid(&ctx_unconnected));
        assert!(!Precondition::TwoSelectedUnconnected.is_valid(&ctx_connected));
    }

    #[test]
    fn precondition_has_connections_between_selected() {
        let map_connected = make_connected_map(1, 2);
        let map_unconnected = make_road_map(&[(1, 0.0, 0.0), (2, 10.0, 0.0)]);
        let selected: HashSet<u64> = [1, 2].into();

        let ctx_yes = PreconditionContext {
            road_map: &map_connected,
            selected_node_ids: &selected,
            distanzen_active: false,
        };
        let ctx_no = PreconditionContext {
            road_map: &map_unconnected,
            selected_node_ids: &selected,
            distanzen_active: false,
        };

        assert!(Precondition::HasConnectionsBetweenSelected.is_valid(&ctx_yes));
        assert!(!Precondition::HasConnectionsBetweenSelected.is_valid(&ctx_no));
    }

    #[test]
    fn precondition_streckenteilung_active() {
        let map = make_road_map(&[]);
        let selected = HashSet::new();

        let ctx_active = PreconditionContext {
            road_map: &map,
            selected_node_ids: &selected,
            distanzen_active: true,
        };
        let ctx_inactive = PreconditionContext {
            road_map: &map,
            selected_node_ids: &selected,
            distanzen_active: false,
        };

        assert!(Precondition::StreckenteilungActive(true).is_valid(&ctx_active));
        assert!(!Precondition::StreckenteilungActive(true).is_valid(&ctx_inactive));
        assert!(Precondition::StreckenteilungActive(false).is_valid(&ctx_inactive));
    }

    // ═══════════════════════════════════════════════════════════════════
    // Katalog-Tests
    // ═══════════════════════════════════════════════════════════════════

    #[test]
    fn catalog_empty_area_shows_tools() {
        let map = make_road_map(&[]);
        let selected = HashSet::new();
        let ctx = PreconditionContext {
            road_map: &map,
            selected_node_ids: &selected,
            distanzen_active: false,
        };
        let intent_ctx = IntentContext {
            node_id: None,
            node_position: None,
            two_node_ids: None,
        };

        let catalog = MenuCatalog::for_empty_area(false);
        let entries = validate_entries(&catalog, &ctx, &intent_ctx);

        assert!(has_command(&entries, CommandId::SetToolSelect));
        assert!(has_command(&entries, CommandId::SetToolConnect));
        assert!(has_command(&entries, CommandId::SetToolAddNode));
        assert_eq!(count_commands(&entries), 3);
    }

    #[test]
    fn catalog_single_node_unselected_shows_marker_create() {
        let map = make_road_map(&[(42, 5.0, 5.0)]);
        let selected = HashSet::new();
        let ctx = PreconditionContext {
            road_map: &map,
            selected_node_ids: &selected,
            distanzen_active: false,
        };
        let intent_ctx = IntentContext {
            node_id: Some(42),
            node_position: Some(Vec2::new(5.0, 5.0)),
            two_node_ids: None,
        };

        let catalog = MenuCatalog::for_single_node_unselected(42);
        let entries = validate_entries(&catalog, &ctx, &intent_ctx);

        // Node hat keinen Marker → CreateMarker sichtbar, EditMarker/RemoveMarker nicht
        assert!(has_command(&entries, CommandId::SelectNode));
        assert!(has_command(&entries, CommandId::AddToSelection));
        assert!(has_command(&entries, CommandId::CreateMarker));
        assert!(!has_command(&entries, CommandId::EditMarker));
        assert!(!has_command(&entries, CommandId::RemoveMarker));
    }

    #[test]
    fn catalog_single_node_unselected_shows_marker_edit_when_marker_exists() {
        let mut map = make_road_map(&[(42, 5.0, 5.0)]);
        map.add_map_marker(MapMarker::new(
            42,
            "Farm".into(),
            "Default".into(),
            1,
            false,
        ));
        let selected = HashSet::new();
        let ctx = PreconditionContext {
            road_map: &map,
            selected_node_ids: &selected,
            distanzen_active: false,
        };
        let intent_ctx = IntentContext {
            node_id: Some(42),
            node_position: Some(Vec2::new(5.0, 5.0)),
            two_node_ids: None,
        };

        let catalog = MenuCatalog::for_single_node_unselected(42);
        let entries = validate_entries(&catalog, &ctx, &intent_ctx);

        // Node hat Marker → EditMarker/RemoveMarker sichtbar, CreateMarker nicht
        assert!(has_command(&entries, CommandId::EditMarker));
        assert!(has_command(&entries, CommandId::RemoveMarker));
        assert!(!has_command(&entries, CommandId::CreateMarker));
    }

    #[test]
    fn catalog_single_node_selected_shows_delete_and_duplicate() {
        let map = make_road_map(&[(10, 1.0, 1.0)]);
        let selected: HashSet<u64> = [10].into();
        let ctx = PreconditionContext {
            road_map: &map,
            selected_node_ids: &selected,
            distanzen_active: false,
        };
        let intent_ctx = IntentContext {
            node_id: Some(10),
            node_position: Some(Vec2::new(1.0, 1.0)),
            two_node_ids: None,
        };

        let catalog = MenuCatalog::for_single_node_selected(10);
        let entries = validate_entries(&catalog, &ctx, &intent_ctx);

        assert!(has_command(&entries, CommandId::DeselectNode));
        assert!(has_command(&entries, CommandId::DeleteSingleNode));
        assert!(has_command(&entries, CommandId::DuplicateSingleNode));
    }

    #[test]
    fn catalog_multi_nodes_connect_only_when_two_unconnected() {
        // 2 Nodes, nicht verbunden → ConnectTwoNodes sichtbar
        let map_unconnected = make_road_map(&[(1, 0.0, 0.0), (2, 10.0, 0.0)]);
        let selected: HashSet<u64> = [1, 2].into();
        let ctx = PreconditionContext {
            road_map: &map_unconnected,
            selected_node_ids: &selected,
            distanzen_active: false,
        };
        let intent_ctx = IntentContext {
            node_id: None,
            node_position: None,
            two_node_ids: Some((1, 2)),
        };

        let catalog = MenuCatalog::for_multiple_nodes_selected();
        let entries = validate_entries(&catalog, &ctx, &intent_ctx);
        assert!(has_command(&entries, CommandId::ConnectTwoNodes));

        // 2 Nodes, verbunden → ConnectTwoNodes NICHT sichtbar
        let map_connected = make_connected_map(1, 2);
        let ctx_connected = PreconditionContext {
            road_map: &map_connected,
            selected_node_ids: &selected,
            distanzen_active: false,
        };
        let entries_connected = validate_entries(&catalog, &ctx_connected, &intent_ctx);
        assert!(!has_command(&entries_connected, CommandId::ConnectTwoNodes));
    }

    #[test]
    fn catalog_multi_nodes_direction_only_when_connected() {
        // Keine Verbindungen → keine Richtungs-Commands
        let map = make_road_map(&[(1, 0.0, 0.0), (2, 10.0, 0.0)]);
        let selected: HashSet<u64> = [1, 2].into();
        let ctx = PreconditionContext {
            road_map: &map,
            selected_node_ids: &selected,
            distanzen_active: false,
        };
        let intent_ctx = IntentContext {
            node_id: None,
            node_position: None,
            two_node_ids: Some((1, 2)),
        };

        let catalog = MenuCatalog::for_multiple_nodes_selected();
        let entries = validate_entries(&catalog, &ctx, &intent_ctx);

        assert!(!has_command(&entries, CommandId::DirectionRegular));
        assert!(!has_command(&entries, CommandId::DirectionDual));
        assert!(!has_command(&entries, CommandId::PriorityRegular));
        assert!(!has_command(&entries, CommandId::RemoveAllConnections));

        // Mit Verbindung → alle Richtungs-Commands sichtbar
        let map_connected = make_connected_map(1, 2);
        let ctx_connected = PreconditionContext {
            road_map: &map_connected,
            selected_node_ids: &selected,
            distanzen_active: false,
        };
        let entries_connected = validate_entries(&catalog, &ctx_connected, &intent_ctx);

        assert!(has_command(&entries_connected, CommandId::DirectionRegular));
        assert!(has_command(&entries_connected, CommandId::DirectionDual));
        assert!(has_command(&entries_connected, CommandId::DirectionReverse));
        assert!(has_command(&entries_connected, CommandId::DirectionInvert));
        assert!(has_command(&entries_connected, CommandId::PriorityRegular));
        assert!(has_command(&entries_connected, CommandId::PrioritySub));
        assert!(has_command(
            &entries_connected,
            CommandId::RemoveAllConnections
        ));
    }

    #[test]
    fn catalog_multi_nodes_route_tools_only_when_two_selected() {
        // 3 Nodes → keine Route-Tools
        let map = make_road_map(&[(1, 0.0, 0.0), (2, 10.0, 0.0), (3, 20.0, 0.0)]);
        let selected: HashSet<u64> = [1, 2, 3].into();
        let ctx = PreconditionContext {
            road_map: &map,
            selected_node_ids: &selected,
            distanzen_active: false,
        };
        let intent_ctx = IntentContext {
            node_id: None,
            node_position: None,
            two_node_ids: None,
        };

        let catalog = MenuCatalog::for_multiple_nodes_selected();
        let entries = validate_entries(&catalog, &ctx, &intent_ctx);

        assert!(!has_command(&entries, CommandId::RouteStraight));
        assert!(!has_command(&entries, CommandId::RouteQuadratic));
        assert!(!has_command(&entries, CommandId::RouteCubic));
    }

    #[test]
    fn catalog_multi_nodes_selection_commands_always_visible() {
        let map = make_road_map(&[(1, 0.0, 0.0), (2, 10.0, 0.0)]);
        let selected: HashSet<u64> = [1, 2].into();
        let ctx = PreconditionContext {
            road_map: &map,
            selected_node_ids: &selected,
            distanzen_active: false,
        };
        let intent_ctx = IntentContext {
            node_id: None,
            node_position: None,
            two_node_ids: Some((1, 2)),
        };

        let catalog = MenuCatalog::for_multiple_nodes_selected();
        let entries = validate_entries(&catalog, &ctx, &intent_ctx);

        assert!(has_command(&entries, CommandId::InvertSelection));
        assert!(has_command(&entries, CommandId::SelectAll));
        assert!(has_command(&entries, CommandId::ClearSelection));
        assert!(has_command(&entries, CommandId::DeleteSelected));
        assert!(has_command(&entries, CommandId::DuplicateSelected));
    }

    #[test]
    fn catalog_route_tool_basic_commands() {
        let map = make_road_map(&[]);
        let selected = HashSet::new();
        let ctx = PreconditionContext {
            road_map: &map,
            selected_node_ids: &selected,
            distanzen_active: false,
        };
        let intent_ctx = IntentContext {
            node_id: None,
            node_position: None,
            two_node_ids: None,
        };

        let catalog = MenuCatalog::for_route_tool();
        let entries = validate_entries(&catalog, &ctx, &intent_ctx);

        assert!(has_command(&entries, CommandId::RouteExecute));
        assert!(has_command(&entries, CommandId::RouteRecreate));
        assert!(has_command(&entries, CommandId::RouteCancel));
        assert_eq!(count_commands(&entries), 3);
    }

    // ═══════════════════════════════════════════════════════════════════
    // Intent-Mapping-Tests
    // ═══════════════════════════════════════════════════════════════════

    #[test]
    fn intent_mapping_select_node() {
        let ctx = IntentContext {
            node_id: Some(42),
            node_position: Some(Vec2::new(5.0, 5.0)),
            two_node_ids: None,
        };
        let intent = CommandId::SelectNode.to_intent(&ctx);
        assert!(matches!(
            intent,
            AppIntent::NodePickRequested {
                additive: false,
                extend_path: false,
                ..
            }
        ));
    }

    #[test]
    fn intent_mapping_connect_two_nodes() {
        let ctx = IntentContext {
            node_id: None,
            node_position: None,
            two_node_ids: Some((1, 2)),
        };
        let intent = CommandId::ConnectTwoNodes.to_intent(&ctx);
        assert!(matches!(intent, AppIntent::ConnectSelectedNodesRequested));
    }

    #[test]
    fn intent_mapping_route_straight() {
        let ctx = IntentContext {
            node_id: None,
            node_position: None,
            two_node_ids: Some((5, 10)),
        };
        let intent = CommandId::RouteStraight.to_intent(&ctx);
        assert!(matches!(
            intent,
            AppIntent::RouteToolWithAnchorsRequested {
                index: 0,
                start_node_id: 5,
                end_node_id: 10,
            }
        ));
    }

    // ═══════════════════════════════════════════════════════════════════
    // Separator/Label-Cleanup-Tests
    // ═══════════════════════════════════════════════════════════════════

    #[test]
    fn cleanup_removes_orphaned_labels() {
        // Simuliere: Label gefolgt von Separator (keine Commands dazwischen)
        let entries = vec![
            ValidatedEntry::Label("Richtung:".into()),
            ValidatedEntry::Separator,
            ValidatedEntry::Command {
                id: CommandId::DeleteSelected,
                label: "Löschen".into(),
                intent: AppIntent::DeleteSelectedRequested,
            },
        ];

        let cleaned = cleanup_separators(entries);

        // Label "Richtung:" sollte entfernt sein (kein Command folgt bis Separator)
        assert!(!cleaned
            .iter()
            .any(|e| matches!(e, ValidatedEntry::Label(l) if l == "Richtung:")));
        // Der Command sollte noch da sein
        assert!(has_command(&cleaned, CommandId::DeleteSelected));
    }

    #[test]
    fn cleanup_keeps_labels_with_commands() {
        let entries = vec![
            ValidatedEntry::Label("🗺 Marker".into()),
            ValidatedEntry::Command {
                id: CommandId::CreateMarker,
                label: "Erstellen".into(),
                intent: AppIntent::CreateMarkerRequested { node_id: 1 },
            },
        ];

        let cleaned = cleanup_separators(entries);

        assert!(cleaned
            .iter()
            .any(|e| matches!(e, ValidatedEntry::Label(l) if l == "🗺 Marker")));
        assert!(has_command(&cleaned, CommandId::CreateMarker));
    }

    #[test]
    fn cleanup_no_double_separators() {
        let entries = vec![
            ValidatedEntry::Command {
                id: CommandId::SelectNode,
                label: "Sel".into(),
                intent: AppIntent::SelectAllRequested,
            },
            ValidatedEntry::Separator,
            ValidatedEntry::Separator,
            ValidatedEntry::Command {
                id: CommandId::DeleteSelected,
                label: "Del".into(),
                intent: AppIntent::DeleteSelectedRequested,
            },
        ];

        let cleaned = cleanup_separators(entries);
        let sep_count = cleaned
            .iter()
            .filter(|e| matches!(e, ValidatedEntry::Separator))
            .count();
        assert_eq!(sep_count, 1);
    }

    #[test]
    fn deleted_node_hides_all_commands() {
        // Node 99 existiert NICHT in der Map
        let map = make_road_map(&[(1, 0.0, 0.0)]);
        let selected = HashSet::new();
        let ctx = PreconditionContext {
            road_map: &map,
            selected_node_ids: &selected,
            distanzen_active: false,
        };
        let intent_ctx = IntentContext {
            node_id: Some(99),
            node_position: None,
            two_node_ids: None,
        };

        let catalog = MenuCatalog::for_single_node_unselected(99);
        let entries = validate_entries(&catalog, &ctx, &intent_ctx);

        // Alle Commands haben NodeExists(99) → alle gefiltert
        assert_eq!(count_commands(&entries), 0);
    }
}
