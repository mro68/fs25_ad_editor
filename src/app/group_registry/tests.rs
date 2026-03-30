use super::*;
use crate::app::tools::ToolAnchor;
use crate::{ConnectionDirection, ConnectionPriority, MapNode, NodeFlag, RoadMap};
use glam::Vec2;

fn make_test_record(
    id: u64,
    node_ids: Vec<u64>,
    positions: Vec<Vec2>,
    locked: bool,
) -> GroupRecord {
    GroupRecord {
        id,
        node_ids,
        start_anchor: ToolAnchor::NewPosition(Vec2::ZERO),
        end_anchor: ToolAnchor::NewPosition(Vec2::ZERO),
        kind: GroupKind::Straight {
            base: GroupBase {
                direction: ConnectionDirection::Regular,
                priority: ConnectionPriority::Regular,
                max_segment_length: 10.0,
            },
        },
        original_positions: positions,
        marker_node_ids: Vec::new(),
        locked,
        entry_node_id: None,
        exit_node_id: None,
    }
}

#[test]
fn groups_for_node_findet_alle_zugehoerigen_segmente() {
    let mut registry = GroupRegistry::new();
    registry.register(make_test_record(0, vec![1, 2, 3], vec![], true));
    registry.register(make_test_record(1, vec![3, 4, 5], vec![], false));
    registry.register(make_test_record(2, vec![6, 7], vec![], true));

    let result = registry.groups_for_node(3);
    assert_eq!(result.len(), 2, "Node 3 gehoert zu Segmenten 0 und 1");
    assert!(result.contains(&0));
    assert!(result.contains(&1));

    let result_solo = registry.groups_for_node(7);
    assert_eq!(result_solo, vec![2]);

    let result_none = registry.groups_for_node(99);
    assert!(result_none.is_empty());
}

#[test]
fn expand_locked_selection_gibt_alle_nodes_locked_segmente() {
    let mut registry = GroupRegistry::new();
    // Locked: Nodes 1, 2, 3
    registry.register(make_test_record(0, vec![1, 2, 3], vec![], true));
    // Unlocked: Nodes 4, 5
    registry.register(make_test_record(1, vec![4, 5], vec![], false));
    // Locked: Nodes 6, 7
    registry.register(make_test_record(2, vec![6, 7], vec![], true));

    // Selektion: nur Node 1 (gehoert zu Segment 0, locked)
    let mut extra = registry.expand_locked_selection(&[1]);
    extra.sort();
    assert_eq!(extra, vec![1, 2, 3]);

    // Selektion: Node 4 (gehoert zu Segment 1, UNlocked) → kein Expand
    let extra_unlocked = registry.expand_locked_selection(&[4]);
    assert!(extra_unlocked.is_empty());

    // Selektion: Node 1 + Node 6 → beide locked Segmente expandieren
    let mut extra_multi = registry.expand_locked_selection(&[1, 6]);
    extra_multi.sort();
    assert_eq!(extra_multi, vec![1, 2, 3, 6, 7]);
}

#[test]
fn update_original_positions_aktualisiert_korrekt() {
    let mut map = RoadMap::new(3);
    map.add_node(MapNode::new(10, Vec2::new(5.0, 0.0), NodeFlag::Regular));
    map.add_node(MapNode::new(11, Vec2::new(15.0, 0.0), NodeFlag::Regular));

    let mut registry = GroupRegistry::new();
    // original_positions absichtlich falsch (alt)
    registry.register(make_test_record(
        0,
        vec![10, 11],
        vec![Vec2::new(0.0, 0.0), Vec2::new(10.0, 0.0)],
        true,
    ));

    registry.update_original_positions(0, &map);

    let record = registry.get(0).expect("Record vorhanden");
    assert_eq!(record.original_positions[0], Vec2::new(5.0, 0.0));
    assert_eq!(record.original_positions[1], Vec2::new(15.0, 0.0));
}

/// Prüft, dass der Reverse-Index nach register/remove konsistent bleibt.
#[test]
fn reverse_index_konsistent_nach_register_und_remove() {
    let mut registry = GroupRegistry::new();
    registry.register(make_test_record(0, vec![1, 2, 3], vec![], false));
    registry.register(make_test_record(1, vec![3, 4], vec![], false));

    // Node 3 gehoert zu beiden Records
    assert_eq!(registry.groups_for_node(3).len(), 2);

    // Record 0 entfernen → Node 3 nur noch in Record 1
    registry.remove(0);
    let segs = registry.groups_for_node(3);
    assert_eq!(segs, vec![1], "Node 3 sollte nur noch Record 1 haben");

    // Nodes 1, 2 sollten keine Zuordnung mehr haben
    assert!(registry.groups_for_node(1).is_empty());
    assert!(registry.groups_for_node(2).is_empty());

    // Node 4 weiterhin Record 1
    assert_eq!(registry.groups_for_node(4), vec![1]);
}

/// Prüft, dass update_record den Reverse-Index korrekt aktualisiert.
#[test]
fn reverse_index_konsistent_nach_update_record() {
    let mut registry = GroupRegistry::new();
    registry.register(make_test_record(0, vec![1, 2, 3], vec![], false));

    // Alte Nodes: 1,2,3 → Neue Nodes: 2,4,5
    let success =
        registry.update_record(0, vec![2, 4, 5], vec![Vec2::ZERO, Vec2::ZERO, Vec2::ZERO]);
    assert!(success, "update_record sollte true zurueckgeben");

    // Node 1, 3 sollten nicht mehr zugeordnet sein
    assert!(registry.groups_for_node(1).is_empty());
    assert!(registry.groups_for_node(3).is_empty());

    // Node 2 sollte weiterhin, 4 und 5 neu zugeordnet sein
    assert_eq!(registry.groups_for_node(2), vec![0]);
    assert_eq!(registry.groups_for_node(4), vec![0]);
    assert_eq!(registry.groups_for_node(5), vec![0]);
}

/// Prüft, dass update_record false zurückgibt bei nicht-existierender ID.
#[test]
fn update_record_nicht_existierend_gibt_false() {
    let mut registry = GroupRegistry::new();
    let result = registry.update_record(99, vec![1], vec![Vec2::ZERO]);
    assert!(!result, "Nicht-existierende ID sollte false ergeben");
}

/// Prüft, dass invalidate_by_node_ids den edit_guard respektiert.
#[test]
fn invalidate_respektiert_edit_guard() {
    let mut registry = GroupRegistry::new();
    registry.register(make_test_record(0, vec![1, 2], vec![], false));
    registry.register(make_test_record(1, vec![2, 3], vec![], false));

    // Record 1 als edit_guard setzen
    registry.set_edit_guard(Some(1));

    // Node 2 invalidieren → sollte nur Record 0 entfernen
    registry.invalidate_by_node_ids(&[2]);

    assert!(registry.get(0).is_none(), "Record 0 sollte entfernt sein");
    assert!(
        registry.get(1).is_some(),
        "Record 1 sollte durch Guard geschuetzt sein"
    );
}

/// Prüft, dass find_by_node_ids korrekte Records findet.
#[test]
fn find_by_node_ids_findet_betroffene_records() {
    let mut registry = GroupRegistry::new();
    registry.register(make_test_record(0, vec![1, 2], vec![], false));
    registry.register(make_test_record(1, vec![3, 4], vec![], false));
    registry.register(make_test_record(2, vec![2, 5], vec![], false));

    let query: indexmap::IndexSet<u64> = [2, 3].into_iter().collect();
    let found = registry.find_by_node_ids(&query);
    let mut found_ids: Vec<u64> = found.iter().map(|r| r.id).collect();
    found_ids.sort();
    assert_eq!(found_ids, vec![0, 1, 2], "Alle Records mit Node 2 oder 3");
}

/// Prüft, dass find_first_by_node_id den ersten Record findet.
#[test]
fn find_first_by_node_id_findet_record() {
    let mut registry = GroupRegistry::new();
    registry.register(make_test_record(0, vec![1, 2], vec![], false));
    registry.register(make_test_record(1, vec![3, 4], vec![], false));

    assert!(registry.find_first_by_node_id(1).is_some());
    assert!(registry.find_first_by_node_id(99).is_none());
}

/// Prüft, dass remove bei nicht-existierender ID nicht panikt.
#[test]
fn remove_nicht_existierend_ist_noop() {
    let mut registry = GroupRegistry::new();
    registry.register(make_test_record(0, vec![1, 2], vec![], false));

    // Doppeltes Remove sollte kein Panic verursachen
    registry.remove(0);
    registry.remove(0);
    registry.remove(99);

    assert!(registry.is_empty());
}

/// Prüft Operationen auf leerer Registry.
#[test]
fn leere_registry_edge_cases() {
    let registry = GroupRegistry::new();

    assert!(registry.groups_for_node(1).is_empty());
    assert!(registry.expand_locked_selection(&[1, 2]).is_empty());
    assert!(registry.find_first_by_node_id(1).is_none());

    let query: indexmap::IndexSet<u64> = [1, 2].into_iter().collect();
    assert!(registry.find_by_node_ids(&query).is_empty());

    assert!(registry.is_empty());
    assert_eq!(registry.len(), 0);
}

// --- Tests fuer remove_nodes_from_record ---

/// Prüft, dass Nodes korrekt aus einem Record entfernt werden.
#[test]
fn remove_nodes_from_record_entfernt_subset() {
    let mut registry = GroupRegistry::new();
    let positions = vec![Vec2::ZERO, Vec2::X, Vec2::Y, Vec2::ONE];
    registry.register(make_test_record(0, vec![1, 2, 3, 4], positions, false));

    let still_alive = registry.remove_nodes_from_record(0, &[2, 3]);
    assert!(
        still_alive,
        "Record sollte bestehen bleiben (2 Nodes uebrig)"
    );

    let record = registry.get(0).expect("Record vorhanden");
    assert_eq!(record.node_ids, vec![1, 4]);
    assert_eq!(record.original_positions.len(), 2);

    // Reverse-Index: entfernte Nodes sollten weg sein
    assert!(registry.groups_for_node(2).is_empty());
    assert!(registry.groups_for_node(3).is_empty());
    // Verbleibende Nodes weiterhin zugeordnet
    assert_eq!(registry.groups_for_node(1), vec![0]);
    assert_eq!(registry.groups_for_node(4), vec![0]);
}

/// Prüft, dass der Record aufgeloest wird wenn weniger als 2 Nodes verbleiben.
#[test]
fn remove_nodes_from_record_dissolve_bei_weniger_als_2() {
    let mut registry = GroupRegistry::new();
    let positions = vec![Vec2::ZERO, Vec2::X, Vec2::Y];
    registry.register(make_test_record(0, vec![1, 2, 3], positions, false));

    let still_alive = registry.remove_nodes_from_record(0, &[1, 2]);
    assert!(
        !still_alive,
        "Record sollte aufgeloest worden sein (<2 Nodes)"
    );
    assert!(
        registry.get(0).is_none(),
        "Record darf nicht mehr existieren"
    );

    // Alle Nodes aus dem Reverse-Index entfernt
    assert!(registry.groups_for_node(1).is_empty());
    assert!(registry.groups_for_node(2).is_empty());
    assert!(registry.groups_for_node(3).is_empty());
}

/// Prüft, dass nicht-existierende Nodes im remove-Set keine Probleme verursachen.
#[test]
fn remove_nodes_from_record_ignoriert_unbekannte_nodes() {
    let mut registry = GroupRegistry::new();
    let positions = vec![Vec2::ZERO, Vec2::X, Vec2::Y];
    registry.register(make_test_record(0, vec![1, 2, 3], positions, false));

    let still_alive = registry.remove_nodes_from_record(0, &[99, 100]);
    assert!(still_alive, "Record bleibt (keine echten Entfernungen)");

    let record = registry.get(0).expect("Record vorhanden");
    assert_eq!(record.node_ids.len(), 3, "Keine Nodes entfernt");
}

/// Prüft, dass remove_nodes_from_record false bei unbekannter Record-ID liefert.
#[test]
fn remove_nodes_from_record_unbekannte_id() {
    let mut registry = GroupRegistry::new();
    let result = registry.remove_nodes_from_record(42, &[1, 2]);
    assert!(!result, "Nicht-existierende ID sollte false ergeben");
}

/// Prüft, dass der Boundary-Cache nach Node-Entfernung invalidiert wird.
#[test]
fn remove_nodes_from_record_invalidiert_boundary_cache() {
    let mut map = RoadMap::new(4);
    map.add_node(MapNode::new(1, Vec2::new(0.0, 0.0), NodeFlag::Regular));
    map.add_node(MapNode::new(2, Vec2::new(10.0, 0.0), NodeFlag::Regular));
    map.add_node(MapNode::new(3, Vec2::new(20.0, 0.0), NodeFlag::Regular));
    map.add_node(MapNode::new(4, Vec2::new(30.0, 0.0), NodeFlag::Regular));

    let mut registry = GroupRegistry::new();
    let positions = vec![
        Vec2::new(0.0, 0.0),
        Vec2::new(10.0, 0.0),
        Vec2::new(20.0, 0.0),
        Vec2::new(30.0, 0.0),
    ];
    registry.register(make_test_record(0, vec![1, 2, 3, 4], positions, false));

    // Cache aufwaermen
    registry.warm_boundary_cache(&map);
    assert!(
        registry.boundary_cache_for(0).is_some(),
        "Cache sollte nach warm vorhanden sein"
    );

    // Node entfernen → Cache muss invalidiert sein
    registry.remove_nodes_from_record(0, &[3]);
    assert!(
        registry.boundary_cache_for(0).is_none(),
        "Cache muss nach Node-Entfernung invalidiert sein"
    );
}
