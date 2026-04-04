//! Host-neutrale Read-Vertraege fuer Viewport-Overlays.

use crate::app::tools::ToolPreview;
use crate::app::BoundaryDirection;
use glam::Vec2;

/// Read-only Snapshot aller aktiven Viewport-Overlays.
#[derive(Debug, Clone, Default)]
pub struct ViewportOverlaySnapshot {
    /// Vorschau-Geometrie des aktiven Route-Tools.
    pub route_tool_preview: Option<ToolPreview>,
    /// Vorschau fuer laufende Paste-Operationen.
    pub clipboard_preview: Option<ClipboardOverlaySnapshot>,
    /// Vorschau-Linie fuer Distanzen-Resampling.
    pub distance_preview: Option<PolylineOverlaySnapshot>,
    /// Klickbare Segment-Lock-Overlay-Elemente.
    pub group_locks: Vec<GroupLockOverlaySnapshot>,
    /// Boundary-Icon-Daten fuer Gruppen.
    pub group_boundaries: Vec<GroupBoundaryOverlaySnapshot>,
    /// Hinweistext anzeigen, wenn keine Karte geladen ist.
    pub show_no_file_hint: bool,
}

/// Read-only Snapshot fuer Clipboard-/Paste-Vorschau.
#[derive(Debug, Clone, Default)]
pub struct ClipboardOverlaySnapshot {
    /// Node-Daten der Paste-Vorschau in Weltkoordinaten.
    pub nodes: Vec<ClipboardPreviewNode>,
    /// Interne Verbindungen als Index-Paare in `nodes`.
    pub connections: Vec<(usize, usize)>,
    /// Gewuenschte Deckkraft der Vorschau.
    pub opacity: f32,
}

/// Ein Node der Clipboard-Vorschau.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct ClipboardPreviewNode {
    /// Position in Weltkoordinaten.
    pub world_pos: Vec2,
    /// Gibt an, ob der Node einen Marker besitzt.
    pub has_marker: bool,
}

/// Read-only Snapshot fuer eine einfache Polyline-Vorschau.
#[derive(Debug, Clone, Default)]
pub struct PolylineOverlaySnapshot {
    /// Punkte der Polyline in Weltkoordinaten.
    pub points: Vec<Vec2>,
}

/// Klickbares Segment-Lock-Overlay.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct GroupLockOverlaySnapshot {
    /// Segment-ID fuer Folgeaktionen.
    pub segment_id: u64,
    /// Weltposition des Overlay-Ankers.
    pub world_pos: Vec2,
    /// Aktueller Lock-Zustand des Segments.
    pub locked: bool,
}

/// Boundary-Icon-Overlay fuer Ein-/Ausfahrt oder bidirektionale Knoten.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct GroupBoundaryOverlaySnapshot {
    /// Zugehoerige Segment-ID.
    pub segment_id: u64,
    /// Zugehoerige Node-ID.
    pub node_id: u64,
    /// Weltposition des Boundary-Ankers.
    pub world_pos: Vec2,
    /// Richtung des Boundary-Icons.
    pub direction: BoundaryDirection,
}
