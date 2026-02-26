//! Keyboard-Shortcuts für den Viewport.
//!
//! Verarbeitet globale Tastenkombinationen und mappt sie auf `AppIntent`s.

use crate::app::{AppIntent, ConnectionDirection, ConnectionPriority, EditorTool};
use std::collections::HashSet;

/// Verarbeitet Keyboard-Shortcuts und gibt AppIntents zurück.
pub(super) fn collect_keyboard_intents(
    ui: &egui::Ui,
    selected_node_ids: &HashSet<u64>,
    active_tool: EditorTool,
    route_tool_is_drawing: bool,
    distanzen_active: bool,
) -> Vec<AppIntent> {
    let mut events = Vec::new();

    // Undo / Redo (Cmd/Ctrl + Z / Y, Shift+Cmd+Z)
    let (modifiers, key_z_pressed, key_y_pressed) = ui.input(|i| {
        (
            i.modifiers,
            i.key_pressed(egui::Key::Z),
            i.key_pressed(egui::Key::Y),
        )
    });

    if modifiers.command && key_z_pressed && !modifiers.shift {
        events.push(AppIntent::UndoRequested);
    }

    if modifiers.command && (key_y_pressed || (modifiers.shift && key_z_pressed)) {
        events.push(AppIntent::RedoRequested);
    }

    // Ctrl+O (Öffnen), Ctrl+S (Speichern), Ctrl+A (Alle selektieren), Escape (Selektion aufheben)
    let (key_o_pressed, key_s_pressed, key_a_pressed, key_escape_pressed) = ui.input(|i| {
        (
            i.key_pressed(egui::Key::O),
            i.key_pressed(egui::Key::S),
            i.key_pressed(egui::Key::A),
            i.key_pressed(egui::Key::Escape),
        )
    });

    if modifiers.command && key_o_pressed {
        events.push(AppIntent::OpenFileRequested);
    }

    if modifiers.command && key_s_pressed && !modifiers.shift {
        events.push(AppIntent::SaveRequested);
    }

    if modifiers.command && key_a_pressed {
        events.push(AppIntent::SelectAllRequested);
    }

    if key_escape_pressed {
        if distanzen_active {
            // Distanzen-Vorschau aktiv → Esc wird im Properties-Panel behandelt
        } else if active_tool == EditorTool::Route && route_tool_is_drawing {
            // Tool zeichnet gerade → Eingabe abbrechen
            events.push(AppIntent::RouteToolCancelled);
        } else if !selected_node_ids.is_empty() {
            // Selektion aufheben (gilt für alle Tools inkl. Route im Leerlauf)
            events.push(AppIntent::ClearSelectionRequested);
        } else if active_tool != EditorTool::Select {
            // Zurück zum Select-Tool
            events.push(AppIntent::SetEditorToolRequested {
                tool: EditorTool::Select,
            });
        }
    }

    // Delete, Tool-Wechsel, Connect/Disconnect, Enter (Route-Tool)
    let (
        key_del_pressed,
        key_1_pressed,
        key_2_pressed,
        key_3_pressed,
        key_c_pressed,
        key_x_pressed,
        key_enter_pressed,
        key_up_pressed,
        key_down_pressed,
        key_left_pressed,
        key_right_pressed,
    ) = ui.input(|i| {
        (
            i.key_pressed(egui::Key::Delete) || i.key_pressed(egui::Key::Backspace),
            i.key_pressed(egui::Key::Num1),
            i.key_pressed(egui::Key::Num2),
            i.key_pressed(egui::Key::Num3),
            i.key_pressed(egui::Key::C),
            i.key_pressed(egui::Key::X),
            i.key_pressed(egui::Key::Enter),
            i.key_pressed(egui::Key::ArrowUp),
            i.key_pressed(egui::Key::ArrowDown),
            i.key_pressed(egui::Key::ArrowLeft),
            i.key_pressed(egui::Key::ArrowRight),
        )
    });

    if key_del_pressed && !selected_node_ids.is_empty() {
        events.push(AppIntent::DeleteSelectedRequested);
    }

    // Enter = Route-Tool ausführen
    if key_enter_pressed && active_tool == EditorTool::Route {
        events.push(AppIntent::RouteToolExecuteRequested);
    }

    if key_1_pressed && !modifiers.command {
        events.push(AppIntent::SetEditorToolRequested {
            tool: EditorTool::Select,
        });
    }
    if key_2_pressed && !modifiers.command {
        events.push(AppIntent::SetEditorToolRequested {
            tool: EditorTool::Connect,
        });
    }
    if key_3_pressed && !modifiers.command {
        events.push(AppIntent::SetEditorToolRequested {
            tool: EditorTool::AddNode,
        });
    }

    // C = Verbinden (bei genau 2 selektierten Nodes)
    // IDs sortiert für deterministische Reihenfolge (HashSet-Iteration ist nicht-deterministisch)
    if key_c_pressed && !modifiers.command && selected_node_ids.len() == 2 {
        let mut ids: Vec<u64> = selected_node_ids.iter().copied().collect();
        ids.sort_unstable();
        events.push(AppIntent::AddConnectionRequested {
            from_id: ids[0],
            to_id: ids[1],
            direction: ConnectionDirection::Regular,
            priority: ConnectionPriority::Regular,
        });
    }

    // X = Trennen (bei genau 2 selektierten Nodes)
    if key_x_pressed && !modifiers.command && selected_node_ids.len() == 2 {
        let mut ids: Vec<u64> = selected_node_ids.iter().copied().collect();
        ids.sort_unstable();
        events.push(AppIntent::RemoveConnectionBetweenRequested {
            node_a: ids[0],
            node_b: ids[1],
        });
    }

    // Arrow Keys für Route-Tool-Konfiguration (nur wenn Route-Tool aktiv und zeichnet)
    if active_tool == EditorTool::Route
        && route_tool_is_drawing
        && !modifiers.command
        && !modifiers.shift
        && !modifiers.alt
    {
        if key_up_pressed {
            events.push(AppIntent::IncreaseRouteToolNodeCount);
        }
        if key_down_pressed {
            events.push(AppIntent::DecreaseRouteToolNodeCount);
        }
        if key_right_pressed {
            events.push(AppIntent::IncreaseRouteToolSegmentLength);
        }
        if key_left_pressed {
            events.push(AppIntent::DecreaseRouteToolSegmentLength);
        }
    }

    events
}

#[cfg(test)]
mod tests {
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
}
