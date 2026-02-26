//! Lifecycle-Zustand und Segment-Konfiguration für Route-Tools.

use super::super::ToolAnchor;
use super::geometry::{node_count_from_length, segment_length_from_count};

/// Welcher Wert wurde zuletzt vom User geändert?
///
/// Bestimmt die Synchronisationsrichtung zwischen Segment-Länge und Node-Anzahl:
/// - `Distance` → Node-Anzahl wird aus Länge und Segment-Abstand berechnet
/// - `NodeCount` → Segment-Abstand wird aus Länge und Node-Anzahl berechnet
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum LastEdited {
    /// User hat Segment-Länge angepasst → Node-Anzahl wird berechnet
    Distance,
    /// User hat Node-Anzahl angepasst → Segment-Länge wird berechnet
    NodeCount,
}

/// Gemeinsamer Lifecycle-Zustand aller Route-Tools.
///
/// Kapselt die vier Felder, die StraightLineTool, CurveTool und SplineTool
/// identisch teilen: letzte IDs, Endpunkt-Anker, Recreate-Flag und Snap-Radius.
#[derive(Debug, Clone)]
pub struct ToolLifecycleState {
    /// IDs der zuletzt erstellten Nodes (für Nachbearbeitung)
    pub last_created_ids: Vec<u64>,
    /// End-Anker der letzten Erstellung (für Verkettung)
    pub last_end_anchor: Option<ToolAnchor>,
    /// Signalisiert, dass Config geändert wurde und Neuberechnung nötig ist
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

    /// Gibt den End-Anker für die Verkettung zurück, wobei `NewPosition`
    /// zu `ExistingNode` hochgestuft wird, da der Node inzwischen erstellt wurde.
    /// Der gespeicherte `last_end_anchor` bleibt unverändert (wichtig für Recreate).
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

    /// Bereitet den Lifecycle-Zustand für Verkettung vor (Reset der gemeinsamen Felder).
    ///
    /// Wird in `on_click()` aller Tools aufgerufen, wenn der letzte Endpunkt
    /// als neuer Startpunkt übernommen wird.
    pub fn prepare_for_chaining(&mut self) {
        self.last_created_ids.clear();
        self.last_end_anchor = None;
        self.recreate_needed = false;
    }

    /// Speichert die zuletzt erstellten Node-IDs und setzt das Recreate-Flag zurück.
    ///
    /// Gemeinsamer Tail-Block aller `set_last_created()`-Implementierungen.
    pub fn save_created_ids(&mut self, ids: &[u64]) {
        self.last_created_ids.clear();
        self.last_created_ids.extend_from_slice(ids);
        self.recreate_needed = false;
    }

    /// Prüft ob eine vorherige Erzeugung existiert (für Adjusting-Modus).
    pub fn has_last_created(&self) -> bool {
        !self.last_created_ids.is_empty()
    }
}

/// Macro für die 7 identischen Lifecycle-Delegationsmethoden aller Route-Tools.
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
    };
}

/// Gekapselte Konfiguration für Segment-Länge und Node-Anzahl.
///
/// Alle Route-Tools nutzen das gleiche Muster: ein Slider für den minimalen
/// Abstand und einer für die Node-Anzahl, die sich gegenseitig ableiten.
/// `SegmentConfig` kapselt diese Logik inkl. der egui-Slider.
#[derive(Debug, Clone)]
pub struct SegmentConfig {
    /// Maximaler Abstand zwischen Zwischen-Nodes
    pub max_segment_length: f32,
    /// Gewünschte Anzahl Nodes (inkl. Start+End)
    pub node_count: usize,
    /// Welcher Parameter zuletzt vom User geändert wurde
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

    /// Synchronisiert den abhängigen Wert anhand der aktuellen Streckenlänge.
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

    /// Rendert die Segment-Slider im Nachbearbeitungs-Modus (mit recreate-Flag).
    ///
    /// Gibt `(changed, recreate_needed)` zurück.
    fn render_adjusting(&mut self, ui: &mut egui::Ui, length: f32, label: &str) -> (bool, bool) {
        let mut changed = false;
        let mut recreate = false;

        ui.label(format!("{}: {:.1} m", label, length));
        ui.add_space(4.0);

        ui.label("Min. Abstand:");
        let max_seg = length.max(1.0);
        if ui
            .add(egui::Slider::new(&mut self.max_segment_length, 1.0..=max_seg).suffix(" m"))
            .changed()
        {
            self.last_edited = LastEdited::Distance;
            self.node_count = node_count_from_length(length, self.max_segment_length);
            recreate = true;
            changed = true;
        }

        ui.add_space(4.0);

        ui.label("Anzahl Nodes:");
        let max_nodes = (length / 1.0).ceil().max(2.0) as usize;
        if ui
            .add(egui::Slider::new(&mut self.node_count, 2..=max_nodes))
            .changed()
        {
            self.last_edited = LastEdited::NodeCount;
            self.max_segment_length = segment_length_from_count(length, self.node_count);
            recreate = true;
            changed = true;
        }

        (changed, recreate)
    }

    /// Rendert die Segment-Slider im Live-Modus (Tool ist bereit, aber noch nicht ausgeführt).
    ///
    /// Gibt `true` zurück wenn sich etwas geändert hat.
    fn render_live(&mut self, ui: &mut egui::Ui, length: f32, label: &str) -> bool {
        let mut changed = false;

        ui.label(format!("{}: {:.1} m", label, length));
        ui.add_space(4.0);

        ui.label("Min. Abstand:");
        let max_seg = length.max(1.0);
        if ui
            .add(egui::Slider::new(&mut self.max_segment_length, 1.0..=max_seg).suffix(" m"))
            .changed()
        {
            self.last_edited = LastEdited::Distance;
            self.sync_from_length(length);
            changed = true;
        }

        ui.add_space(4.0);

        ui.label("Anzahl Nodes:");
        let max_nodes = (length / 1.0).ceil().max(2.0) as usize;
        if ui
            .add(egui::Slider::new(&mut self.node_count, 2..=max_nodes))
            .changed()
        {
            self.last_edited = LastEdited::NodeCount;
            self.sync_from_length(length);
            changed = true;
        }

        changed
    }

    /// Rendert den Segment-Slider im Default-Modus (Tool noch nicht bereit).
    ///
    /// Gibt `true` zurück wenn sich etwas geändert hat.
    fn render_default(&mut self, ui: &mut egui::Ui) -> bool {
        let mut changed = false;

        ui.label("Max. Segment-Länge:");
        if ui
            .add(egui::Slider::new(&mut self.max_segment_length, 1.0..=20.0).suffix(" m"))
            .changed()
        {
            self.last_edited = LastEdited::Distance;
            changed = true;
        }

        changed
    }

    /// Erhöht die Anzahl der Nodes um 1.
    pub fn increase_node_count(&mut self) {
        self.node_count = self.node_count.saturating_add(1);
        self.last_edited = LastEdited::NodeCount;
    }

    /// Verringert die Anzahl der Nodes um 1 (min. 2).
    pub fn decrease_node_count(&mut self) {
        self.node_count = self.node_count.saturating_sub(1).max(2);
        self.last_edited = LastEdited::NodeCount;
    }

    /// Erhöht den minimalen Abstand zwischen Nodes um 0.25.
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

/// Rendert die 3-Modus-Segment-Konfiguration (adjusting / live / default).
///
/// Gemeinsames Pattern aller Route-Tools. Gibt `(changed, recreate_needed)` zurück.
/// - `adjusting`: Nachbearbeitungs-Modus (Segment wurde bereits platziert)
/// - `ready`: Tool ist bereit zur Ausführung
/// - `length`: Aktuelle Streckenlänge (irrelevant für Default-Modus)
/// - `label`: Anzeige-Label für die Länge (z.B. "Kurvenlänge", "Spline-Länge")
pub fn render_segment_config_3modes(
    seg: &mut SegmentConfig,
    ui: &mut egui::Ui,
    adjusting: bool,
    ready: bool,
    length: f32,
    label: &str,
) -> (bool, bool) {
    if adjusting {
        seg.render_adjusting(ui, length, label)
    } else if ready {
        (seg.render_live(ui, length, label), false)
    } else {
        (seg.render_default(ui), false)
    }
}
