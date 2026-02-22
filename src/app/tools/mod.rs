//! Trait-basiertes Route-Tool-System für erweiterbare Strecken-Werkzeuge.
//!
//! Jedes Route-Tool implementiert den `RouteTool`-Trait und wird beim
//! `ToolManager` registriert. Tools erzeugen reine Daten (`ToolResult`),
//! die Mutation erfolgt zentral in `apply_tool_result`.

pub mod curve;
pub mod spline;
pub mod straight_line;

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

// ── Trait ────────────────────────────────────────────────────────

/// Schnittstelle für alle Route-Tools (Linie, Parkplatz, Kurve, …).
///
/// Tools sind zustandsbehaftet (Klick-Phasen) und erzeugen Preview-Geometrie
/// sowie ein `ToolResult` mit neuen Nodes/Connections.
pub trait RouteTool {
    /// Anzeigename für Toolbar
    fn name(&self) -> &str;

    /// Kurzbeschreibung / Tooltip
    fn description(&self) -> &str;

    /// Statustext für das Properties-Panel (z.B. "Startpunkt wählen")
    fn status_text(&self) -> &str;

    /// Viewport-Klick verarbeiten. Gibt die nächste Aktion zurück.
    /// `ctrl` ist true wenn Ctrl/Cmd gedrückt war.
    fn on_click(&mut self, pos: Vec2, road_map: &RoadMap, ctrl: bool) -> ToolAction;

    /// Preview-Geometrie für die aktuelle Mausposition berechnen.
    fn preview(&self, cursor_pos: Vec2, road_map: &RoadMap) -> ToolPreview;

    /// Tool-spezifische Konfiguration im Properties-Panel rendern.
    /// Gibt `true` zurück wenn sich Einstellungen geändert haben.
    fn render_config(&mut self, ui: &mut egui::Ui) -> bool;

    /// Ergebnis erzeugen (Nodes + Connections als reine Daten).
    fn execute(&self, road_map: &RoadMap) -> Option<ToolResult>;

    /// Tool-Zustand zurücksetzen (Escape / Tool-Wechsel).
    fn reset(&mut self);

    /// Ist das Tool bereit zur Ausführung?
    fn is_ready(&self) -> bool;

    /// Verbindungsrichtung vom Editor-Default übernehmen.
    fn set_direction(&mut self, _dir: ConnectionDirection) {}

    /// Straßenart vom Editor-Default übernehmen.
    fn set_priority(&mut self, _prio: ConnectionPriority) {}

    /// Snap-Radius (Welteinheiten) vom Editor übernehmen.
    fn set_snap_radius(&mut self, _radius: f32) {}

    /// Speichert die IDs der zuletzt erstellten Nodes (für nachträgliche Anpassung).
    /// `road_map` erlaubt tools, Nachbar-Informationen für Feintuning zu cachen.
    fn set_last_created(&mut self, _ids: Vec<u64>, _road_map: &RoadMap) {}

    /// Gibt die IDs der zuletzt erstellten Nodes zurück.
    fn last_created_ids(&self) -> &[u64] {
        &[]
    }

    /// Gibt den letzten Endpunkt zurück (für Verkettung).
    fn last_end_anchor(&self) -> Option<ToolAnchor> {
        None
    }

    /// Signalisiert, ob eine Neuberechnung nötig ist (Config geändert nach Erstellung).
    fn needs_recreate(&self) -> bool {
        false
    }

    /// Setzt das Recreate-Flag zurück.
    fn clear_recreate_flag(&mut self) {}

    /// Erzeugt ein ToolResult aus den gespeicherten Ankern (für Nachbearbeitung).
    fn execute_from_anchors(&self, _road_map: &RoadMap) -> Option<ToolResult> {
        None
    }

    /// Gibt die Weltpositionen aller verschiebbaren Punkte zurück (für Drag-Hit-Test).
    ///
    /// Nur nicht-leer wenn alle nötigen Punkte gesetzt sind und das Tool
    /// im Drag-Modus bereitsteht (z.B. Phase::Control mit vollständigen CPs).
    fn drag_targets(&self) -> Vec<Vec2> {
        vec![]
    }

    /// Startet einen Drag auf einem Punkt nahe `pos`.
    ///
    /// Gibt `true` zurück wenn ein Punkt gegriffen wurde, `false` wenn nichts in Reichweite.
    fn on_drag_start(&mut self, _pos: Vec2, _road_map: &RoadMap, _pick_radius: f32) -> bool {
        false
    }

    /// Aktualisiert die Position des gegriffenen Punkts während eines Drags.
    fn on_drag_update(&mut self, _pos: Vec2) {}

    /// Beendet den Drag (ggf. Re-Snap auf existierenden Node).
    fn on_drag_end(&mut self, _road_map: &RoadMap) {}
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
