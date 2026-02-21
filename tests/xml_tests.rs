/// Integration-Tests für XML-Parsing
use fs25_auto_drive_editor::xml::parse_autodrive_config;

#[test]
fn test_parse_simple_config() {
    let xml_content = include_str!("fixtures/simple_config.xml");
    let road_map = parse_autodrive_config(xml_content).unwrap();

    assert_eq!(road_map.version, 3);
    assert_eq!(road_map.node_count(), 4);
    assert_eq!(road_map.connection_count(), 5);
}
