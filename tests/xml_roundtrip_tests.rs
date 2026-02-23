use fs25_auto_drive_editor::{parse_autodrive_config, write_autodrive_config};

#[test]
fn test_xml_roundtrip_preserves_core_counts_and_ids() {
    let xml_content = include_str!("fixtures/simple_config.xml");

    let parsed = parse_autodrive_config(xml_content).expect("Initiales Parsing fehlgeschlagen");
    let written_xml =
        write_autodrive_config(&parsed, None, 255.0).expect("XML-Export fehlgeschlagen");
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

    let written = write_autodrive_config(&parsed, None, 255.0).expect("Export fehlgeschlagen");
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

#[test]
fn test_xml_roundtrip_dual_connections() {
    // Dual-Verbindung: 1↔2 (bidirektional, beide in out+incoming)
    let xml = r#"<?xml version="1.0" encoding="UTF-8"?>
<AutoDrive version="3">
    <waypoints>
        <id>1,2</id>
        <x>0.0,100.0</x>
        <y>0.0,0.0</y>
        <z>0.0,0.0</z>
        <out>2;1</out>
        <incoming>2;1</incoming>
        <flags>0,0</flags>
    </waypoints>
    <mapmarker>
    </mapmarker>
</AutoDrive>"#;

    let parsed = parse_autodrive_config(xml).expect("Parsing fehlgeschlagen");
    // AutoDrive XML: beide Richtungen in out/incoming → Parser erzeugt Dual-Connection
    assert!(parsed.has_connection(1, 2), "Verbindung 1→2 fehlt");

    let written = write_autodrive_config(&parsed, None, 255.0).expect("Export fehlgeschlagen");
    let reparsed = parse_autodrive_config(&written).expect("Re-Parsing fehlgeschlagen");

    // Nach Roundtrip: Counts müssen stimmen
    assert_eq!(reparsed.node_count(), 2);
    assert_eq!(reparsed.connection_count(), parsed.connection_count());
    assert!(
        reparsed.has_connection(1, 2),
        "Verbindung 1→2 fehlt nach Roundtrip"
    );
}

#[test]
fn test_xml_roundtrip_marker_remapping() {
    // Marker auf Node 5, das zu ID 2 remappt wird
    let xml = r#"<?xml version="1.0" encoding="UTF-8"?>
<AutoDrive version="3">
    <waypoints>
        <id>3,5</id>
        <x>10.0,20.0</x>
        <y>0.0,0.0</y>
        <z>30.0,40.0</z>
        <out>5;</out>
        <incoming>;3</incoming>
        <flags>0,0</flags>
    </waypoints>
    <mapmarker>
        <mm1>
            <id>5.000000</id>
            <name>Hof</name>
            <group>default</group>
        </mm1>
    </mapmarker>
</AutoDrive>"#;

    let parsed = parse_autodrive_config(xml).expect("Parsing fehlgeschlagen");
    assert_eq!(parsed.marker_count(), 1);

    let written = write_autodrive_config(&parsed, None, 255.0).expect("Export fehlgeschlagen");
    let reparsed = parse_autodrive_config(&written).expect("Re-Parsing fehlgeschlagen");

    // Marker muss auf die neue ID 2 zeigen (5 → 2)
    assert_eq!(reparsed.marker_count(), 1);
    let marker = &reparsed.map_markers[0];
    assert_eq!(marker.id, 2, "Marker-ID sollte nach Remapping 2 sein");
    assert_eq!(marker.name, "Hof");
    assert_eq!(marker.group, "default");
}

#[test]
fn test_xml_roundtrip_meta_data() {
    let xml = r#"<?xml version="1.0" encoding="UTF-8"?>
<AutoDrive>
    <version>42</version>
    <MapName>TestMap</MapName>
    <ADRouteVersion>1.5</ADRouteVersion>
    <ADRouteAuthor>Tester</ADRouteAuthor>
    <waypoints>
        <id>1,2</id>
        <x>0.0,10.0</x>
        <y>0.0,0.0</y>
        <z>0.0,10.0</z>
        <out>2;</out>
        <incoming>;1</incoming>
        <flags>0,0</flags>
    </waypoints>
    <mapmarker>
    </mapmarker>
</AutoDrive>"#;

    let parsed = parse_autodrive_config(xml).expect("Parsing fehlgeschlagen");
    let written = write_autodrive_config(&parsed, None, 255.0).expect("Export fehlgeschlagen");
    let reparsed = parse_autodrive_config(&written).expect("Re-Parsing fehlgeschlagen");

    assert_eq!(
        reparsed.meta.config_version.as_deref(),
        Some("42"),
        "Config-Version fehlt nach Roundtrip"
    );
    assert_eq!(
        reparsed.map_name.as_deref(),
        Some("TestMap"),
        "MapName fehlt nach Roundtrip"
    );
    assert_eq!(
        reparsed.meta.route_version.as_deref(),
        Some("1.5"),
        "RouteVersion fehlt nach Roundtrip"
    );
    assert_eq!(
        reparsed.meta.route_author.as_deref(),
        Some("Tester"),
        "RouteAuthor fehlt nach Roundtrip"
    );
}

#[test]
fn test_xml_roundtrip_special_characters_in_marker() {
    let xml = r#"<?xml version="1.0" encoding="UTF-8"?>
<AutoDrive version="3">
    <waypoints>
        <id>1,2</id>
        <x>0.0,10.0</x>
        <y>0.0,0.0</y>
        <z>0.0,10.0</z>
        <out>2;</out>
        <incoming>;1</incoming>
        <flags>0,0</flags>
    </waypoints>
    <mapmarker>
        <mm1>
            <id>1.000000</id>
            <name>Bauer &amp; Sohn</name>
            <group>Höfe</group>
        </mm1>
    </mapmarker>
</AutoDrive>"#;

    let parsed = parse_autodrive_config(xml).expect("Parsing fehlgeschlagen");
    assert_eq!(parsed.map_markers[0].name, "Bauer & Sohn");

    let written = write_autodrive_config(&parsed, None, 255.0).expect("Export fehlgeschlagen");

    // Die geschriebene XML muss & korrekt escapen
    assert!(
        written.contains("Bauer &amp; Sohn"),
        "Ampersand muss escaped werden: {}",
        written
    );

    let reparsed = parse_autodrive_config(&written).expect("Re-Parsing fehlgeschlagen");
    assert_eq!(
        reparsed.map_markers[0].name, "Bauer & Sohn",
        "Sonderzeichen müssen nach Roundtrip erhalten bleiben"
    );
}
