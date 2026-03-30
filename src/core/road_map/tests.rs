use indexmap::IndexSet;

use super::*;
use crate::core::{ConnectionDirection, ConnectionPriority, NodeFlag};
use glam::Vec2;

#[test]
fn test_roadmap_creation() {
    let mut map = RoadMap::new(3);

    let node = MapNode::new(1, Vec2::new(100.0, 300.0), NodeFlag::Regular);
    map.add_node(node);

    assert_eq!(map.node_count(), 1);
    assert_eq!(map.connection_count(), 0);
    assert_eq!(map.marker_count(), 0);
}

#[test]
fn test_rebuild_connection_geometry() {
    let mut map = RoadMap::new(3);

    let node_a = MapNode::new(1, Vec2::new(0.0, 0.0), NodeFlag::Regular);
    let node_b = MapNode::new(2, Vec2::new(10.0, 0.0), NodeFlag::Regular);
    map.add_node(node_a);
    map.add_node(node_b);

    let connection = Connection::new(
        1,
        2,
        ConnectionDirection::Regular,
        ConnectionPriority::Regular,
        Vec2::new(0.0, 0.0),
        Vec2::new(10.0, 0.0),
    );
    map.add_connection(connection);

    map.rebuild_connection_geometry();

    let connection = map.connections_iter().next().expect("Verbindung erwartet");
    assert_eq!(connection.midpoint, Vec2::new(5.0, 0.0));
    assert_eq!(connection.angle, 0.0);
}

/// Verifiziert, dass die Geometrie auch nach Positionsänderungen neu berechnet wird.
#[test]
fn test_rebuild_connection_geometry_updates_after_node_move() {
    let mut map = RoadMap::new(3);

    map.add_node(MapNode::new(1, Vec2::new(0.0, 0.0), NodeFlag::Regular));
    map.add_node(MapNode::new(2, Vec2::new(1.0, 1.0), NodeFlag::Regular));

    let connection = Connection::new(
        1,
        2,
        ConnectionDirection::Regular,
        ConnectionPriority::Regular,
        Vec2::new(0.0, 0.0),
        Vec2::new(1.0, 1.0),
    );
    map.add_connection(connection);

    map.nodes.get_mut(&2).unwrap().position = Vec2::new(3.0, 4.0);
    map.rebuild_connection_geometry();

    let connection = map.connections_iter().next().expect("Verbindung erwartet");
    assert_eq!(connection.midpoint, Vec2::new(1.5, 2.0));
    let expected_angle = 4.0f32.atan2(3.0f32);
    assert!((connection.angle - expected_angle).abs() < 1e-6);
}

#[test]
fn test_spatial_queries() {
    let mut map = RoadMap::new(3);
    map.add_node(MapNode::new(1, Vec2::new(0.0, 0.0), NodeFlag::Regular));
    map.add_node(MapNode::new(2, Vec2::new(10.0, 0.0), NodeFlag::Regular));
    map.add_node(MapNode::new(3, Vec2::new(5.0, 5.0), NodeFlag::Regular));
    map.ensure_spatial_index();

    let nearest = map
        .nearest_node(Vec2::new(5.2, 5.1))
        .expect("Treffer erwartet");
    assert_eq!(nearest.node_id, 3);

    let mut in_rect = map.nodes_within_rect(Vec2::new(-1.0, -1.0), Vec2::new(6.0, 6.0));
    in_rect.sort_unstable();
    assert_eq!(in_rect, vec![1, 3]);
}

#[test]
fn test_spatial_index_consistency_on_remove_and_update() {
    let mut map = RoadMap::new(3);
    map.add_node(MapNode::new(1, Vec2::new(0.0, 0.0), NodeFlag::Regular));
    map.add_node(MapNode::new(2, Vec2::new(10.0, 0.0), NodeFlag::Regular));
    map.ensure_spatial_index();

    assert_eq!(
        map.nearest_node(Vec2::new(9.8, 0.1)).map(|m| m.node_id),
        Some(2)
    );

    assert!(map.update_node_position(2, Vec2::new(2.0, 0.0)));
    map.ensure_spatial_index();
    assert_eq!(
        map.nearest_node(Vec2::new(2.1, 0.0)).map(|m| m.node_id),
        Some(2)
    );

    let removed = map.remove_node(2);
    assert!(removed.is_some());
    map.ensure_spatial_index();
    assert_eq!(
        map.nearest_node(Vec2::new(2.1, 0.0)).map(|m| m.node_id),
        Some(1)
    );

    let mut ids = map.nodes_within_rect(Vec2::new(-1.0, -1.0), Vec2::new(3.0, 1.0));
    ids.sort_unstable();
    assert_eq!(ids, vec![1]);
}

#[test]
fn test_recalculate_node_flags_subprio_only() {
    let mut map = RoadMap::new(3);
    map.add_node(MapNode::new(1, Vec2::ZERO, NodeFlag::Regular));
    map.add_node(MapNode::new(2, Vec2::new(10.0, 0.0), NodeFlag::Regular));

    let conn = Connection::new(
        1,
        2,
        ConnectionDirection::Regular,
        ConnectionPriority::SubPriority,
        Vec2::ZERO,
        Vec2::new(10.0, 0.0),
    );
    map.add_connection(conn);
    map.recalculate_node_flags(&[1, 2]);

    assert_eq!(map.nodes[&1].flag, NodeFlag::SubPrio);
    assert_eq!(map.nodes[&2].flag, NodeFlag::SubPrio);
}

#[test]
fn test_recalculate_node_flags_mixed_priority() {
    let mut map = RoadMap::new(3);
    map.add_node(MapNode::new(1, Vec2::ZERO, NodeFlag::Regular));
    map.add_node(MapNode::new(2, Vec2::new(10.0, 0.0), NodeFlag::Regular));
    map.add_node(MapNode::new(3, Vec2::new(20.0, 0.0), NodeFlag::Regular));

    let c1 = Connection::new(
        1,
        2,
        ConnectionDirection::Regular,
        ConnectionPriority::Regular,
        Vec2::ZERO,
        Vec2::new(10.0, 0.0),
    );
    let c2 = Connection::new(
        2,
        3,
        ConnectionDirection::Regular,
        ConnectionPriority::SubPriority,
        Vec2::new(10.0, 0.0),
        Vec2::new(20.0, 0.0),
    );
    map.add_connection(c1);
    map.add_connection(c2);
    map.recalculate_node_flags(&[1, 2, 3]);

    assert_eq!(map.nodes[&1].flag, NodeFlag::Regular);
    assert_eq!(map.nodes[&2].flag, NodeFlag::Regular);
    assert_eq!(map.nodes[&3].flag, NodeFlag::SubPrio);
}

#[test]
fn test_recalculate_node_flags_preserves_warning() {
    let mut map = RoadMap::new(3);
    map.add_node(MapNode::new(1, Vec2::ZERO, NodeFlag::Warning));
    map.add_node(MapNode::new(2, Vec2::new(10.0, 0.0), NodeFlag::Regular));

    let conn = Connection::new(
        1,
        2,
        ConnectionDirection::Regular,
        ConnectionPriority::SubPriority,
        Vec2::ZERO,
        Vec2::new(10.0, 0.0),
    );
    map.add_connection(conn);
    map.recalculate_node_flags(&[1, 2]);

    assert_eq!(map.nodes[&1].flag, NodeFlag::Warning);
    assert_eq!(map.nodes[&2].flag, NodeFlag::SubPrio);
}

/// Stellt sicher, dass Reserved-Nodes beim Flag-Update unberührt bleiben.
#[test]
fn test_recalculate_node_flags_preserves_reserved() {
    let mut map = RoadMap::new(3);
    map.add_node(MapNode::new(1, Vec2::ZERO, NodeFlag::Reserved));
    map.add_node(MapNode::new(2, Vec2::new(10.0, 0.0), NodeFlag::Regular));

    let conn = Connection::new(
        1,
        2,
        ConnectionDirection::Regular,
        ConnectionPriority::SubPriority,
        Vec2::ZERO,
        Vec2::new(10.0, 0.0),
    );
    map.add_connection(conn);
    map.recalculate_node_flags(&[1, 2]);

    assert_eq!(map.nodes[&1].flag, NodeFlag::Reserved);
    assert_eq!(map.nodes[&2].flag, NodeFlag::SubPrio);
}

#[test]
fn test_recalculate_node_flags_no_connections() {
    let mut map = RoadMap::new(3);
    map.add_node(MapNode::new(1, Vec2::ZERO, NodeFlag::SubPrio));

    map.recalculate_node_flags(&[1]);
    assert_eq!(map.nodes[&1].flag, NodeFlag::Regular);
}

#[test]
fn test_deduplicate_no_duplicates() {
    let mut map = RoadMap::new(3);
    map.add_node(MapNode::new(1, Vec2::new(0.0, 0.0), NodeFlag::Regular));
    map.add_node(MapNode::new(2, Vec2::new(10.0, 0.0), NodeFlag::Regular));

    let result = map.deduplicate_nodes(0.01);
    assert!(!result.had_duplicates());
    assert_eq!(map.node_count(), 2);
}

#[test]
fn test_deduplicate_removes_exact_duplicates() {
    let mut map = RoadMap::new(3);
    map.add_node(MapNode::new(1, Vec2::new(100.0, 200.0), NodeFlag::Regular));
    map.add_node(MapNode::new(2, Vec2::new(100.0, 200.0), NodeFlag::Regular));
    map.add_node(MapNode::new(3, Vec2::new(50.0, 50.0), NodeFlag::Regular));

    let c1 = Connection::new(
        1,
        3,
        ConnectionDirection::Regular,
        ConnectionPriority::Regular,
        Vec2::new(100.0, 200.0),
        Vec2::new(50.0, 50.0),
    );
    let c2 = Connection::new(
        2,
        3,
        ConnectionDirection::Regular,
        ConnectionPriority::Regular,
        Vec2::new(100.0, 200.0),
        Vec2::new(50.0, 50.0),
    );
    map.add_connection(c1);
    map.add_connection(c2);

    let result = map.deduplicate_nodes(0.01);
    assert!(result.had_duplicates());
    assert_eq!(result.removed_nodes, 1);
    assert_eq!(result.duplicate_groups, 1);
    assert_eq!(map.node_count(), 2);
    assert!(map.nodes.contains_key(&1));
    assert!(!map.nodes.contains_key(&2));
    assert!(map.has_connection(1, 3));
}

#[test]
fn test_deduplicate_keeps_lowest_id() {
    let mut map = RoadMap::new(3);
    map.add_node(MapNode::new(5, Vec2::new(10.0, 10.0), NodeFlag::Regular));
    map.add_node(MapNode::new(2, Vec2::new(10.0, 10.0), NodeFlag::Regular));
    map.add_node(MapNode::new(8, Vec2::new(10.0, 10.0), NodeFlag::Regular));

    let result = map.deduplicate_nodes(0.01);
    assert_eq!(result.removed_nodes, 2);
    assert_eq!(map.node_count(), 1);
    assert!(map.nodes.contains_key(&2));
}

#[test]
fn test_deduplicate_remaps_connections() {
    let mut map = RoadMap::new(3);
    map.add_node(MapNode::new(1, Vec2::new(0.0, 0.0), NodeFlag::Regular));
    map.add_node(MapNode::new(10, Vec2::new(0.0, 0.0), NodeFlag::Regular));
    map.add_node(MapNode::new(2, Vec2::new(10.0, 0.0), NodeFlag::Regular));
    map.add_node(MapNode::new(20, Vec2::new(10.0, 0.0), NodeFlag::Regular));

    let c1 = Connection::new(
        1,
        2,
        ConnectionDirection::Dual,
        ConnectionPriority::Regular,
        Vec2::new(0.0, 0.0),
        Vec2::new(10.0, 0.0),
    );
    let c2 = Connection::new(
        10,
        20,
        ConnectionDirection::Regular,
        ConnectionPriority::Regular,
        Vec2::new(0.0, 0.0),
        Vec2::new(10.0, 0.0),
    );
    map.add_connection(c1);
    map.add_connection(c2);

    let result = map.deduplicate_nodes(0.01);
    assert_eq!(result.removed_nodes, 2);
    assert_eq!(map.node_count(), 2);
    assert_eq!(map.connection_count(), 1);
    let conn = map.find_connection(1, 2).expect("Verbindung 1→2 erwartet");
    assert_eq!(conn.direction, ConnectionDirection::Dual);
}

#[test]
fn test_deduplicate_removes_self_connections() {
    let mut map = RoadMap::new(3);
    map.add_node(MapNode::new(1, Vec2::new(0.0, 0.0), NodeFlag::Regular));
    map.add_node(MapNode::new(2, Vec2::new(0.0, 0.0), NodeFlag::Regular));

    let conn = Connection::new(
        1,
        2,
        ConnectionDirection::Regular,
        ConnectionPriority::Regular,
        Vec2::new(0.0, 0.0),
        Vec2::new(0.0, 0.0),
    );
    map.add_connection(conn);

    let result = map.deduplicate_nodes(0.01);
    assert_eq!(result.removed_nodes, 1);
    assert_eq!(result.removed_self_connections, 1);
    assert_eq!(map.connection_count(), 0);
}

#[test]
fn test_deduplicate_updates_markers() {
    let mut map = RoadMap::new(3);
    map.add_node(MapNode::new(1, Vec2::new(0.0, 0.0), NodeFlag::Regular));
    map.add_node(MapNode::new(5, Vec2::new(0.0, 0.0), NodeFlag::Regular));

    use crate::core::MapMarker;
    map.map_markers.push(MapMarker::new(
        5,
        "TestMarker".to_string(),
        "All".to_string(),
        1,
        false,
    ));

    let result = map.deduplicate_nodes(0.01);
    assert_eq!(result.remapped_markers, 1);
    assert_eq!(map.map_markers.len(), 1);
    assert_eq!(map.map_markers[0].id, 1);
}

#[test]
fn test_deduplicate_within_epsilon_tolerance() {
    let mut map = RoadMap::new(3);
    map.add_node(MapNode::new(
        1,
        Vec2::new(100.004, 200.004),
        NodeFlag::Regular,
    ));
    map.add_node(MapNode::new(
        2,
        Vec2::new(100.004, 200.004),
        NodeFlag::Regular,
    ));

    let result = map.deduplicate_nodes(0.01);
    assert!(result.had_duplicates());
    assert_eq!(map.node_count(), 1);
}

#[test]
fn test_deduplicate_outside_epsilon_no_merge() {
    let mut map = RoadMap::new(3);
    map.add_node(MapNode::new(1, Vec2::new(100.0, 200.0), NodeFlag::Regular));
    map.add_node(MapNode::new(2, Vec2::new(100.02, 200.0), NodeFlag::Regular));

    let result = map.deduplicate_nodes(0.01);
    assert!(!result.had_duplicates());
    assert_eq!(map.node_count(), 2);
}

// ═══════════════════════════════════════════════════════════════════
// is_resampleable_chain Tests
// ═══════════════════════════════════════════════════════════════════

/// Hilfsfunktion: RoadMap mit Nodes und gerichteten Verbindungen aufbauen.
fn make_chain_map(nodes: &[(u64, f32, f32)], edges: &[(u64, u64)]) -> RoadMap {
    let mut map = RoadMap::new(3);
    for &(id, x, y) in nodes {
        map.add_node(MapNode::new(id, Vec2::new(x, y), NodeFlag::Regular));
    }
    for &(a, b) in edges {
        let pa = map.nodes[&a].position;
        let pb = map.nodes[&b].position;
        map.add_connection(Connection::new(
            a,
            b,
            ConnectionDirection::Regular,
            ConnectionPriority::Regular,
            pa,
            pb,
        ));
    }
    map
}

#[test]
fn resampleable_chain_simple_path() {
    // 1 → 2 → 3 → 4: einfache Kette
    let map = make_chain_map(
        &[
            (1, 0.0, 0.0),
            (2, 10.0, 0.0),
            (3, 20.0, 0.0),
            (4, 30.0, 0.0),
        ],
        &[(1, 2), (2, 3), (3, 4)],
    );
    let sel: IndexSet<u64> = [1, 2, 3, 4].into();
    assert!(map.is_resampleable_chain(&sel));
}

#[test]
fn resampleable_chain_too_few_nodes() {
    let map = make_chain_map(&[(1, 0.0, 0.0)], &[]);
    let sel: IndexSet<u64> = [1].into();
    assert!(!map.is_resampleable_chain(&sel));
}

#[test]
fn resampleable_chain_disconnected() {
    // 1 → 2, 3 → 4 (zwei getrennte Paare)
    let map = make_chain_map(
        &[
            (1, 0.0, 0.0),
            (2, 10.0, 0.0),
            (3, 20.0, 0.0),
            (4, 30.0, 0.0),
        ],
        &[(1, 2), (3, 4)],
    );
    let sel: IndexSet<u64> = [1, 2, 3, 4].into();
    assert!(!map.is_resampleable_chain(&sel));
}

#[test]
fn resampleable_chain_intersection_at_endpoint_ok() {
    // Kreuzung an Node 1 (Grad 3), aber Node 1 ist Endpunkt → erlaubt
    // 5 → 1 (nicht selektiert), 1 → 2 → 3
    let map = make_chain_map(
        &[
            (1, 0.0, 0.0),
            (2, 10.0, 0.0),
            (3, 20.0, 0.0),
            (5, -10.0, 0.0),
        ],
        &[(5, 1), (1, 2), (2, 3)],
    );
    let sel: IndexSet<u64> = [1, 2, 3].into();
    assert!(map.is_resampleable_chain(&sel));
}

#[test]
fn resampleable_chain_intersection_in_middle_rejected() {
    // Kreuzung an Node 2 (innerhalb der Selektion, Grad 3)
    // 1 → 2 → 3, 2 → 4 (alle 4 selektiert → Baum, keine Kette)
    let map = make_chain_map(
        &[
            (1, 0.0, 0.0),
            (2, 10.0, 0.0),
            (3, 20.0, 0.0),
            (4, 10.0, 10.0),
        ],
        &[(1, 2), (2, 3), (2, 4)],
    );
    let sel: IndexSet<u64> = [1, 2, 3, 4].into();
    assert!(!map.is_resampleable_chain(&sel));
}

#[test]
fn resampleable_chain_two_connected_nodes() {
    let map = make_chain_map(&[(1, 0.0, 0.0), (2, 10.0, 0.0)], &[(1, 2)]);
    let sel: IndexSet<u64> = [1, 2].into();
    assert!(map.is_resampleable_chain(&sel));
}

#[test]
fn resampleable_chain_two_unconnected_nodes() {
    let map = make_chain_map(&[(1, 0.0, 0.0), (2, 10.0, 0.0)], &[]);
    let sel: IndexSet<u64> = [1, 2].into();
    assert!(!map.is_resampleable_chain(&sel));
}

// ═══════════════════════════════════════════════════════════════════
// Adjacency-Index Tests
// ═══════════════════════════════════════════════════════════════════

/// Hilfsfunktion fuer einfache Verbindungen (dupliziert make_conn aus neighbors::tests)
fn make_adj_conn(s: u64, e: u64, sx: f32, sy: f32, ex: f32, ey: f32) -> Connection {
    Connection::new(
        s,
        e,
        ConnectionDirection::Regular,
        ConnectionPriority::Regular,
        Vec2::new(sx, sy),
        Vec2::new(ex, ey),
    )
}

/// Leerer Node hat degree 0 und leere Nachbar-Liste.
#[test]
fn test_adjacency_empty_node() {
    let mut map = RoadMap::new(3);
    map.add_node(MapNode::new(1, Vec2::ZERO, NodeFlag::Regular));

    assert_eq!(map.degree(1), 0);
    assert!(map.neighbors(1).is_empty());
    assert_eq!(map.outgoing_neighbors(1).count(), 0);
    assert_eq!(map.incoming_neighbors(1).count(), 0);
}

/// Unbekannte Node-ID liefert leeren Slice statt Panic.
#[test]
fn test_adjacency_unknown_node_returns_empty() {
    let map = RoadMap::new(3);
    assert!(map.neighbors(999).is_empty());
    assert_eq!(map.degree(999), 0);
}

/// Nach add_connection korrekte Adjacency fuer beide Endpunkte.
#[test]
fn test_adjacency_after_add_connection() {
    let mut map = RoadMap::new(3);
    map.add_node(MapNode::new(1, Vec2::new(0.0, 0.0), NodeFlag::Regular));
    map.add_node(MapNode::new(2, Vec2::new(10.0, 0.0), NodeFlag::Regular));
    map.add_connection(make_adj_conn(1, 2, 0.0, 0.0, 10.0, 0.0));

    // Node 1: ausgehend zu Node 2
    assert_eq!(map.degree(1), 1);
    let n1 = map.neighbors(1);
    assert_eq!(n1.len(), 1);
    assert_eq!(n1[0], (2, true));

    // Node 2: eingehend von Node 1
    assert_eq!(map.degree(2), 1);
    let n2 = map.neighbors(2);
    assert_eq!(n2.len(), 1);
    assert_eq!(n2[0], (1, false));

    // Iterator-API pruefen
    let out: Vec<u64> = map.outgoing_neighbors(1).collect();
    assert_eq!(out, vec![2]);
    assert_eq!(map.incoming_neighbors(1).count(), 0);

    let inc: Vec<u64> = map.incoming_neighbors(2).collect();
    assert_eq!(inc, vec![1]);
    assert_eq!(map.outgoing_neighbors(2).count(), 0);
}

/// Nach remove_connection wird der Index bereinigt.
#[test]
fn test_adjacency_after_remove_connection() {
    let mut map = RoadMap::new(3);
    map.add_node(MapNode::new(1, Vec2::new(0.0, 0.0), NodeFlag::Regular));
    map.add_node(MapNode::new(2, Vec2::new(10.0, 0.0), NodeFlag::Regular));
    map.add_connection(make_adj_conn(1, 2, 0.0, 0.0, 10.0, 0.0));

    let removed = map.remove_connection(1, 2);
    assert!(removed);

    assert_eq!(map.degree(1), 0);
    assert_eq!(map.degree(2), 0);
    assert!(map.neighbors(1).is_empty());
    assert!(map.neighbors(2).is_empty());
}

/// Bidirektionale Verbindungen (A\u2192B und B\u2192A) erzeugen korrekte Adjacency.
#[test]
fn test_adjacency_bidirectional_connections() {
    let mut map = RoadMap::new(3);
    map.add_node(MapNode::new(1, Vec2::new(0.0, 0.0), NodeFlag::Regular));
    map.add_node(MapNode::new(2, Vec2::new(10.0, 0.0), NodeFlag::Regular));
    map.add_connection(make_adj_conn(1, 2, 0.0, 0.0, 10.0, 0.0));
    map.add_connection(make_adj_conn(2, 1, 10.0, 0.0, 0.0, 0.0));

    // Beide Richtungen zaehlen → degree 2 pro Node
    assert_eq!(map.degree(1), 2);
    assert_eq!(map.degree(2), 2);

    // Node 1: (2, true) = ausgehend, (2, false) = eingehend
    let out1: Vec<u64> = map.outgoing_neighbors(1).collect();
    let inc1: Vec<u64> = map.incoming_neighbors(1).collect();
    assert_eq!(out1, vec![2]);
    assert_eq!(inc1, vec![2]);
}

/// remove_node bereinigt auch die Adjacency-Eintraege der Nachbarn.
#[test]
fn test_adjacency_after_remove_node() {
    let mut map = RoadMap::new(3);
    map.add_node(MapNode::new(1, Vec2::new(0.0, 0.0), NodeFlag::Regular));
    map.add_node(MapNode::new(2, Vec2::new(10.0, 0.0), NodeFlag::Regular));
    map.add_node(MapNode::new(3, Vec2::new(20.0, 0.0), NodeFlag::Regular));
    map.add_connection(make_adj_conn(1, 2, 0.0, 0.0, 10.0, 0.0));
    map.add_connection(make_adj_conn(2, 3, 10.0, 0.0, 20.0, 0.0));

    // Node 2 entfernen → alle Verbindungen weg, Nachbarn bereinigt
    map.remove_node(2);

    assert!(
        map.neighbors(1).is_empty(),
        "Node 1 sollte keine Nachbarn mehr haben"
    );
    assert!(
        map.neighbors(3).is_empty(),
        "Node 3 sollte keine Nachbarn mehr haben"
    );
    assert!(
        map.neighbors(2).is_empty(),
        "Geloeschter Node hat keine Adjacency mehr"
    );
    assert_eq!(map.degree(2), 0);
}

/// rebuild_adjacency_index stimmt mit inkrementeller Pflege ueberein.
#[test]
fn test_adjacency_rebuild_consistent_with_incremental() {
    let mut map = RoadMap::new(3);
    map.add_node(MapNode::new(1, Vec2::new(0.0, 0.0), NodeFlag::Regular));
    map.add_node(MapNode::new(2, Vec2::new(10.0, 0.0), NodeFlag::Regular));
    map.add_node(MapNode::new(3, Vec2::new(5.0, 5.0), NodeFlag::Regular));
    map.add_connection(make_adj_conn(1, 2, 0.0, 0.0, 10.0, 0.0));
    map.add_connection(make_adj_conn(1, 3, 0.0, 0.0, 5.0, 5.0));
    map.add_connection(make_adj_conn(3, 2, 5.0, 5.0, 10.0, 0.0));

    // Zustand nach inkrementeller Pflege
    let deg1_before = map.degree(1);
    let deg2_before = map.degree(2);
    let deg3_before = map.degree(3);

    // Rebuild erzeugt identischen Zustand
    map.rebuild_adjacency_index();

    assert_eq!(map.degree(1), deg1_before);
    assert_eq!(map.degree(2), deg2_before);
    assert_eq!(map.degree(3), deg3_before);
}

/// invert_connection aktualisiert die Richtungs-Flags im Adjacency-Index korrekt.
#[test]
fn test_adjacency_after_invert_connection() {
    let mut map = RoadMap::new(3);
    map.add_node(MapNode::new(1, Vec2::new(0.0, 0.0), NodeFlag::Regular));
    map.add_node(MapNode::new(2, Vec2::new(10.0, 0.0), NodeFlag::Regular));
    map.add_connection(make_adj_conn(1, 2, 0.0, 0.0, 10.0, 0.0));

    // Vor Invertierung: 1\u2192 outgoing nach 2, 2 \u2190 incoming von 1
    assert!(map.outgoing_neighbors(1).any(|id| id == 2));
    assert!(map.incoming_neighbors(2).any(|id| id == 1));

    map.invert_connection(1, 2);

    // Nach Invertierung: umgekehrt
    assert_eq!(
        map.outgoing_neighbors(1).count(),
        0,
        "Node 1 hat keine ausgehenden mehr"
    );
    assert!(
        map.incoming_neighbors(1).any(|id| id == 2),
        "Node 1 erhaelt jetzt von 2"
    );
    assert!(
        map.outgoing_neighbors(2).any(|id| id == 1),
        "Node 2 sendet jetzt zu 1"
    );
    assert_eq!(
        map.incoming_neighbors(2).count(),
        0,
        "Node 2 hat keine eingehenden mehr"
    );
}

/// remove_connections_between bereinigt beide Richtungen im Index.
#[test]
fn test_adjacency_after_remove_connections_between() {
    let mut map = RoadMap::new(3);
    map.add_node(MapNode::new(1, Vec2::new(0.0, 0.0), NodeFlag::Regular));
    map.add_node(MapNode::new(2, Vec2::new(10.0, 0.0), NodeFlag::Regular));
    map.add_connection(make_adj_conn(1, 2, 0.0, 0.0, 10.0, 0.0));
    map.add_connection(make_adj_conn(2, 1, 10.0, 0.0, 0.0, 0.0));

    let count = map.remove_connections_between(1, 2);
    assert_eq!(count, 2);

    assert_eq!(map.degree(1), 0);
    assert_eq!(map.degree(2), 0);
}
