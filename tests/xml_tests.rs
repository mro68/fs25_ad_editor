/// Integration-Tests fuer XML-Parsing
use fs25_auto_drive_editor::core::NodeFlag;
use fs25_auto_drive_editor::xml::parse_autodrive_config;

#[test]
fn test_parse_simple_config() {
    let xml_content = include_str!("fixtures/simple_config.xml");
    let road_map = parse_autodrive_config(xml_content).unwrap();

    assert_eq!(road_map.version, 3);
    assert_eq!(road_map.node_count(), 4);
    assert_eq!(road_map.connection_count(), 5);
}

#[test]
fn test_parse_normalizes_legacy_flags_two_and_four_to_regular() {
    let xml = r#"
        <?xml version="1.0" encoding="UTF-8"?>
        <AutoDrive version="3">
            <waypoints>
                <id>1,2</id>
                <x>0,10</x>
                <y>0,0</y>
                <z>0,0</z>
                <out>2;</out>
                <incoming>;1</incoming>
                <flags>2,4</flags>
            </waypoints>
            <mapmarker>
                <mmID></mmID>
                <name></name>
                <group></group>
                <markerID></markerID>
            </mapmarker>
        </AutoDrive>
    "#;

    let road_map = parse_autodrive_config(xml).expect("Legacy-Flags muessen parsebar sein");
    let node1 = road_map.node(1).expect("Node 1 muss vorhanden sein");
    let node2 = road_map.node(2).expect("Node 2 muss vorhanden sein");

    assert_eq!(node1.flag, NodeFlag::Regular);
    assert_eq!(node2.flag, NodeFlag::Regular);
}
