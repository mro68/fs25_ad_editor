use crate::app::{AppCommand, AppIntent, AppState, ConnectionDirection};

use super::map_intent_to_commands;

#[test]
fn save_requested_maps_to_save_file_without_path() {
    let state = AppState::new();

    let commands = map_intent_to_commands(&state, AppIntent::SaveRequested);

    assert_eq!(commands.len(), 1);
    assert!(matches!(commands[0], AppCommand::SaveFile { path: None }));
}

#[test]
fn heightmap_selection_requested_maps_to_two_commands_in_order() {
    let state = AppState::new();

    let commands = map_intent_to_commands(&state, AppIntent::HeightmapSelectionRequested);

    assert_eq!(commands.len(), 2);
    assert!(matches!(commands[0], AppCommand::DismissHeightmapWarning));
    assert!(matches!(commands[1], AppCommand::RequestHeightmapDialog));
}

#[test]
fn set_default_direction_requested_maps_to_command() {
    let state = AppState::new();

    let commands = map_intent_to_commands(
        &state,
        AppIntent::SetDefaultDirectionRequested {
            direction: ConnectionDirection::Dual,
        },
    );

    assert_eq!(commands.len(), 1);
    assert!(matches!(
        commands[0],
        AppCommand::SetDefaultDirection {
            direction: ConnectionDirection::Dual
        }
    ));
}
