//! Blueprint-Serien-Layout-Generator fuer Parkplaetze.
//!
//! Erzeugt skalierbare Parkplatz-Serien anhand eines Referenz-Blueprints.

use crate::core::{ConnectionDirection, ConnectionPriority};
use glam::Vec2;

use super::super::state::{ParkingConfig, RampSide};
use super::ParkingLayout;

/// Erzeugt ein skalierbares Blueprint-Serien-Layout fuer `config.num_rows` Parkplaetze.
///
/// Grundlage ist ein Referenz-Blueprint in relativen Koordinaten (Gesamtlaenge 80m).
/// Alle Nodes werden skaliert, versetzt und in Weltkoordinaten transformiert.
pub fn generate_blueprint_series_layout(
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

        // Einfahrt/Ausfahrt-Position als Anteil der Gesamtlaenge — Rampenwinkel bleibt konstant
        let n2 = Vec2::new(config.entry_t * config.bay_length, y_offset);
        let n3 = Vec2::new(config.exit_t * config.bay_length, y_offset);
        // n2/n3 aus base_nodes-Loop mit Bias-Positionen ueberschreiben
        nodes[base_idx + 1] = to_world(n2.x, n2.y);
        nodes[base_idx + 2] = to_world(n3.x, n3.y);
        let n7 = Vec2::new(
            n2.x - config.ramp_length,
            n2.y + super::side_sign_y(config.entry_side) * config.ramp_length,
        );
        let n8 = Vec2::new(
            n3.x + config.ramp_length,
            n3.y + super::side_sign_y(config.exit_side) * config.ramp_length,
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
