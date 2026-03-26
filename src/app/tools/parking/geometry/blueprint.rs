//! Blueprint-Serien-Layout-Generator fuer Parkplaetze.
//!
//! Erzeugt skalierbare Parkplatz-Serien anhand eines Referenz-Blueprints.

use crate::core::{ConnectionDirection, ConnectionPriority};
use glam::Vec2;

use super::super::state::{ParkingConfig, RampSide};
use super::ParkingLayout;

/// Erzeugt gleichmaessig verteilte Zwischenpositionen zwischen zwei Punkten.
///
/// Gibt eine leere Liste zurueck, wenn der Abstand `max_distance` nicht ueberschritten wird.
/// Die Endpunkte selbst sind nicht enthalten.
fn interpolate_positions(from: Vec2, to: Vec2, max_distance: f32) -> Vec<Vec2> {
    let dist = from.distance(to);
    if dist <= max_distance {
        return vec![];
    }
    let n = (dist / max_distance).ceil() as usize;
    (1..n)
        .map(|i| from.lerp(to, i as f32 / n as f32))
        .collect()
}

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
    let count = config.num_rows.max(1);

    // Geometrisches Zentrum des Row-Grids (Mitte von X- und Y-Ausdehnung aller Reihen).
    // Die Rotation erfolgt um diesen Punkt, sodass sich das Layout beim Drehen
    // um seine eigene Mitte dreht und nicht um die Ecke Row 0.
    let layout_center = Vec2::new(
        config.bay_length / 2.0,
        (count - 1) as f32 * config.row_spacing / 2.0,
    );

    // Transformiert lokale Koordinaten nach Weltkoordinaten mit Rotation um layout_center.
    // Formel: world = origin + rotate(local - center, angle) + center
    let to_world = |lx: f32, ly: f32| -> Vec2 {
        let dx = lx - layout_center.x;
        let dy = ly - layout_center.y;
        Vec2::new(
            origin.x + cos_a * dx - sin_a * dy + layout_center.x,
            origin.y + sin_a * dx + cos_a * dy + layout_center.y,
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

    let scale = (config.bay_length / 80.0).max(0.1);
    let spacing = config.row_spacing;

    let mut nodes: Vec<Vec2> = Vec::with_capacity(count * base_nodes.len());
    let mut connections: Vec<(usize, usize, ConnectionDirection, ConnectionPriority)> =
        Vec::with_capacity(count * (base_connections.len() + 2));
    let mut markers: Vec<(usize, String, String)> = Vec::with_capacity(count);
    // Rampenpunkt-Indizes pro Reihe fuer die Ketten-Verbindungen am Ende.
    let mut row_ramp_indices: Vec<(usize, usize)> = Vec::with_capacity(count);

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

        let entry_ramp_idx = base_idx + 6;
        let exit_ramp_idx = base_idx + 7;
        row_ramp_indices.push((entry_ramp_idx, exit_ramp_idx));

        // Verbindungen: Bay-Strecken (0→1, 1→2, 2→3) mit optionalen Zwischenknoten;
        // alle anderen Verbindungen werden direkt uebernommen.
        for (from, to, dir) in base_connections {
            if matches!((from, to), (0, 1) | (1, 2) | (2, 3)) {
                // Bay-Verbindungen: bei Bedarf Zwischenknoten einfuegen
                let p_from = nodes[base_idx + from];
                let p_to = nodes[base_idx + to];
                let intermediates =
                    interpolate_positions(p_from, p_to, config.max_node_distance);
                if intermediates.is_empty() {
                    connections.push((base_idx + from, base_idx + to, dir, priority));
                } else {
                    let mut prev = base_idx + from;
                    for pos in intermediates {
                        let new_idx = nodes.len();
                        nodes.push(pos);
                        connections.push((prev, new_idx, dir, priority));
                        prev = new_idx;
                    }
                    connections.push((prev, base_idx + to, dir, priority));
                }
            } else {
                connections.push((base_idx + from, base_idx + to, dir, priority));
            }
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
        for i in 0..(count - 1) {
            let (curr_entry, curr_exit) = row_ramp_indices[i];
            let (next_entry, next_exit) = row_ramp_indices[i + 1];

            // Einfahrt-Kette: Richtung folgt dem Anfahrts-Traffic-Flow.
            let (ef, et) = match config.entry_side {
                RampSide::Right => (curr_entry, next_entry), // suedlich → nordwaerts
                RampSide::Left => (next_entry, curr_entry),  // noerdlich → suedwaerts
            };
            connections.push((ef, et, ConnectionDirection::Regular, priority));

            // Ausfahrt-Kette: Richtung entgegengesetzt zur Einfahrt (gleicher Ansatz).
            let (xf, xt) = match config.exit_side {
                RampSide::Right => (next_exit, curr_exit), // suedlich → suedwaerts raus
                RampSide::Left => (curr_exit, next_exit),  // noerdlich → nordwaerts raus
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
