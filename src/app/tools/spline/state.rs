//! State-Definitionen und Hilfsmethoden für das Catmull-Rom-Spline-Tool.

use super::super::common::{self, SegmentConfig, TangentSource, TangentState, ToolLifecycleState};
use super::super::{ToolAnchor, ToolResult};
use super::geometry::{catmull_rom_chain_with_tangents, polyline_length, resample_by_distance};
use crate::core::{ConnectionDirection, ConnectionPriority, RoadMap};
use glam::Vec2;

/// Zwischenpunkte pro Catmull-Rom-Segment beim Dicht-Sampling.
///
/// 16 reicht für eine flüssige Preview und genaue Längenberechnung;
/// doppelt so viele (32) liefern keinen sichtbaren Qualitätsunterschied.
const SPLINE_SAMPLES_PER_SEGMENT: usize = 16;

/// Spline-Tool: Interpolierender Catmull-Rom-Spline durch geklickte Punkte.
pub struct SplineTool {
    /// Alle bestätigten Kontrollpunkte (geklickt)
    pub(crate) anchors: Vec<ToolAnchor>,
    /// Segment-Konfiguration (Abstand / Node-Anzahl)
    pub(crate) seg: SegmentConfig,
    pub direction: ConnectionDirection,
    pub priority: ConnectionPriority,
    /// Gemeinsamer Lifecycle-Zustand (IDs, Endpunkt-Anker, Recreate-Flag, Snap-Radius)
    pub(crate) lifecycle: ToolLifecycleState,
    /// Anker der letzten Erstellung (für Nachbearbeitung)
    pub(crate) last_anchors: Vec<ToolAnchor>,
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
            lifecycle: ToolLifecycleState::new(3.0), // Default, wird vom Handler überschrieben
            last_anchors: Vec::new(),
            tangents: TangentState::new(),
        }
    }

    /// Sammelt die Positionen aller Anker.
    pub(crate) fn anchor_positions(&self) -> Vec<Vec2> {
        self.anchors.iter().map(|a| a.position()).collect()
    }

    /// Berechnet einen Phantom-Punkt aus einer Tangente.
    ///
    /// Der Phantom-Punkt liegt in der Gegenrichtung der Verbindung zum Nachbar,
    /// im gleichen Abstand wie `anchor_pos` → `neighbor_pos`.
    /// Wird als virtueller p0/p3-Punkt in Catmull-Rom übergeben, um die Kurve
    /// am Rand tangential an einer bestehenden Verbindung auszurichten.
    ///
    /// - `anchor_pos`: Start- oder Endpunkt des Splines
    /// - `tangent_angle`: Winkel der Verbindung vom Anchor-Node zum Nachbar-Node (Radiant)
    /// - `neighbor_pos`: Weltposition des nächsten Spline-Kontrollpunkts (für Abstandsschätzung)
    pub(crate) fn phantom_from_tangent(
        anchor_pos: Vec2,
        tangent_angle: f32,
        neighbor_pos: Vec2,
    ) -> Vec2 {
        let dist = anchor_pos.distance(neighbor_pos).max(1.0);
        let dir = Vec2::from_angle(tangent_angle + std::f32::consts::PI);
        anchor_pos + dir * dist
    }

    /// Berechnet optionale Phantom-Punkte für Start und Ende des Splines.
    ///
    /// Gibt `(start_phantom, end_phantom)` zurück — `None` bedeutet Standard-Spiegelung.
    pub(crate) fn compute_phantoms(
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
    pub(crate) fn compute_dense_polyline(&self, extra_cursor: Option<Vec2>) -> Vec<Vec2> {
        let mut pts = self.anchor_positions();
        if let Some(c) = extra_cursor {
            pts.push(c);
        }
        if pts.len() < 2 {
            return pts;
        }
        let (start_phantom, end_phantom) =
            Self::compute_phantoms(&pts, self.tangents.tangent_start, self.tangents.tangent_end);
        catmull_rom_chain_with_tangents(
            &pts,
            SPLINE_SAMPLES_PER_SEGMENT,
            start_phantom,
            end_phantom,
        )
    }

    /// Berechnet die verteilt gesampelten Positionen (für Nodes).
    pub(crate) fn compute_resampled(&self, extra_cursor: Option<Vec2>) -> Vec<Vec2> {
        let dense = self.compute_dense_polyline(extra_cursor);
        resample_by_distance(&dense, self.seg.max_segment_length)
    }

    /// Spline-Länge über aktuelle Anker.
    pub(crate) fn spline_length(&self) -> f32 {
        let dense = self.compute_dense_polyline(None);
        polyline_length(&dense)
    }

    /// Synchronisiert den jeweils abhängigen Wert.
    pub(crate) fn sync_derived(&mut self) {
        let length = self.spline_length();
        self.seg.sync_from_length(length);
    }

    /// Spline-Länge aus gegebenen Ankern (mit Tangenten).
    pub(crate) fn spline_length_from_anchors(
        anchors: &[ToolAnchor],
        tangent_start: TangentSource,
        tangent_end: TangentSource,
    ) -> f32 {
        let pts: Vec<Vec2> = anchors.iter().map(|a| a.position()).collect();
        if pts.len() < 2 {
            return 0.0;
        }
        let (start_phantom, end_phantom) = Self::compute_phantoms(&pts, tangent_start, tangent_end);
        let dense = catmull_rom_chain_with_tangents(
            &pts,
            SPLINE_SAMPLES_PER_SEGMENT,
            start_phantom,
            end_phantom,
        );
        polyline_length(&dense)
    }

    /// Baut ein `ToolResult` aus gegebenen Ankern.
    ///
    /// Zentrale Logik für `execute()` und `execute_from_anchors()`.
    pub(crate) fn build_result_from_anchors(
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
        let dense = catmull_rom_chain_with_tangents(
            &pts,
            SPLINE_SAMPLES_PER_SEGMENT,
            start_phantom,
            end_phantom,
        );
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
