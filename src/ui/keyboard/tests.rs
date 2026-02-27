use super::*;

fn collect_with_key_event(event: egui::Event, selected: HashSet<u64>) -> Vec<AppIntent> {
    collect_with_key_event_full(event, selected, EditorTool::Select, false)
}

fn collect_with_key_event_and_tool(
    event: egui::Event,
    selected: HashSet<u64>,
    active_tool: EditorTool,
) -> Vec<AppIntent> {
    collect_with_key_event_full(event, selected, active_tool, false)
}

fn collect_with_key_event_full(
    event: egui::Event,
    selected: HashSet<u64>,
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
            );
        });
    });

    events
}

#[test]
fn test_num2_emits_connect_tool_intent() {
    let events = collect_with_key_event(
        egui::Event::Key {
            key: egui::Key::Num2,
            physical_key: None,
            pressed: true,
            repeat: false,
            modifiers: egui::Modifiers::default(),
        },
        HashSet::new(),
    );

    assert!(events.iter().any(|event| matches!(
        event,
        AppIntent::SetEditorToolRequested {
            tool: EditorTool::Connect
        }
    )));
}

#[test]
fn test_delete_with_selection_emits_delete_intent() {
    let mut selected = HashSet::new();
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
    let mut selected = HashSet::new();
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
        HashSet::new(),
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
        HashSet::new(),
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
        HashSet::new(),
        EditorTool::Route,
        true, // is_drawing = true
    );

    assert!(events
        .iter()
        .any(|e| matches!(e, AppIntent::RouteToolCancelled)));
}

#[test]
fn test_escape_route_tool_idle_with_selection_clears() {
    let mut selected = HashSet::new();
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
        HashSet::new(),
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
