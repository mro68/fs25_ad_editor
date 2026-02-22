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