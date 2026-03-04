//! State-Definitionen, Phase-Enum und Konstruktor fuer das Constraint-Route-Tool.

use super::super::common::{SegmentConfig, ToolLifecycleState};
use super::super::ToolAnchor;
use crate::core::{ConnectionDirection, ConnectionPriority};
use glam::Vec2;

/// Phasen des Constraint-Route-Tools.
///
/// Steuert die Klick-Abfolge:
/// 1. `Start` — Startpunkt setzen
/// 2. `End` — Endpunkt setzen
/// 3. `ControlNodes` — optionale Zwischen-Kontrollpunkte hinzufuegen (Enter bestaetigt)
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum Phase {
    /// Warten auf Startpunkt-Klick
    Start,
    /// Warten auf Endpunkt-Klick
    End,
    /// Kontrollpunkte platzieren (Enter bestaetigt, Escape bricht ab)
    ControlNodes,
}

/// Constraint-Route-Tool: Erzeugt Routen mit automatischer Winkelglaettung.
///
/// Nach Start/End-Platzierung koennen beliebig viele Zwischen-Kontrollpunkte
/// gesetzt werden. Der Solver erzeugt eine geglaettete Route die
/// Winkel-Constraints einhaelt und glatt in bestehende Verbindungen uebergeht.
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
    /// Maximale Richtungsaenderung pro Segment (Grad)
    pub(crate) max_angle_deg: f32,
    /// Richtung fuer die erzeugten Verbindungen
    pub direction: ConnectionDirection,
    /// Prioritaet fuer die erzeugten Verbindungen
    pub priority: ConnectionPriority,
    /// Gemeinsamer Lifecycle-Zustand (IDs, Endpunkt-Anker, Recreate-Flag, Snap-Radius)
    pub(crate) lifecycle: ToolLifecycleState,
    /// Start-Anker der letzten Erstellung (fuer Neuberechnung)
    pub(crate) last_start_anchor: Option<ToolAnchor>,
    /// End-Anker der letzten Erstellung (fuer Neuberechnung)
    pub(crate) last_end_anchor: Option<ToolAnchor>,
    /// Kontrollpunkte der letzten Erstellung (fuer Neuberechnung)
    pub(crate) last_control_nodes: Vec<Vec2>,
    /// Gecachte Solver-Ausgabe fuer Preview-Rendering
    pub(crate) preview_positions: Vec<Vec2>,
    /// Gecachte lineare Connection-Indizes fuer `preview_positions`
    pub(crate) preview_connections: Vec<(usize, usize)>,
    /// Gecachte Nachbar-Richtungsvektoren am Startpunkt
    pub(crate) start_neighbor_dirs: Vec<Vec2>,
    /// Gecachte Nachbar-Richtungsvektoren am Endpunkt
    pub(crate) end_neighbor_dirs: Vec<Vec2>,
    /// Vom Solver berechneter Approach-Steuerpunkt (manuell ueberschreibbar)
    pub(crate) approach_steerer: Option<Vec2>,
    /// Vom Solver berechneter Departure-Steuerpunkt (manuell ueberschreibbar)
    pub(crate) departure_steerer: Option<Vec2>,
    /// Ob der Approach-Steuerpunkt manuell verschoben wurde
    pub(crate) approach_manual: bool,
    /// Ob der Departure-Steuerpunkt manuell verschoben wurde
    pub(crate) departure_manual: bool,
    /// Minimaldistanz: Nodes die naeher beieinander liegen werden gefiltert (Meter)
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
            seg: SegmentConfig::new(10.0),
            max_angle_deg: 10.0,
            direction: ConnectionDirection::Dual,
            priority: ConnectionPriority::Regular,
            lifecycle: ToolLifecycleState::new(3.0),
            last_start_anchor: None,
            last_end_anchor: None,
            last_control_nodes: Vec::new(),
            preview_positions: Vec::new(),
            preview_connections: Vec::new(),
            start_neighbor_dirs: Vec::new(),
            end_neighbor_dirs: Vec::new(),
            approach_steerer: None,
            departure_steerer: None,
            approach_manual: false,
            departure_manual: false,
            min_distance: 2.0,
        }
    }

    /// Berechnet die Gesamtlaenge der Polyline (Start → Control-Nodes → End).
    pub(crate) fn total_distance(&self) -> f32 {
        let (Some(s), Some(e)) = (&self.start, &self.end) else {
            return 0.0;
        };
        let mut points = vec![s.position()];
        points.extend_from_slice(&self.control_nodes);
        points.push(e.position());
        polyline_length(&points)
    }

    /// Synchronisiert den jeweils abhaengigen Wert (Node-Anzahl ↔ Segment-Laenge).
    pub(crate) fn sync_derived(&mut self) {
        self.seg.sync_from_length(self.total_distance());
    }

    /// Berechnet die Solver-Vorschau und cacht sie in `preview_positions`.
    ///
    /// Manuell verschobene Steuerpunkte werden als zusaetzliche Kontrollpunkte
    /// an den Solver uebergeben. Automatisch berechnete Steuerpunkte werden
    /// aus dem Solver-Ergebnis uebernommen (sofern nicht manuell ueberschrieben).
    pub(crate) fn update_preview(&mut self) {
        let (Some(start), Some(end)) = (&self.start, &self.end) else {
            self.preview_positions.clear();
            self.preview_connections.clear();
            return;
        };

        // Kontrollpunkte fuer den Solver zusammenbauen:
        // Manuell verschobene Steuerpunkte werden als regulaere Kontrollpunkte eingefuegt.
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
        self.preview_connections = (0..self.preview_positions.len().saturating_sub(1))
            .map(|i| (i, i + 1))
            .collect();

        // Auto-Steuerpunkte uebernehmen (sofern nicht manuell ueberschrieben)
        if !self.approach_manual {
            self.approach_steerer = result.approach_steerer;
        }
        if !self.departure_manual {
            self.departure_steerer = result.departure_steerer;
        }
    }

    /// Sammelt die Nachbar-Richtungsvektoren fuer einen Anker aus der RoadMap.
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

/// Berechnet die Gesamtlaenge einer Polyline.
fn polyline_length(points: &[Vec2]) -> f32 {
    points.windows(2).map(|w| w[0].distance(w[1])).sum()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn constraint_route_defaults_match_requested_values() {
        let tool = ConstraintRouteTool::new();
        assert_eq!(tool.max_angle_deg, 10.0);
        assert_eq!(tool.seg.max_segment_length, 10.0);
        assert_eq!(tool.min_distance, 2.0);
    }
}
