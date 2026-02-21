//! Test-Beispiel: Duplikat-Erkennung auf einer realen AutoDrive-Config.
//! Aufruf: cargo run --example dedup_test -- <pfad_zur_xml>

use fs25_auto_drive_editor::xml::parse_autodrive_config;

fn main() {
    let path = std::env::args()
        .nth(1)
        .unwrap_or_else(|| "ad_sample_data/AutoDrive_config.xml".to_string());

    let xml = std::fs::read_to_string(&path).expect("Datei konnte nicht gelesen werden");
    let mut rm = parse_autodrive_config(&xml).expect("Parse-Fehler");

    let before_nodes = rm.node_count();
    let before_conns = rm.connection_count();
    let before_markers = rm.marker_count();

    let result = rm.deduplicate_nodes(0.01);

    println!("=== Duplikat-Bereinigung ===");
    println!(
        "Vorher:  {} Nodes, {} Connections, {} Marker",
        before_nodes, before_conns, before_markers
    );
    println!(
        "Nachher: {} Nodes, {} Connections, {} Marker",
        rm.node_count(),
        rm.connection_count(),
        rm.marker_count()
    );
    println!();
    println!("Entfernte Duplikat-Nodes:    {}", result.removed_nodes);
    println!("Positions-Gruppen:           {}", result.duplicate_groups);
    println!(
        "Umgeleitete Verbindungen:    {}",
        result.remapped_connections
    );
    println!(
        "Entfernte Selbstreferenzen:  {}",
        result.removed_self_connections
    );
    println!("Umgeleitete Marker:          {}", result.remapped_markers);
}
