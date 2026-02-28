//! State-Definitionen und Konstruktor für das Gerade-Strecke-Tool.

use super::super::common::{SegmentConfig, ToolLifecycleState};
use super::super::ToolAnchor;
use crate::core::{ConnectionDirection, ConnectionPriority};

/// Gerade-Strecke-Tool
pub struct StraightLineTool {
    pub(crate) start: Option<ToolAnchor>,
    pub(crate) end: Option<ToolAnchor>,
    /// Segment-Konfiguration (Abstand / Node-Anzahl)
    pub(crate) seg: SegmentConfig,
    /// Richtung für die erzeugten Verbindungen (aus Editor-Defaults)
    pub direction: ConnectionDirection,
    /// Priorität für die erzeugten Verbindungen (aus Editor-Defaults)
    pub priority: ConnectionPriority,
    /// Gemeinsamer Lifecycle-Zustand (IDs, Endpunkt-Anker, Recreate-Flag, Snap-Radius)
    pub(crate) lifecycle: ToolLifecycleState,
    /// Start-Anker der letzten Erstellung (für Neuberechnung)
    pub(crate) last_start_anchor: Option<ToolAnchor>,
}

impl StraightLineTool {
    /// Erstellt ein neues Gerade-Strecke-Tool mit Standardwerten.
    pub fn new() -> Self {
        Self {
            start: None,
            end: None,
            seg: SegmentConfig::new(6.0),
            direction: ConnectionDirection::Dual,
            priority: ConnectionPriority::Regular,
            lifecycle: ToolLifecycleState::new(3.0), // Default, wird vom Handler überschrieben
            last_start_anchor: None,
        }
    }

    /// Berechnet die Gesamtlänge der Strecke (0.0 wenn nicht bereit).
    pub(crate) fn total_distance(&self) -> f32 {
        match (&self.start, &self.end) {
            (Some(s), Some(e)) => s.position().distance(e.position()),
            _ => 0.0,
        }
    }

    /// Synchronisiert den jeweils abhängigen Wert.
    pub(crate) fn sync_derived(&mut self) {
        self.seg.sync_from_length(self.total_distance());
    }
}

impl Default for StraightLineTool {
    fn default() -> Self {
        Self::new()
    }
}
