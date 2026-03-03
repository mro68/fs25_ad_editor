//! State-Definitionen, Phase-Enum und Konstruktor für das Constraint-Route-Tool.

use super::super::common::{SegmentConfig, ToolLifecycleState};
use super::super::ToolAnchor;
use crate::core::{ConnectionDirection, ConnectionPriority};
use glam::Vec2;

/// Phasen des Constraint-Route-Tools.
///
/// Steuert die Klick-Abfolge:
/// 1. `Start` — Startpunkt setzen
/// 2. `End` — Endpunkt setzen
/// 3. `ControlNodes` — optionale Zwischen-Kontrollpunkte hinzufügen (Enter bestätigt)
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum Phase {
    /// Warten auf Startpunkt-Klick
    Start,
    /// Warten auf Endpunkt-Klick
    End,
    /// Kontrollpunkte platzieren (Enter bestätigt, Escape bricht ab)
    ControlNodes,
}

/// Constraint-Route-Tool: Erzeugt Routen mit automatischer Winkelglättung.
///
/// Nach Start/End-Platzierung können beliebig viele Zwischen-Kontrollpunkte
/// gesetzt werden. Der Solver erzeugt eine geglättete Route die
/// Winkel-Constraints einhält und glatt in bestehende Verbindungen übergeht.
/// Automatische Steuerpunkte (Approach/Departure) sind sichtbar und verschiebbar.
pub struct ConstraintRouteTool {
    /// Startpunkt der Route
    pub(crate) start: Option<ToolAnchor>,
    /// Endpunkt der Route
    pub(crate) end: Option<ToolAnchor>,
    /// Zwischen-Kontrollpunkte (beliebig viele, vom User geklickt)
    pub(crate) control_nodes: Vec<Vec2>,
    /// Aktuelle Klick-Phase
    pub(crate) phase: Phase,
    /// Index des aktuell gedraggerten Punkts (None = kein Drag aktiv)
    pub(crate) dragging: Option<DragTarget>,
    /// Segment-Konfiguration (Abstand / Node-Anzahl)
    pub(crate) seg: SegmentConfig,
    /// Maximale Richtungsänderung pro Segment (Grad)
    pub(crate) max_angle_deg: f32,
    /// Richtung für die erzeugten Verbindungen
    pub direction: ConnectionDirection,
    /// Priorität für die erzeugten Verbindungen
    pub priority: ConnectionPriority,
    /// Gemeinsamer Lifecycle-Zustand (IDs, Endpunkt-Anker, Recreate-Flag, Snap-Radius)
    pub(crate) lifecycle: ToolLifecycleState,
    /// Start-Anker der letzten Erstellung (für Neuberechnung)
    pub(crate) last_start_anchor: Option<ToolAnchor>,
    /// End-Anker der letzten Erstellung (für Neuberechnung)
    pub(crate) last_end_anchor: Option<ToolAnchor>,
    /// Kontrollpunkte der letzten Erstellung (für Neuberechnung)
    pub(crate) last_control_nodes: Vec<Vec2>,
    /// Gecachte Solver-Ausgabe für Preview-Rendering
    pub(crate) preview_positions: Vec<Vec2>,
    /// Gecachte Nachbar-Richtungsvektoren am Startpunkt
    pub(crate) start_neighbor_dirs: Vec<Vec2>,
    /// Gecachte Nachbar-Richtungsvektoren am Endpunkt
    pub(crate) end_neighbor_dirs: Vec<Vec2>,
    /// Vom Solver berechneter Approach-Steuerpunkt (manuell überschreibbar)
    pub(crate) approach_steerer: Option<Vec2>,
    /// Vom Solver berechneter Departure-Steuerpunkt (manuell überschreibbar)
    pub(crate) departure_steerer: Option<Vec2>,
    /// Ob der Approach-Steuerpunkt manuell verschoben wurde
    pub(crate) approach_manual: bool,
    /// Ob der Departure-Steuerpunkt manuell verschoben wurde
    pub(crate) departure_manual: bool,
    /// Minimaldistanz: Nodes die näher beieinander liegen werden gefiltert (Meter)
    pub(crate) min_distance: f32,
}

/// Ziel eines Drag-Vorgangs.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum DragTarget {
    /// Start-Endpunkt
    Start,
    /// End-Endpunkt
    End,
    /// Kontrollpunkt an Index i
    Control(usize),
    /// Approach-Steuerpunkt am Start
    ApproachSteerer,
    /// Departure-Steuerpunkt am Ende
    DepartureSteerer,
}

impl ConstraintRouteTool {
    /// Erstellt ein neues Constraint-Route-Tool mit Standardwerten.
    pub fn new() -> Self {
        Self {
            start: None,
            end: None,
            control_nodes: Vec::new(),
            phase: Phase::Start,
            dragging: None,
            seg: SegmentConfig::new(6.0),
            max_angle_deg: 45.0,
            direction: ConnectionDirection::Dual,
            priority: ConnectionPriority::Regular,
            lifecycle: ToolLifecycleState::new(3.0),
            last_start_anchor: None,
            last_end_anchor: None,
            last_control_nodes: Vec::new(),
            preview_positions: Vec::new(),
            start_neighbor_dirs: Vec::new(),
            end_neighbor_dirs: Vec::new(),
            approach_steerer: None,
            departure_steerer: None,
            approach_manual: false,
            departure_manual: false,
            min_distance: 1.0,
        }
    }

    /// Berechnet die Gesamtlänge der Polyline (Start → Control-Nodes → End).
    pub(crate) fn total_distance(&self) -> f32 {
        let (Some(s), Some(e)) = (&self.start, &self.end) else {
            return 0.0;
        };
        let mut points = vec![s.position()];
        points.extend_from_slice(&self.control_nodes);
        points.push(e.position());
        polyline_length(&points)
    }

    /// Synchronisiert den jeweils abhängigen Wert (Node-Anzahl ↔ Segment-Länge).
    pub(crate) fn sync_derived(&mut self) {
        self.seg.sync_from_length(self.total_distance());
    }

    /// Berechnet die Solver-Vorschau und cacht sie in `preview_positions`.
    ///
    /// Manuell verschobene Steuerpunkte werden als zusätzliche Kontrollpunkte
    /// an den Solver übergeben. Automatisch berechnete Steuerpunkte werden
    /// aus dem Solver-Ergebnis übernommen (sofern nicht manuell überschrieben).
    pub(crate) fn update_preview(&mut self) {
        let (Some(start), Some(end)) = (&self.start, &self.end) else {
            self.preview_positions.clear();
            return;
        };

        // Kontrollpunkte für den Solver zusammenbauen:
        // Manuell verschobene Steuerpunkte werden als reguläre Kontrollpunkte eingefügt.
        let mut solver_control = Vec::new();
        if self.approach_manual {
            if let Some(ap) = self.approach_steerer {
                solver_control.push(ap);
            }
        }
        solver_control.extend_from_slice(&self.control_nodes);
        if self.departure_manual {
            if let Some(dp) = self.departure_steerer {
                solver_control.push(dp);
            }
        }

        let input = super::geometry::ConstraintRouteInput {
            start: start.position(),
            end: end.position(),
            control_nodes: solver_control,
            max_segment_length_m: self.seg.max_segment_length,
            max_direction_change_deg: self.max_angle_deg,
            // Wenn Steuerpunkte manuell gesetzt, keine Auto-Berechnung
            start_neighbor_directions: if self.approach_manual {
                vec![]
            } else {
                self.start_neighbor_dirs.clone()
            },
            end_neighbor_directions: if self.departure_manual {
                vec![]
            } else {
                self.end_neighbor_dirs.clone()
            },
            min_distance: self.min_distance,
        };
        let result = super::geometry::solve_route(&input);
        self.preview_positions = result.positions;

        // Auto-Steuerpunkte übernehmen (sofern nicht manuell überschrieben)
        if !self.approach_manual {
            self.approach_steerer = result.approach_steerer;
        }
        if !self.departure_manual {
            self.departure_steerer = result.departure_steerer;
        }
    }

    /// Sammelt die Nachbar-Richtungsvektoren für einen Anker aus der RoadMap.
    pub(crate) fn collect_neighbor_dirs(
        anchor: &ToolAnchor,
        road_map: &crate::core::RoadMap,
    ) -> Vec<Vec2> {
        match anchor {
            ToolAnchor::ExistingNode(id, pos) => {
                let neighbors = road_map.connected_neighbors(*id);
                neighbors
                    .iter()
                    .map(|n| {
                        // Richtungsvektor vom Anchor zum Nachbar
                        let neighbor_pos = road_map
                            .nodes
                            .get(&n.neighbor_id)
                            .map(|node| node.position)
                            .unwrap_or(*pos);
                        (neighbor_pos - *pos).normalize_or_zero()
                    })
                    .filter(|v| v.length() > 0.0)
                    .collect()
            }
            ToolAnchor::NewPosition(_) => Vec::new(),
        }
    }
}

impl Default for ConstraintRouteTool {
    fn default() -> Self {
        Self::new()
    }
}

/// Berechnet die Gesamtlänge einer Polyline.
fn polyline_length(points: &[Vec2]) -> f32 {
    points.windows(2).map(|w| w[0].distance(w[1])).sum()
}
