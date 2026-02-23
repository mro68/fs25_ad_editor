//! Datei-Dialoge und modale Fenster.

use crate::app::{AppIntent, RoadMap, UiState};
use std::collections::BTreeSet;

fn path_to_ui_string(path: &std::path::Path) -> String {
    path.to_string_lossy().into_owned()
}

/// Formatiert eine Dateigröße menschenlesbar (KB, MB, GB).
fn format_file_size(bytes: u64) -> String {
    const KB: u64 = 1024;
    const MB: u64 = 1024 * 1024;
    const GB: u64 = 1024 * 1024 * 1024;
    if bytes >= GB {
        format!("{:.1} GB", bytes as f64 / GB as f64)
    } else if bytes >= MB {
        format!("{:.1} MB", bytes as f64 / MB as f64)
    } else if bytes >= KB {
        format!("{:.0} KB", bytes as f64 / KB as f64)
    } else {
        format!("{} B", bytes)
    }
}

/// Verarbeitet ausstehende Datei-Dialoge und gibt AppIntents zurück.
pub fn handle_file_dialogs(ui_state: &mut UiState) -> Vec<AppIntent> {
    let mut events = Vec::new();

    // Open-Datei-Dialog
    if ui_state.show_file_dialog {
        ui_state.show_file_dialog = false;

        if let Some(path) = rfd::FileDialog::new()
            .add_filter("AutoDrive Config", &["xml"])
            .pick_file()
        {
            events.push(AppIntent::FileSelected {
                path: path_to_ui_string(&path),
            });
        }
    }

    // Save-Datei-Dialog
    if ui_state.show_save_file_dialog {
        ui_state.show_save_file_dialog = false;

        let default_name = ui_state
            .current_file_path
            .as_ref()
            .and_then(|p| std::path::Path::new(p).file_name())
            .and_then(|n| n.to_str())
            .unwrap_or("AutoDrive_config.xml");

        if let Some(path) = rfd::FileDialog::new()
            .add_filter("AutoDrive Config", &["xml"])
            .set_file_name(default_name)
            .save_file()
        {
            events.push(AppIntent::SaveFilePathSelected {
                path: path_to_ui_string(&path),
            });
        }
    }

    // Heightmap-Auswahl-Dialog
    if ui_state.show_heightmap_dialog {
        ui_state.show_heightmap_dialog = false;

        if let Some(path) = rfd::FileDialog::new()
            .add_filter("Heightmap Image", &["png", "jpg", "jpeg"])
            .pick_file()
        {
            events.push(AppIntent::HeightmapSelected {
                path: path_to_ui_string(&path),
            });
        }
    }

    // Background-Map-Auswahl-Dialog
    if ui_state.show_background_map_dialog {
        ui_state.show_background_map_dialog = false;

        if let Some(path) = rfd::FileDialog::new()
            .add_filter("Map Background", &["png", "jpg", "jpeg", "dds", "zip"])
            .pick_file()
        {
            let path_str = path_to_ui_string(&path);
            if path_str.to_lowercase().ends_with(".zip") {
                // ZIP-Datei: Browser-Dialog öffnen
                events.push(AppIntent::ZipBackgroundBrowseRequested { path: path_str });
            } else {
                // Direktes Bild: wie bisher laden
                events.push(AppIntent::BackgroundMapSelected {
                    path: path_str,
                    crop_size: None,
                });
            }
        }
    }

    events
}

/// Zeigt die Heightmap-Warnung als modales Fenster.
pub fn show_heightmap_warning(ctx: &egui::Context, show: bool) -> Vec<AppIntent> {
    let mut events = Vec::new();

    if !show {
        return events;
    }

    egui::Window::new("Heightmap nicht vorhanden")
        .collapsible(false)
        .resizable(false)
        .anchor(egui::Align2::CENTER_CENTER, [0.0, 0.0])
        .show(ctx, |ui| {
            ui.vertical_centered(|ui| {
                ui.add_space(10.0);
                ui.label("Es wurde keine Heightmap ausgewählt.");
                ui.label("Alle Y-Koordinaten werden auf 0 gesetzt (flache Map).");
                ui.add_space(10.0);

                ui.horizontal(|ui| {
                    if ui.button("Heightmap auswählen").clicked() {
                        events.push(AppIntent::HeightmapSelectionRequested);
                    }

                    if ui.button("Ohne Heightmap fortfahren").clicked() {
                        events.push(AppIntent::HeightmapWarningConfirmed);
                    }

                    if ui.button("Abbrechen").clicked() {
                        events.push(AppIntent::HeightmapWarningCancelled);
                    }
                });
            });
        });

    events
}

/// Zeigt den Marker-Bearbeiten-Dialog als modales Fenster.
pub fn show_marker_dialog(
    ctx: &egui::Context,
    ui_state: &mut UiState,
    road_map: Option<&RoadMap>,
) -> Vec<AppIntent> {
    let mut events = Vec::new();

    if !ui_state.marker_dialog.visible {
        return events;
    }

    let node_id = match ui_state.marker_dialog.node_id {
        Some(id) => id,
        None => return events,
    };

    let title = if ui_state.marker_dialog.is_new {
        "Marker erstellen"
    } else {
        "Marker ändern"
    };

    // Bestehende Gruppen aus der RoadMap ableiten
    let existing_groups: BTreeSet<String> = road_map
        .map(|rm| rm.map_markers.iter().map(|m| m.group.clone()).collect())
        .unwrap_or_default();

    let mut confirmed = false;
    let mut cancelled = false;

    egui::Window::new(title)
        .collapsible(false)
        .resizable(false)
        .anchor(egui::Align2::CENTER_CENTER, [0.0, 0.0])
        .show(ctx, |ui| {
            ui.set_min_width(280.0);

            ui.label(format!("Node: {}", node_id));
            ui.add_space(6.0);

            // Name
            ui.horizontal(|ui| {
                ui.label("Name:");
                ui.text_edit_singleline(&mut ui_state.marker_dialog.name);
            });

            ui.add_space(4.0);

            // Gruppe: TextEdit + Dropdown mit bestehenden Gruppen
            ui.horizontal(|ui| {
                ui.label("Gruppe:");
                ui.text_edit_singleline(&mut ui_state.marker_dialog.group);
            });

            if !existing_groups.is_empty() {
                ui.add_space(2.0);
                ui.horizontal_wrapped(|ui| {
                    ui.spacing_mut().item_spacing.x = 4.0;
                    ui.label("Bestehend:");
                    for group in &existing_groups {
                        let selected = ui_state.marker_dialog.group == *group;
                        if ui.selectable_label(selected, group).clicked() {
                            ui_state.marker_dialog.group = group.clone();
                        }
                    }
                });
            }

            ui.add_space(10.0);

            ui.horizontal(|ui| {
                let name_valid = !ui_state.marker_dialog.name.trim().is_empty();
                let group_valid = !ui_state.marker_dialog.group.trim().is_empty();

                ui.add_enabled_ui(name_valid && group_valid, |ui| {
                    if ui.button("OK").clicked() {
                        confirmed = true;
                    }
                });

                if ui.button("Abbrechen").clicked() {
                    cancelled = true;
                }
            });
        });

    if confirmed {
        events.push(AppIntent::MarkerDialogConfirmed {
            node_id,
            name: ui_state.marker_dialog.name.trim().to_string(),
            group: ui_state.marker_dialog.group.trim().to_string(),
            is_new: ui_state.marker_dialog.is_new,
        });
    } else if cancelled {
        events.push(AppIntent::MarkerDialogCancelled);
    }

    events
}

/// Zeigt den Duplikat-Bestätigungsdialog.
///
/// Wird nach dem Laden einer XML-Datei angezeigt, wenn duplizierte Nodes
/// erkannt wurden. Der Benutzer kann wählen, ob die Bereinigung durchgeführt
/// werden soll.
pub fn show_dedup_dialog(ctx: &egui::Context, ui_state: &UiState) -> Vec<AppIntent> {
    let mut events = Vec::new();

    if !ui_state.dedup_dialog.visible {
        return events;
    }

    egui::Window::new("Duplizierte Wegpunkte erkannt")
        .collapsible(false)
        .resizable(false)
        .anchor(egui::Align2::CENTER_CENTER, [0.0, 0.0])
        .show(ctx, |ui| {
            ui.set_min_width(380.0);
            ui.vertical_centered(|ui| {
                ui.add_space(8.0);
                ui.label(
                    egui::RichText::new("⚠ AutoDrive hat Teile des Netzwerks mehrfach erstellt.")
                        .color(egui::Color32::YELLOW),
                );
                ui.add_space(6.0);
                ui.label(format!(
                    "Gefunden: {} duplizierte Nodes in {} Positions-Gruppen",
                    ui_state.dedup_dialog.duplicate_count, ui_state.dedup_dialog.group_count
                ));
                ui.add_space(4.0);
                ui.label("Die Bereinigung entfernt Duplikate und leitet deren Verbindungen um.");
                ui.label("Das Original-Netzwerk bleibt vollständig erhalten.");
                ui.add_space(12.0);

                ui.horizontal(|ui| {
                    if ui.button("Bereinigen").clicked() {
                        events.push(AppIntent::DeduplicateConfirmed);
                    }
                    if ui.button("Nicht bereinigen").clicked() {
                        events.push(AppIntent::DeduplicateCancelled);
                    }
                });
            });
        });

    events
}

/// Zeigt den ZIP-Browser-Dialog zur Auswahl einer Bilddatei aus einem ZIP-Archiv.
pub fn show_zip_browser(ctx: &egui::Context, ui_state: &mut UiState) -> Vec<AppIntent> {
    let mut events = Vec::new();

    let Some(browser) = &mut ui_state.zip_browser else {
        return events;
    };

    let mut open = true;
    egui::Window::new("Bild aus ZIP wählen")
        .collapsible(false)
        .resizable(true)
        .open(&mut open)
        .default_width(400.0)
        .anchor(egui::Align2::CENTER_CENTER, [0.0, 0.0])
        .show(ctx, |ui| {
            ui.label(
                egui::RichText::new(format!("{} Bilddateien gefunden:", browser.entries.len()))
                    .strong(),
            );
            ui.add_space(4.0);

            egui::ScrollArea::vertical()
                .max_height(300.0)
                .show(ui, |ui| {
                    for (i, entry) in browser.entries.iter().enumerate() {
                        let selected = browser.selected == Some(i);
                        let label = format!("{} ({})", entry.name, format_file_size(entry.size));
                        let response = ui.selectable_label(selected, &label);
                        if response.clicked() {
                            browser.selected = Some(i);
                        }
                        if response.double_clicked() {
                            events.push(AppIntent::ZipBackgroundFileSelected {
                                zip_path: browser.zip_path.clone(),
                                entry_name: entry.name.clone(),
                            });
                        }
                    }
                });

            ui.add_space(8.0);
            ui.horizontal(|ui| {
                let can_confirm = browser.selected.is_some();
                if ui
                    .add_enabled(can_confirm, egui::Button::new("Übernehmen"))
                    .clicked()
                {
                    if let Some(idx) = browser.selected {
                        if let Some(entry) = browser.entries.get(idx) {
                            events.push(AppIntent::ZipBackgroundFileSelected {
                                zip_path: browser.zip_path.clone(),
                                entry_name: entry.name.clone(),
                            });
                        }
                    }
                }
                if ui.button("Abbrechen").clicked() {
                    events.push(AppIntent::ZipBrowserCancelled);
                }
            });
        });

    // X-Button zum Schließen
    if !open {
        events.push(AppIntent::ZipBrowserCancelled);
    }

    events
}
