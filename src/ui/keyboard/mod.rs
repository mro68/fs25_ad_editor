//! Keyboard-Shortcuts fuer den Viewport.
//!
//! Verarbeitet globale Tastenkombinationen und mappt sie auf `AppIntent`s.

use crate::app::{AppIntent, ConnectionDirection, ConnectionPriority, EditorTool};
use indexmap::IndexSet;

fn collect_escape_intents(
    selected_node_ids: &IndexSet<u64>,
    active_tool: EditorTool,
    route_tool_is_drawing: bool,
    distanzen_active: bool,
) -> Vec<AppIntent> {
    if distanzen_active {
        // Distanzen-Vorschau aktiv -> Esc wird im Properties-Panel behandelt
        return vec![];
    }

    if active_tool == EditorTool::Route && route_tool_is_drawing {
        // Tool zeichnet gerade -> Eingabe abbrechen
        return vec![AppIntent::RouteToolCancelled];
    }

    if !selected_node_ids.is_empty() {
        // Selektion aufheben (gilt fuer alle Tools inkl. Route im Leerlauf)
        return vec![AppIntent::ClearSelectionRequested];
    }

    if active_tool != EditorTool::Select {
        // Zurueck zum Select-Tool
        return vec![AppIntent::SetEditorToolRequested {
            tool: EditorTool::Select,
        }];
    }

    vec![]
}

/// Verarbeitet Keyboard-Shortcuts und gibt AppIntents zurueck.
///
/// `clipboard_has_data`: true wenn die Zwischenablage Nodes enthaelt (fuer Ctrl+V).
pub(super) fn collect_keyboard_intents(
    ui: &egui::Ui,
    selected_node_ids: &IndexSet<u64>,
    active_tool: EditorTool,
    route_tool_is_drawing: bool,
    distanzen_active: bool,
    clipboard_has_data: bool,
    command_palette_open: bool,
) -> Vec<AppIntent> {
    // Solange die Command Palette offen ist, verarbeitet ausschliesslich die Palette Tastatureingaben.
    if command_palette_open {
        return vec![];
    }

    // Guard: Shortcuts unterdruecken wenn ein Widget Keyboard-Input haben moechte.
    // Ctrl+K und Escape bleiben erlaubt.
    if ui.ctx().wants_keyboard_input() {
        let (ctrl_or_cmd_k_pressed, key_escape_pressed) = ui.input(|i| {
            let ctrl_or_cmd_k_pressed = i.events.iter().any(|event| {
                matches!(
                    event,
                    egui::Event::Key {
                        key: egui::Key::K,
                        pressed: true,
                        modifiers,
                        ..
                    } if modifiers.command || modifiers.ctrl
                )
            });

            (ctrl_or_cmd_k_pressed, i.key_pressed(egui::Key::Escape))
        });

        if ctrl_or_cmd_k_pressed {
            return vec![AppIntent::CommandPaletteToggled];
        }

        if key_escape_pressed {
            return collect_escape_intents(
                selected_node_ids,
                active_tool,
                route_tool_is_drawing,
                distanzen_active,
            );
        }

        return vec![];
    }

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

    // Ctrl+O (Oeffnen), Ctrl+S (Speichern), Ctrl+A (Alle selektieren), Escape (Selektion aufheben)
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
        events.extend(collect_escape_intents(
            selected_node_ids,
            active_tool,
            route_tool_is_drawing,
            distanzen_active,
        ));
    }

    // Delete, Tool-Wechsel, Connect/Disconnect, Enter (Route-Tool)
    let (
        key_del_pressed,
        key_1_pressed,
        key_2_pressed,
        key_3_pressed,
        key_k_pressed,
        key_c_pressed,
        key_v_pressed,
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
            i.key_pressed(egui::Key::K),
            i.key_pressed(egui::Key::C),
            i.key_pressed(egui::Key::V),
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

    // Enter = Route-Tool ausfuehren
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

    // T-Taste: Tool-Palette toggling
    let should_toggle_tool_palette = ui.input(|i| {
        i.events.iter().any(|event| {
            matches!(
                event,
                egui::Event::Key {
                    key: egui::Key::T,
                    pressed: true,
                    modifiers,
                    ..
                } if {
                    let command = modifiers.command || modifiers.ctrl;
                    !command && !modifiers.shift && !modifiers.alt
                }
            )
        })
    });
    if should_toggle_tool_palette {
        events.push(AppIntent::ToggleToolPalette);
    }

    if (modifiers.command || modifiers.ctrl) && key_k_pressed {
        events.push(AppIntent::CommandPaletteToggled);
    }

    // C = Verbinden (bei genau 2 selektierten Nodes)
    // Reihenfolge aus IndexSet: erster selektierter Node = from, zweiter = to
    if key_c_pressed && !modifiers.command && selected_node_ids.len() == 2 {
        let ids: Vec<u64> = selected_node_ids.iter().copied().collect();
        events.push(AppIntent::AddConnectionRequested {
            from_id: ids[0],
            to_id: ids[1],
            direction: ConnectionDirection::Regular,
            priority: ConnectionPriority::Regular,
        });
    }

    // Ctrl+C: Selektion kopieren
    if modifiers.command && key_c_pressed && !selected_node_ids.is_empty() {
        events.push(AppIntent::CopySelectionRequested);
    }

    // Ctrl+V: Paste-Vorschau starten
    if modifiers.command && key_v_pressed && clipboard_has_data {
        events.push(AppIntent::PasteStartRequested);
    }

    // X = Trennen (bei genau 2 selektierten Nodes)
    if key_x_pressed && !modifiers.command && selected_node_ids.len() == 2 {
        let ids: Vec<u64> = selected_node_ids.iter().copied().collect();
        events.push(AppIntent::RemoveConnectionBetweenRequested {
            node_a: ids[0],
            node_b: ids[1],
        });
    }

    // Arrow Keys fuer Route-Tool-Konfiguration (nur wenn Route-Tool aktiv und zeichnet)
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
mod tests;
