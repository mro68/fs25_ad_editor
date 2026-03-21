use super::validation::cleanup_separators;
use super::*;
use crate::app::{
    Connection, ConnectionDirection, ConnectionPriority, MapMarker, MapNode, NodeFlag, RoadMap,
};
use crate::shared::{t, I18nKey, Language};
use glam::Vec2;
use indexmap::IndexSet;

/// Erstellt eine RoadMap mit gegebenen Nodes (IDs und Positionen).
fn make_road_map(nodes: &[(u64, f32, f32)]) -> RoadMap {
    let mut map = RoadMap::new(3);
    for &(id, x, y) in nodes {
        map.add_node(MapNode::new(id, Vec2::new(x, y), NodeFlag::Regular));
    }
    map
}

/// Erstellt eine RoadMap mit 2 Nodes und einer Verbindung dazwischen.
fn make_connected_map(id_a: u64, id_b: u64) -> RoadMap {
    let mut map = make_road_map(&[(id_a, 0.0, 0.0), (id_b, 10.0, 10.0)]);
    let pos_a = map.nodes[&id_a].position;
    let pos_b = map.nodes[&id_b].position;
    map.add_connection(Connection::new(
        id_a,
        id_b,
        ConnectionDirection::Regular,
        ConnectionPriority::Regular,
        pos_a,
        pos_b,
    ));
    map
}

/// Zaehlt Commands in der validierten Entry-Liste (rekursiv, inkl. Submenues).
fn count_commands(entries: &[ValidatedEntry]) -> usize {
    entries
        .iter()
        .map(|e| match e {
            ValidatedEntry::Command { .. } => 1,
            ValidatedEntry::Submenu { entries, .. } => count_commands(entries),
            _ => 0,
        })
        .sum()
}

/// Prueft ob ein bestimmter CommandId in den Entries enthalten ist (rekursiv, inkl. Submenues).
fn has_command(entries: &[ValidatedEntry], target: CommandId) -> bool {
    entries.iter().any(|e| match e {
        ValidatedEntry::Command { id, .. } => *id == target,
        ValidatedEntry::Submenu { entries, .. } => has_command(entries, target),
        _ => false,
    })
}

// ═══════════════════════════════════════════════════════════════════
// Precondition-Tests
// ═══════════════════════════════════════════════════════════════════

#[test]
fn precondition_node_exists() {
    let map = make_road_map(&[(1, 0.0, 0.0)]);
    let selected = IndexSet::new();
    let ctx = PreconditionContext {
        road_map: &map,
        selected_node_ids: &selected,
        distanzen_active: false,
        clipboard_has_data: false,
        segment_record_id: None,
        group_editing_active: false,
        farmland_polygons_loaded: false,
    };

    assert!(Precondition::NodeExists(1).is_valid(&ctx));
    assert!(!Precondition::NodeExists(999).is_valid(&ctx));
}

#[test]
fn precondition_has_marker() {
    let mut map = make_road_map(&[(1, 0.0, 0.0), (2, 10.0, 0.0)]);
    map.add_map_marker(MapMarker::new(1, "Test".into(), "Default".into(), 1, false));
    let selected = IndexSet::new();
    let ctx = PreconditionContext {
        road_map: &map,
        selected_node_ids: &selected,
        distanzen_active: false,
        clipboard_has_data: false,
        segment_record_id: None,
        group_editing_active: false,
        farmland_polygons_loaded: false,
    };

    assert!(Precondition::HasMarker(1).is_valid(&ctx));
    assert!(!Precondition::HasMarker(2).is_valid(&ctx));
    assert!(!Precondition::HasNoMarker(1).is_valid(&ctx));
    assert!(Precondition::HasNoMarker(2).is_valid(&ctx));
}

#[test]
fn precondition_exactly_two_selected() {
    let map = make_road_map(&[(1, 0.0, 0.0), (2, 10.0, 0.0), (3, 20.0, 0.0)]);
    let two: IndexSet<u64> = [1, 2].into();
    let three: IndexSet<u64> = [1, 2, 3].into();
    let one: IndexSet<u64> = [1].into();

    let ctx2 = PreconditionContext {
        road_map: &map,
        selected_node_ids: &two,
        distanzen_active: false,
        clipboard_has_data: false,
        segment_record_id: None,
        group_editing_active: false,
        farmland_polygons_loaded: false,
    };
    let ctx3 = PreconditionContext {
        road_map: &map,
        selected_node_ids: &three,
        distanzen_active: false,
        clipboard_has_data: false,
        segment_record_id: None,
        group_editing_active: false,
        farmland_polygons_loaded: false,
    };
    let ctx1 = PreconditionContext {
        road_map: &map,
        selected_node_ids: &one,
        distanzen_active: false,
        clipboard_has_data: false,
        segment_record_id: None,
        group_editing_active: false,
        farmland_polygons_loaded: false,
    };

    assert!(Precondition::ExactlyTwoSelected.is_valid(&ctx2));
    assert!(!Precondition::ExactlyTwoSelected.is_valid(&ctx3));
    assert!(!Precondition::ExactlyTwoSelected.is_valid(&ctx1));
}

#[test]
fn precondition_two_selected_unconnected() {
    let map_unconnected = make_road_map(&[(1, 0.0, 0.0), (2, 10.0, 0.0)]);
    let map_connected = make_connected_map(1, 2);
    let selected: IndexSet<u64> = [1, 2].into();

    let ctx_unconnected = PreconditionContext {
        road_map: &map_unconnected,
        selected_node_ids: &selected,
        distanzen_active: false,
        clipboard_has_data: false,
        segment_record_id: None,
        group_editing_active: false,
        farmland_polygons_loaded: false,
    };
    let ctx_connected = PreconditionContext {
        road_map: &map_connected,
        selected_node_ids: &selected,
        distanzen_active: false,
        clipboard_has_data: false,
        segment_record_id: None,
        group_editing_active: false,
        farmland_polygons_loaded: false,
    };

    assert!(Precondition::TwoSelectedUnconnected.is_valid(&ctx_unconnected));
    assert!(!Precondition::TwoSelectedUnconnected.is_valid(&ctx_connected));
}

#[test]
fn precondition_has_connections_between_selected() {
    let map_connected = make_connected_map(1, 2);
    let map_unconnected = make_road_map(&[(1, 0.0, 0.0), (2, 10.0, 0.0)]);
    let selected: IndexSet<u64> = [1, 2].into();

    let ctx_yes = PreconditionContext {
        road_map: &map_connected,
        selected_node_ids: &selected,
        distanzen_active: false,
        clipboard_has_data: false,
        segment_record_id: None,
        group_editing_active: false,
        farmland_polygons_loaded: false,
    };
    let ctx_no = PreconditionContext {
        road_map: &map_unconnected,
        selected_node_ids: &selected,
        distanzen_active: false,
        clipboard_has_data: false,
        segment_record_id: None,
        group_editing_active: false,
        farmland_polygons_loaded: false,
    };

    assert!(Precondition::HasConnectionsBetweenSelected.is_valid(&ctx_yes));
    assert!(!Precondition::HasConnectionsBetweenSelected.is_valid(&ctx_no));
}

#[test]
fn precondition_streckenteilung_active() {
    let map = make_road_map(&[]);
    let selected = IndexSet::new();

    let ctx_active = PreconditionContext {
        road_map: &map,
        selected_node_ids: &selected,
        distanzen_active: true,
        clipboard_has_data: false,
        segment_record_id: None,
        group_editing_active: false,
        farmland_polygons_loaded: false,
    };
    let ctx_inactive = PreconditionContext {
        road_map: &map,
        selected_node_ids: &selected,
        distanzen_active: false,
        clipboard_has_data: false,
        segment_record_id: None,
        group_editing_active: false,
        farmland_polygons_loaded: false,
    };

    assert!(Precondition::StreckenteilungActive(true).is_valid(&ctx_active));
    assert!(!Precondition::StreckenteilungActive(true).is_valid(&ctx_inactive));
    assert!(Precondition::StreckenteilungActive(false).is_valid(&ctx_inactive));
}

// ═══════════════════════════════════════════════════════════════════
// Katalog-Tests
// ═══════════════════════════════════════════════════════════════════

#[test]
fn catalog_empty_area_shows_tools() {
    let map = make_road_map(&[]);
    let selected = IndexSet::new();
    let ctx = PreconditionContext {
        road_map: &map,
        selected_node_ids: &selected,
        distanzen_active: false,
        clipboard_has_data: false,
        segment_record_id: None,
        group_editing_active: false,
        farmland_polygons_loaded: false,
    };
    let intent_ctx = IntentContext {
        node_id: None,
        node_position: None,
        two_node_ids: None,
        segment_record_id: None,
    };

    let catalog = MenuCatalog::for_empty_area(Language::De);
    let entries = validate_entries(&catalog, &ctx, &intent_ctx);

    assert!(has_command(&entries, CommandId::SetToolSelect));
    assert!(has_command(&entries, CommandId::SetToolConnect));
    assert!(has_command(&entries, CommandId::SetToolAddNode));
    assert!(has_command(&entries, CommandId::SetToolRouteSmoothCurve));
    assert!(has_command(&entries, CommandId::SetToolRouteStraight));
    assert!(has_command(&entries, CommandId::SetToolRouteQuadratic));
    assert!(has_command(&entries, CommandId::SetToolRouteCubic));
    assert!(has_command(&entries, CommandId::ZoomToFit));
    assert_eq!(count_commands(&entries), 8);
}

#[test]
fn catalog_empty_area_smooth_curve_is_first_route_entry() {
    let catalog = MenuCatalog::for_empty_area(Language::De);
    let route_submenu = catalog
        .entries
        .iter()
        .find_map(|entry| match entry {
            MenuEntry::Submenu { label, entries }
                if label == t(Language::De, I18nKey::CtxRouteSubmenu) =>
            {
                Some(entries)
            }
            _ => None,
        })
        .expect("Strecke-Untermenue erwartet");

    let first_route_id = route_submenu.iter().find_map(|entry| match entry {
        MenuEntry::Command { id, .. } => Some(*id),
        _ => None,
    });

    assert_eq!(first_route_id, Some(CommandId::SetToolRouteSmoothCurve));
}

#[test]
fn catalog_node_focused_shows_marker_create() {
    let map = make_road_map(&[(42, 5.0, 5.0)]);
    let selected: IndexSet<u64> = [42].into();
    let ctx = PreconditionContext {
        road_map: &map,
        selected_node_ids: &selected,
        distanzen_active: false,
        clipboard_has_data: false,
        segment_record_id: None,
        group_editing_active: false,
        farmland_polygons_loaded: false,
    };
    let intent_ctx = IntentContext {
        node_id: Some(42),
        node_position: Some(Vec2::new(5.0, 5.0)),
        two_node_ids: None,
        segment_record_id: None,
    };

    let catalog = MenuCatalog::for_node_focused(42, Language::De);
    let entries = validate_entries(&catalog, &ctx, &intent_ctx);

    assert!(has_command(&entries, CommandId::CreateMarker));
    assert!(!has_command(&entries, CommandId::EditMarker));
    assert!(!has_command(&entries, CommandId::RemoveMarker));
}

#[test]
fn catalog_node_focused_shows_marker_edit_when_marker_exists() {
    let mut map = make_road_map(&[(42, 5.0, 5.0)]);
    map.add_map_marker(MapMarker::new(
        42,
        "Farm".into(),
        "Default".into(),
        1,
        false,
    ));
    let selected: IndexSet<u64> = [42].into();
    let ctx = PreconditionContext {
        road_map: &map,
        selected_node_ids: &selected,
        distanzen_active: false,
        clipboard_has_data: false,
        segment_record_id: None,
        group_editing_active: false,
        farmland_polygons_loaded: false,
    };
    let intent_ctx = IntentContext {
        node_id: Some(42),
        node_position: Some(Vec2::new(5.0, 5.0)),
        two_node_ids: None,
        segment_record_id: None,
    };

    let catalog = MenuCatalog::for_node_focused(42, Language::De);
    let entries = validate_entries(&catalog, &ctx, &intent_ctx);

    assert!(has_command(&entries, CommandId::EditMarker));
    assert!(has_command(&entries, CommandId::RemoveMarker));
    assert!(!has_command(&entries, CommandId::CreateMarker));
}

#[test]
fn catalog_node_focused_shows_delete_and_duplicate() {
    let map = make_road_map(&[(10, 1.0, 1.0)]);
    let selected: IndexSet<u64> = [10].into();
    let ctx = PreconditionContext {
        road_map: &map,
        selected_node_ids: &selected,
        distanzen_active: false,
        clipboard_has_data: false,
        segment_record_id: None,
        group_editing_active: false,
        farmland_polygons_loaded: false,
    };
    let intent_ctx = IntentContext {
        node_id: Some(10),
        node_position: Some(Vec2::new(1.0, 1.0)),
        two_node_ids: None,
        segment_record_id: None,
    };

    let catalog = MenuCatalog::for_node_focused(10, Language::De);
    let entries = validate_entries(&catalog, &ctx, &intent_ctx);

    assert!(has_command(&entries, CommandId::DeleteSelected));
}

#[test]
fn catalog_multi_nodes_connect_only_when_two_unconnected() {
    let map_unconnected = make_road_map(&[(1, 0.0, 0.0), (2, 10.0, 0.0)]);
    let selected: IndexSet<u64> = [1, 2].into();
    let ctx = PreconditionContext {
        road_map: &map_unconnected,
        selected_node_ids: &selected,
        distanzen_active: false,
        clipboard_has_data: false,
        segment_record_id: None,
        group_editing_active: false,
        farmland_polygons_loaded: false,
    };
    let intent_ctx = IntentContext {
        node_id: None,
        node_position: None,
        two_node_ids: Some((1, 2)),
        segment_record_id: None,
    };

    let catalog = MenuCatalog::for_selection_only(Language::De);
    let entries = validate_entries(&catalog, &ctx, &intent_ctx);
    assert!(has_command(&entries, CommandId::ConnectTwoNodes));

    let map_connected = make_connected_map(1, 2);
    let ctx_connected = PreconditionContext {
        road_map: &map_connected,
        selected_node_ids: &selected,
        distanzen_active: false,
        clipboard_has_data: false,
        segment_record_id: None,
        group_editing_active: false,
        farmland_polygons_loaded: false,
    };
    let entries_connected = validate_entries(&catalog, &ctx_connected, &intent_ctx);
    assert!(!has_command(&entries_connected, CommandId::ConnectTwoNodes));
}

#[test]
fn catalog_multi_nodes_direction_only_when_connected() {
    let map = make_road_map(&[(1, 0.0, 0.0), (2, 10.0, 0.0)]);
    let selected: IndexSet<u64> = [1, 2].into();
    let ctx = PreconditionContext {
        road_map: &map,
        selected_node_ids: &selected,
        distanzen_active: false,
        clipboard_has_data: false,
        segment_record_id: None,
        group_editing_active: false,
        farmland_polygons_loaded: false,
    };
    let intent_ctx = IntentContext {
        node_id: None,
        node_position: None,
        two_node_ids: Some((1, 2)),
        segment_record_id: None,
    };

    let catalog = MenuCatalog::for_selection_only(Language::De);
    let entries = validate_entries(&catalog, &ctx, &intent_ctx);

    assert!(!has_command(&entries, CommandId::DirectionRegular));
    assert!(!has_command(&entries, CommandId::PriorityRegular));

    let map_connected = make_connected_map(1, 2);
    let ctx_connected = PreconditionContext {
        road_map: &map_connected,
        selected_node_ids: &selected,
        distanzen_active: false,
        clipboard_has_data: false,
        segment_record_id: None,
        group_editing_active: false,
        farmland_polygons_loaded: false,
    };
    let entries_connected = validate_entries(&catalog, &ctx_connected, &intent_ctx);

    assert!(has_command(&entries_connected, CommandId::DirectionRegular));
    assert!(has_command(&entries_connected, CommandId::PriorityRegular));
}

#[test]
fn catalog_multi_nodes_route_tools_only_when_two_selected() {
    let map = make_road_map(&[(1, 0.0, 0.0), (2, 10.0, 0.0), (3, 20.0, 0.0)]);
    let selected: IndexSet<u64> = [1, 2, 3].into();
    let ctx = PreconditionContext {
        road_map: &map,
        selected_node_ids: &selected,
        distanzen_active: false,
        clipboard_has_data: false,
        segment_record_id: None,
        group_editing_active: false,
        farmland_polygons_loaded: false,
    };
    let intent_ctx = IntentContext {
        node_id: None,
        node_position: None,
        two_node_ids: None,
        segment_record_id: None,
    };

    let catalog = MenuCatalog::for_selection_only(Language::De);
    let entries = validate_entries(&catalog, &ctx, &intent_ctx);

    assert!(!has_command(&entries, CommandId::RouteStraight));
    assert!(!has_command(&entries, CommandId::RouteSmoothCurve));
    assert!(!has_command(&entries, CommandId::RouteQuadratic));
    assert!(!has_command(&entries, CommandId::RouteCubic));
}

#[test]
fn catalog_selection_smooth_curve_is_first_generate_entry() {
    let entries = MenuCatalog::for_selection_only(Language::De).entries;
    let generate_submenu = entries
        .iter()
        .find_map(|entry| match entry {
            MenuEntry::Submenu { label, entries }
                if label == t(Language::De, I18nKey::CtxCreateRoute) =>
            {
                Some(entries)
            }
            _ => None,
        })
        .expect("Strecke-erzeugen-Untermenue erwartet");

    let first_route_id = generate_submenu.iter().find_map(|entry| match entry {
        MenuEntry::Command { id, .. } => Some(*id),
        _ => None,
    });

    assert_eq!(first_route_id, Some(CommandId::RouteSmoothCurve));
}

#[test]
fn catalog_multi_nodes_selection_commands_always_visible() {
    let map = make_road_map(&[(1, 0.0, 0.0), (2, 10.0, 0.0)]);
    let selected: IndexSet<u64> = [1, 2].into();
    let ctx = PreconditionContext {
        road_map: &map,
        selected_node_ids: &selected,
        distanzen_active: false,
        clipboard_has_data: false,
        segment_record_id: None,
        group_editing_active: false,
        farmland_polygons_loaded: false,
    };
    let intent_ctx = IntentContext {
        node_id: None,
        node_position: None,
        two_node_ids: Some((1, 2)),
        segment_record_id: None,
    };

    let catalog = MenuCatalog::for_selection_only(Language::De);
    let entries = validate_entries(&catalog, &ctx, &intent_ctx);

    assert!(has_command(&entries, CommandId::DeleteSelected));
}

#[test]
fn catalog_route_tool_basic_commands() {
    let map = make_road_map(&[]);
    let selected = IndexSet::new();
    let ctx = PreconditionContext {
        road_map: &map,
        selected_node_ids: &selected,
        distanzen_active: false,
        clipboard_has_data: false,
        segment_record_id: None,
        group_editing_active: false,
        farmland_polygons_loaded: false,
    };
    let intent_ctx = IntentContext {
        node_id: None,
        node_position: None,
        two_node_ids: None,
        segment_record_id: None,
    };

    let catalog = MenuCatalog::for_route_tool();
    let entries = validate_entries(&catalog, &ctx, &intent_ctx);

    assert!(has_command(&entries, CommandId::RouteExecute));
    assert!(has_command(&entries, CommandId::RouteRecreate));
    assert!(has_command(&entries, CommandId::RouteCancel));
    assert_eq!(count_commands(&entries), 3);
}

#[test]
fn intent_mapping_delete_selected() {
    let ctx = IntentContext {
        node_id: Some(42),
        node_position: Some(Vec2::new(5.0, 5.0)),
        two_node_ids: None,
        segment_record_id: None,
    };
    let intent = CommandId::DeleteSelected.to_intent(&ctx);
    assert!(matches!(intent, AppIntent::DeleteSelectedRequested));
}

#[test]
fn intent_mapping_connect_two_nodes() {
    let ctx = IntentContext {
        node_id: None,
        node_position: None,
        two_node_ids: Some((1, 2)),
        segment_record_id: None,
    };
    let intent = CommandId::ConnectTwoNodes.to_intent(&ctx);
    assert!(matches!(intent, AppIntent::ConnectSelectedNodesRequested));
}

#[test]
fn cleanup_removes_orphaned_labels() {
    let entries = vec![
        ValidatedEntry::Label("Richtung:".into()),
        ValidatedEntry::Separator,
        ValidatedEntry::Command {
            id: CommandId::DeleteSelected,
            label: "Loeschen".into(),
            intent: Box::new(AppIntent::DeleteSelectedRequested),
        },
    ];

    let cleaned = cleanup_separators(&entries);

    assert!(!cleaned
        .iter()
        .any(|e| matches!(e, ValidatedEntry::Label(l) if l == "Richtung:")));
    assert!(has_command(&cleaned, CommandId::DeleteSelected));
}

#[test]
fn cleanup_keeps_labels_with_commands() {
    let entries = vec![
        ValidatedEntry::Label("🗺 Marker".into()),
        ValidatedEntry::Command {
            id: CommandId::CreateMarker,
            label: "Erstellen".into(),
            intent: Box::new(AppIntent::CreateMarkerRequested { node_id: 1 }),
        },
    ];

    let cleaned = cleanup_separators(&entries);

    assert!(cleaned
        .iter()
        .any(|e| matches!(e, ValidatedEntry::Label(l) if l == "🗺 Marker")));
    assert!(has_command(&cleaned, CommandId::CreateMarker));
}

#[test]
fn cleanup_no_double_separators() {
    let entries = vec![
        ValidatedEntry::Command {
            id: CommandId::SelectAll,
            label: "Sel".into(),
            intent: Box::new(AppIntent::SelectAllRequested),
        },
        ValidatedEntry::Separator,
        ValidatedEntry::Separator,
        ValidatedEntry::Command {
            id: CommandId::DeleteSelected,
            label: "Del".into(),
            intent: Box::new(AppIntent::DeleteSelectedRequested),
        },
    ];

    let cleaned = cleanup_separators(&entries);
    let sep_count = cleaned
        .iter()
        .filter(|e| matches!(e, ValidatedEntry::Separator))
        .count();
    assert_eq!(sep_count, 1);
}

#[test]
fn deleted_node_hides_all_commands() {
    let map = make_road_map(&[(1, 0.0, 0.0)]);
    let selected = IndexSet::new();
    let ctx = PreconditionContext {
        road_map: &map,
        selected_node_ids: &selected,
        distanzen_active: false,
        clipboard_has_data: false,
        segment_record_id: None,
        group_editing_active: false,
        farmland_polygons_loaded: false,
    };
    let intent_ctx = IntentContext {
        node_id: Some(99),
        node_position: None,
        two_node_ids: None,
        segment_record_id: None,
    };

    let catalog = MenuCatalog::for_node_focused(99, Language::De);
    let entries = validate_entries(&catalog, &ctx, &intent_ctx);

    // Geloeschter Node: NodeExists-Precondition schlaegt fehl → keine node-spezifischen Commands
    assert!(!has_command(&entries, CommandId::CreateMarker));
    assert!(!has_command(&entries, CommandId::CreateMarker));
}
