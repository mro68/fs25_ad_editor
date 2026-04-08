//! Command Palette Overlay mit Suchfeld und Tastatur-Navigation.

use crate::app::tools::route_tool_label_key;
use crate::app::{AppIntent, EditorTool};
use crate::shared::{t, I18nKey, Language};
use crate::ui::common::{
    host_route_tool_disabled_reason_key, host_route_tool_entries_for, host_route_tool_to_engine,
};
use fs25_auto_drive_host_bridge::{HostChromeSnapshot, HostRouteToolGroup, HostRouteToolSurface};

#[derive(Clone)]
struct PaletteEntry {
    /// Anzeigename des Befehls.
    label: String,
    /// Shortcut-Anzeige (leer wenn keiner vorhanden).
    shortcut: String,
    /// Intent, der bei Auswahl emittiert wird.
    intent: AppIntent,
    /// `true` wenn der Eintrag aktuell ausgefuehrt werden darf.
    enabled: bool,
    /// Optionaler Disabled-Grund fuer sichtbare, aber deaktivierte Eintraege.
    disabled_reason: Option<String>,
}

#[derive(Clone, Default)]
struct PaletteState {
    search_text: String,
    selected_index: usize,
    focus_requested: bool,
}

fn palette_entry(label: String, shortcut: &str, intent: AppIntent) -> PaletteEntry {
    PaletteEntry {
        label,
        shortcut: shortcut.to_owned(),
        intent,
        enabled: true,
        disabled_reason: None,
    }
}

fn selected_catalog_intent(
    catalog: &[PaletteEntry],
    filtered_indices: &[usize],
    selected_index: usize,
) -> Option<AppIntent> {
    let catalog_idx = filtered_indices.get(selected_index).copied()?;
    let entry = &catalog[catalog_idx];
    entry.enabled.then(|| entry.intent.clone())
}

/// Baut den Command-Katalog aus statischen Befehlen und allen katalogsichtbaren Route-Tools.
fn build_catalog(lang: Language, host_chrome_snapshot: &HostChromeSnapshot) -> Vec<PaletteEntry> {
    let mut catalog = vec![
        palette_entry(
            t(lang, I18nKey::PaletteOpenFile).to_owned(),
            "Ctrl+O",
            AppIntent::OpenFileRequested,
        ),
        palette_entry(
            t(lang, I18nKey::PaletteSave).to_owned(),
            "Ctrl+S",
            AppIntent::SaveRequested,
        ),
        palette_entry(
            t(lang, I18nKey::PaletteUndo).to_owned(),
            "Ctrl+Z",
            AppIntent::UndoRequested,
        ),
        palette_entry(
            t(lang, I18nKey::PaletteRedo).to_owned(),
            "Ctrl+Y",
            AppIntent::RedoRequested,
        ),
        palette_entry(
            t(lang, I18nKey::PaletteSelectAll).to_owned(),
            "Ctrl+A",
            AppIntent::SelectAllRequested,
        ),
        palette_entry(
            t(lang, I18nKey::PaletteDeleteSelected).to_owned(),
            "Del",
            AppIntent::DeleteSelectedRequested,
        ),
        palette_entry(
            t(lang, I18nKey::PaletteCopy).to_owned(),
            "Ctrl+C",
            AppIntent::CopySelectionRequested,
        ),
        palette_entry(
            t(lang, I18nKey::PalettePaste).to_owned(),
            "Ctrl+V",
            AppIntent::PasteStartRequested,
        ),
        palette_entry(
            t(lang, I18nKey::PaletteResetCamera).to_owned(),
            "Home",
            AppIntent::ResetCameraRequested,
        ),
        palette_entry(
            t(lang, I18nKey::PaletteToolSelect).to_owned(),
            "T",
            AppIntent::SetEditorToolRequested {
                tool: EditorTool::Select,
            },
        ),
        palette_entry(
            t(lang, I18nKey::PaletteToolConnect).to_owned(),
            "T",
            AppIntent::SetEditorToolRequested {
                tool: EditorTool::Connect,
            },
        ),
        palette_entry(
            t(lang, I18nKey::PaletteToolAddNode).to_owned(),
            "T",
            AppIntent::SetEditorToolRequested {
                tool: EditorTool::AddNode,
            },
        ),
    ];

    let prefix = t(lang, I18nKey::PaletteRouteToolPrefix);
    for entry in [
        HostRouteToolGroup::Basics,
        HostRouteToolGroup::Section,
        HostRouteToolGroup::Analysis,
    ]
    .into_iter()
    .flat_map(|group| {
        host_route_tool_entries_for(
            host_chrome_snapshot,
            HostRouteToolSurface::CommandPalette,
            group,
        )
    }) {
        let engine_tool = host_route_tool_to_engine(entry.tool);
        catalog.push(PaletteEntry {
            label: format!("{prefix} {}", t(lang, route_tool_label_key(engine_tool))),
            shortcut: String::new(),
            intent: AppIntent::SelectRouteToolRequested {
                tool_id: engine_tool,
            },
            enabled: entry.enabled,
            disabled_reason: entry
                .disabled_reason
                .map(|reason| t(lang, host_route_tool_disabled_reason_key(reason)).to_owned()),
        });
    }

    catalog
}

/// Rendert die Command Palette als zentriertes Overlay-Fenster.
///
/// Deaktivierte Route-Tools bleiben im Katalog sichtbar und tragen ihren
/// Disabled-Grund, koennen aber weder per Klick noch per Enter ausgefuehrt
/// werden.
pub fn render_command_palette(
    ctx: &egui::Context,
    show: &mut bool,
    host_chrome_snapshot: &HostChromeSnapshot,
) -> Vec<AppIntent> {
    if !*show {
        return Vec::new();
    }

    let mut intents = Vec::new();
    let mut window_open = *show;
    let state_id = egui::Id::new("command_palette_state");
    let lang = host_chrome_snapshot.options.language;

    let mut palette_state =
        ctx.data_mut(|d| d.get_temp_mut_or_default::<PaletteState>(state_id).clone());
    let catalog = build_catalog(lang, host_chrome_snapshot);
    let needle = palette_state.search_text.to_lowercase();
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

    if palette_state.selected_index >= filtered_indices.len() {
        palette_state.selected_index = 0; // layer-ok
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
                egui::TextEdit::singleline(&mut palette_state.search_text)
                    .hint_text(t(lang, I18nKey::PaletteSearchHint)),
            );
            if !palette_state.focus_requested {
                search_response.request_focus();
                palette_state.focus_requested = true; // layer-ok
            }
            if search_response.changed() {
                palette_state.selected_index = 0; // layer-ok
            }

            ui.separator();

            egui::ScrollArea::vertical()
                .max_height(280.0)
                .show(ui, |ui| {
                    if filtered_indices.is_empty() {
                        ui.label(t(lang, I18nKey::PaletteNoResults));
                        return;
                    }

                    for (visible_idx, catalog_idx) in filtered_indices.iter().copied().enumerate() {
                        let entry = &catalog[catalog_idx];
                        let selected = visible_idx == palette_state.selected_index;
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
                                    ui.label(if entry.enabled {
                                        egui::RichText::new(&entry.label)
                                    } else {
                                        egui::RichText::new(&entry.label).weak()
                                    });
                                    ui.with_layout(
                                        egui::Layout::right_to_left(egui::Align::Center),
                                        |ui| {
                                            if let Some(reason) = &entry.disabled_reason {
                                                ui.label(egui::RichText::new(reason).weak());
                                            } else if !entry.shortcut.is_empty() {
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
                            .response;

                        let row = if entry.enabled {
                            row.interact(egui::Sense::click())
                        } else if let Some(reason) = &entry.disabled_reason {
                            row.on_hover_text(reason)
                        } else {
                            row
                        };

                        if row.clicked() {
                            palette_state.selected_index = visible_idx; // layer-ok
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
            let next_index = (palette_state.selected_index + 1) % filtered_indices.len();
            palette_state.selected_index = next_index; // layer-ok
        }
        if arrow_up {
            let selected_index = palette_state.selected_index;
            let wrapped_index = if selected_index == 0 {
                filtered_indices.len() - 1
            } else {
                selected_index - 1
            };
            palette_state.selected_index = wrapped_index; // layer-ok
        }
    }

    if enter_pressed && !filtered_indices.is_empty() {
        trigger_selected = true;
    }

    if trigger_selected
        && let Some(intent) =
            selected_catalog_intent(&catalog, &filtered_indices, palette_state.selected_index)
    {
        intents.push(intent);
        window_open = false;
    }

    if escape_pressed || ctrl_k_pressed {
        window_open = false;
    }

    if let (Some(rect), Some(pointer_pos)) = (window_rect, pointer_click)
        && !rect.contains(pointer_pos)
    {
        window_open = false;
    }

    if window_open {
        ctx.data_mut(|d| {
            d.insert_temp(state_id, palette_state);
        });
    } else {
        ctx.data_mut(|d| {
            d.remove::<PaletteState>(state_id);
        });
    }

    *show = window_open;
    intents
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::app::tool_contract::RouteToolId;
    use crate::app::AppState;

    fn route_tool_entry(catalog: &[PaletteEntry], tool_id: RouteToolId) -> (usize, &PaletteEntry) {
        catalog
            .iter()
            .enumerate()
            .find(|(_, entry)| match &entry.intent {
                AppIntent::SelectRouteToolRequested {
                    tool_id: entry_tool_id,
                } => *entry_tool_id == tool_id,
                _ => false,
            })
            .expect("Route-Tool-Eintrag muss im Palette-Katalog vorhanden sein")
    }

    #[test]
    fn command_palette_zeigt_route_tools_trotz_disabled_state() {
        let state = AppState::new();
        let chrome = fs25_auto_drive_host_bridge::build_host_chrome_snapshot(&state);
        let catalog = build_catalog(state.options.language, &chrome);

        for tool_id in RouteToolId::ALL {
            route_tool_entry(&catalog, tool_id);
        }

        for tool_id in [
            RouteToolId::Straight,
            RouteToolId::CurveQuad,
            RouteToolId::CurveCubic,
            RouteToolId::Spline,
            RouteToolId::SmoothCurve,
            RouteToolId::Parking,
        ] {
            let (_, entry) = route_tool_entry(&catalog, tool_id);
            assert!(entry.enabled, "{:?} sollte aktivierbar bleiben", tool_id);
            assert!(
                entry.disabled_reason.is_none(),
                "{:?} sollte keinen Disabled-Grund tragen",
                tool_id
            );
        }

        for tool_id in [
            RouteToolId::Bypass,
            RouteToolId::FieldBoundary,
            RouteToolId::FieldPath,
            RouteToolId::RouteOffset,
            RouteToolId::ColorPath,
        ] {
            let (_, entry) = route_tool_entry(&catalog, tool_id);
            assert!(
                !entry.enabled,
                "{:?} muss sichtbar, aber disabled sein",
                tool_id
            );
            assert!(
                entry.disabled_reason.is_some(),
                "{:?} muss seinen Disabled-Grund in der Palette behalten",
                tool_id
            );
        }
    }

    #[test]
    fn command_palette_blockiert_enter_auf_disabled_route_tools() {
        let state = AppState::new();
        let chrome = fs25_auto_drive_host_bridge::build_host_chrome_snapshot(&state);
        let catalog = build_catalog(state.options.language, &chrome);
        let (disabled_idx, _) = route_tool_entry(&catalog, RouteToolId::Bypass);
        let (enabled_idx, _) = route_tool_entry(&catalog, RouteToolId::Straight);

        assert!(selected_catalog_intent(&catalog, &[disabled_idx], 0).is_none());
        assert!(matches!(
            selected_catalog_intent(&catalog, &[enabled_idx], 0),
            Some(AppIntent::SelectRouteToolRequested {
                tool_id: RouteToolId::Straight,
            })
        ));
    }
}
