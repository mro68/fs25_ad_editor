//! Spline-Tool: Interpolierender Catmull-Rom-Spline durch alle geklickten Punkte.
//!
//! **Ablauf:** Punkte per Klick setzen → Vorschau wird fortlaufend aktualisiert
//! (Cursor = nächster Punkt) → Enter bestätigt den Kurs.
//!
//! Einstellungsmöglichkeiten: Max. Segment-Länge / Node-Anzahl, Richtung, Priorität.
//! Mindestens 2 Punkte für eine gerade Strecke, ab 3 Punkten entsteht eine Kurve.

mod config_ui;
mod geometry;

use self::geometry::{catmull_rom_chain_with_tangents, polyline_length, resample_by_distance};
use super::{
    common::{
        self, populate_neighbors, SegmentConfig, TangentSource, TangentState, ToolLifecycleState,
    },
    snap_to_node, RouteTool, ToolAction, ToolAnchor, ToolPreview, ToolResult,
};
use crate::core::{ConnectionDirection, ConnectionPriority, RoadMap};
use crate::shared::SNAP_RADIUS;
use glam::Vec2;

// ── Spline-Tool ──────────────────────────────────────────────────

/// Spline-Tool: Interpolierender Catmull-Rom-Spline durch geklickte Punkte.
pub struct SplineTool {
    /// Alle bestätigten Kontrollpunkte (geklickt)
    anchors: Vec<ToolAnchor>,
    /// Segment-Konfiguration (Abstand / Node-Anzahl)
    pub(crate) seg: SegmentConfig,
    pub direction: ConnectionDirection,
    pub priority: ConnectionPriority,
    /// Gemeinsamer Lifecycle-Zustand (IDs, Endpunkt-Anker, Recreate-Flag, Snap-Radius)
    pub(crate) lifecycle: ToolLifecycleState,
    /// Anker der letzten Erstellung (für Nachbearbeitung)
    last_anchors: Vec<ToolAnchor>,
    /// Tangenten-Zustand (Start/Ende, Nachbarn-Cache, Recreation-Kopien)
    pub(crate) tangents: TangentState,
}

impl SplineTool {
    /// Erstellt ein neues Spline-Tool mit Standardwerten.
    pub fn new() -> Self {
        Self {
            anchors: Vec::new(),
            seg: SegmentConfig::new(2.0),
            direction: ConnectionDirection::Dual,
            priority: ConnectionPriority::Regular,
            lifecycle: ToolLifecycleState::new(SNAP_RADIUS),
            last_anchors: Vec::new(),
            tangents: TangentState::new(),
        }
    }

    /// Sammelt die Positionen aller Anker.
    fn anchor_positions(&self) -> Vec<Vec2> {
        self.anchors.iter().map(|a| a.position()).collect()
    }

    /// Berechnet einen Phantom-Punkt aus einer Tangente.
    ///
    /// Der Phantom-Punkt liegt in der Verlängerung der Tangente,
    /// im Abstand des ersten/letzten Segments (wie bei der Spiegelung).
    fn phantom_from_tangent(anchor_pos: Vec2, tangent_angle: f32, neighbor_pos: Vec2) -> Vec2 {
        // Phantom-Punkt in Richtung weg vom Nachbar, im gleichen Abstand
        let dist = anchor_pos.distance(neighbor_pos).max(1.0);
        // Tangent-Angle zeigt zum Nachbar → Phantom in Gegenrichtung
        let dir = Vec2::from_angle(tangent_angle + std::f32::consts::PI);
        anchor_pos + dir * dist
    }

    /// Berechnet die Phantom-Punkte für Start und Ende basierend auf Tangenten.
    fn compute_phantoms(
        points: &[Vec2],
        tangent_start: TangentSource,
        tangent_end: TangentSource,
    ) -> (Option<Vec2>, Option<Vec2>) {
        let start_phantom = if let TangentSource::Connection { angle, .. } = tangent_start {
            if points.len() >= 2 {
                Some(Self::phantom_from_tangent(points[0], angle, points[1]))
            } else {
                None
            }
        } else {
            None
        };

        let end_phantom = if let TangentSource::Connection { angle, .. } = tangent_end {
            if points.len() >= 2 {
                let n = points.len();
                Some(Self::phantom_from_tangent(
                    points[n - 1],
                    angle,
                    points[n - 2],
                ))
            } else {
                None
            }
        } else {
            None
        };

        (start_phantom, end_phantom)
    }

    /// Berechnet die dichte Spline-Polyline aus den Ankern (+ optionaler Cursor-Position).
    fn compute_dense_polyline(&self, extra_cursor: Option<Vec2>) -> Vec<Vec2> {
        let mut pts = self.anchor_positions();
        if let Some(c) = extra_cursor {
            pts.push(c);
        }
        if pts.len() < 2 {
            return pts;
        }
        let (start_phantom, end_phantom) =
            Self::compute_phantoms(&pts, self.tangents.tangent_start, self.tangents.tangent_end);
        catmull_rom_chain_with_tangents(&pts, 32, start_phantom, end_phantom)
    }

    /// Berechnet die verteilt gesampelten Positionen (für Nodes).
    fn compute_resampled(&self, extra_cursor: Option<Vec2>) -> Vec<Vec2> {
        let dense = self.compute_dense_polyline(extra_cursor);
        resample_by_distance(&dense, self.seg.max_segment_length)
    }

    /// Spline-Länge über aktuelle Anker.
    fn spline_length(&self) -> f32 {
        let dense = self.compute_dense_polyline(None);
        polyline_length(&dense)
    }

    /// Synchronisiert den jeweils abhängigen Wert.
    fn sync_derived(&mut self) {
        let length = self.spline_length();
        self.seg.sync_from_length(length);
    }

    /// Spline-Länge aus gegebenen Ankern (mit Tangenten).
    fn spline_length_from_anchors(
        anchors: &[ToolAnchor],
        tangent_start: TangentSource,
        tangent_end: TangentSource,
    ) -> f32 {
        let pts: Vec<Vec2> = anchors.iter().map(|a| a.position()).collect();
        if pts.len() < 2 {
            return 0.0;
        }
        let (start_phantom, end_phantom) = Self::compute_phantoms(&pts, tangent_start, tangent_end);
        let dense = catmull_rom_chain_with_tangents(&pts, 32, start_phantom, end_phantom);
        polyline_length(&dense)
    }

    /// Baut ToolResult aus gegebenen Ankern.
    fn build_result_from_anchors(
        anchors: &[ToolAnchor],
        max_segment_length: f32,
        direction: ConnectionDirection,
        priority: ConnectionPriority,
        tangent_start: TangentSource,
        tangent_end: TangentSource,
        road_map: &RoadMap,
    ) -> Option<ToolResult> {
        if anchors.len() < 2 {
            return None;
        }

        let pts: Vec<Vec2> = anchors.iter().map(|a| a.position()).collect();
        let (start_phantom, end_phantom) = Self::compute_phantoms(&pts, tangent_start, tangent_end);
        let dense = catmull_rom_chain_with_tangents(&pts, 32, start_phantom, end_phantom);
        let positions = resample_by_distance(&dense, max_segment_length);

        let first_anchor = anchors.first()?;
        let last_anchor = anchors.last()?;

        Some(common::assemble_tool_result(
            &positions,
            first_anchor,
            last_anchor,
            direction,
            priority,
            road_map,
        ))
    }
}

impl Default for SplineTool {
    fn default() -> Self {
        Self::new()
    }
}

impl RouteTool for SplineTool {
    fn name(&self) -> &str {
        "〰️ Spline"
    }

    fn description(&self) -> &str {
        "Zeichnet einen Catmull-Rom-Spline durch alle geklickten Punkte"
    }

    fn status_text(&self) -> &str {
        match self.anchors.len() {
            0 => "Startpunkt klicken",
            1 => "Nächsten Punkt klicken (mind. 2 Punkte)",
            _ => "Weitere Punkte klicken — Enter bestätigt, Escape abbrechen",
        }
    }

    fn on_click(&mut self, pos: Vec2, road_map: &RoadMap, _ctrl: bool) -> ToolAction {
        let anchor = snap_to_node(pos, road_map, self.lifecycle.snap_radius);

        if self.anchors.is_empty() {
            // Verkettung: letzten Endpunkt als Start verwenden
            if let Some(last_end) = self.lifecycle.last_end_anchor {
                self.lifecycle.last_created_ids.clear();
                self.last_anchors.clear();
                self.lifecycle.last_end_anchor = None;
                self.lifecycle.recreate_needed = false;
                self.tangents.reset_tangents();
                self.anchors.push(last_end);
                self.anchors.push(anchor);
                self.sync_derived();
                return ToolAction::UpdatePreview;
            }
        }

        self.anchors.push(anchor);

        if self.anchors.len() >= 2 {
            self.sync_derived();
            ToolAction::UpdatePreview
        } else {
            ToolAction::Continue
        }
    }

    fn preview(&self, cursor_pos: Vec2, road_map: &RoadMap) -> ToolPreview {
        if self.anchors.is_empty() {
            return ToolPreview::default();
        }

        // Cursor als nächster Punkt (gesnappt)
        let snapped_cursor =
            snap_to_node(cursor_pos, road_map, self.lifecycle.snap_radius).position();

        let positions = if self.anchors.len() == 1 {
            // Nur Start + Cursor → gerade Linie (Preview)
            let start = self.anchors[0].position();
            vec![start, snapped_cursor]
        } else {
            // Spline durch alle Anker + Cursor
            self.compute_resampled(Some(snapped_cursor))
        };

        let connections: Vec<(usize, usize)> = (0..positions.len().saturating_sub(1))
            .map(|i| (i, i + 1))
            .collect();

        // Kontrollpunkte (Anker) als zusätzliche Preview-Nodes (für visuelle Markierung)
        let mut nodes = positions;
        for anchor in &self.anchors {
            nodes.push(anchor.position());
        }
        nodes.push(snapped_cursor);

        ToolPreview { nodes, connections }
    }

    fn render_config(&mut self, ui: &mut egui::Ui) -> bool {
        self.render_config_view(ui)
    }

    fn execute(&self, road_map: &RoadMap) -> Option<ToolResult> {
        Self::build_result_from_anchors(
            &self.anchors,
            self.seg.max_segment_length,
            self.direction,
            self.priority,
            self.tangents.tangent_start,
            self.tangents.tangent_end,
            road_map,
        )
    }

    fn reset(&mut self) {
        self.anchors.clear();
        self.tangents.reset_tangents();
        // start_neighbors / end_neighbors bleiben erhalten —
        // werden in set_last_created befüllt und im Nachbearbeitungs-Modus benötigt
    }

    fn is_ready(&self) -> bool {
        self.anchors.len() >= 2
    }

    fn set_direction(&mut self, dir: ConnectionDirection) {
        self.direction = dir;
    }

    fn set_priority(&mut self, prio: ConnectionPriority) {
        self.priority = prio;
    }

    fn set_snap_radius(&mut self, radius: f32) {
        self.lifecycle.snap_radius = radius;
    }

    fn set_last_created(&mut self, ids: Vec<u64>, road_map: &RoadMap) {
        // Nur bei Erst-Erstellung Anker übernehmen; bei Recreate bleiben last_anchors erhalten
        if !self.anchors.is_empty() {
            self.last_anchors = self.anchors.clone();
            if let Some(last) = self.anchors.last() {
                self.lifecycle.last_end_anchor = Some(*last);
            }
        }
        // Nachbarn aus den richtigen Ankern befüllen (anchors oder last_anchors)
        let source = if !self.anchors.is_empty() {
            &self.anchors
        } else {
            &self.last_anchors
        };
        if let Some(first) = source.first() {
            self.tangents.start_neighbors = populate_neighbors(first, road_map);
        }
        if let Some(last) = source.last() {
            self.tangents.end_neighbors = populate_neighbors(last, road_map);
        }
        self.tangents.save_for_recreate();
        self.lifecycle.last_created_ids = ids;
        self.lifecycle.recreate_needed = false;
    }

    fn last_created_ids(&self) -> &[u64] {
        &self.lifecycle.last_created_ids
    }

    fn last_end_anchor(&self) -> Option<ToolAnchor> {
        self.lifecycle.last_end_anchor
    }

    fn needs_recreate(&self) -> bool {
        self.lifecycle.recreate_needed
    }

    fn clear_recreate_flag(&mut self) {
        self.lifecycle.recreate_needed = false;
    }

    fn execute_from_anchors(&self, road_map: &RoadMap) -> Option<ToolResult> {
        // Aktuelle Tangenten verwenden (nicht last_tangent_*),
        // damit Änderungen im Nachbearbeitungs-Modus wirksam werden
        Self::build_result_from_anchors(
            &self.last_anchors,
            self.seg.max_segment_length,
            self.direction,
            self.priority,
            self.tangents.tangent_start,
            self.tangents.tangent_end,
            road_map,
        )
    }
}

#[cfg(test)]
mod tests;
