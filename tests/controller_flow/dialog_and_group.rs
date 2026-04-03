use fs25_auto_drive_editor::app::{AppController, AppIntent, AppState, GroupRecord};

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

    assert!(!state.ui.show_command_palette);

    controller
        .handle_intent(&mut state, AppIntent::CommandPaletteToggled)
        .expect("CommandPaletteToggled sollte die Palette oeffnen");
    assert!(state.ui.show_command_palette);

    controller
        .handle_intent(&mut state, AppIntent::CommandPaletteToggled)
        .expect("CommandPaletteToggled sollte die Palette wieder schliessen");
    assert!(!state.ui.show_command_palette);
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

    assert_eq!(state.ui.confirm_dissolve_group_id, Some(record_id));
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
