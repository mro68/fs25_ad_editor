//! Datei-Dialoge und modale Fenster.

use crate::app::{AppIntent, RoadMap, UiState};
use std::collections::BTreeSet;

fn path_to_ui_string(path: &std::path::Path) -> String {
    path.to_string_lossy().into_owned()
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
            .add_filter("Map Background", &["png", "jpg", "jpeg", "dds"])
            .pick_file()
        {
            // Für MVP: keine Crop-Size-Auswahl, User bekommt Original
            events.push(AppIntent::BackgroundMapSelected {
                path: path_to_ui_string(&path),
                crop_size: None,
            });
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
