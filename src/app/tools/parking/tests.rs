//! Integrations-Tests fuer das Parkplatz-Layout-Tool.

use super::geometry::{build_parking_result, build_preview, generate_parking_layout};
use super::state::{ParkingConfig, ParkingPhase, ParkingTool, RampSide};
use crate::app::group_registry::GroupKind;
use crate::app::tools::{RouteTool, RouteToolCore, RouteToolRotate};
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
        max_node_distance: f32::MAX,
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
        max_node_distance: f32::MAX,
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
        max_node_distance: f32::MAX,
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
        max_node_distance: f32::MAX,
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
        max_node_distance: f32::MAX,
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
        max_node_distance: f32::MAX,
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
        entry_t: 0.25,
        exit_t: 0.5,
        ramp_length: 5.0,
        entry_side: RampSide::Left,
        exit_side: RampSide::Right,
        marker_group: "Blueprint".to_string(),
        max_node_distance: f32::MAX,
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
        max_node_distance: f32::MAX,
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

/// Prueft exakte Groessen des Blueprint-Layouts fuer mehrere Reihen.
#[test]
fn test_blueprint_series_has_expected_node_and_connection_counts() {
    let config = ParkingConfig {
        num_rows: 2,
        row_spacing: 9.0,
        bay_length: 80.0,
        entry_t: 0.4,
        exit_t: 0.6,
        ramp_length: 5.0,
        entry_side: RampSide::Right,
        exit_side: RampSide::Right,
        marker_group: "Serie2".to_string(),
        max_node_distance: f32::MAX,
    };

    let layout = generate_parking_layout(
        Vec2::ZERO,
        0.0,
        &config,
        ConnectionDirection::Dual,
        ConnectionPriority::Regular,
    );

    // Pro Reihe entstehen 8 Nodes (6 Blueprint + n7 + n8).
    assert_eq!(layout.nodes.len(), 16, "Falsche Anzahl Nodes fuer 2 Reihen");
    assert_eq!(layout.markers.len(), 2, "Es muessen 2 Marker entstehen");

    // 8 Basisverbindungen pro Reihe + 2 Serienverbindungen fuer count=2.
    assert_eq!(
        layout.connections.len(),
        18,
        "Falsche Anzahl Verbindungen fuer 2 Reihen"
    );
}

/// Prueft Marker-Positionen bei 0° fuer definierte Reihenabstaende.
#[test]
fn test_marker_positions_follow_origin_and_row_spacing_without_rotation() {
    let config = ParkingConfig {
        num_rows: 2,
        row_spacing: 11.0,
        bay_length: 80.0,
        entry_t: 0.5,
        exit_t: 0.5,
        ramp_length: 5.0,
        entry_side: RampSide::Right,
        exit_side: RampSide::Right,
        marker_group: "Pos".to_string(),
        max_node_distance: f32::MAX,
    };

    let origin = Vec2::new(10.0, 20.0);
    let layout = generate_parking_layout(
        origin,
        0.0,
        &config,
        ConnectionDirection::Dual,
        ConnectionPriority::Regular,
    );

    let marker_1 = layout.nodes[layout.markers[0].0];
    let marker_2 = layout.nodes[layout.markers[1].0];

    assert!((marker_1 - Vec2::new(10.0, 20.0)).length() < 0.001);
    assert!((marker_2 - Vec2::new(10.0, 31.0)).length() < 0.001);
}

/// Prueft 90°-Drehung: Rotation erfolgt um das geometrische Zentrum des Layouts,
/// nicht um die Ecke Row 0. Das Zentrum (bay_length/2, (rows-1)*spacing/2) = (40, 6)
/// bleibt in Weltkoordinaten an seiner Ausgangsposition (40, 6), waehrend alle
/// anderen Nodes um diesen Punkt rotieren.
#[test]
fn test_marker_positions_rotate_correctly_at_ninety_degrees() {
    let config = ParkingConfig {
        num_rows: 2,
        row_spacing: 12.0,
        bay_length: 80.0,
        entry_t: 0.5,
        exit_t: 0.5,
        ramp_length: 5.0,
        entry_side: RampSide::Right,
        exit_side: RampSide::Right,
        marker_group: "Rot90".to_string(),
        max_node_distance: f32::MAX,
    };

    let layout = generate_parking_layout(
        Vec2::ZERO,
        std::f32::consts::FRAC_PI_2,
        &config,
        ConnectionDirection::Dual,
        ConnectionPriority::Regular,
    );

    let marker_1 = layout.nodes[layout.markers[0].0];
    let marker_2 = layout.nodes[layout.markers[1].0];

    // Bei 90°-Drehung um Zentrum (40, 6):
    //   Marker 1 (lokal 0,0):  world = (46, -34)
    //   Marker 2 (lokal 0,12): world = (34, -34)
    assert!(
        (marker_1 - Vec2::new(46.0, -34.0)).length() < 0.01,
        "Marker 1 bei 90°: erwartet (46, -34), got {:?}",
        marker_1
    );
    assert!(
        (marker_2 - Vec2::new(34.0, -34.0)).length() < 0.01,
        "Marker 2 bei 90°: erwartet (34, -34), got {:?}",
        marker_2
    );
}

/// Prueft Richtungslogik der Serien-Ketten fuer rechte Seite.
#[test]
fn test_blueprint_series_chain_direction_for_right_side() {
    let config = ParkingConfig {
        num_rows: 3,
        row_spacing: 10.0,
        bay_length: 80.0,
        entry_t: 0.4,
        exit_t: 0.7,
        ramp_length: 6.0,
        entry_side: RampSide::Right,
        exit_side: RampSide::Right,
        marker_group: "Rechts".to_string(),
        max_node_distance: f32::MAX,
    };

    let layout = generate_parking_layout(
        Vec2::ZERO,
        0.0,
        &config,
        ConnectionDirection::Dual,
        ConnectionPriority::Regular,
    );

    let has_regular = |from: usize, to: usize| {
        layout
            .connections
            .iter()
            .any(|&(a, b, d, _)| a == from && b == to && d == ConnectionDirection::Regular)
    };

    // Entry-Kette rechts: 6 -> 14 -> 22.
    assert!(has_regular(6, 14));
    assert!(has_regular(14, 22));
    // Exit-Kette rechts: 23 -> 15 -> 7.
    assert!(has_regular(23, 15));
    assert!(has_regular(15, 7));
}

/// Prueft robuste Skalierung bei sehr kleiner Laenge und sehr grossem Abstand.
#[test]
fn test_blueprint_extreme_scale_and_spacing_remain_finite() {
    let config = ParkingConfig {
        num_rows: 2,
        row_spacing: 500.0,
        bay_length: 0.5,
        entry_t: 0.0,
        exit_t: 1.0,
        ramp_length: 0.2,
        entry_side: RampSide::Left,
        exit_side: RampSide::Right,
        marker_group: "Extreme".to_string(),
        max_node_distance: f32::MAX,
    };

    let layout = generate_parking_layout(
        Vec2::new(1000.0, -2000.0),
        0.0,
        &config,
        ConnectionDirection::Dual,
        ConnectionPriority::Regular,
    );

    assert_eq!(layout.nodes.len(), 16);
    assert!(
        layout
            .nodes
            .iter()
            .all(|p| p.x.is_finite() && p.y.is_finite()),
        "Alle Punkte muessen endlich sein"
    );
}

/// Prueft, dass Preview-Konvertierung Topologie und Stil konsistent uebernimmt.
#[test]
fn test_build_preview_preserves_connection_order_and_styles() {
    let config = ParkingConfig {
        num_rows: 1,
        row_spacing: 7.0,
        bay_length: 80.0,
        entry_t: 0.5,
        exit_t: 0.5,
        ramp_length: 5.0,
        entry_side: RampSide::Right,
        exit_side: RampSide::Left,
        marker_group: "Preview".to_string(),
        max_node_distance: f32::MAX,
    };
    let layout = generate_parking_layout(
        Vec2::ZERO,
        0.0,
        &config,
        ConnectionDirection::Dual,
        ConnectionPriority::SubPriority,
    );

    let preview = build_preview(&layout);

    assert_eq!(preview.nodes.len(), layout.nodes.len());
    assert_eq!(preview.connections.len(), layout.connections.len());
    assert_eq!(preview.connection_styles.len(), layout.connections.len());

    for (i, &(from, to, dir, prio)) in layout.connections.iter().enumerate() {
        assert_eq!(preview.connections[i], (from, to));
        assert_eq!(preview.connection_styles[i], (dir, prio));
    }
}

/// Prueft, dass Result-Konvertierung NodeFlags und Marker unveraendert abbildet.
#[test]
fn test_build_parking_result_sets_regular_flags_and_empty_external_connections() {
    let config = ParkingConfig {
        num_rows: 1,
        row_spacing: 7.0,
        bay_length: 80.0,
        entry_t: 0.5,
        exit_t: 0.5,
        ramp_length: 5.0,
        entry_side: RampSide::Right,
        exit_side: RampSide::Right,
        marker_group: "Result".to_string(),
        max_node_distance: f32::MAX,
    };
    let layout = generate_parking_layout(
        Vec2::new(5.0, 6.0),
        0.0,
        &config,
        ConnectionDirection::Dual,
        ConnectionPriority::Regular,
    );

    let expected_node_count = layout.nodes.len();
    let expected_conn_count = layout.connections.len();
    let expected_markers = layout.markers.clone();

    let result = build_parking_result(layout);

    assert_eq!(result.new_nodes.len(), expected_node_count);
    assert_eq!(result.internal_connections.len(), expected_conn_count);
    assert_eq!(result.external_connections.len(), 0);
    assert_eq!(result.markers, expected_markers);
    assert!(result.nodes_to_remove.is_empty());

    assert!(
        result
            .new_nodes
            .iter()
            .all(|(_, flag)| *flag == crate::core::NodeFlag::Regular),
        "Alle erzeugten Nodes muessen NodeFlag::Regular tragen"
    );
}

// ─── GroupRecord-Tests ──────────────────────────────────────────────────────

/// Roundtrip: make_group_record erstellt den korrekten Record,
/// load_for_edit stellt alle Felder exakt wieder her.
#[test]
fn parking_segment_record_roundtrip() {
    let mut tool = ParkingTool::new();
    tool.origin = Some(Vec2::new(100.0, 200.0));
    tool.angle = 1.5;
    tool.phase = ParkingPhase::Configuring;
    tool.config.num_rows = 3;
    tool.config.row_spacing = 8.0;
    tool.direction = ConnectionDirection::Regular;
    tool.priority = ConnectionPriority::SubPriority;

    let record = tool.make_group_record(99, &[200, 201, 202]);
    assert!(record.is_some(), "Record muss vorhanden sein");
    let record = record.unwrap();

    let GroupKind::Parking {
        origin,
        angle,
        ref config,
        ref base,
    } = record.kind
    else {
        panic!("Erwartetes GroupKind::Parking, bekam etwas anderes");
    };
    assert_eq!(
        origin,
        Vec2::new(100.0, 200.0),
        "origin muss uebereinstimmen"
    );
    assert_eq!(angle, 1.5, "angle muss uebereinstimmen");
    assert_eq!(config.num_rows, 3, "num_rows muss uebereinstimmen");
    assert_eq!(config.row_spacing, 8.0, "row_spacing muss uebereinstimmen");
    assert_eq!(
        base.direction,
        ConnectionDirection::Regular,
        "direction im base"
    );
    assert_eq!(
        base.priority,
        ConnectionPriority::SubPriority,
        "priority im base"
    );

    // Roundtrip: neues Tool, load_for_edit
    let mut tool2 = ParkingTool::new();
    tool2.load_for_edit(&record, &record.kind);

    assert_eq!(
        tool2.origin,
        Some(Vec2::new(100.0, 200.0)),
        "origin nach load_for_edit"
    );
    assert_eq!(tool2.angle, 1.5, "angle nach load_for_edit");
    assert_eq!(
        tool2.phase,
        ParkingPhase::Configuring,
        "phase muss Configuring sein nach load_for_edit"
    );
    assert_eq!(tool2.config.num_rows, 3, "num_rows nach load_for_edit");
    assert_eq!(
        tool2.config.row_spacing, 8.0,
        "row_spacing nach load_for_edit"
    );
    assert_eq!(
        tool2.direction,
        ConnectionDirection::Regular,
        "direction nach load_for_edit"
    );
    assert_eq!(
        tool2.priority,
        ConnectionPriority::SubPriority,
        "priority nach load_for_edit"
    );
}

/// Ohne gesetzten Origin muss make_group_record None liefern.
#[test]
fn parking_segment_record_none_ohne_origin() {
    let tool = ParkingTool::new();
    let record = tool.make_group_record(0, &[]);
    assert!(
        record.is_none(),
        "Ohne origin muss make_group_record None liefern"
    );
}

// ─── Neue Interaktionsflow-Tests ──────────────────────────────────────────────

#[test]
fn parking_neuer_interaktionsflow() {
    use crate::core::RoadMap;
    let mut tool = ParkingTool::new();
    let rm = RoadMap::new(0);

    // Idle → Klick → Configuring
    let action = tool.on_click(Vec2::new(10.0, 20.0), &rm, false);
    assert_eq!(action, crate::app::tools::ToolAction::Continue);
    assert_eq!(tool.phase, ParkingPhase::Configuring);
    assert!(tool.is_ready());

    // Configuring → Viewport-Klick → Adjusting
    let action = tool.on_click(Vec2::new(30.0, 40.0), &rm, false);
    assert_eq!(action, crate::app::tools::ToolAction::Continue);
    assert_eq!(tool.phase, ParkingPhase::Adjusting);
    assert!(!tool.is_ready());

    // Adjusting → Klick → zurueck zu Configuring mit neuer Position
    let action = tool.on_click(Vec2::new(50.0, 60.0), &rm, false);
    assert_eq!(action, crate::app::tools::ToolAction::Continue);
    assert_eq!(tool.phase, ParkingPhase::Configuring);
    assert_eq!(tool.origin, Some(Vec2::new(50.0, 60.0)));
    assert!(tool.is_ready());
}

#[test]
fn parking_scroll_rotation() {
    let mut tool = ParkingTool::new();
    assert_eq!(tool.angle, 0.0);

    // Idle → rotierbar
    tool.on_scroll_rotate(15.0);
    assert!(tool.angle > 0.0);

    // Configuring → NICHT rotierbar
    let angle_before = tool.angle;
    tool.phase = ParkingPhase::Configuring;
    tool.on_scroll_rotate(15.0);
    assert_eq!(tool.angle, angle_before);

    // Adjusting → wieder rotierbar
    tool.phase = ParkingPhase::Adjusting;
    tool.on_scroll_rotate(15.0);
    assert!(tool.angle > angle_before);
}

#[test]
fn parking_execute_nur_in_configuring() {
    use crate::core::RoadMap;
    let mut tool = ParkingTool::new();
    let rm = RoadMap::new(0);

    // Idle → kein Execute
    assert!(tool.execute(&rm).is_none());

    // Configuring → Execute moeglich
    tool.on_click(Vec2::new(10.0, 20.0), &rm, false);
    assert!(tool.execute(&rm).is_some());

    // Adjusting → kein Execute
    tool.on_click(Vec2::new(30.0, 40.0), &rm, false);
    assert_eq!(tool.phase, ParkingPhase::Adjusting);
    assert!(tool.execute(&rm).is_none());
}
