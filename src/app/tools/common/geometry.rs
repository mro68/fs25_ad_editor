//! Rein-mathematische Hilfsfunktionen ohne egui-Abhängigkeit.

use crate::core::{ConnectedNeighbor, RoadMap};

use super::super::ToolAnchor;

/// Wandelt einen Winkel (Radiant) in eine Kompass-Richtung um.
///
/// FS25-Koordinatensystem: +X = Ost, +Z = Süd in der Draufsicht.
pub fn angle_to_compass(angle: f32) -> &'static str {
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
pub fn node_count_from_length(length: f32, max_segment_length: f32) -> usize {
    let segments = (length / max_segment_length).ceil().max(1.0) as usize;
    segments + 1
}

/// Leitet den Segmentabstand aus Länge und gewünschter Node-Anzahl ab.
pub fn segment_length_from_count(length: f32, node_count: usize) -> f32 {
    let segments = (node_count.max(2) - 1) as f32;
    length / segments
}

/// Liefert alle verbundenen Nachbarn eines Snap-Ankers aus der RoadMap.
///
/// Gibt einen leeren Vec zurück wenn der Anker kein existierender Node ist.
pub fn populate_neighbors(anchor: &ToolAnchor, road_map: &RoadMap) -> Vec<ConnectedNeighbor> {
    match anchor {
        ToolAnchor::ExistingNode(id, _) => road_map.connected_neighbors(*id),
        ToolAnchor::NewPosition(_) => Vec::new(),
    }
}

/// Erzeugt lineare Connections `[(0,1), (1,2), ...]` für eine Polyline.
///
/// Gemeinsames Pattern aller Route-Tool-Previews.
pub fn linear_connections(count: usize) -> Vec<(usize, usize)> {
    (0..count.saturating_sub(1)).map(|i| (i, i + 1)).collect()
}

/// Formatiert Tangenten-Optionen aus Nachbar-Liste als `(TangentSource, Label)`-Paare.
///
/// Gemeinsame Daten-Aufbereitung für ComboBox und Kontextmenü.
pub fn tangent_options(
    neighbors: &[ConnectedNeighbor],
) -> Vec<(super::TangentSource, String)> {
    let mut opts = vec![(super::TangentSource::None, "Manuell".to_string())];
    for n in neighbors {
        opts.push((
            super::TangentSource::Connection {
                neighbor_id: n.neighbor_id,
                angle: n.angle,
            },
            format!("→ Node #{} ({})", n.neighbor_id, angle_to_compass(n.angle)),
        ));
    }
    opts
}
