//! Keyboard-Shortcuts fuer den Viewport.
//!
//! Verarbeitet globale Tastenkombinationen und mappt sie auf `AppIntent`s.

use crate::app::tools::RouteToolGroup;
use crate::app::{
    AppIntent, ConnectionDirection, ConnectionPriority, EditorTool, FloatingMenuKind,
};
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

    let (
        key_a_no_mod,
        key_g_no_mod,
        key_t_no_mod,
        key_b_no_mod,
        key_r_no_mod,
        key_z_no_mod,
        key_k_no_mod,
        key_k_ctrl_or_cmd,
    ) = ui.input(|i| {
        let mut key_a_no_mod = false;
        let mut key_g_no_mod = false;
        let mut key_t_no_mod = false;
        let mut key_b_no_mod = false;
        let mut key_r_no_mod = false;
        let mut key_z_no_mod = false;
        let mut key_k_no_mod = false;
        let mut key_k_ctrl_or_cmd = false;

        for event in &i.events {
            if let egui::Event::Key {
                key,
                pressed: true,
                modifiers,
                ..
            } = event
            {
                let no_mod =
                    !modifiers.command && !modifiers.ctrl && !modifiers.shift && !modifiers.alt;
                match key {
                    egui::Key::A if no_mod => key_a_no_mod = true,
                    egui::Key::G if no_mod => key_g_no_mod = true,
                    egui::Key::T if no_mod => key_t_no_mod = true,
                    egui::Key::B if no_mod => key_b_no_mod = true,
                    egui::Key::R if no_mod => key_r_no_mod = true,
                    egui::Key::Z if no_mod => key_z_no_mod = true,
                    egui::Key::K if no_mod => key_k_no_mod = true,
                    egui::Key::K if modifiers.command || modifiers.ctrl => {
                        key_k_ctrl_or_cmd = true;
                    }
                    _ => {}
                }
            }
        }

        (
            key_a_no_mod,
            key_g_no_mod,
            key_t_no_mod,
            key_b_no_mod,
            key_r_no_mod,
            key_z_no_mod,
            key_k_no_mod,
            key_k_ctrl_or_cmd,
        )
    });

    if key_a_no_mod {
        events.push(AppIntent::ToggleFloatingMenu {
            kind: FloatingMenuKind::RouteTools(RouteToolGroup::Analysis),
        });
    }

    if key_g_no_mod {
        events.push(AppIntent::ToggleFloatingMenu {
            kind: FloatingMenuKind::RouteTools(RouteToolGroup::Basics),
        });
    }

    if key_t_no_mod {
        events.push(AppIntent::ToggleFloatingMenu {
            kind: FloatingMenuKind::Tools,
        });
    }

    if key_b_no_mod {
        events.push(AppIntent::ToggleFloatingMenu {
            kind: FloatingMenuKind::RouteTools(RouteToolGroup::Section),
        });
    }

    if key_r_no_mod {
        events.push(AppIntent::ToggleFloatingMenu {
            kind: FloatingMenuKind::DirectionPriority,
        });
    }

    if key_z_no_mod {
        events.push(AppIntent::ToggleFloatingMenu {
            kind: FloatingMenuKind::Zoom,
        });
    }

    if key_k_no_mod {
        events.push(AppIntent::CommandPaletteToggled);
    }

    if key_k_ctrl_or_cmd {
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
    } else if !modifiers.command && !modifiers.shift && !modifiers.alt {
        // Pfeiltasten fuer Kamera-Pan (außer im Route-Tool beim Zeichnen)
        const PAN_STEP: f32 = 100.0;
        if key_up_pressed {
            events.push(AppIntent::CameraPan {
                delta: glam::Vec2::new(0.0, -PAN_STEP),
            });
        }
        if key_down_pressed {
            events.push(AppIntent::CameraPan {
                delta: glam::Vec2::new(0.0, PAN_STEP),
            });
        }
        if key_left_pressed {
            events.push(AppIntent::CameraPan {
                delta: glam::Vec2::new(-PAN_STEP, 0.0),
            });
        }
        if key_right_pressed {
            events.push(AppIntent::CameraPan {
                delta: glam::Vec2::new(PAN_STEP, 0.0),
            });
        }
    }

    // Plus/Minus fuer stufenweises Zoomen (keine Modifier)
    let (key_plus_no_mod, key_minus_no_mod) = ui.input(|i| {
        let mut plus = false;
        let mut minus = false;
        for event in &i.events {
            if let egui::Event::Key {
                key,
                pressed: true,
                modifiers,
                ..
            } = event
            {
                let no_mod =
                    !modifiers.command && !modifiers.ctrl && !modifiers.shift && !modifiers.alt;
                match key {
                    egui::Key::Plus if no_mod => plus = true,
                    egui::Key::Minus if no_mod => minus = true,
                    _ => {}
                }
            }
        }
        (plus, minus)
    });

    if key_plus_no_mod {
        events.push(AppIntent::ZoomInRequested);
    }
    if key_minus_no_mod {
        events.push(AppIntent::ZoomOutRequested);
    }

    events
}

#[cfg(test)]
mod tests;
