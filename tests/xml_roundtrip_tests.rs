use fs25_auto_drive_editor::{parse_autodrive_config, write_autodrive_config};

#[test]
fn test_xml_roundtrip_preserves_core_counts_and_ids() {
    let xml_content = include_str!("fixtures/simple_config.xml");

    let parsed = parse_autodrive_config(xml_content).expect("Initiales Parsing fehlgeschlagen");
    let written_xml = write_autodrive_config(&parsed, None).expect("XML-Export fehlgeschlagen");
    let reparsed = parse_autodrive_config(&written_xml).expect("Re-Parsing fehlgeschlagen");

    assert_eq!(parsed.version, reparsed.version);
    assert_eq!(parsed.node_count(), reparsed.node_count());
    assert_eq!(parsed.connection_count(), reparsed.connection_count());
    assert_eq!(parsed.marker_count(), reparsed.marker_count());

    // IDs sind nach Export lückenlos 1..N
    let mut reparsed_ids: Vec<u64> = reparsed.nodes.keys().copied().collect();
    reparsed_ids.sort_unstable();
    let expected_ids: Vec<u64> = (1..=parsed.node_count() as u64).collect();
    assert_eq!(reparsed_ids, expected_ids);
}

#[test]
fn test_xml_roundtrip_renumbers_gapped_ids() {
    // IDs mit Lücken: 2, 5, 10
    let xml_gapped = r#"<?xml version="1.0" encoding="UTF-8"?>
<AutoDrive version="3">
    <waypoints>
        <id>2,5,10</id>
        <x>100.0,200.0,300.0</x>
        <y>0.0,0.0,0.0</y>
        <z>100.0,200.0,300.0</z>
        <out>5;10;</out>
        <incoming>;2;5</incoming>
        <flags>0,0,0</flags>
    </waypoints>
    <mapmarker>
    </mapmarker>
</AutoDrive>"#;

    let parsed = parse_autodrive_config(xml_gapped).expect("Parsing fehlgeschlagen");
    assert_eq!(parsed.node_count(), 3);

    let written = write_autodrive_config(&parsed, None).expect("Export fehlgeschlagen");
    let reparsed = parse_autodrive_config(&written).expect("Re-Parsing fehlgeschlagen");

    // Nach Export: IDs sind 1, 2, 3
    let mut ids: Vec<u64> = reparsed.nodes.keys().copied().collect();
    ids.sort_unstable();
    assert_eq!(ids, vec![1, 2, 3]);

    // Verbindungen müssen korrekt umgemappt sein: 2→5 wird 1→2, 5→10 wird 2→3
    assert!(reparsed.has_connection(1, 2), "Verbindung 1→2 fehlt");
    assert!(reparsed.has_connection(2, 3), "Verbindung 2→3 fehlt");
    assert_eq!(reparsed.connection_count(), parsed.connection_count());
}
