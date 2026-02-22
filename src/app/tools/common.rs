//! Gemeinsame Hilfsfunktionen für Route-Tools.

use super::{ToolAnchor, ToolResult};
use crate::core::{ConnectedNeighbor, ConnectionDirection, ConnectionPriority, NodeFlag, RoadMap};
use glam::Vec2;

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
pub(crate) fn populate_neighbors(
    anchor: &ToolAnchor,
    road_map: &RoadMap,
) -> Vec<ConnectedNeighbor> {
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

/// Rendert eine Tangenten-ComboBox und gibt `true` zurück wenn die Auswahl geändert wurde.
///
/// Gemeinsamer UI-Baustein für Curve- und Spline-Tool.
pub(crate) fn render_tangent_combo(
    ui: &mut egui::Ui,
    id_salt: &str,
    label: &str,
    none_label: &str,
    current: &mut TangentSource,
    neighbors: &[ConnectedNeighbor],
) -> bool {
    let old = *current;
    let selected_text = match *current {
        TangentSource::None => none_label.to_string(),
        TangentSource::Connection { neighbor_id, angle } => {
            format!("→ Node #{} ({})", neighbor_id, angle_to_compass(angle))
        }
    };
    ui.label(label);
    egui::ComboBox::from_id_salt(id_salt)
        .selected_text(selected_text)
        .show_ui(ui, |ui| {
            ui.selectable_value(current, TangentSource::None, none_label);
            for neighbor in neighbors {
                let text = format!(
                    "→ Node #{} ({})",
                    neighbor.neighbor_id,
                    angle_to_compass(neighbor.angle)
                );
                ui.selectable_value(
                    current,
                    TangentSource::Connection {
                        neighbor_id: neighbor.neighbor_id,
                        angle: neighbor.angle,
                    },
                    text,
                );
            }
        });
    *current != old
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

// ── Gemeinsamer build_result ─────────────────────────────────

/// Baut ein `ToolResult` aus einer Positions-Sequenz und Start-/End-Ankern.
///
/// Diese Funktion enthält die gemeinsame Logik aller Route-Tools:
/// 1. Neue Nodes für Positionen erzeugen (existierende Nodes überspringen)
/// 2. Interne und externe Verbindungen zwischen aufeinanderfolgenden Positionen aufbauen
///
/// Die Geometrie (Positionen) wird vorher tool-spezifisch berechnet und übergeben.
pub(crate) fn assemble_tool_result(
    positions: &[Vec2],
    start: &ToolAnchor,
    end: &ToolAnchor,
    direction: ConnectionDirection,
    priority: ConnectionPriority,
    road_map: &RoadMap,
) -> ToolResult {
    let mut new_nodes: Vec<(Vec2, NodeFlag)> = Vec::new();
    let mut internal_connections: Vec<(usize, usize, ConnectionDirection, ConnectionPriority)> =
        Vec::new();
    let mut external_connections: Vec<(usize, u64, ConnectionDirection, ConnectionPriority)> =
        Vec::new();

    // Phase 1: Positions → neue Nodes oder existierende Nodes zuordnen
    let mut pos_to_new_idx: Vec<Option<usize>> = Vec::with_capacity(positions.len());

    for (i, &pos) in positions.iter().enumerate() {
        let is_start = i == 0;
        let is_end = i == positions.len() - 1;

        let existing_id = if is_start {
            match start {
                ToolAnchor::ExistingNode(id, _) => Some(*id),
                _ => None,
            }
        } else if is_end {
            match end {
                ToolAnchor::ExistingNode(id, _) => Some(*id),
                _ => None,
            }
        } else {
            road_map
                .nearest_node(pos)
                .filter(|hit| hit.distance < 0.01)
                .map(|hit| hit.node_id)
        };

        if existing_id.is_some() {
            pos_to_new_idx.push(None);
        } else {
            let idx = new_nodes.len();
            new_nodes.push((pos, NodeFlag::Regular));
            pos_to_new_idx.push(Some(idx));
        }
    }

    // Phase 2: Verbindungen zwischen aufeinanderfolgenden Positionen aufbauen
    for i in 0..positions.len().saturating_sub(1) {
        let a_new_idx = pos_to_new_idx[i];
        let b_new_idx = pos_to_new_idx[i + 1];

        let is_start_a = i == 0;
        let is_end_b = i + 1 == positions.len() - 1;

        let a_existing = if is_start_a {
            match start {
                ToolAnchor::ExistingNode(id, _) => Some(*id),
                _ => None,
            }
        } else {
            None
        };

        let b_existing = if is_end_b {
            match end {
                ToolAnchor::ExistingNode(id, _) => Some(*id),
                _ => None,
            }
        } else if pos_to_new_idx[i + 1].is_none() {
            road_map
                .nearest_node(positions[i + 1])
                .filter(|hit| hit.distance < 0.01)
                .map(|hit| hit.node_id)
        } else {
            None
        };

        match (a_new_idx, a_existing, b_new_idx, b_existing) {
            (Some(a), _, Some(b), _) => {
                internal_connections.push((a, b, direction, priority));
            }
            (Some(a), _, None, Some(b_id)) => {
                external_connections.push((a, b_id, direction, priority));
            }
            (None, Some(a_id), Some(b), _) => {
                external_connections.push((b, a_id, direction, priority));
            }
            _ => {}
        }
    }

    ToolResult {
        new_nodes,
        internal_connections,
        external_connections,
    }
}
