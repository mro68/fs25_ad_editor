//! Integrations-Tests fuer das Parkplatz-Layout-Tool.

use super::geometry::generate_parking_layout;
use super::state::{ParkingConfig, RampSide};
use crate::core::{ConnectionDirection, ConnectionPriority};
use glam::Vec2;

#[test]
fn test_single_row_layout() {
    let config = ParkingConfig {
        num_rows: 1,
        row_spacing: 7.0,
        bay_length: 18.0,
        entry_t: 0.3,
        exit_t: 0.7,
        ramp_length: 5.0,
        entry_side: RampSide::Right,
        exit_side: RampSide::Right,
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
        ramp_length: 5.0,
        entry_side: RampSide::Right,
        exit_side: RampSide::Right,
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
fn test_marker_names_are_numbered_in_series() {
    let config = ParkingConfig {
        num_rows: 3,
        row_spacing: 8.0,
        bay_length: 80.0,
        entry_t: 0.4,
        exit_t: 0.7,
        ramp_length: 5.0,
        entry_side: RampSide::Right,
        exit_side: RampSide::Left,
        marker_group: "Marker_Name".to_string(),
    };
    let layout = generate_parking_layout(
        Vec2::ZERO,
        0.0,
        &config,
        ConnectionDirection::Dual,
        ConnectionPriority::Regular,
    );

    assert_eq!(layout.markers.len(), 3);
    assert_eq!(layout.markers[0].1, "Marker_Name - 01");
    assert_eq!(layout.markers[1].1, "Marker_Name - 02");
    assert_eq!(layout.markers[2].1, "Marker_Name - 03");
}

#[test]
fn test_entry_and_exit_ramps_are_45_deg_and_away_from_marker() {
    let config = ParkingConfig {
        num_rows: 2,
        row_spacing: 8.0,
        bay_length: 24.0,
        entry_t: 0.5,
        exit_t: 0.5,
        ramp_length: 5.0,
        entry_side: RampSide::Right,
        exit_side: RampSide::Right,
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

#[test]
fn test_multi_parking_connects_entries_and_exits_one_way() {
    let config = ParkingConfig {
        num_rows: 3,
        row_spacing: 10.0,
        bay_length: 80.0,
        entry_t: 0.4,
        exit_t: 0.7,
        ramp_length: 6.0,
        entry_side: RampSide::Left,
        exit_side: RampSide::Right,
        marker_group: "Serie".to_string(),
    };
    let layout = generate_parking_layout(
        Vec2::ZERO,
        0.0,
        &config,
        ConnectionDirection::Dual,
        ConnectionPriority::Regular,
    );

    let has = |from: usize, to: usize| {
        layout
            .connections
            .iter()
            .any(|&(a, b, d, _)| a == from && b == to && d == ConnectionDirection::Regular)
    };

    // Blockgroesse = 8 Nodes (n1..n8)
    // entry_side=Left → Einfahrtkette laeuft SUEDWAERTS (von hohem Index→niedrig)
    // n7_3 -> n7_2 -> n7_1
    assert!(has(22, 14));
    assert!(has(14, 6));
    // exit_side=Right → Ausfahrtkette laeuft SUEDWAERTS (von hohem Index→niedrig)
    // n8_3 -> n8_2 -> n8_1
    assert!(has(23, 15));
    assert!(has(15, 7));
}

#[test]
fn test_reverse_maneuver_edge_exists_in_teardrop() {
    let config = ParkingConfig {
        num_rows: 2,
        row_spacing: 7.0,
        bay_length: 24.0,
        entry_t: 0.4,
        exit_t: 0.7,
        ramp_length: 5.0,
        entry_side: RampSide::Right,
        exit_side: RampSide::Right,
        marker_group: "Test".to_string(),
    };
    let layout = generate_parking_layout(
        Vec2::ZERO,
        0.0,
        &config,
        ConnectionDirection::Dual,
        ConnectionPriority::Regular,
    );

    let has_reverse_edge = layout
        .connections
        .iter()
        .any(|&(from, to, dir, _)| from == 5 && to == 3 && dir == ConnectionDirection::Reverse);
    assert!(
        has_reverse_edge,
        "Rueckwaerts-Kante fuer Rangiermanoever (n6->n4) fehlt"
    );
}

#[test]
fn test_single_parking_matches_requested_blueprint() {
    let config = ParkingConfig {
        num_rows: 1,
        row_spacing: 7.0,
        bay_length: 80.0,
        entry_t: 0.5,
        exit_t: 0.5,
        ramp_length: 5.0,
        entry_side: RampSide::Left,
        exit_side: RampSide::Right,
        marker_group: "Blueprint".to_string(),
    };
    let layout = generate_parking_layout(
        Vec2::ZERO,
        0.0,
        &config,
        ConnectionDirection::Dual,
        ConnectionPriority::Regular,
    );

    assert_eq!(layout.nodes.len(), 8, "Blueprint muss genau 8 Nodes haben");
    assert_eq!(
        layout.markers.len(),
        1,
        "Blueprint muss genau 1 Marker haben"
    );

    // Koordinatenpruefung
    let expected = [
        Vec2::new(0.0, 0.0),
        Vec2::new(20.0, 0.0),
        Vec2::new(40.0, 0.0),
        Vec2::new(75.0, 0.0),
        Vec2::new(77.5, -0.5),
        Vec2::new(80.0, 0.0),
        Vec2::new(15.0, 5.0),
        Vec2::new(45.0, -5.0),
    ];
    for (idx, exp) in expected.iter().enumerate() {
        let got = layout.nodes[idx];
        assert!(
            (got.x - exp.x).abs() < 0.01,
            "Node {} x stimmt nicht",
            idx + 1
        );
        assert!(
            (got.y - exp.y).abs() < 0.01,
            "Node {} y stimmt nicht",
            idx + 1
        );
    }

    // Topologiepruefung
    let has = |from: usize, to: usize, dir: ConnectionDirection| {
        layout
            .connections
            .iter()
            .any(|&(a, b, d, _)| a == from && b == to && d == dir)
    };

    assert!(has(0, 1, ConnectionDirection::Dual));
    assert!(has(1, 2, ConnectionDirection::Dual));
    assert!(has(2, 3, ConnectionDirection::Dual));
    assert!(has(3, 4, ConnectionDirection::Regular));
    assert!(has(4, 5, ConnectionDirection::Regular));
    assert!(has(5, 3, ConnectionDirection::Reverse));
    assert!(has(6, 1, ConnectionDirection::Regular));
    assert!(has(2, 7, ConnectionDirection::Regular));
}

#[test]
fn test_preview_params_move_entry_and_exit_nodes() {
    let mut config = ParkingConfig {
        num_rows: 1,
        row_spacing: 7.0,
        bay_length: 80.0,
        entry_t: 0.5,
        exit_t: 0.5,
        ramp_length: 5.0,
        entry_side: RampSide::Right,
        exit_side: RampSide::Right,
        marker_group: "Cfg".to_string(),
    };
    let a = generate_parking_layout(
        Vec2::ZERO,
        0.0,
        &config,
        ConnectionDirection::Dual,
        ConnectionPriority::Regular,
    );

    config.entry_t = 0.9;
    config.exit_t = 0.1;
    config.ramp_length = 8.0;
    config.entry_side = RampSide::Left;
    config.exit_side = RampSide::Left;
    let b = generate_parking_layout(
        Vec2::ZERO,
        0.0,
        &config,
        ConnectionDirection::Dual,
        ConnectionPriority::Regular,
    );

    // n7 und n8 muessen sich durch Konfig-Parameter sichtbar veraendern.
    let a_n7 = a.nodes[6];
    let a_n8 = a.nodes[7];
    let b_n7 = b.nodes[6];
    let b_n8 = b.nodes[7];
    assert!((a_n7 - b_n7).length() > 0.1, "n7 reagiert nicht auf Config");
    assert!((a_n8 - b_n8).length() > 0.1, "n8 reagiert nicht auf Config");
}
