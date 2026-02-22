//! Gemeinsame Hilfsfunktionen für Route-Tools.

use crate::core::{ConnectedNeighbor, RoadMap};
use super::ToolAnchor;

/// Welcher Wert wurde zuletzt vom User geändert?
///
/// Bestimmt die Synchronisationsrichtung zwischen Segment-Länge und Node-Anzahl:
/// - `Distance` → Node-Anzahl wird aus Länge und Segment-Abstand berechnet
/// - `NodeCount` → Segment-Abstand wird aus Länge und Node-Anzahl berechnet
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum LastEdited {
    /// User hat Segment-Länge angepasst → Node-Anzahl wird berechnet
    Distance,
    /// User hat Node-Anzahl angepasst → Segment-Länge wird berechnet
    NodeCount,
}

/// Wandelt einen Winkel (Radiant) in eine Kompass-Richtung um.
///
/// FS25-Koordinatensystem: +X = Ost, +Z = Süd in der Draufsicht.
pub(crate) fn angle_to_compass(angle: f32) -> &'static str {
    let deg = angle.to_degrees().rem_euclid(360.0) as u32;
    match deg {
        0..=22 | 338..=360 => "O",
        23..=67 => "SO",
        68..=112 => "S",
        113..=157 => "SW",
        158..=202 => "W",
        203..=247 => "NW",
        248..=292 => "N",
        293..=337 => "NO",
        _ => "?",
    }
}

/// Leitet die gewünschte Node-Anzahl (inkl. Start/Ende) aus Länge und Segmentabstand ab.
pub(crate) fn node_count_from_length(length: f32, max_segment_length: f32) -> usize {
    let segments = (length / max_segment_length).ceil().max(1.0) as usize;
    segments + 1
}

/// Leitet den Segmentabstand aus Länge und gewünschter Node-Anzahl ab.
pub(crate) fn segment_length_from_count(length: f32, node_count: usize) -> f32 {
    let segments = (node_count.max(2) - 1) as f32;
    length / segments
}

/// Liefert alle verbundenen Nachbarn eines Snap-Ankers aus der RoadMap.
///
/// Gibt einen leeren Vec zurück wenn der Anker kein existierender Node ist.
pub(crate) fn populate_neighbors(anchor: &ToolAnchor, road_map: &RoadMap) -> Vec<ConnectedNeighbor> {
    match anchor {
        ToolAnchor::ExistingNode(id, _) => road_map.connected_neighbors(*id),
        ToolAnchor::NewPosition(_) => Vec::new(),
    }
}

/// Quelle einer Tangente am Start- oder Endpunkt eines Route-Tools.
///
/// Wird von Curve- und Spline-Tool verwendet, um den Kontrollpunkt
/// bzw. Phantom-Punkt tangential an einer bestehenden Verbindung auszurichten.
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum TangentSource {
    /// Kein Tangenten-Vorschlag — Punkt wird manuell gesetzt
    None,
    /// Tangente aus bestehender Verbindung
    Connection { neighbor_id: u64, angle: f32 },
}

// ── Segment-Konfiguration ────────────────────────────────────

/// Gekapselte Konfiguration für Segment-Länge und Node-Anzahl.
///
/// Alle Route-Tools nutzen das gleiche Muster: ein Slider für den minimalen
/// Abstand und einer für die Node-Anzahl, die sich gegenseitig ableiten.
/// `SegmentConfig` kapselt diese Logik inkl. der egui-Slider.
#[derive(Debug, Clone)]
pub(crate) struct SegmentConfig {
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
            .add(egui::Slider::new(&mut self.max_segment_length, 1.0..=50.0).suffix(" m"))
            .changed()
        {
            self.last_edited = LastEdited::Distance;
            changed = true;
        }

        changed
    }
}