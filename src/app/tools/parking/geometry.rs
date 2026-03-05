//! Geometrie-Generator fuer Parkplatz-Layouts.
//!
//! Erzeugt Nodes, Connections und Marker im lokalen Koordinatensystem,
//! transformiert anschliessend nach Welt-Koordinaten.

use crate::app::tools::ToolResult;
use crate::core::{ConnectionDirection, ConnectionPriority, NodeFlag};
use glam::Vec2;

use super::state::{ParkingConfig, RampSide};

fn row_index_for_side(side: RampSide, row_count: usize) -> usize {
    if row_count <= 1 {
        0
    } else {
        match side {
            RampSide::Right => 0,
            RampSide::Left => row_count - 1,
        }
    }
}

fn side_sign_y(side: RampSide) -> f32 {
    match side {
        RampSide::Right => -1.0,
        RampSide::Left => 1.0,
    }
}

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
    if config.num_rows > 0 {
        return generate_blueprint_series_layout(origin, angle, config, priority);
    }

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

    // Rueckwaerts-Manoever (entscheidender Parking-Trick):
    // Vorwaerts in die Tasche (t0 -> t1 -> t2), dann nur rueckwaerts zurueck (t2 -> t0).
    // Damit kann der Pfadfinder ein realistisches Rangier-Manoever wie in Parking.xml abbilden.
    if tropfen_indices.len() >= 3 {
        connections.push((
            tropfen_indices[2],
            tropfen_indices[0],
            ConnectionDirection::Reverse,
            priority,
        ));
    }

    // ════════════════════════════════════════════════════════════
    // SCHRITT C: Einfahrt-Node (45°-Rampe)
    // ════════════════════════════════════════════════════════════
    // Rampen verlaufen immer in +X-Richtung (weg vom Marker am oestlichen Ende).
    let entry_target_seg =
        ((config.entry_t * num_segments as f32).round() as usize).min(num_segments);
    let entry_row_idx = row_index_for_side(config.entry_side, n);
    let entry_target_idx = row_nodes[entry_row_idx][entry_target_seg];
    let entry_target_lx = entry_target_seg as f32 * seg_len;
    let entry_target_ly = if n == 1 {
        0.0
    } else {
        (entry_row_idx as f32 - (n - 1) as f32 / 2.0) * spacing
    };
    let ramp_offset = config.ramp_length.max(0.5);
    let entry_lx = entry_target_lx - ramp_offset;
    let entry_ly = entry_target_ly + side_sign_y(config.entry_side) * ramp_offset;
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
    let exit_row_idx = row_index_for_side(config.exit_side, n);
    let exit_target_idx = row_nodes[exit_row_idx][exit_target_seg];
    let exit_target_lx = exit_target_seg as f32 * seg_len;
    let exit_target_ly = if n == 1 {
        0.0
    } else {
        (exit_row_idx as f32 - (n - 1) as f32 / 2.0) * spacing
    };
    let exit_lx = exit_target_lx + ramp_offset;
    let exit_ly = exit_target_ly + side_sign_y(config.exit_side) * ramp_offset;
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

fn generate_blueprint_series_layout(
    origin: Vec2,
    angle: f32,
    config: &ParkingConfig,
    priority: ConnectionPriority,
) -> ParkingLayout {
    let (sin_a, cos_a) = angle.sin_cos();
    let to_world = |lx: f32, ly: f32| -> Vec2 {
        Vec2::new(
            origin.x + cos_a * lx - sin_a * ly,
            origin.y + sin_a * lx + cos_a * ly,
        )
    };

    // Referenz-Blueprint in relativen Koordinaten bezogen auf Gesamtlänge L = 80m.
    // n1 Marker/Parkplatz, n4-n6 Wendegruppe, n7 Einfahrt, n8 Ausfahrt.
    let base_nodes = [
        Vec2::new(0.0, 0.0),
        Vec2::new(20.0, 0.0),
        Vec2::new(40.0, 0.0),
        Vec2::new(75.0, 0.0),
        Vec2::new(77.5, -0.5),
        Vec2::new(80.0, 0.0),
    ];
    let base_connections = [
        (0usize, 1usize, ConnectionDirection::Dual),
        (1, 2, ConnectionDirection::Dual),
        (2, 3, ConnectionDirection::Dual),
        (3, 4, ConnectionDirection::Regular),
        (4, 5, ConnectionDirection::Regular),
        (5, 3, ConnectionDirection::Reverse),
        (6, 1, ConnectionDirection::Regular),
        (2, 7, ConnectionDirection::Regular),
    ];

    let count = config.num_rows.max(1);
    let scale = (config.bay_length / 80.0).max(0.1);
    let spacing = config.row_spacing;

    let mut nodes: Vec<Vec2> = Vec::with_capacity(count * base_nodes.len());
    let mut connections: Vec<(usize, usize, ConnectionDirection, ConnectionPriority)> =
        Vec::with_capacity(count * (base_connections.len() + 2));
    let mut markers: Vec<(usize, String, String)> = Vec::with_capacity(count);

    for i in 0..count {
        let base_idx = nodes.len();
        let y_offset = i as f32 * spacing;

        for p in base_nodes {
            let lp = Vec2::new(p.x * scale, p.y * scale + y_offset);
            nodes.push(to_world(lp.x, lp.y));
        }

        // n7/n8 reagieren auf alle einstellbaren Parameter:
        // - ramp_length: Distanz
        // - entry_side/exit_side: Nord/Sued aus Marker-Sicht
        // - entry_t/exit_t: X-Bias entlang der Hauptachse
        let n2 = Vec2::new(20.0 * scale, y_offset);
        let n3 = Vec2::new(40.0 * scale, y_offset);
        let entry_bias_x = (config.entry_t - 0.5) * 10.0 * scale;
        let exit_bias_x = (config.exit_t - 0.5) * 10.0 * scale;
        let n7 = Vec2::new(
            n2.x - config.ramp_length + entry_bias_x,
            n2.y + side_sign_y(config.entry_side) * config.ramp_length,
        );
        let n8 = Vec2::new(
            n3.x + config.ramp_length + exit_bias_x,
            n3.y + side_sign_y(config.exit_side) * config.ramp_length,
        );
        nodes.push(to_world(n7.x, n7.y));
        nodes.push(to_world(n8.x, n8.y));

        for (from, to, dir) in base_connections {
            connections.push((base_idx + from, base_idx + to, dir, priority));
        }

        markers.push((
            base_idx,
            format!("{} - {:02}", config.marker_group, i + 1),
            config.marker_group.clone(),
        ));
    }

    // Mehrere Parkplaetze: Einfahrt-Kette (n7) + Ausfahrt-Kette (n8), jeweils Einbahn.
    //
    // Richtungslogik: Der Winkel an jedem Kettenpunkt darf max. ~45° betragen.
    //
    // Right-Seite (suedlich): n7/n8 liegen bei y = y_offset - ramp (suedlich der Bucht).
    //   Fahrzeuge naehern sich von Sueden → Einfahrkette nordwaerts (curr→next).
    //   Ausfahrt zeigt SE (+x, -y) → Ausfahrtkette muss suedwaerts laufen (next→curr),
    //   damit der Winkel (SE→S) = 45° bleibt statt 135°.
    //
    // Left-Seite (noerdlich): Fahrtrichtungen gespiegelt.
    //   Einfahrkette suedwaerts (next→curr), Ausfahrtkette nordwaerts (curr→next).
    if count > 1 {
        let block = base_nodes.len() + 2;
        for i in 0..(count - 1) {
            let curr = i * block;
            let next = (i + 1) * block;

            // Einfahrt-Kette: Richtung folgt dem Anfahrts-Traffic-Flow.
            let (ef, et) = match config.entry_side {
                RampSide::Right => (curr + 6, next + 6), // suedlich → nordwaerts
                RampSide::Left => (next + 6, curr + 6),  // noerdlich → suedwaerts
            };
            connections.push((ef, et, ConnectionDirection::Regular, priority));

            // Ausfahrt-Kette: Richtung entgegengesetzt zur Einfahrt (gleicher Ansatz).
            let (xf, xt) = match config.exit_side {
                RampSide::Right => (next + 7, curr + 7), // suedlich → suedwaerts raus
                RampSide::Left => (curr + 7, next + 7),  // noerdlich → nordwaerts raus
            };
            connections.push((xf, xt, ConnectionDirection::Regular, priority));
        }
    }

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
