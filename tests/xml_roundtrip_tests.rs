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

    let mut parsed_ids: Vec<u64> = parsed.nodes.keys().copied().collect();
    let mut reparsed_ids: Vec<u64> = reparsed.nodes.keys().copied().collect();
    parsed_ids.sort_unstable();
    reparsed_ids.sort_unstable();

    assert_eq!(parsed_ids, reparsed_ids);
}
