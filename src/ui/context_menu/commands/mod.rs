//! Validiertes Context-Menu-System: Command-Definitionen, Preconditions und Kataloge.
//!
//! Architektur:
//! - `CommandId`: Eindeutige Identifikation jedes Menü-Eintrags (mod.rs)
//! - `Precondition`: Enum mit Vorbedingungen (preconditions.rs)
//! - `MenuCatalog`: Statischer Katalog pro `MenuVariant` (catalogs.rs)
//! - `validate_entries()`: Filtert nur gültige Commands (validation.rs)
//!
//! Garantie: Nur Commands mit erfüllten Preconditions werden gerendert.

mod catalogs;
pub mod preconditions;
mod validation;

// Re-Exports für Konsumenten
pub use preconditions::{Precondition, PreconditionContext};
pub use validation::{validate_entries, ValidatedEntry};

use crate::app::segment_registry::{
    TOOL_INDEX_CURVE_CUBIC, TOOL_INDEX_CURVE_QUAD, TOOL_INDEX_STRAIGHT,
};
use crate::app::{AppIntent, ConnectionDirection, ConnectionPriority, EditorTool};

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

#[cfg(test)]
mod tests;
