use fs25_auto_drive_editor::app::ui_contract::{DialogRequest, DialogRequestKind};
use fs25_auto_drive_editor::app::{
    AppController, AppIntent, AppState, GroupRecord, OverviewSourceContext,
};
use fs25_auto_drive_editor::core::{
    Connection, ConnectionDirection, ConnectionPriority, MapNode, NodeFlag, RoadMap,
};
use std::sync::Arc;

fn make_manual_group_record(id: u64) -> GroupRecord {
    GroupRecord {
        id,
        node_ids: vec![1, 2],
        original_positions: vec![glam::Vec2::ZERO, glam::Vec2::new(10.0, 0.0)],
        marker_node_ids: Vec::new(),
        locked: false,
        entry_node_id: None,
        exit_node_id: None,
    }
}

#[test]
fn command_palette_toggled_flips_visibility() {
    let mut controller = AppController::new();
    let mut state = AppState::new();

    // Vor dem ersten Toggle: keine DialogRequest-Eintraege
    assert!(state.ui.dialog_requests.is_empty());

    controller
        .handle_intent(&mut state, AppIntent::CommandPaletteToggled)
        .expect("CommandPaletteToggled sollte die Palette oeffnen");
    assert!(
        state
            .ui
            .dialog_requests
            .iter()
            .any(|r| matches!(r, DialogRequest::ToggleCommandPalette)),
        "ToggleCommandPalette muss in dialog_requests stehen"
    );

    // Zweiter Toggle: nochmals in Queue
    controller
        .handle_intent(&mut state, AppIntent::CommandPaletteToggled)
        .expect("CommandPaletteToggled sollte die Palette wieder schliessen");
    assert_eq!(
        state
            .ui
            .dialog_requests
            .iter()
            .filter(|r| matches!(r, DialogRequest::ToggleCommandPalette))
            .count(),
        2,
        "Zweiter Toggle muss zweiten ToggleCommandPalette-Request erzeugen"
    );
}

#[test]
fn dissolve_group_requested_opens_confirm_dialog_without_mutating_registry() {
    let mut controller = AppController::new();
    let mut state = AppState::new();
    let record_id = 7;
    state
        .group_registry
        .register(make_manual_group_record(record_id));

    controller
        .handle_intent(
            &mut state,
            AppIntent::DissolveGroupRequested {
                segment_id: record_id,
            },
        )
        .expect("DissolveGroupRequested sollte den Confirm-Dialog oeffnen");

    assert!(
        state
            .ui
            .dialog_requests
            .iter()
            .any(|r| matches!(r, DialogRequest::ShowDissolveGroupConfirm(id) if *id == record_id)),
        "ShowDissolveGroupConfirm muss in dialog_requests stehen"
    );
    assert!(state.group_registry.get(record_id).is_some());
}

#[test]
fn dissolve_group_confirmed_removes_record() {
    let mut controller = AppController::new();
    let mut state = AppState::new();
    let record_id = 9;
    state
        .group_registry
        .register(make_manual_group_record(record_id));

    controller
        .handle_intent(
            &mut state,
            AppIntent::DissolveGroupConfirmed {
                segment_id: record_id,
            },
        )
        .expect("DissolveGroupConfirmed sollte den Record entfernen");

    assert!(state.group_registry.get(record_id).is_none());
}

// ======================== Group-Operations-Flow-Tests ========================

/// Erstellt einen AppState mit 3 miteinander verbundenen Nodes (1→2→3).
fn make_connected_map_3nodes() -> AppState {
    let mut map = RoadMap::new(3);
    map.add_node(MapNode::new(
        1,
        glam::Vec2::new(0.0, 0.0),
        NodeFlag::Regular,
    ));
    map.add_node(MapNode::new(
        2,
        glam::Vec2::new(10.0, 0.0),
        NodeFlag::Regular,
    ));
    map.add_node(MapNode::new(
        3,
        glam::Vec2::new(20.0, 0.0),
        NodeFlag::Regular,
    ));
    map.add_connection(Connection::new(
        1,
        2,
        ConnectionDirection::Regular,
        ConnectionPriority::Regular,
        glam::Vec2::new(0.0, 0.0),
        glam::Vec2::new(10.0, 0.0),
    ));
    map.add_connection(Connection::new(
        2,
        3,
        ConnectionDirection::Regular,
        ConnectionPriority::Regular,
        glam::Vec2::new(10.0, 0.0),
        glam::Vec2::new(20.0, 0.0),
    ));
    map.ensure_spatial_index();
    let mut state = AppState::new();
    state.road_map = Some(Arc::new(map));
    state
}

#[test]
fn test_group_selection_as_group_creates_registry_entry() {
    let mut state = make_connected_map_3nodes();
    let mut controller = AppController::new();

    // Alle 3 Nodes selektieren (bilden zusammenhaengenden Subgraphen)
    state.selection.ids_mut().insert(1);
    state.selection.ids_mut().insert(2);
    state.selection.ids_mut().insert(3);

    controller
        .handle_intent(&mut state, AppIntent::GroupSelectionAsGroupRequested)
        .expect("GroupSelectionAsGroupRequested muss erfolgreich sein");

    // next_id startet bei 0 → erster Record erhaelt ID 0
    let record = state
        .group_registry
        .get(0)
        .expect("GroupRegistry muss einen Record mit ID 0 enthalten");

    assert_eq!(
        record.node_ids.len(),
        3,
        "Record muss genau 3 Nodes enthalten"
    );
    assert!(
        record.node_ids.contains(&1),
        "Node 1 muss im Record enthalten sein"
    );
    assert!(
        record.node_ids.contains(&2),
        "Node 2 muss im Record enthalten sein"
    );
    assert!(
        record.node_ids.contains(&3),
        "Node 3 muss im Record enthalten sein"
    );
    assert!(!record.locked, "Neuer Record muss initial entsperrt sein");
}

#[test]
fn test_remove_selected_nodes_from_group_updates_registry() {
    let mut state = AppState::new();
    let mut controller = AppController::new();

    // Gruppe mit 3 Nodes manuell registrieren
    let record_id = state.group_registry.next_id();
    state.group_registry.register(GroupRecord {
        id: record_id,
        node_ids: vec![1, 2, 3],
        original_positions: vec![
            glam::Vec2::new(0.0, 0.0),
            glam::Vec2::new(10.0, 0.0),
            glam::Vec2::new(20.0, 0.0),
        ],
        marker_node_ids: Vec::new(),
        locked: false,
        entry_node_id: None,
        exit_node_id: None,
    });

    // Node 3 selektieren und aus der Gruppe entfernen
    state.selection.ids_mut().insert(3);

    controller
        .handle_intent(&mut state, AppIntent::RemoveSelectedNodesFromGroupRequested)
        .expect("RemoveSelectedNodesFromGroupRequested muss erfolgreich sein");

    let record = state
        .group_registry
        .get(record_id)
        .expect("Record muss nach Entfernung eines Nodes weiterhin existieren");

    assert_eq!(
        record.node_ids.len(),
        2,
        "Record muss genau 2 Nodes enthalten"
    );
    assert!(
        !record.node_ids.contains(&3),
        "Node 3 darf nicht mehr im Record enthalten sein"
    );
    assert!(
        record.node_ids.contains(&1),
        "Node 1 muss noch im Record enthalten sein"
    );
    assert!(
        record.node_ids.contains(&2),
        "Node 2 muss noch im Record enthalten sein"
    );
}

#[test]
fn test_toggle_group_lock_changes_lock_state() {
    let mut state = AppState::new();
    let mut controller = AppController::new();

    // Gruppe mit bekannter ID registrieren
    let segment_id = 5;
    state
        .group_registry
        .register(make_manual_group_record(segment_id));

    // Vorbedingung: Gruppe ist initial entsperrt
    assert!(
        !state.group_registry.is_locked(segment_id),
        "Neue Gruppe muss initial entsperrt sein"
    );

    // Erster Toggle: sperren
    controller
        .handle_intent(
            &mut state,
            AppIntent::ToggleGroupLockRequested { segment_id },
        )
        .expect("Erster ToggleGroupLockRequested muss erfolgreich sein");

    assert!(
        state.group_registry.is_locked(segment_id),
        "Gruppe muss nach erstem Toggle gesperrt sein"
    );

    // Zweiter Toggle: entsperren
    controller
        .handle_intent(
            &mut state,
            AppIntent::ToggleGroupLockRequested { segment_id },
        )
        .expect("Zweiter ToggleGroupLockRequested muss erfolgreich sein");

    assert!(
        !state.group_registry.is_locked(segment_id),
        "Gruppe muss nach zweitem Toggle wieder entsperrt sein"
    );
}

#[test]
fn test_set_group_boundary_nodes_updates_boundary_candidates() {
    let mut state = AppState::new();
    let mut controller = AppController::new();

    // Gruppe mit 3 Nodes manuell registrieren
    let record_id = state.group_registry.next_id();
    state.group_registry.register(GroupRecord {
        id: record_id,
        node_ids: vec![1, 2, 3],
        original_positions: vec![
            glam::Vec2::new(0.0, 0.0),
            glam::Vec2::new(10.0, 0.0),
            glam::Vec2::new(20.0, 0.0),
        ],
        marker_node_ids: Vec::new(),
        locked: false,
        entry_node_id: None,
        exit_node_id: None,
    });

    // Entry- und Exit-Nodes setzen (muessen im Record enthalten sein)
    controller
        .handle_intent(
            &mut state,
            AppIntent::SetGroupBoundaryNodes {
                record_id,
                entry_node_id: Some(1),
                exit_node_id: Some(3),
            },
        )
        .expect("SetGroupBoundaryNodes muss erfolgreich sein");

    let record = state
        .group_registry
        .get(record_id)
        .expect("Record muss nach SetGroupBoundaryNodes weiterhin existieren");

    assert_eq!(
        record.entry_node_id,
        Some(1),
        "entry_node_id muss auf Node 1 gesetzt sein"
    );
    assert_eq!(
        record.exit_node_id,
        Some(3),
        "exit_node_id muss auf Node 3 gesetzt sein"
    );
}

#[test]
fn generate_overview_requested_opens_manual_source_dialog() {
    let mut controller = AppController::new();
    let mut state = AppState::new();

    controller
        .handle_intent(&mut state, AppIntent::GenerateOverviewRequested)
        .expect("GenerateOverviewRequested sollte den Source-Dialog oeffnen");

    assert!(state.ui.post_load_dialog.visible);
    assert_eq!(
        state.ui.post_load_dialog.context,
        OverviewSourceContext::ManualMenu
    );
    assert!(state.ui.post_load_dialog.matching_zips.is_empty());
    assert!(!state.ui.post_load_dialog.heightmap_set);
    assert!(!state.ui.post_load_dialog.overview_loaded);
    assert!(state.ui.post_load_dialog.map_name.is_empty());
}

#[test]
fn overview_zip_browse_requested_queues_native_picker() {
    let mut controller = AppController::new();
    let mut state = AppState::new();

    controller
        .handle_intent(&mut state, AppIntent::OverviewZipBrowseRequested)
        .expect("OverviewZipBrowseRequested sollte den nativen Picker anfordern");

    assert!(state.ui.dialog_requests.iter().any(|request| {
        matches!(
            request,
            DialogRequest::PickPath {
                kind: DialogRequestKind::OverviewZip,
                ..
            }
        )
    }));
}
