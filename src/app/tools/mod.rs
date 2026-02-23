//! Trait-basiertes Route-Tool-System für erweiterbare Strecken-Werkzeuge.
//!
//! Jedes Route-Tool implementiert den `RouteTool`-Trait und wird beim
//! `ToolManager` registriert. Tools erzeugen reine Daten (`ToolResult`),
//! die Mutation erfolgt zentral in `apply_tool_result`.

/// Gemeinsame Hilfsfunktionen für Route-Tools.
pub mod common;
/// Bézier-Kurven-Tool (Grad 2 + 3) mit sequentieller Punkt-Platzierung.
pub mod curve;
/// RouteTool-Trait — Schnittstelle für alle Route-Tools.
mod route_tool;
/// Catmull-Rom-Spline-Tool — interpolierende Kurve durch alle geklickten Punkte.
pub mod spline;
/// Gerade-Linie-Tool mit konfigurierbarem Node-Abstand.
pub mod straight_line;

pub use route_tool::RouteTool;

use crate::core::{ConnectionDirection, ConnectionPriority, NodeFlag, RoadMap};
use glam::Vec2;

// ── Gemeinsame Utilities ─────────────────────────────────────

/// Versucht, auf einen existierenden Node innerhalb des Snap-Radius zu snappen.
///
/// Gibt `ToolAnchor::ExistingNode` zurück wenn ein Node in Reichweite ist,
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

/// Anker-Punkt: entweder ein existierender Node oder eine freie Position.
#[derive(Debug, Clone, Copy)]
pub enum ToolAnchor {
    /// Snap auf existierenden Node
    ExistingNode(u64, Vec2),
    /// Freie Position (neuer Node wird erstellt)
    NewPosition(Vec2),
}

impl ToolAnchor {
    /// Gibt die Welt-Position des Ankers zurück.
    pub fn position(&self) -> Vec2 {
        match self {
            ToolAnchor::ExistingNode(_, pos) => *pos,
            ToolAnchor::NewPosition(pos) => *pos,
        }
    }
}

/// Rückgabe von `on_click` — steuert den Tool-Flow.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ToolAction {
    /// Punkt registriert, weitere Eingabe nötig
    Continue,
    /// Alle nötigen Punkte gesetzt — bereit zur Ausführung
    ReadyToExecute,
    /// Vorschau aktualisiert — Klick ändert Parameter, Enter bestätigt
    UpdatePreview,
}

/// Preview-Geometrie für das Rendering (halbtransparent im Viewport).
#[derive(Debug, Clone, Default)]
pub struct ToolPreview {
    /// Vorschau-Node-Positionen
    pub nodes: Vec<Vec2>,
    /// Vorschau-Verbindungen als Index-Paare in `nodes`
    pub connections: Vec<(usize, usize)>,
}

/// Ergebnis eines Route-Tools — reine Daten, keine Mutation.
#[derive(Debug, Clone)]
pub struct ToolResult {
    /// Neue Nodes: (Position, Flag)
    pub new_nodes: Vec<(Vec2, NodeFlag)>,
    /// Verbindungen zwischen neuen Nodes: (from_idx, to_idx, Richtung, Priorität)
    /// Indizes beziehen sich auf `new_nodes`.
    pub internal_connections: Vec<(usize, usize, ConnectionDirection, ConnectionPriority)>,
    /// Verbindungen von neuen Nodes zu existierenden Nodes:
    /// (new_node_idx, existing_node_id, Richtung, Priorität)
    pub external_connections: Vec<(usize, u64, ConnectionDirection, ConnectionPriority)>,
}

// ── ToolManager ──────────────────────────────────────────────────

/// Verwaltet registrierte Route-Tools und den aktiven Tool-Index.
pub struct ToolManager {
    tools: Vec<Box<dyn RouteTool>>,
    active_index: Option<usize>,
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
        // Standard-Tools registrieren
        manager.register(Box::new(straight_line::StraightLineTool::new()));
        manager.register(Box::new(curve::CurveTool::new()));
        manager.register(Box::new(curve::CurveTool::new_cubic()));
        manager.register(Box::new(spline::SplineTool::new()));
        manager
    }

    /// Registriert ein neues Route-Tool.
    pub fn register(&mut self, tool: Box<dyn RouteTool>) {
        self.tools.push(tool);
    }

    /// Gibt die Anzahl registrierter Tools zurück.
    pub fn tool_count(&self) -> usize {
        self.tools.len()
    }

    /// Gibt Name und Index aller registrierten Tools zurück.
    pub fn tool_names(&self) -> Vec<(usize, &str)> {
        self.tools
            .iter()
            .enumerate()
            .map(|(i, t)| (i, t.name()))
            .collect()
    }

    /// Gibt Index, Name und Icon aller registrierten Tools zurück.
    pub fn tool_entries(&self) -> Vec<(usize, &str, &str)> {
        self.tools
            .iter()
            .enumerate()
            .map(|(i, t)| (i, t.name(), t.icon()))
            .collect()
    }

    /// Setzt das aktive Route-Tool per Index.
    pub fn set_active(&mut self, index: usize) {
        if index < self.tools.len() {
            // Altes Tool zurücksetzen
            if let Some(old) = self.active_index {
                if old != index {
                    self.tools[old].reset();
                }
            }
            self.active_index = Some(index);
        }
    }

    /// Gibt den Index des aktiven Tools zurück.
    pub fn active_index(&self) -> Option<usize> {
        self.active_index
    }

    /// Gibt eine Referenz auf das aktive Tool zurück.
    pub fn active_tool(&self) -> Option<&dyn RouteTool> {
        self.active_index.map(|i| self.tools[i].as_ref())
    }

    /// Gibt eine mutable Referenz auf das aktive Tool zurück.
    pub fn active_tool_mut(&mut self) -> Option<&mut dyn RouteTool> {
        let i = self.active_index?;
        Some(self.tools[i].as_mut())
    }

    /// Setzt alle Tools zurück und deaktiviert das aktive Tool.
    pub fn reset(&mut self) {
        if let Some(i) = self.active_index {
            self.tools[i].reset();
        }
        self.active_index = None;
    }
}
