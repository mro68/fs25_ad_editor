//! Command Palette Overlay mit Suchfeld und Tastatur-Navigation.

use crate::app::tools::ToolManager;
use crate::app::{AppIntent, EditorTool};

#[derive(Clone)]
struct PaletteEntry {
    /// Anzeigename des Befehls.
    label: String,
    /// Shortcut-Anzeige (leer wenn keiner vorhanden).
    shortcut: String,
    /// Intent, der bei Auswahl emittiert wird.
    intent: AppIntent,
}

#[derive(Clone, Default)]
struct PaletteState {
    search_text: String,
    selected_index: usize,
    focus_requested: bool,
}

/// Baut den Command-Katalog aus statischen Befehlen und verfuegbaren Route-Tools.
fn build_catalog(tool_manager: Option<&ToolManager>) -> Vec<PaletteEntry> {
    let mut catalog = vec![
        PaletteEntry {
            label: "Datei oeffnen".to_owned(),
            shortcut: "Ctrl+O".to_owned(),
            intent: AppIntent::OpenFileRequested,
        },
        PaletteEntry {
            label: "Speichern".to_owned(),
            shortcut: "Ctrl+S".to_owned(),
            intent: AppIntent::SaveRequested,
        },
        PaletteEntry {
            label: "Rueckgaengig".to_owned(),
            shortcut: "Ctrl+Z".to_owned(),
            intent: AppIntent::UndoRequested,
        },
        PaletteEntry {
            label: "Wiederholen".to_owned(),
            shortcut: "Ctrl+Y".to_owned(),
            intent: AppIntent::RedoRequested,
        },
        PaletteEntry {
            label: "Alles auswaehlen".to_owned(),
            shortcut: "Ctrl+A".to_owned(),
            intent: AppIntent::SelectAllRequested,
        },
        PaletteEntry {
            label: "Auswahl loeschen".to_owned(),
            shortcut: "Del".to_owned(),
            intent: AppIntent::DeleteSelectedRequested,
        },
        PaletteEntry {
            label: "Kopieren".to_owned(),
            shortcut: "Ctrl+C".to_owned(),
            intent: AppIntent::CopySelectionRequested,
        },
        PaletteEntry {
            label: "Einfuegen".to_owned(),
            shortcut: "Ctrl+V".to_owned(),
            intent: AppIntent::PasteStartRequested,
        },
        PaletteEntry {
            label: "Kamera zuruecksetzen".to_owned(),
            shortcut: "Home".to_owned(),
            intent: AppIntent::ResetCameraRequested,
        },
        PaletteEntry {
            label: "Select-Tool".to_owned(),
            shortcut: "1".to_owned(),
            intent: AppIntent::SetEditorToolRequested {
                tool: EditorTool::Select,
            },
        },
        PaletteEntry {
            label: "Connect-Tool".to_owned(),
            shortcut: "2".to_owned(),
            intent: AppIntent::SetEditorToolRequested {
                tool: EditorTool::Connect,
            },
        },
        PaletteEntry {
            label: "Add-Node-Tool".to_owned(),
            shortcut: "3".to_owned(),
            intent: AppIntent::SetEditorToolRequested {
                tool: EditorTool::AddNode,
            },
        },
    ];

    if let Some(tm) = tool_manager {
        for (index, (_, name, _icon)) in tm.tool_entries().iter().enumerate() {
            catalog.push(PaletteEntry {
                label: format!("Route-Tool: {name}"),
                shortcut: String::new(),
                intent: AppIntent::SelectRouteToolRequested { index },
            });
        }
    }

    catalog
}

/// Rendert die Command Palette als zentriertes Overlay-Fenster.
pub fn render_command_palette(
    ctx: &egui::Context,
    show: &mut bool,
    tool_manager: Option<&ToolManager>,
) -> Vec<AppIntent> {
    if !*show {
        return Vec::new();
    }

    let mut intents = Vec::new();
    let mut window_open = *show;
    let state_id = egui::Id::new("command_palette_state");

    let mut state = ctx.data_mut(|d| d.get_temp_mut_or_default::<PaletteState>(state_id).clone());
    let catalog = build_catalog(tool_manager);
    let needle = state.search_text.to_lowercase();
    let filtered_indices: Vec<usize> = catalog
        .iter()
        .enumerate()
        .filter_map(|(idx, entry)| {
            if needle.is_empty() || entry.label.to_lowercase().contains(&needle) {
                Some(idx)
            } else {
                None
            }
        })
        .collect();

    if state.selected_index >= filtered_indices.len() {
        state.selected_index = 0; // layer-ok
    }

    let mut trigger_selected = false;
    let mut window_rect = None;

    let window_output = egui::Window::new("Command Palette")
        .anchor(egui::Align2::CENTER_TOP, egui::vec2(0.0, 80.0))
        .fixed_size(egui::vec2(500.0, 360.0))
        .collapsible(false)
        .resizable(false)
        .open(&mut window_open)
        .show(ctx, |ui| {
            let search_response = ui.add(
                egui::TextEdit::singleline(&mut state.search_text).hint_text("Befehl eingeben..."),
            );
            if !state.focus_requested {
                search_response.request_focus();
                state.focus_requested = true; // layer-ok
            }
            if search_response.changed() {
                state.selected_index = 0; // layer-ok
            }

            ui.separator();

            egui::ScrollArea::vertical()
                .max_height(280.0)
                .show(ui, |ui| {
                    if filtered_indices.is_empty() {
                        ui.label("Keine Treffer");
                        return;
                    }

                    for (visible_idx, catalog_idx) in filtered_indices.iter().copied().enumerate() {
                        let entry = &catalog[catalog_idx];
                        let selected = visible_idx == state.selected_index;
                        let fill = if selected {
                            ui.visuals().selection.bg_fill
                        } else {
                            egui::Color32::TRANSPARENT
                        };

                        let row = egui::Frame::default()
                            .fill(fill)
                            .inner_margin(egui::Margin::symmetric(8, 4))
                            .show(ui, |ui| {
                                ui.horizontal(|ui| {
                                    ui.label(&entry.label);
                                    ui.with_layout(
                                        egui::Layout::right_to_left(egui::Align::Center),
                                        |ui| {
                                            if !entry.shortcut.is_empty() {
                                                ui.label(
                                                    egui::RichText::new(&entry.shortcut)
                                                        .weak()
                                                        .monospace(),
                                                );
                                            }
                                        },
                                    );
                                });
                            })
                            .response
                            .interact(egui::Sense::click());

                        if row.clicked() {
                            state.selected_index = visible_idx; // layer-ok
                            trigger_selected = true;
                        }
                    }
                });
        });

    if let Some(output) = window_output {
        window_rect = Some(output.response.rect);
    }

    let (arrow_up, arrow_down, enter_pressed, escape_pressed, ctrl_k_pressed, pointer_click) = ctx
        .input(|i| {
            let ctrl_k_pressed = i.events.iter().any(|event| {
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

            (
                i.key_pressed(egui::Key::ArrowUp),
                i.key_pressed(egui::Key::ArrowDown),
                i.key_pressed(egui::Key::Enter),
                i.key_pressed(egui::Key::Escape),
                ctrl_k_pressed,
                i.pointer
                    .primary_clicked()
                    .then(|| i.pointer.interact_pos())
                    .flatten(),
            )
        });

    if !filtered_indices.is_empty() {
        if arrow_down {
            let next_index = (state.selected_index + 1) % filtered_indices.len();
            state.selected_index = next_index; // layer-ok
        }
        if arrow_up {
            let selected_index = state.selected_index;
            let wrapped_index = if selected_index == 0 {
                filtered_indices.len() - 1
            } else {
                selected_index - 1
            };
            state.selected_index = wrapped_index; // layer-ok
        }
    }

    if enter_pressed && !filtered_indices.is_empty() {
        trigger_selected = true;
    }

    if trigger_selected {
        if let Some(catalog_idx) = filtered_indices.get(state.selected_index).copied() {
            intents.push(catalog[catalog_idx].intent.clone());
            window_open = false;
        }
    }

    if escape_pressed || ctrl_k_pressed {
        window_open = false;
    }

    if let (Some(rect), Some(pointer_pos)) = (window_rect, pointer_click) {
        if !rect.contains(pointer_pos) {
            window_open = false;
        }
    }

    if window_open {
        ctx.data_mut(|d| {
            d.insert_temp(state_id, state);
        });
    } else {
        ctx.data_mut(|d| {
            d.remove::<PaletteState>(state_id);
        });
    }

    *show = window_open;
    intents
}
