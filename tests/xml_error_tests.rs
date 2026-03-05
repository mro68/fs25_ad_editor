use fs25_auto_drive_editor::{
    parse_autodrive_config, write_autodrive_config, AutoDriveMeta, Connection, ConnectionDirection,
    ConnectionPriority, MapNode, NodeFlag, RoadMap,
};
use glam::Vec2;

/// Prüft, dass der Parser nicht-numerische ID-Listen ablehnt.
#[test]
fn test_parser_errors_on_non_numeric_ids() {
    let xml = r#"
    <AutoDrive version="3">
        <waypoints>
            <id>1,a</id>
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

    let err = parse_autodrive_config(xml).expect_err("Parser sollte bei ungültigen IDs scheitern");
    let msg = format!("{err:#}");
    assert!(msg.contains("Fehler beim Parsen der ID-Liste"));
}

/// Prüft, dass fehlende Pflicht-Listen im XML erkannt werden.
#[test]
fn test_parser_errors_on_missing_waypoint_lists() {
    let xml = r#"
    <AutoDrive version="3">
        <waypoints>
            <id>1</id>
            <x>0</x>
            <y>0</y>
            <z>0</z>
            <out></out>
            <flags>0</flags>
        </waypoints>
        <mapmarker></mapmarker>
    </AutoDrive>
    "#;

    let err = parse_autodrive_config(xml).expect_err("Parser sollte fehlende Listen melden");
    let msg = format!("{err:#}");
    assert!(msg.contains("Pflichtfelder in <waypoints> fehlen"));
}

/// Prüft, dass unterschiedlich lange Waypoint-Listen als Fehler behandelt werden.
#[test]
fn test_parser_errors_on_mismatched_waypoint_lengths() {
    let xml = r#"
    <AutoDrive version="3">
        <waypoints>
            <id>1,2,3</id>
            <x>0,10</x>
            <y>0,0,0</y>
            <z>0,0,0</z>
            <out>2;1;3</out>
            <incoming>2;1;3</incoming>
            <flags>0,0,0</flags>
        </waypoints>
        <mapmarker></mapmarker>
    </AutoDrive>
    "#;

    let err = parse_autodrive_config(xml).expect_err("Parser sollte Längeninkonsistenzen melden");
    let msg = format!("{err:#}");
    assert!(msg.contains("Laengen der Waypoint-Listen stimmen nicht ueberein"));
}

/// Prüft, dass Meta-Optionen und Version nach Export und Import erhalten bleiben.
#[test]
fn test_meta_options_roundtrip_preserves_meta() {
    let mut map = RoadMap::new(3);
    map.add_node(MapNode::new(1, Vec2::new(0.0, 0.0), NodeFlag::Regular));
    map.add_node(MapNode::new(2, Vec2::new(3.0, 4.0), NodeFlag::Regular));

    let connection = Connection::new(
        1,
        2,
        ConnectionDirection::Regular,
        ConnectionPriority::Regular,
        Vec2::new(0.0, 0.0),
        Vec2::new(3.0, 4.0),
    );
    map.add_connection(connection);

    map.meta = AutoDriveMeta {
        config_version: Some("3.0.0.5".to_string()),
        route_version: Some("1.5".to_string()),
        route_author: Some("MetaTester".to_string()),
        options: vec![
            ("OptionAlpha".to_string(), "First".to_string()),
            ("OptionBeta".to_string(), "Second".to_string()),
        ],
    };
    map.map_name = Some("MetaMap".to_string());

    let expected_options = map.meta.options.clone();
    let xml = write_autodrive_config(&map, None, 255.0).expect("Export fehlgeschlagen");
    let reparsed = parse_autodrive_config(&xml).expect("Re-Parsing fehlgeschlagen");

    assert_eq!(reparsed.version, 3);
    assert_eq!(reparsed.map_name.as_deref(), Some("MetaMap"));
    assert_eq!(reparsed.meta.config_version.as_deref(), Some("3.0.0.5"));
    assert_eq!(reparsed.meta.route_version.as_deref(), Some("1.5"));
    assert_eq!(reparsed.meta.route_author.as_deref(), Some("MetaTester"));
    assert_eq!(reparsed.meta.options, expected_options);
}
