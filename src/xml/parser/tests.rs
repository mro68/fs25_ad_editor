use super::*;
use crate::ConnectionDirection;
use crate::xml::parser::waypoints::{parse_list, parse_nested_list};

#[test]
fn test_parse_simple_list() {
    let result = parse_list::<u64>("1,2,3,4", ',').unwrap();
    assert_eq!(result, vec![1, 2, 3, 4]);
}

#[test]
fn test_parse_nested_list() {
    let result = parse_nested_list("2,3;4,5;;1").unwrap();
    assert_eq!(result, vec![vec![2, 3], vec![4, 5], vec![], vec![1],]);
}

#[test]
fn test_parse_fails_for_invalid_marker_id() {
    let xml = r#"
    <AutoDrive version="3">
        <waypoints>
            <id>1</id>
            <x>0</x>
            <y>0</y>
            <z>0</z>
            <out></out>
            <incoming></incoming>
            <flags>0</flags>
        </waypoints>
        <mapmarker>
            <mm1>
                <id>abc</id>
                <name>Test</name>
                <group>All</group>
            </mm1>
        </mapmarker>
    </AutoDrive>
    "#;

    let err = parse_autodrive_config(xml).expect_err("Parser sollte fehlschlagen");
    let msg = format!("{err:#}");
    assert!(msg.contains("Ungueltige Marker-ID"));
}

#[test]
fn test_bidirectional_creates_single_connection() {
    let xml = r#"
    <AutoDrive version="3">
        <waypoints>
            <id>1,2</id>
            <x>0,10</x>
            <y>0,0</y>
            <z>0,0</z>
            <out>2;1</out>
            <incoming>2;1</incoming>
            <flags>0,0</flags>
        </waypoints>
        <mapmarker></mapmarker>
    </AutoDrive>
    "#;

    let road_map = parse_autodrive_config(xml).expect("Parsing fehlgeschlagen");
    assert_eq!(
        road_map.connection_count(),
        1,
        "Bidirektional soll nur 1 Connection erzeugen"
    );

    let conn = road_map
        .connections_iter()
        .next()
        .expect("Connection erwartet");
    assert_eq!(conn.direction, ConnectionDirection::Dual);
}

#[test]
fn test_bidirectional_roundtrip_preserves_connections() {
    use crate::xml::writer::write_autodrive_config;

    let xml = r#"
    <AutoDrive version="3">
        <waypoints>
            <id>1,2,3</id>
            <x>0,10,20</x>
            <y>0,0,0</y>
            <z>0,0,0</z>
            <out>2;1,3;-1</out>
            <incoming>2;1;2</incoming>
            <flags>0,0,0</flags>
        </waypoints>
        <mapmarker></mapmarker>
    </AutoDrive>
    "#;

    let road_map = parse_autodrive_config(xml).expect("Parsing fehlgeschlagen");
    assert_eq!(
        road_map.connection_count(),
        2,
        "1 Dual + 1 Regular = 2 Connections"
    );

    let written = write_autodrive_config(&road_map, None, 255.0).expect("Export fehlgeschlagen");
    let reparsed = parse_autodrive_config(&written).expect("Re-Parsing fehlgeschlagen");
    assert_eq!(reparsed.connection_count(), road_map.connection_count());
}
