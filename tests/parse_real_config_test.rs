/// Integrationstest: Parst echte AutoDrive-Configs
use fs25_auto_drive_editor::xml::parse_autodrive_config;

#[test]
fn test_parse_autodrive_config() {
    let xml = std::fs::read_to_string("ad_sample_data/AutoDrive_config.xml").unwrap();
    match parse_autodrive_config(&xml) {
        Ok(rm) => {
            println!(
                "OK: {} nodes, {} connections, {} markers",
                rm.node_count(),
                rm.connection_count(),
                rm.marker_count()
            );
            assert!(rm.node_count() > 0);
        }
        Err(e) => panic!("Parse-Fehler: {:#}", e),
    }
}

#[test]
fn test_parse_autodrive_config1() {
    let xml = std::fs::read_to_string("ad_sample_data/AutoDrive_config1.xml").unwrap();
    match parse_autodrive_config(&xml) {
        Ok(rm) => {
            println!(
                "OK: {} nodes, {} connections, {} markers",
                rm.node_count(),
                rm.connection_count(),
                rm.marker_count()
            );
            assert!(rm.node_count() > 0);
        }
        Err(e) => panic!("Parse-Fehler: {:#}", e),
    }
}

/// Testet Duplikat-Erkennung auf einer XML mit duplizierten Nodes
#[test]
fn test_deduplicate_on_duplicated_xml() {
    // XML mit 4 Nodes: 1→2 und 3→4 wobei 3&4 Duplikate von 1&2 sind
    let xml = r#"<?xml version="1.0" encoding="UTF-8"?>
<AutoDrive version="3">
    <waypoints>
        <id>1,2,3,4</id>
        <x>100.0,200.0,100.0,200.0</x>
        <y>0,0,0,0</y>
        <z>300.0,400.0,300.0,400.0</z>
        <out>2;-1;4;-1</out>
        <incoming>-1;1;-1;3</incoming>
        <flags>0,0,2,2</flags>
    </waypoints>
    <mapmarker>
        <mm1>
            <id>3</id>
            <name>DupMarker</name>
            <group>All</group>
        </mm1>
    </mapmarker>
</AutoDrive>"#;

    let mut road_map = parse_autodrive_config(xml).expect("Parse fehlgeschlagen");

    assert_eq!(road_map.node_count(), 4);
    assert_eq!(road_map.connection_count(), 2); // 1→2 und 3→4

    let result = road_map.deduplicate_nodes(0.01);

    assert!(result.had_duplicates());
    assert_eq!(result.removed_nodes, 2); // Nodes 3 und 4 entfernt
    assert_eq!(result.duplicate_groups, 2);
    assert_eq!(road_map.node_count(), 2); // nur 1 und 2 bleiben
    assert_eq!(road_map.connection_count(), 1); // 3→4 wird zu 1→2 (gemergt)
    assert!(road_map.has_connection(1, 2));

    // Marker wurde von Node 3 auf Node 1 umgeleitet
    assert_eq!(result.remapped_markers, 1);
    assert_eq!(road_map.map_markers[0].id, 1);
}
