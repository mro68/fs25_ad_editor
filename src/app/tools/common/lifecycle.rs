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

    /// Setzt alle Lifecycle-Felder auf die Ausgangswerte zurück (für Tool-Reset).
    #[allow(dead_code)]
    pub fn clear(&mut self) {
        self.last_created_ids.clear();
        self.last_end_anchor = None;
        self.recreate_needed = false;
    }
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
    pub fn render_adjusting(
        &mut self,
        ui: &mut egui::Ui,
        length: f32,
        label: &str,
    ) -> (bool, bool) {
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
    pub fn render_live(&mut self, ui: &mut egui::Ui, length: f32, label: &str) -> bool {
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
    pub fn render_default(&mut self, ui: &mut egui::Ui) -> bool {
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
}
