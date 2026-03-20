use super::*;
use indexmap::IndexSet;

fn collect_with_key_event_and_modifiers(
    event: egui::Event,
    raw_modifiers: egui::Modifiers,
    selected: IndexSet<u64>,
) -> Vec<AppIntent> {
    let ctx = egui::Context::default();
    let mut raw_input = egui::RawInput::default();
    raw_input.modifiers = raw_modifiers;
    raw_input.events.push(event);

    let mut events = Vec::new();
    let _ = ctx.run(raw_input, |ctx| {
        egui::CentralPanel::default().show(ctx, |ui| {
            events = collect_keyboard_intents(
                ui,
                &selected,
                EditorTool::Select,
                false,
                false,
                false,
                false,
            );
        });
    });

    events
}

fn collect_with_key_event(event: egui::Event, selected: IndexSet<u64>) -> Vec<AppIntent> {
    collect_with_key_event_full(event, selected, EditorTool::Select, false)
}
fn collect_with_key_event_and_tool(
    event: egui::Event,
    selected: IndexSet<u64>,
    active_tool: EditorTool,
) -> Vec<AppIntent> {
    collect_with_key_event_full(event, selected, active_tool, false)
}

fn collect_with_key_event_full(
    event: egui::Event,
    selected: IndexSet<u64>,
    active_tool: EditorTool,
    route_tool_is_drawing: bool,
) -> Vec<AppIntent> {
    let ctx = egui::Context::default();
    let mut raw_input = egui::RawInput::default();
    raw_input.events.push(event);

    let mut events = Vec::new();
    let _ = ctx.run(raw_input, |ctx| {
        egui::CentralPanel::default().show(ctx, |ui| {
            events = collect_keyboard_intents(
                ui,
                &selected,
                active_tool,
                route_tool_is_drawing,
                false,
                false,
                false,
            );
        });
    });

    events
}

fn collect_with_key_event_text_input_focus(
    event: egui::Event,
    selected: IndexSet<u64>,
    active_tool: EditorTool,
) -> Vec<AppIntent> {
    let ctx = egui::Context::default();
    let mut raw_input = egui::RawInput::default();
    raw_input.events.push(event);

    let mut events = Vec::new();
    let _ = ctx.run(raw_input, |ctx| {
        egui::CentralPanel::default().show(ctx, |ui| {
            let mut text = String::new();
            let response = ui.text_edit_singleline(&mut text);
            ui.memory_mut(|m| m.request_focus(response.id));

            events =
                collect_keyboard_intents(ui, &selected, active_tool, false, false, false, false);
        });
    });

    events
}

#[test]
fn test_delete_with_selection_emits_delete_intent() {
    let mut selected = IndexSet::new();
    selected.insert(10);

    let events = collect_with_key_event(
        egui::Event::Key {
            key: egui::Key::Delete,
            physical_key: None,
            pressed: true,
            repeat: false,
            modifiers: egui::Modifiers::default(),
        },
        selected,
    );

    assert!(events
        .iter()
        .any(|event| matches!(event, AppIntent::DeleteSelectedRequested)));
}

#[test]
fn test_escape_with_selection_clears_selection() {
    let mut selected = IndexSet::new();
    selected.insert(5);

    let events = collect_with_key_event(
        egui::Event::Key {
            key: egui::Key::Escape,
            physical_key: None,
            pressed: true,
            repeat: false,
            modifiers: egui::Modifiers::default(),
        },
        selected,
    );

    assert!(events
        .iter()
        .any(|event| matches!(event, AppIntent::ClearSelectionRequested)));
}

#[test]
fn test_escape_without_selection_switches_to_select_tool() {
    let events = collect_with_key_event_and_tool(
        egui::Event::Key {
            key: egui::Key::Escape,
            physical_key: None,
            pressed: true,
            repeat: false,
            modifiers: egui::Modifiers::default(),
        },
        IndexSet::new(),
        EditorTool::Connect,
    );

    assert!(events.iter().any(|event| matches!(
        event,
        AppIntent::SetEditorToolRequested {
            tool: EditorTool::Select
        }
    )));
}

#[test]
fn test_escape_in_select_tool_without_selection_does_nothing() {
    let events = collect_with_key_event(
        egui::Event::Key {
            key: egui::Key::Escape,
            physical_key: None,
            pressed: true,
            repeat: false,
            modifiers: egui::Modifiers::default(),
        },
        IndexSet::new(),
    );

    assert!(events.is_empty());
}

#[test]
fn test_escape_route_tool_drawing_cancels() {
    let events = collect_with_key_event_full(
        egui::Event::Key {
            key: egui::Key::Escape,
            physical_key: None,
            pressed: true,
            repeat: false,
            modifiers: egui::Modifiers::default(),
        },
        IndexSet::new(),
        EditorTool::Route,
        true, // is_drawing = true
    );

    assert!(events
        .iter()
        .any(|e| matches!(e, AppIntent::RouteToolCancelled)));
}

#[test]
fn test_escape_route_tool_idle_with_selection_clears() {
    let mut selected = IndexSet::new();
    selected.insert(42);

    let events = collect_with_key_event_full(
        egui::Event::Key {
            key: egui::Key::Escape,
            physical_key: None,
            pressed: true,
            repeat: false,
            modifiers: egui::Modifiers::default(),
        },
        selected,
        EditorTool::Route,
        false, // is_drawing = false (idle nach Enter)
    );

    assert!(events
        .iter()
        .any(|e| matches!(e, AppIntent::ClearSelectionRequested)));
    assert!(!events
        .iter()
        .any(|e| matches!(e, AppIntent::RouteToolCancelled)));
}

#[test]
fn test_escape_route_tool_idle_no_selection_switches_to_select() {
    let events = collect_with_key_event_full(
        egui::Event::Key {
            key: egui::Key::Escape,
            physical_key: None,
            pressed: true,
            repeat: false,
            modifiers: egui::Modifiers::default(),
        },
        IndexSet::new(),
        EditorTool::Route,
        false,
    );

    assert!(events.iter().any(|e| matches!(
        e,
        AppIntent::SetEditorToolRequested {
            tool: EditorTool::Select
        }
    )));
}

#[test]
fn test_text_input_focus_blocks_regular_shortcuts() {
    let events = collect_with_key_event_text_input_focus(
        egui::Event::Key {
            key: egui::Key::Num2,
            physical_key: None,
            pressed: true,
            repeat: false,
            modifiers: egui::Modifiers::default(),
        },
        IndexSet::new(),
        EditorTool::Select,
    );

    assert!(events.is_empty());
}

#[test]
fn test_text_input_focus_allows_ctrl_k() {
    let events = collect_with_key_event_text_input_focus(
        egui::Event::Key {
            key: egui::Key::K,
            physical_key: None,
            pressed: true,
            repeat: false,
            modifiers: egui::Modifiers {
                ctrl: true,
                ..egui::Modifiers::default()
            },
        },
        IndexSet::new(),
        EditorTool::Select,
    );

    assert!(events
        .iter()
        .any(|event| matches!(event, AppIntent::CommandPaletteToggled)));
}

#[test]
fn test_text_input_focus_allows_escape_behavior() {
    let mut selected = IndexSet::new();
    selected.insert(12);

    let events = collect_with_key_event_text_input_focus(
        egui::Event::Key {
            key: egui::Key::Escape,
            physical_key: None,
            pressed: true,
            repeat: false,
            modifiers: egui::Modifiers::default(),
        },
        selected,
        EditorTool::Select,
    );

    assert!(events
        .iter()
        .any(|event| matches!(event, AppIntent::ClearSelectionRequested)));
}

#[test]
fn test_t_toggles_tool_palette_without_modifiers() {
    let events = collect_with_key_event(
        egui::Event::Key {
            key: egui::Key::T,
            physical_key: None,
            pressed: true,
            repeat: false,
            modifiers: egui::Modifiers::default(),
        },
        IndexSet::new(),
    );

    assert!(events.iter().any(|event| matches!(
        event,
        AppIntent::ToggleFloatingMenu {
            kind: FloatingMenuKind::Tools
        }
    )));
}

#[test]
fn test_t_with_ctrl_does_not_toggle_tool_palette() {
    let events = collect_with_key_event(
        egui::Event::Key {
            key: egui::Key::T,
            physical_key: None,
            pressed: true,
            repeat: false,
            modifiers: egui::Modifiers {
                ctrl: true,
                ..egui::Modifiers::default()
            },
        },
        IndexSet::new(),
    );

    assert!(!events.iter().any(|event| matches!(
        event,
        AppIntent::ToggleFloatingMenu {
            kind: FloatingMenuKind::Tools
        }
    )));
}

#[test]
fn test_g_toggles_basics_floating_menu() {
    let events = collect_with_key_event(
        egui::Event::Key {
            key: egui::Key::G,
            physical_key: None,
            pressed: true,
            repeat: false,
            modifiers: egui::Modifiers::default(),
        },
        IndexSet::new(),
    );

    assert!(events.iter().any(|event| matches!(
        event,
        AppIntent::ToggleFloatingMenu {
            kind: FloatingMenuKind::Basics
        }
    )));
}

#[test]
fn test_k_without_modifiers_toggles_command_palette() {
    let events = collect_with_key_event(
        egui::Event::Key {
            key: egui::Key::K,
            physical_key: None,
            pressed: true,
            repeat: false,
            modifiers: egui::Modifiers::default(),
        },
        IndexSet::new(),
    );

    assert!(events
        .iter()
        .any(|event| matches!(event, AppIntent::CommandPaletteToggled)));
}

#[test]
fn b_taste_oeffnet_section_tools_floating_menu() {
    let events = collect_with_key_event(
        egui::Event::Key {
            key: egui::Key::B,
            physical_key: None,
            pressed: true,
            repeat: false,
            modifiers: egui::Modifiers::default(),
        },
        IndexSet::new(),
    );

    assert!(events.iter().any(|event| matches!(
        event,
        AppIntent::ToggleFloatingMenu {
            kind: FloatingMenuKind::SectionTools
        }
    )));
}

#[test]
fn r_taste_oeffnet_direction_priority_floating_menu() {
    let events = collect_with_key_event(
        egui::Event::Key {
            key: egui::Key::R,
            physical_key: None,
            pressed: true,
            repeat: false,
            modifiers: egui::Modifiers::default(),
        },
        IndexSet::new(),
    );

    assert!(events.iter().any(|event| matches!(
        event,
        AppIntent::ToggleFloatingMenu {
            kind: FloatingMenuKind::DirectionPriority
        }
    )));
}

#[test]
fn z_taste_oeffnet_zoom_floating_menu() {
    let events = collect_with_key_event(
        egui::Event::Key {
            key: egui::Key::Z,
            physical_key: None,
            pressed: true,
            repeat: false,
            modifiers: egui::Modifiers::default(),
        },
        IndexSet::new(),
    );

    assert!(events.iter().any(|event| matches!(
        event,
        AppIntent::ToggleFloatingMenu {
            kind: FloatingMenuKind::Zoom
        }
    )));
}

#[test]
fn ctrl_z_bleibt_undo() {
    // Auf Linux setzt eframe/winit bei Ctrl-Taste sowohl ctrl als auch command.
    // RawInput.modifiers muss ebenfalls gesetzt sein, da i.modifiers daraus gelesen wird.
    let ctrl_cmd = egui::Modifiers {
        ctrl: true,
        command: true,
        ..egui::Modifiers::default()
    };
    let events = collect_with_key_event_and_modifiers(
        egui::Event::Key {
            key: egui::Key::Z,
            physical_key: None,
            pressed: true,
            repeat: false,
            modifiers: ctrl_cmd,
        },
        ctrl_cmd,
        IndexSet::new(),
    );

    assert!(events
        .iter()
        .any(|event| matches!(event, AppIntent::UndoRequested)));
    assert!(!events.iter().any(|event| matches!(
        event,
        AppIntent::ToggleFloatingMenu {
            kind: FloatingMenuKind::Zoom
        }
    )));
}
