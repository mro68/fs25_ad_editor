//! Trait-basiertes Route-Tool-System fuer erweiterbare Strecken-Werkzeuge.
//!
//! Jedes Route-Tool implementiert den `RouteTool`-Trait und wird beim
//! `ToolManager` registriert. Tools erzeugen reine Daten (`ToolResult`),
//! die Mutation erfolgt zentral in `apply_tool_result`.

/// Ausweichstrecken-Tool — generiert eine parallele Strecke zur selektierten Kette.
pub mod bypass;
/// Kanonischer Tool-Katalog mit stabilen Tool-IDs und Surface-Metadaten.
mod catalog;
/// Farb-Pfad-Tool: erkennt Wege anhand der Farbe im Hintergrundbild.
pub mod color_path;
/// Gemeinsame Hilfsfunktionen fuer Route-Tools.
pub mod common;
/// Bézier-Kurven-Tool (Grad 2 + 3) mit sequentieller Punkt-Platzierung.
pub mod curve;
/// Feldgrenz-Erkennungs-Tool: erzeugt eine Route entlang eines erkannten Feldumrisses.
pub mod field_boundary;
/// Feldweg-Erkennungs-Tool: berechnet eine Mittellinie zwischen zwei Farmland-Seiten.
pub mod field_path;
/// Parkplatz-Layout-Tool mit Wendekreis und konfigurierbaren Parkreihen.
pub mod parking;
/// Strecken-Versatz-Tool — generiert parallele Versatz-Kette(n) zur selektierten Kette.
pub mod route_offset;
/// RouteTool-Trait — Schnittstelle fuer alle Route-Tools.
mod route_tool;
/// Geglättete-Kurve-Tool — winkelgeglaettete Route mit automatischen Tangenten-Uebergaengen.
pub mod smooth_curve;
/// Catmull-Rom-Spline-Tool — interpolierende Kurve durch alle geklickten Punkte.
pub mod spline;
/// Gerade-Linie-Tool mit konfigurierbarem Node-Abstand.
pub mod straight_line;

pub use crate::app::tool_contract::{RouteToolId, ToolAnchor};
pub use catalog::{
    resolve_route_tool_entries, route_tool_catalog, route_tool_defaults_tooltip_key,
    route_tool_descriptor, route_tool_descriptor_by_slot, route_tool_disabled_reason,
    route_tool_disabled_reason_key, route_tool_group_label_key, route_tool_label_key,
    route_tool_slot, ResolvedRouteToolEntry, RouteToolAvailabilityContext, RouteToolBackingMode,
    RouteToolDescriptor, RouteToolDisabledReason, RouteToolGroup, RouteToolRequirement,
    RouteToolSurface,
};
pub use route_tool::RouteTool;

use crate::core::{ConnectionDirection, ConnectionPriority, NodeFlag, RoadMap};
use glam::Vec2;

// ── Gemeinsame Utilities ─────────────────────────────────────

/// Versucht, auf einen existierenden Node innerhalb des Snap-Radius zu snappen.
///
/// Gibt `ToolAnchor::ExistingNode` zurueck wenn ein Node in Reichweite ist,
/// sonst `ToolAnchor::NewPosition` mit der Original-Position.
pub fn snap_to_node(pos: Vec2, road_map: &RoadMap, snap_radius: f32) -> ToolAnchor {
    if let Some(hit) = road_map.nearest_node(pos) {
        if hit.distance <= snap_radius {
            if let Some(node) = road_map.nodes.get(&hit.node_id) {
                return ToolAnchor::ExistingNode(hit.node_id, node.position);
            }
        }
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

// ── ToolManager ──────────────────────────────────────────────────

/// Verwaltet registrierte Route-Tools und den aktiven Tool-Index.
pub struct ToolManager {
    tools: Vec<RegisteredTool>,
    active_index: Option<usize>,
}

struct RegisteredTool {
    id: RouteToolId,
    tool: Box<dyn RouteTool>,
}

impl Default for ToolManager {
    fn default() -> Self {
        Self::new()
    }
}

impl ToolManager {
    /// Erstellt einen neuen ToolManager mit vorregistrierten Standard-Tools.
    pub fn new() -> Self {
        let mut manager = Self {
            tools: Vec::new(),
            active_index: None,
        };
        for descriptor in route_tool_catalog() {
            manager.register(descriptor.id, (descriptor.factory)());
        }
        manager
    }

    /// Registriert ein neues Route-Tool.
    pub fn register(&mut self, tool_id: RouteToolId, tool: Box<dyn RouteTool>) {
        self.tools.push(RegisteredTool { id: tool_id, tool });
    }

    /// Gibt die Anzahl registrierter Tools zurueck.
    pub fn tool_count(&self) -> usize {
        self.tools.len()
    }

    /// Gibt Name und Index aller registrierten Tools zurueck.
    pub fn tool_names(&self) -> Vec<(RouteToolId, &str)> {
        self.tools
            .iter()
            .map(|entry| (entry.id, entry.tool.name()))
            .collect()
    }

    /// Gibt Index, Name und Icon aller registrierten Tools zurueck.
    pub fn tool_entries(&self) -> Vec<(RouteToolId, &str, &str)> {
        self.tools
            .iter()
            .map(|entry| (entry.id, entry.tool.name(), entry.tool.icon()))
            .collect()
    }

    fn set_active_slot(&mut self, index: usize) {
        if index < self.tools.len() {
            // Altes Tool zuruecksetzen
            if let Some(old) = self.active_index {
                if old != index {
                    self.tools[old].tool.reset();
                }
            }
            self.active_index = Some(index);
        }
    }

    /// Setzt das aktive Route-Tool per stabiler Tool-ID.
    pub fn set_active_by_id(&mut self, tool_id: RouteToolId) {
        if let Some(slot) = route_tool_slot(tool_id) {
            self.set_active_slot(slot);
        }
    }

    /// Gibt die Tool-ID des aktiven Tools zurueck.
    pub fn active_id(&self) -> Option<RouteToolId> {
        self.active_index.map(|index| self.tools[index].id)
    }

    /// Gibt den Descriptor des aktiven Tools zurueck.
    pub fn active_descriptor(&self) -> Option<&'static RouteToolDescriptor> {
        self.active_id().map(route_tool_descriptor)
    }

    /// Gibt eine Referenz auf das aktive Tool zurueck.
    pub fn active_tool(&self) -> Option<&dyn RouteTool> {
        self.active_index
            .map(|index| self.tools[index].tool.as_ref())
    }

    /// Gibt eine mutable Referenz auf das aktive Tool zurueck.
    pub fn active_tool_mut(&mut self) -> Option<&mut dyn RouteTool> {
        let index = self.active_index?;
        Some(self.tools[index].tool.as_mut())
    }

    /// Setzt alle Tools zurueck und deaktiviert das aktive Tool.
    pub fn reset(&mut self) {
        if let Some(i) = self.active_index {
            self.tools[i].tool.reset();
        }
        self.active_index = None;
    }
}
