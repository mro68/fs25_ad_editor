//! Geometrie-Generator fuer Parkplatz-Layouts.
//!
//! Erzeugt Nodes, Connections und Marker im lokalen Koordinatensystem,
//! transformiert anschliessend nach Welt-Koordinaten.

use crate::app::tools::ToolResult;
use crate::core::{ConnectionDirection, ConnectionPriority, NodeFlag};
use glam::Vec2;

use super::state::ParkingConfig;

/// Internes Ergebnis des Generators vor ToolResult-Konvertierung.
pub(super) struct ParkingLayout {
    /// Positionen aller Nodes in Weltkoordinaten.
    pub nodes: Vec<Vec2>,
    /// (from_idx, to_idx, direction, priority)
    pub connections: Vec<(usize, usize, ConnectionDirection, ConnectionPriority)>,
    /// (node_idx, marker_name, marker_group)
    pub markers: Vec<(usize, String, String)>,
}

/// Erzeugt ein Parkplatz-Layout aus Konfiguration + Weltposition + Winkel.
///
/// Koordinatensystem: Ursprung = Mitte oestliche Enden.
/// Lokale X-Achse = Reihenrichtung (positiv = weg vom Marker).
/// Lokale Y-Achse = senkrecht zu Reihen.
/// Danach Rotation um `angle` und Translation nach `origin`.
pub fn generate_parking_layout(
    origin: Vec2,
    angle: f32,
    config: &ParkingConfig,
    _lane_direction: ConnectionDirection,
    priority: ConnectionPriority,
) -> ParkingLayout {
    let n = config.num_rows;
    let spacing = config.row_spacing;
    let length = config.bay_length;

    // Segment-Abstand ca. 6m, mindestens 3 Nodes pro Reihe
    let num_segments = (length / 6.0).round().max(2.0) as usize;
    let seg_len = length / num_segments as f32;

    // Rotation: local → world
    let (sin_a, cos_a) = angle.sin_cos();
    let to_world = |lx: f32, ly: f32| -> Vec2 {
        Vec2::new(
            origin.x + cos_a * lx - sin_a * ly,
            origin.y + sin_a * lx + cos_a * ly,
        )
    };

    let mut nodes: Vec<Vec2> = Vec::new();
    let mut connections: Vec<(usize, usize, ConnectionDirection, ConnectionPriority)> = Vec::new();
    let mut markers: Vec<(usize, String, String)> = Vec::new();

    // Gesamthoehe aller Reihen
    let total_span = (n.max(1) - 1) as f32 * spacing;

    // ════════════════════════════════════════════════════════════
    // SCHRITT A: Reihen-Nodes (bidirektional)
    // ════════════════════════════════════════════════════════════
    // row_start_indices[i] = Index des ersten Nodes (Ost/Marker) fuer Reihe i
    let mut row_start_indices: Vec<usize> = Vec::with_capacity(n);
    // row_end_indices[i] = Index des letzten Nodes (West) fuer Reihe i
    let mut row_end_indices: Vec<usize> = Vec::with_capacity(n);
    // row_nodes[i] = Indizes aller Nodes der Reihe (Ost→West)
    let mut row_nodes: Vec<Vec<usize>> = Vec::with_capacity(n);

    for row in 0..n {
        let ly = if n == 1 {
            0.0
        } else {
            (row as f32 - (n - 1) as f32 / 2.0) * spacing
        };

        let mut this_row: Vec<usize> = Vec::with_capacity(num_segments + 1);
        for seg in 0..=num_segments {
            let lx = seg as f32 * seg_len;
            let idx = nodes.len();
            nodes.push(to_world(lx, ly));
            this_row.push(idx);

            // Bidirektionale Verbindung zum vorherigen Node in der Reihe
            if seg > 0 {
                connections.push((idx - 1, idx, ConnectionDirection::Dual, priority));
            }
        }

        let first = *this_row.first().unwrap();
        let last = *this_row.last().unwrap();
        row_start_indices.push(first);
        row_end_indices.push(last);

        // Marker am oestlichen Ende (Index 0 = Marker-Position)
        markers.push((
            first,
            format!("Parken - {} - {:02}", config.marker_group, row + 1),
            config.marker_group.clone(),
        ));

        row_nodes.push(this_row);
    }

    // ════════════════════════════════════════════════════════════
    // SCHRITT B: Tropfen-Wendekreis am Westende
    // ════════════════════════════════════════════════════════════
    // Der Tropfen verbindet die westlichsten Nodes aller Reihen
    // ueber einen Halbkreis (unidirektional).
    let tropfen_cx = length + 3.0; // Mittelpunkt 3m westlich der letzten Nodes
    let tropfen_radius = if n == 1 { 3.0 } else { total_span / 2.0 + 1.5 };

    // Tropfen-Nodes: Halbkreis von unterster Reihe (Sueden) nach oberster (Norden)
    // Richtung: Sueden (- Y) → Westen → Norden (+ Y) = Uhrzeigersinn im lokalen KS
    let tropfen_segments = 6.max(n * 2);
    let mut tropfen_indices: Vec<usize> = Vec::with_capacity(tropfen_segments + 1);

    // Startwinkel: von der untersten Reihe (-Y) = -PI/2
    // Endwinkel: zur obersten Reihe (+Y) = +PI/2
    for i in 0..=tropfen_segments {
        let t = i as f32 / tropfen_segments as f32;
        let theta = -std::f32::consts::FRAC_PI_2 + t * std::f32::consts::PI;
        let lx = tropfen_cx + tropfen_radius * theta.cos();
        let ly = tropfen_radius * theta.sin();
        let idx = nodes.len();
        nodes.push(to_world(lx, ly));
        tropfen_indices.push(idx);

        // Unidirektionale Kette: nur vorwaerts
        if i > 0 {
            connections.push((
                tropfen_indices[i - 1],
                idx,
                ConnectionDirection::Regular,
                priority,
            ));
        }
    }

    // Verbindung: letzte Reihe (suedlichste, row 0) → Tropfen-Start (unidirektional)
    if let Some(&last_of_last_row) = row_end_indices.first() {
        connections.push((
            last_of_last_row,
            tropfen_indices[0],
            ConnectionDirection::Regular,
            priority,
        ));
    }

    // Verbindung: Tropfen-Ende → erste Reihe (noerdlichste, row N-1) (unidirektional)
    if let Some(&last_of_first_row) = row_end_indices.last() {
        connections.push((
            *tropfen_indices.last().unwrap(),
            last_of_first_row,
            ConnectionDirection::Regular,
            priority,
        ));
    }

    // ════════════════════════════════════════════════════════════
    // SCHRITT C: Einfahrt-Node (45°-Rampe)
    // ════════════════════════════════════════════════════════════
    // Rampen verlaufen immer in +X-Richtung (weg vom Marker am oestlichen Ende).
    let entry_target_seg =
        ((config.entry_t * num_segments as f32).round() as usize).min(num_segments);
    let entry_row_idx = 0; // suedlichste Reihe
    let entry_target_idx = row_nodes[entry_row_idx][entry_target_seg];
    let entry_target_lx = entry_target_seg as f32 * seg_len;
    let entry_target_ly = if n == 1 {
        0.0
    } else {
        (entry_row_idx as f32 - (n - 1) as f32 / 2.0) * spacing
    };
    let ramp_offset = 5.0;
    let entry_lx = entry_target_lx - ramp_offset;
    let entry_ly = entry_target_ly - ramp_offset;
    let entry_idx = nodes.len();
    nodes.push(to_world(entry_lx, entry_ly));

    // Unidirektional: Einfahrt → Reihe, Richtung +X und 45° zur Reihe.
    connections.push((
        entry_idx,
        entry_target_idx,
        ConnectionDirection::Regular,
        priority,
    ));

    // ════════════════════════════════════════════════════════════
    // SCHRITT D: Ausfahrt-Node (45°-Rampe)
    // ════════════════════════════════════════════════════════════
    let exit_target_seg =
        ((config.exit_t * num_segments as f32).round() as usize).min(num_segments);
    let exit_row_idx = n - 1; // noerdlichste Reihe
    let exit_target_idx = row_nodes[exit_row_idx][exit_target_seg];
    let exit_target_lx = exit_target_seg as f32 * seg_len;
    let exit_target_ly = if n == 1 {
        0.0
    } else {
        (exit_row_idx as f32 - (n - 1) as f32 / 2.0) * spacing
    };
    let exit_lx = exit_target_lx + ramp_offset;
    let exit_ly = exit_target_ly + ramp_offset;
    let exit_idx = nodes.len();
    nodes.push(to_world(exit_lx, exit_ly));

    // Unidirektional: Reihe → Ausfahrt, ebenfalls Richtung +X (weg vom Marker).
    connections.push((
        exit_target_idx,
        exit_idx,
        ConnectionDirection::Regular,
        priority,
    ));

    ParkingLayout {
        nodes,
        connections,
        markers,
    }
}

/// Konvertiert ein ParkingLayout in ein ToolResult.
pub(super) fn build_parking_result(layout: ParkingLayout) -> ToolResult {
    ToolResult {
        new_nodes: layout
            .nodes
            .into_iter()
            .map(|pos| (pos, NodeFlag::Regular))
            .collect(),
        internal_connections: layout.connections,
        external_connections: vec![],
        markers: layout.markers,
    }
}

/// Konvertiert ein ParkingLayout in eine ToolPreview.
pub(super) fn build_preview(layout: &ParkingLayout) -> super::super::ToolPreview {
    super::super::ToolPreview {
        nodes: layout.nodes.clone(),
        connections: layout
            .connections
            .iter()
            .map(|&(a, b, _dir, _prio)| (a, b))
            .collect(),
        connection_styles: layout
            .connections
            .iter()
            .map(|&(_a, _b, dir, prio)| (dir, prio))
            .collect(),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_single_row_layout() {
        let config = ParkingConfig {
            num_rows: 1,
            row_spacing: 7.0,
            bay_length: 18.0,
            entry_t: 0.3,
            exit_t: 0.7,
            marker_group: "Test".to_string(),
        };
        let layout = generate_parking_layout(
            Vec2::ZERO,
            0.0,
            &config,
            ConnectionDirection::Dual,
            ConnectionPriority::Regular,
        );
        // Mindestens: Reihen-Nodes + Tropfen-Nodes + Entry + Exit
        assert!(
            layout.nodes.len() >= 5,
            "Zu wenig Nodes: {}",
            layout.nodes.len()
        );
        assert!(!layout.connections.is_empty(), "Keine Connections");
        assert_eq!(layout.markers.len(), 1, "Genau 1 Marker fuer 1 Reihe");
    }

    #[test]
    fn test_two_row_layout() {
        let config = ParkingConfig::default(); // 2 Reihen
        let layout = generate_parking_layout(
            Vec2::new(100.0, 100.0),
            0.0,
            &config,
            ConnectionDirection::Dual,
            ConnectionPriority::Regular,
        );
        assert_eq!(layout.markers.len(), 2, "2 Marker fuer 2 Reihen");
        // Pruefen dass Marker am oestlichen Ende liegen (x nahe origin)
        for &(idx, _, _) in &layout.markers {
            let pos = layout.nodes[idx];
            assert!(
                (pos.x - 100.0).abs() < 1.0,
                "Marker-Node sollte nahe am Ursprung sein, ist bei x={}",
                pos.x
            );
        }
    }

    #[test]
    fn test_rotation() {
        let config = ParkingConfig {
            num_rows: 2,
            row_spacing: 7.0,
            bay_length: 20.0,
            entry_t: 0.5,
            exit_t: 0.5,
            marker_group: "Rot".to_string(),
        };
        let layout_0 = generate_parking_layout(
            Vec2::ZERO,
            0.0,
            &config,
            ConnectionDirection::Dual,
            ConnectionPriority::Regular,
        );
        let layout_90 = generate_parking_layout(
            Vec2::ZERO,
            std::f32::consts::FRAC_PI_2,
            &config,
            ConnectionDirection::Dual,
            ConnectionPriority::Regular,
        );
        // Bei 2 Reihen hat der letzte Reihen-Node unterschiedliche Position nach Rotation
        let last_0 = layout_0.nodes[layout_0.nodes.len() - 3]; // Node vor Entry/Exit
        let last_90 = layout_90.nodes[layout_90.nodes.len() - 3];
        assert!(
            (last_0.x - last_90.x).abs() > 0.01 || (last_0.y - last_90.y).abs() > 0.01,
            "Rotation muss Positionen veraendern"
        );
    }

    #[test]
    fn test_bidirectional_connections() {
        let config = ParkingConfig::default();
        let layout = generate_parking_layout(
            Vec2::ZERO,
            0.0,
            &config,
            ConnectionDirection::Dual,
            ConnectionPriority::Regular,
        );
        let dual_count = layout
            .connections
            .iter()
            .filter(|c| c.2 == ConnectionDirection::Dual)
            .count();
        let fwd_count = layout
            .connections
            .iter()
            .filter(|c| c.2 == ConnectionDirection::Regular)
            .count();
        assert!(
            dual_count > 0,
            "Es muessen bidirektionale Verbindungen existieren"
        );
        assert!(
            fwd_count > 0,
            "Es muessen unidirektionale Verbindungen existieren (Tropfen, Ein-/Ausfahrt)"
        );
    }

    #[test]
    fn test_entry_and_exit_are_on_opposite_sides() {
        let config = ParkingConfig {
            num_rows: 2,
            row_spacing: 8.0,
            bay_length: 24.0,
            entry_t: 0.4,
            exit_t: 0.7,
            marker_group: "Test".to_string(),
        };
        let layout = generate_parking_layout(
            Vec2::ZERO,
            0.0,
            &config,
            ConnectionDirection::Dual,
            ConnectionPriority::Regular,
        );

        // Die letzten zwei Nodes sind Entry und Exit.
        let entry = layout.nodes[layout.nodes.len() - 2];
        let exit = layout.nodes[layout.nodes.len() - 1];
        assert!(entry.y < 0.0, "Entry sollte suedlich liegen, y={}", entry.y);
        assert!(exit.y > 0.0, "Exit sollte noerdlich liegen, y={}", exit.y);
    }

    #[test]
    fn test_entry_and_exit_ramps_are_45_deg_and_away_from_marker() {
        let config = ParkingConfig {
            num_rows: 2,
            row_spacing: 8.0,
            bay_length: 24.0,
            entry_t: 0.4,
            exit_t: 0.7,
            marker_group: "Test".to_string(),
        };
        let layout = generate_parking_layout(
            Vec2::ZERO,
            0.0,
            &config,
            ConnectionDirection::Dual,
            ConnectionPriority::Regular,
        );

        let entry_idx = layout.nodes.len() - 2;
        let exit_idx = layout.nodes.len() - 1;

        let entry_conn = layout
            .connections
            .iter()
            .find(|(from, _, dir, _)| *from == entry_idx && *dir == ConnectionDirection::Regular)
            .expect("Entry-Connection fehlt");
        let exit_conn = layout
            .connections
            .iter()
            .find(|(_, to, dir, _)| *to == exit_idx && *dir == ConnectionDirection::Regular)
            .expect("Exit-Connection fehlt");

        let entry_from = layout.nodes[entry_conn.0];
        let entry_to = layout.nodes[entry_conn.1];
        let exit_from = layout.nodes[exit_conn.0];
        let exit_to = layout.nodes[exit_conn.1];

        let entry_dx = entry_to.x - entry_from.x;
        let entry_dy = entry_to.y - entry_from.y;
        let exit_dx = exit_to.x - exit_from.x;
        let exit_dy = exit_to.y - exit_from.y;

        // Richtung immer weg vom Marker = +X in lokalem KS (hier Winkel 0).
        assert!(entry_dx > 0.0, "Entry muss in +X zeigen, dx={}", entry_dx);
        assert!(exit_dx > 0.0, "Exit muss in +X zeigen, dx={}", exit_dx);

        // 45°-Rampen: |dx| == |dy| (mit kleiner Toleranz).
        assert!(
            (entry_dx.abs() - entry_dy.abs()).abs() < 0.01,
            "Entry-Rampe ist nicht 45°, dx={}, dy={}",
            entry_dx,
            entry_dy
        );
        assert!(
            (exit_dx.abs() - exit_dy.abs()).abs() < 0.01,
            "Exit-Rampe ist nicht 45°, dx={}, dy={}",
            exit_dx,
            exit_dy
        );
    }
}
