//! Lifecycle-Zustand und Segment-Konfiguration fuer Route-Tools.

use super::super::ToolAnchor;
use super::geometry::{
    node_count_from_length, segment_length_from_count,
    snap_with_neighbors as geom_snap_with_neighbors,
};
use crate::app::tools::snap_to_node;
use crate::app::ui_contract::{
    SegmentConfigPanelAction, SegmentConfigPanelState, SegmentPanelMode,
};
use crate::core::{ConnectedNeighbor, RoadMap};

/// Welcher Wert wurde zuletzt vom User geaendert?
///
/// Bestimmt die Synchronisationsrichtung zwischen Segment-Laenge und Node-Anzahl:
/// - `Distance` → Node-Anzahl wird aus Laenge und Segment-Abstand berechnet
/// - `NodeCount` → Segment-Abstand wird aus Laenge und Node-Anzahl berechnet
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum LastEdited {
    /// User hat Segment-Laenge angepasst → Node-Anzahl wird berechnet
    Distance,
    /// User hat Node-Anzahl angepasst → Segment-Laenge wird berechnet
    NodeCount,
}

/// Gemeinsamer Lifecycle-Zustand aller Route-Tools.
///
/// Kapselt die vier Felder, die StraightLineTool, CurveTool und SplineTool
/// identisch teilen: letzte IDs, Endpunkt-Anker, Recreate-Flag und Snap-Radius.
#[derive(Debug, Clone)]
pub struct ToolLifecycleState {
    /// IDs der zuletzt erstellten Nodes (fuer Nachbearbeitung)
    pub last_created_ids: Vec<u64>,
    /// End-Anker der letzten Erstellung (fuer Verkettung)
    pub last_end_anchor: Option<ToolAnchor>,
    /// Signalisiert, dass Config geaendert wurde und Neuberechnung noetig ist
    pub recreate_needed: bool,
    /// Snap-Radius in Welteinheiten (aus EditorOptions)
    pub snap_radius: f32,
}

impl ToolLifecycleState {
    /// Erstellt einen neuen Lifecycle-Zustand mit einem vorgegebenen Snap-Radius.
    pub fn new(snap_radius: f32) -> Self {
        Self {
            last_created_ids: Vec::new(),
            last_end_anchor: None,
            recreate_needed: false,
            snap_radius,
        }
    }

    /// Gibt den End-Anker fuer die Verkettung zurueck, wobei `NewPosition`
    /// zu `ExistingNode` hochgestuft wird, da der Node inzwischen erstellt wurde.
    /// Der gespeicherte `last_end_anchor` bleibt unveraendert (wichtig fuer Recreate).
    pub fn chaining_start_anchor(&self) -> Option<ToolAnchor> {
        let anchor = self.last_end_anchor?;
        Some(match anchor {
            ToolAnchor::NewPosition(pos) => {
                if let Some(&last_id) = self.last_created_ids.last() {
                    ToolAnchor::ExistingNode(last_id, pos)
                } else {
                    anchor
                }
            }
            existing => existing,
        })
    }

    /// Bereitet den Lifecycle-Zustand fuer Verkettung vor (Reset der gemeinsamen Felder).
    ///
    /// Wird in `on_click()` aller Tools aufgerufen, wenn der letzte Endpunkt
    /// als neuer Startpunkt uebernommen wird.
    pub fn prepare_for_chaining(&mut self) {
        self.last_created_ids.clear();
        self.last_end_anchor = None;
        self.recreate_needed = false;
    }

    /// Speichert die zuletzt erstellten Node-IDs und setzt das Recreate-Flag zurueck.
    ///
    /// Gemeinsamer Tail-Block aller `set_last_created()`-Implementierungen.
    pub fn save_created_ids(&mut self, ids: &[u64]) {
        self.last_created_ids.clear();
        self.last_created_ids.extend_from_slice(ids);
        self.recreate_needed = false;
    }

    /// Prueft ob eine vorherige Erzeugung existiert (fuer Adjusting-Modus).
    pub fn has_last_created(&self) -> bool {
        !self.last_created_ids.is_empty()
    }

    /// Snappt auf den naechsten Node am Cursor-Punkt.
    ///
    /// Kuerzel fuer `snap_to_node(pos, road_map, self.snap_radius)`.
    /// Vermeidet das explizite Weitergeben des Snap-Radius an jeder Aufrufstelle.
    pub fn snap_at(&self, pos: glam::Vec2, road_map: &RoadMap) -> ToolAnchor {
        snap_to_node(pos, road_map, self.snap_radius)
    }

    /// Snappt auf den naechsten Node und liefert dessen verbundene Nachbarn.
    ///
    /// Kuerzel fuer `snap_with_neighbors(pos, road_map, self.snap_radius)`.
    pub fn snap_with_neighbors(
        &self,
        pos: glam::Vec2,
        road_map: &RoadMap,
    ) -> (ToolAnchor, Vec<ConnectedNeighbor>) {
        geom_snap_with_neighbors(pos, road_map, self.snap_radius)
    }
}

/// Macro fuer die 7 identischen Lifecycle-Delegationsmethoden aller Route-Tools.
///
/// Vermeidet ~35 Zeilen Boilerplate pro Tool-Implementierung.
/// Erwartet, dass der Typ `self.lifecycle` (ToolLifecycleState),
/// `self.direction` (ConnectionDirection) und `self.priority` (ConnectionPriority) hat.
///
/// Wird innerhalb eines `impl RouteTool for X { ... }`-Blocks aufgerufen.
#[macro_export]
macro_rules! impl_lifecycle_delegation {
    () => {
        fn set_direction(&mut self, dir: $crate::core::ConnectionDirection) {
            self.direction = dir;
        }

        fn set_priority(&mut self, prio: $crate::core::ConnectionPriority) {
            self.priority = prio;
        }

        fn set_snap_radius(&mut self, radius: f32) {
            self.lifecycle.snap_radius = radius;
        }

        fn last_created_ids(&self) -> &[u64] {
            &self.lifecycle.last_created_ids
        }

        fn last_end_anchor(&self) -> Option<$crate::app::tools::ToolAnchor> {
            self.lifecycle.last_end_anchor
        }

        fn needs_recreate(&self) -> bool {
            self.lifecycle.recreate_needed
        }

        fn clear_recreate_flag(&mut self) {
            self.lifecycle.recreate_needed = false;
        }

        fn set_last_created(&mut self, ids: &[u64], road_map: &$crate::core::RoadMap) {
            self.lifecycle.last_end_anchor = self.current_end_anchor();
            self.save_anchors_for_recreate(road_map);
            self.lifecycle.save_created_ids(ids);
        }

        fn increase_node_count(&mut self) {
            self.seg.increase_node_count();
            self.lifecycle.recreate_needed = true;
        }

        fn decrease_node_count(&mut self) {
            self.seg.decrease_node_count();
            self.lifecycle.recreate_needed = true;
        }

        fn increase_segment_length(&mut self) {
            self.seg.increase_segment_length();
            self.lifecycle.recreate_needed = true;
        }

        fn decrease_segment_length(&mut self) {
            self.seg.decrease_segment_length();
            self.lifecycle.recreate_needed = true;
        }
    };
}

/// Macro fuer die 8 identischen Lifecycle-Delegationsmethoden fuer Tools ohne SegmentConfig.
///
/// Fuer Tools ohne `self.seg: SegmentConfig` (Bypass, Parking, FieldBoundary, RouteOffset).
/// Deckt `set_direction`, `set_priority`, `set_snap_radius` und die
/// fuenf `ToolLifecycleState`-Delegationen ab.
///
/// Erwartet `self.lifecycle` (ToolLifecycleState), `self.direction` (ConnectionDirection)
/// und `self.priority` (ConnectionPriority) am Ziel-Typ.
///
/// Wird innerhalb eines `impl RouteTool for X { ... }`-Blocks aufgerufen.
#[macro_export]
macro_rules! impl_lifecycle_delegation_no_seg {
    () => {
        fn set_direction(&mut self, dir: $crate::core::ConnectionDirection) {
            self.direction = dir;
        }

        fn set_priority(&mut self, prio: $crate::core::ConnectionPriority) {
            self.priority = prio;
        }

        fn set_snap_radius(&mut self, radius: f32) {
            self.lifecycle.snap_radius = radius;
        }

        fn last_created_ids(&self) -> &[u64] {
            &self.lifecycle.last_created_ids
        }

        fn last_end_anchor(&self) -> Option<$crate::app::tools::ToolAnchor> {
            self.lifecycle.last_end_anchor
        }

        fn needs_recreate(&self) -> bool {
            self.lifecycle.recreate_needed
        }

        fn clear_recreate_flag(&mut self) {
            self.lifecycle.recreate_needed = false;
        }

        fn set_last_created(&mut self, ids: &[u64], _road_map: &$crate::core::RoadMap) {
            self.lifecycle.save_created_ids(ids);
        }
    };
}

/// Gekapselte Konfiguration fuer Segment-Laenge und Node-Anzahl.
///
/// Alle Route-Tools nutzen das gleiche Muster: minimaler Abstand und
/// Node-Anzahl leiten sich gegenseitig aus der aktuellen Streckenlaenge ab.
#[derive(Debug, Clone)]
pub struct SegmentConfig {
    /// Maximaler Abstand zwischen Zwischen-Nodes
    pub max_segment_length: f32,
    /// Gewuenschte Anzahl Nodes (inkl. Start+End)
    pub node_count: usize,
    /// Welcher Parameter zuletzt vom User geaendert wurde
    pub last_edited: LastEdited,
}

impl SegmentConfig {
    /// Erstellt eine neue Segment-Konfiguration mit gegebenem Standard-Abstand.
    pub fn new(default_segment_length: f32) -> Self {
        Self {
            max_segment_length: default_segment_length,
            node_count: 2,
            last_edited: LastEdited::Distance,
        }
    }

    /// Synchronisiert den abhaengigen Wert anhand der aktuellen Streckenlaenge.
    pub fn sync_from_length(&mut self, length: f32) {
        if length < f32::EPSILON {
            return;
        }
        match self.last_edited {
            LastEdited::Distance => {
                self.node_count = node_count_from_length(length, self.max_segment_length);
            }
            LastEdited::NodeCount => {
                self.max_segment_length = segment_length_from_count(length, self.node_count);
            }
        }
    }

    /// Liefert den semantischen Panelzustand fuer die gemeinsame Segment-Konfiguration.
    pub fn panel_state(
        &self,
        adjusting: bool,
        ready: bool,
        length: f32,
        label: &str,
        with_node_count: bool,
    ) -> SegmentConfigPanelState {
        let mode = panel_mode(adjusting, ready);
        let (length_m, max_segment_length_max, node_count, node_count_min, node_count_max) =
            match mode {
                SegmentPanelMode::Default => (None, 20.0, None, None, None),
                SegmentPanelMode::Ready | SegmentPanelMode::Adjusting => {
                    let max_nodes = (length / 1.0).ceil().max(2.0) as usize;
                    (
                        Some(length),
                        length.max(1.0),
                        with_node_count.then_some(self.node_count),
                        with_node_count.then_some(2),
                        with_node_count.then_some(max_nodes),
                    )
                }
            };

        SegmentConfigPanelState {
            mode,
            length_label: label.to_owned(),
            length_m,
            max_segment_length: self.max_segment_length,
            max_segment_length_min: 1.0,
            max_segment_length_max,
            node_count,
            node_count_min,
            node_count_max,
        }
    }

    /// Wendet eine semantische Panel-Aktion auf die Segment-Konfiguration an.
    pub fn apply_panel_action(
        &mut self,
        action: SegmentConfigPanelAction,
        adjusting: bool,
        ready: bool,
        length: f32,
        with_node_count: bool,
    ) -> SegmentConfigApplyResult {
        let mode = panel_mode(adjusting, ready);
        match action {
            SegmentConfigPanelAction::SetMaxSegmentLength(value) => {
                let max_value = match mode {
                    SegmentPanelMode::Default => 20.0,
                    SegmentPanelMode::Ready | SegmentPanelMode::Adjusting => length.max(1.0),
                };
                let clamped = value.clamp(1.0, max_value);
                if (self.max_segment_length - clamped).abs() < f32::EPSILON {
                    return SegmentConfigApplyResult::default();
                }
                self.max_segment_length = clamped;
                self.last_edited = LastEdited::Distance;
                let recreate = if mode == SegmentPanelMode::Adjusting {
                    self.node_count = node_count_from_length(length, self.max_segment_length);
                    true
                } else {
                    if mode == SegmentPanelMode::Ready {
                        self.sync_from_length(length);
                    }
                    false
                };
                SegmentConfigApplyResult {
                    changed: true,
                    recreate,
                }
            }
            SegmentConfigPanelAction::SetNodeCount(value) => {
                if !with_node_count || mode == SegmentPanelMode::Default {
                    return SegmentConfigApplyResult::default();
                }
                let max_nodes = (length / 1.0).ceil().max(2.0) as usize;
                let clamped = value.clamp(2, max_nodes);
                if self.node_count == clamped {
                    return SegmentConfigApplyResult::default();
                }
                self.node_count = clamped;
                self.last_edited = LastEdited::NodeCount;
                let recreate = if mode == SegmentPanelMode::Adjusting {
                    self.max_segment_length = segment_length_from_count(length, self.node_count);
                    true
                } else {
                    self.sync_from_length(length);
                    false
                };
                SegmentConfigApplyResult {
                    changed: true,
                    recreate,
                }
            }
        }
    }

    /// Erhoeht die Anzahl der Nodes um 1.
    pub fn increase_node_count(&mut self) {
        self.node_count = self.node_count.saturating_add(1);
        self.last_edited = LastEdited::NodeCount;
    }

    /// Verringert die Anzahl der Nodes um 1 (min. 2).
    pub fn decrease_node_count(&mut self) {
        self.node_count = self.node_count.saturating_sub(1).max(2);
        self.last_edited = LastEdited::NodeCount;
    }

    /// Erhoeht den minimalen Abstand zwischen Nodes um 0.25.
    pub fn increase_segment_length(&mut self) {
        self.max_segment_length = (self.max_segment_length + 0.25).min(20.0);
        self.last_edited = LastEdited::Distance;
    }

    /// Verringert den minimalen Abstand zwischen Nodes um 0.25 (min. 0.1).
    pub fn decrease_segment_length(&mut self) {
        self.max_segment_length = (self.max_segment_length - 0.25).max(0.1);
        self.last_edited = LastEdited::Distance;
    }
}

/// Ergebnis einer semantischen Segment-Aktion.
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub struct SegmentConfigApplyResult {
    /// Mindestens ein Wert wurde geaendert.
    pub changed: bool,
    /// Die Aenderung erfordert eine Neuberechnung bestehender Geometrie.
    pub recreate: bool,
}

fn panel_mode(adjusting: bool, ready: bool) -> SegmentPanelMode {
    if adjusting {
        SegmentPanelMode::Adjusting
    } else if ready {
        SegmentPanelMode::Ready
    } else {
        SegmentPanelMode::Default
    }
}
