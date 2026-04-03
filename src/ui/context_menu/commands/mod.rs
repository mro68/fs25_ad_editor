//! Validiertes Context-Menu-System: Command-Definitionen, Preconditions und Kataloge.
//!
//! Architektur:
//! - `CommandId`: Eindeutige Identifikation jedes Menue-Eintrags (mod.rs)
//! - `Precondition`: Enum mit Vorbedingungen (preconditions.rs)
//! - `MenuCatalog`: Statischer Katalog pro `MenuVariant` (catalogs.rs)
//! - `validate_entries()`: Filtert nur gueltige Commands (validation.rs)
//!
//! Garantie: Nur Commands mit erfuellten Preconditions werden gerendert.

mod catalogs;
pub mod preconditions;
mod validation;

// Re-Exports fuer Konsumenten
pub use preconditions::{Precondition, PreconditionContext};
pub use validation::{validate_entries, ValidatedEntry};

use crate::app::tool_contract::RouteToolId;
use crate::app::{AppIntent, ConnectionDirection, ConnectionPriority, EditorTool};

// =============================================================================
// CommandId — Eindeutige Identifikation jedes Menue-Befehls
// =============================================================================

/// Eindeutige ID fuer jeden Context-Menu-Befehl.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum CommandId {
    // ── EmptyArea ────────────────────────────────────────────────────
    /// Auswahl-Werkzeug aktivieren
    SetToolSelect,
    /// Verbinden-Werkzeug aktivieren
    SetToolConnect,
    /// Node-Hinzufuegen-Werkzeug aktivieren
    SetToolAddNode,
    /// Route-Tool: Gerade Strecke aktivieren
    SetToolRouteStraight,
    /// Route-Tool: Geglättete Kurve aktivieren
    SetToolRouteSmoothCurve,
    /// Route-Tool: Bézier Grad 2 aktivieren
    SetToolRouteQuadratic,
    /// Route-Tool: Bézier Grad 3 aktivieren
    SetToolRouteCubic,
    /// Streckenteilung (in EmptyArea nur wenn aktiv)
    StreckenteilungEmptyArea,

    // ── NodeFocused (Einzelnode-Befehle) ─────────────────────────────
    /// Marker erstellen
    CreateMarker,
    /// Marker bearbeiten
    EditMarker,
    /// Marker loeschen
    RemoveMarker,
    // ── Selection-Befehle (SelectionOnly + NodeFocused) ─────────────
    /// Zwei Nodes verbinden (nur bei genau 2 unverbundenen)
    ConnectTwoNodes,
    /// Gerade Strecke erzeugen (2 Nodes)
    RouteStraight,
    /// Geglättete Kurve erzeugen (2 Nodes)
    RouteSmoothCurve,
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
    /// Prioritaet: Hauptstrasse
    PriorityRegular,
    /// Prioritaet: Nebenstrasse
    PrioritySub,
    /// Alle Verbindungen trennen
    RemoveAllConnections,
    /// Streckenteilung (bei selektierten Nodes)
    StreckenteilungMulti,
    /// Selektion invertieren
    InvertSelection,
    /// Alle Nodes auswaehlen
    SelectAll,
    /// Selektion aufheben
    ClearSelection,
    /// Selektierte Nodes loeschen
    DeleteSelected,
    // ── RouteTool ────────────────────────────────────────────────────
    /// Route ausfuehren
    RouteExecute,
    /// Route neu berechnen
    RouteRecreate,
    /// Route abbrechen
    RouteCancel,

    // ── Copy/Paste ─────────────────────────────────────────────────
    /// Selektion in die Zwischenablage kopieren
    CopySelection,
    /// Einfuegen-Vorschau starten
    PasteHere,

    // ── Segment ──────────────────────────────────────────────────────
    /// Selektiertes Segment nachtraeglich bearbeiten
    EditGroup,
    /// Selektierte zusammenhaengende Nodes als neues Segment gruppieren
    GroupSelectionAsGroup,
    /// Selektierte Nodes aus ihrer Gruppe entfernen (Nodes bleiben erhalten)
    RemoveFromGroup,
    /// Gruppe komplett aufloesen (alle Nodes bleiben erhalten)
    DissolveGroup,

    // ── Extras ───────────────────────────────────────────────────────
    /// FieldBoundaryTool aktivieren
    SetToolFieldBoundary,

    // ── Zoom ─────────────────────────────────────────────────────────
    /// Alles in den Viewport einpassen
    ZoomToFit,
    /// Auf die Grenzen der aktuellen Selektion zoomen
    ZoomToSelection,
}

// =============================================================================
// MenuEntry + MenuCatalog — Statische Beschreibung der Menue-Struktur
// =============================================================================

/// Ein einzelner Eintrag im Menue-Katalog.
#[derive(Debug, Clone)]
pub enum MenuEntry {
    /// Ueberschrift / Label (wird nur angezeigt wenn kein Submenu)
    Label(String),
    /// Trennlinie
    Separator,
    /// Befehl mit optionalen Vorbedingungen
    Command {
        id: CommandId,
        label: String,
        preconditions: Vec<Precondition>,
    },
    /// Einklappbares Untermenue mit eigenem Label und Kind-Eintraegen
    Submenu {
        label: String,
        entries: Vec<MenuEntry>,
    },
}

/// Katalog fuer eine bestimmte `MenuVariant` — definiert Reihenfolge und Preconditions.
#[derive(Debug, Clone)]
pub struct MenuCatalog {
    pub entries: Vec<MenuEntry>,
}

// =============================================================================
// Intent-Erzeugung — Wie wird aus einem CommandId ein AppIntent?
// =============================================================================

/// Kontext fuer die Intent-Erzeugung — enthaelt Node-IDs und Tool-Daten.
pub struct IntentContext {
    /// Node-ID (fuer SingleNode-Varianten)
    pub node_id: Option<u64>,
    /// Node-Position (fuer NodePick)
    pub node_position: Option<glam::Vec2>,
    /// Sortierte Zwei-Node-IDs (fuer RouteToolWithAnchorsRequested)
    pub two_node_ids: Option<(u64, u64)>,
    /// Record-ID eines validen Segments (fuer EditGroup-Command)
    pub group_record_id: Option<u64>,
}

impl CommandId {
    fn route_tool_selection_id(&self) -> Option<RouteToolId> {
        match self {
            Self::SetToolRouteStraight => Some(RouteToolId::Straight),
            Self::SetToolRouteSmoothCurve => Some(RouteToolId::SmoothCurve),
            Self::SetToolRouteQuadratic => Some(RouteToolId::CurveQuad),
            Self::SetToolRouteCubic => Some(RouteToolId::CurveCubic),
            Self::SetToolFieldBoundary => Some(RouteToolId::FieldBoundary),
            _ => None,
        }
    }

    fn route_tool_anchor_id(&self) -> Option<RouteToolId> {
        match self {
            Self::RouteStraight => Some(RouteToolId::Straight),
            Self::RouteSmoothCurve => Some(RouteToolId::SmoothCurve),
            Self::RouteQuadratic => Some(RouteToolId::CurveQuad),
            Self::RouteCubic => Some(RouteToolId::CurveCubic),
            _ => None,
        }
    }

    /// Erzeugt den passenden `AppIntent` fuer diesen Command.
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
            Self::SetToolRouteStraight
            | Self::SetToolRouteSmoothCurve
            | Self::SetToolRouteQuadratic
            | Self::SetToolRouteCubic
            | Self::SetToolFieldBoundary => AppIntent::SelectRouteToolRequested {
                tool_id: self
                    .route_tool_selection_id()
                    .expect("invariant: Route-Tool-Selektionsbefehl muss Tool-ID liefern"),
            },
            Self::StreckenteilungEmptyArea | Self::StreckenteilungMulti => {
                AppIntent::StreckenteilungAktivieren
            }

            // ── NodeFocused (Einzelnode-Befehle) ─────────────────────
            Self::CreateMarker => AppIntent::CreateMarkerRequested {
                node_id: ctx.node_id.unwrap_or(0),
            },
            Self::EditMarker => AppIntent::EditMarkerRequested {
                node_id: ctx.node_id.unwrap_or(0),
            },
            Self::RemoveMarker => AppIntent::RemoveMarkerRequested {
                node_id: ctx.node_id.unwrap_or(0),
            },
            Self::DeleteSelected => AppIntent::DeleteSelectedRequested,

            // ── Selection-Befehle ────────────────────────────────────
            Self::ConnectTwoNodes => AppIntent::ConnectSelectedNodesRequested,
            Self::RouteStraight
            | Self::RouteSmoothCurve
            | Self::RouteQuadratic
            | Self::RouteCubic => {
                let (s, e) = ctx.two_node_ids.unwrap_or((0, 0));
                AppIntent::RouteToolWithAnchorsRequested {
                    tool_id: self
                        .route_tool_anchor_id()
                        .expect("invariant: Route-Tool-Ankerbefehl muss Tool-ID liefern"),
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
            // ── Copy/Paste ────────────────────────────────────────────────
            Self::CopySelection => AppIntent::CopySelectionRequested,
            Self::PasteHere => AppIntent::PasteStartRequested,

            // ── Segment ──────────────────────────────────────────────────────
            Self::EditGroup => AppIntent::GroupEditStartRequested {
                record_id: ctx.group_record_id.unwrap_or(0),
            },
            Self::GroupSelectionAsGroup => AppIntent::GroupSelectionAsGroupRequested,
            Self::RemoveFromGroup => AppIntent::RemoveSelectedNodesFromGroupRequested,
            Self::DissolveGroup => AppIntent::DissolveGroupRequested {
                segment_id: ctx.group_record_id.unwrap_or(0),
            },
            // ── Zoom ─────────────────────────────────────────────────────────
            Self::ZoomToFit => AppIntent::ZoomToFitRequested,
            Self::ZoomToSelection => AppIntent::ZoomToSelectionBoundsRequested,
        }
    }
}

#[cfg(test)]
mod tests;
