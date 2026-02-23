use crate::app::{AppIntent, RoadMap, UiState};
use std::collections::BTreeSet;

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

            ui.horizontal(|ui| {
                ui.label("Name:");
                ui.text_edit_singleline(&mut ui_state.marker_dialog.name);
            });

            ui.add_space(4.0);

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
