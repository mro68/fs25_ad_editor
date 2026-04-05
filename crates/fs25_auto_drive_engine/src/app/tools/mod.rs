//! Trait-basiertes Route-Tool-System fuer erweiterbare Strecken-Werkzeuge.
//!
//! Jedes Route-Tool implementiert den kleinen Umbrella `RouteTool` ueber
//! Kernvertrag, Panel-Bruecke und Host-Sync. Optionale Interaktionen laufen
//! ueber additive Capabilities, die der `ToolManager` gezielt entdeckt.

/// Ausweichstrecken-Tool — generiert eine parallele Strecke zur selektierten Kette.
pub mod bypass;
/// Additive Capabilities fuer optionale Tool-Faehigkeiten.
mod capabilities;
/// Kanonischer Tool-Katalog mit stabilen Tool-IDs und Surface-Metadaten.
mod catalog;
/// Farb-Pfad-Tool: erkennt Wege anhand der Farbe im Hintergrundbild.
pub mod color_path;
/// Gemeinsame Hilfsfunktionen fuer Route-Tools.
pub mod common;
/// Feste Basisvertraege fuer Route-Tools.
mod contracts;
/// Bézier-Kurven-Tool (Grad 2 + 3) mit sequentieller Punkt-Platzierung.
pub mod curve;
/// Feldgrenz-Erkennungs-Tool: erzeugt eine Route entlang eines erkannten Feldumrisses.
pub mod field_boundary;
/// Feldweg-Erkennungs-Tool: berechnet eine Mittellinie zwischen zwei Farmland-Seiten.
pub mod field_path;
/// ToolManager und Capability-Discovery.
mod manager;
/// Parkplatz-Layout-Tool mit Wendekreis und konfigurierbaren Parkreihen.
pub mod parking;
/// Strecken-Versatz-Tool — generiert parallele Versatz-Kette(n) zur selektierten Kette.
pub mod route_offset;
/// Kleiner Umbrella-Vertrag fuer Route-Tools.
mod route_tool;
/// Geglättete-Kurve-Tool — winkelgeglaettete Route mit automatischen Tangenten-Uebergaengen.
pub mod smooth_curve;
/// Catmull-Rom-Spline-Tool — interpolierende Kurve durch alle geklickten Punkte.
pub mod spline;
/// Gerade-Linie-Tool mit konfigurierbarem Node-Abstand.
pub mod straight_line;

pub use crate::app::tool_contract::ToolAnchor;
pub use capabilities::{
    OrderedNodeChain, RouteToolChainInput, RouteToolDrag, RouteToolGroupEdit, RouteToolLassoInput,
    RouteToolRecreate, RouteToolRotate, RouteToolSegmentAdjustments, RouteToolTangent,
};
pub use catalog::{
    resolve_route_tool_entries, route_tool_catalog, route_tool_defaults_tooltip_key,
    route_tool_descriptor, route_tool_descriptor_by_slot, route_tool_disabled_reason,
    route_tool_disabled_reason_key, route_tool_group_label_key, route_tool_label_key,
    route_tool_slot, ResolvedRouteToolEntry, RouteToolAvailabilityContext, RouteToolBackingMode,
    RouteToolDescriptor, RouteToolDisabledReason, RouteToolGroup, RouteToolIconKey,
    RouteToolRequirement, RouteToolSurface,
};
pub use contracts::{RouteToolCore, RouteToolHostSync, RouteToolPanelBridge, ToolHostContext};
pub use manager::ToolManager;
pub use route_tool::RouteTool;

use crate::core::{ConnectionDirection, ConnectionPriority, NodeFlag, RoadMap};
use glam::Vec2;

// ── Gemeinsame Utilities ─────────────────────────────────────

/// Versucht, auf einen existierenden Node innerhalb des Snap-Radius zu snappen.
///
/// Gibt `ToolAnchor::ExistingNode` zurueck wenn ein Node in Reichweite ist,
/// sonst `ToolAnchor::NewPosition` mit der Original-Position.
pub fn snap_to_node(pos: Vec2, road_map: &RoadMap, snap_radius: f32) -> ToolAnchor {
    if let Some(hit) = road_map.nearest_node(pos)
        && hit.distance <= snap_radius
        && let Some(node) = road_map.node(hit.node_id)
    {
        return ToolAnchor::ExistingNode(hit.node_id, node.position);
    }
    ToolAnchor::NewPosition(pos)
}

// ── Typen ────────────────────────────────────────────────────────

/// Rueckgabe von `on_click` — steuert den Tool-Flow.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ToolAction {
    /// Punkt registriert, weitere Eingabe noetig
    Continue,
    /// Alle noetigen Punkte gesetzt — bereit zur Ausfuehrung
    ReadyToExecute,
    /// Vorschau aktualisiert — Klick aendert Parameter, Enter bestaetigt
    UpdatePreview,
}

/// Preview-Geometrie fuer das Rendering (halbtransparent im Viewport).
#[derive(Debug, Clone, Default)]
pub struct ToolPreview {
    /// Vorschau-Node-Positionen
    pub nodes: Vec<Vec2>,
    /// Vorschau-Verbindungen als Index-Paare in `nodes`
    pub connections: Vec<(usize, usize)>,
    /// Stil pro Verbindung (Index passt zu `connections`)
    pub connection_styles: Vec<(ConnectionDirection, ConnectionPriority)>,
    /// Beschriftungen pro Node: (node_index, Labeltext)
    pub labels: Vec<(usize, String)>,
}

impl ToolPreview {
    /// Erzeugt eine Vorschau aus einer Polyline mit einheitlicher Richtung und Prioritaet.
    ///
    /// Gemeinsames Konstruktor-Pattern aller Route-Tool-`preview()`-Methoden:
    /// Verbindet `positions` linear (`[(0,1), (1,2), ...]`) und weist jeder Verbindung
    /// denselben `direction`/`priority`-Stil zu.
    pub fn from_polyline(
        positions: Vec<Vec2>,
        direction: ConnectionDirection,
        priority: ConnectionPriority,
    ) -> Self {
        let connections = common::linear_connections(positions.len());
        let connection_styles = vec![(direction, priority); connections.len()];
        Self {
            nodes: positions,
            connections,
            connection_styles,
            labels: vec![],
        }
    }
}

/// Ergebnis eines Route-Tools — reine Daten, keine Mutation.
///
/// Dieses Struct enthaelt alle geometrischen Daten, die von einem Tool erzeugt werden:
/// neue Nodes mit ihren Positionen und Flags, sowie Verbindungen zwischen
/// diesen Nodes sowie zu bestehenden Nodes in der Road Map.
/// Optionale Sammlungen bleiben fuer einfache Tools leer und werden intern
/// ueber gemeinsame Helper in `app::tools::common` kanonisch initialisiert.
///
/// Die Ausfuehrung erfolgt zentral in `apply_tool_result()` — das Tool
/// selbst verursacht keine direkten State-Mutationen.
///
/// # Beispiel
///
/// ```rust,ignore
/// let result = ToolResult {
///     new_nodes: vec![(Vec2::new(0.0, 0.0), NodeFlag::Road)],
///     internal_connections: vec![],
///     external_connections: vec![(0, 42, true, ConnectionDirection::Both, Regular)],
/// };
/// ```
///
/// Dies würde einen neuen Node erstellen und ihn bidirektional mit existiertem Node #42 verbinden.
#[derive(Debug, Clone)]
pub struct ToolResult {
    /// Neue Nodes als Vektor von (Position, NodeFlag).
    ///
    /// **NodeFlag** beschreibt den Typ des Nodes (z.B. `Road`, `Intersection`, `Turn-Restriction`).
    /// Indizes in diesem Vektor (0, 1, 2, ...) werden in `internal_connections`
    /// und `external_connections` verwendet.
    pub new_nodes: Vec<(Vec2, NodeFlag)>,
    /// Verbindungen innerhalb der neuen Nodes.
    ///
    /// Jeder Eintrag ist `(from_idx, to_idx, direction, priority)`, wobei die Indizes
    /// sich auf `new_nodes` beziehen. Die Verbindungen werden in der angegebenen
    /// Richtung etabliert.
    pub internal_connections: Vec<(usize, usize, ConnectionDirection, ConnectionPriority)>,
    /// Verbindungen von neuen Nodes zu existierenden Nodes in der Road Map.
    ///
    /// Jeder Eintrag ist `(new_node_idx, existing_node_id, existing_to_new, direction, priority)`:
    /// - `new_node_idx` — Index in `new_nodes`
    /// - `existing_node_id` — ID eines existierenden Nodes in der Road Map
    /// - `existing_to_new` — `true`: Verbindung von existierend zu neu; `false`: von neu zu existierend
    /// - `direction` — Richtung der Verbindung
    /// - `priority` — Strassenkategorisierung (Regular, Preferred, etc.)
    pub external_connections: Vec<(usize, u64, bool, ConnectionDirection, ConnectionPriority)>,
    /// Optionale Map-Marker: (new_node_idx, name, group).
    ///
    /// Jeder Eintrag erzeugt einen Map-Marker am Node mit dem angegebenen Index
    /// in `new_nodes`. Wird z.B. vom ParkingTool genutzt.
    pub markers: Vec<(usize, String, String)>,
    /// IDs von Nodes, die beim Anwenden des Results entfernt werden sollen.
    ///
    /// Wird vom `RouteOffsetTool` befuellt wenn "Original entfernen" aktiv ist.
    /// `apply_tool_result` loescht diese Nodes (inkl. aller zugehoerigen Connections)
    /// im selben Undo-Snapshot wie die Erstellung der neuen Nodes.
    /// Fuer alle anderen Tools ist dieser Vec leer.
    pub nodes_to_remove: Vec<u64>,
}
