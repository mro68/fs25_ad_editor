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
