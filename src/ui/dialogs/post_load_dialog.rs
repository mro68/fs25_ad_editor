//! Post-Load-Dialog: zeigt automatisch erkannte Heightmap und passende Map-Mod-ZIPs.

use crate::app::{AppIntent, UiState};

/// Zeigt den Post-Load-Dialog nach dem Laden einer XML-Datei.
///
/// Informiert den Benutzer über automatisch erkannte Heightmap und bietet
/// die Möglichkeit, eine Übersichtskarte aus einem passenden Map-Mod-ZIP
/// zu generieren.
pub fn show_post_load_dialog(ctx: &egui::Context, ui_state: &mut UiState) -> Vec<AppIntent> {
    let mut events = Vec::new();

    if !ui_state.post_load_dialog.visible {
        return events;
    }

    egui::Window::new("Nach dem Laden erkannt")
        .collapsible(false)
        .resizable(false)
        .anchor(egui::Align2::CENTER_CENTER, [0.0, 0.0])
        .show(ctx, |ui| {
            ui.set_min_width(400.0);
            ui.vertical_centered(|ui| {
                ui.add_space(8.0);

                // Heightmap-Info
                if ui_state.post_load_dialog.heightmap_set {
                    ui.horizontal(|ui| {
                        ui.label(
                            egui::RichText::new("✓ Heightmap automatisch geladen")
                                .color(egui::Color32::from_rgb(100, 200, 100)),
                        );
                    });
                    if let Some(ref hm_path) = ui_state.post_load_dialog.heightmap_path {
                        if let Some(filename) = std::path::Path::new(hm_path).file_name() {
                            ui.label(
                                egui::RichText::new(format!("   {}", filename.to_string_lossy()))
                                    .weak(),
                            );
                        }
                    }
                    ui.add_space(8.0);
                }

                // overview.png-Info
                if ui_state.post_load_dialog.overview_loaded {
                    ui.horizontal(|ui| {
                        ui.label(
                            egui::RichText::new("✓ Hintergrundbild automatisch geladen")
                                .color(egui::Color32::from_rgb(100, 200, 100)),
                        );
                    });
                    ui.add_space(8.0);
                }

                // ZIP-Auswahl
                if !ui_state.post_load_dialog.matching_zips.is_empty() {
                    if !ui_state.post_load_dialog.map_name.is_empty() {
                        ui.label(format!("Karte: \"{}\"", ui_state.post_load_dialog.map_name));
                        ui.add_space(4.0);
                    }

                    let zip_count = ui_state.post_load_dialog.matching_zips.len();
                    if zip_count == 1 {
                        ui.label("Passender Map-Mod gefunden:");
                        let zip_name = ui_state.post_load_dialog.matching_zips[0]
                            .file_name()
                            .and_then(|n| n.to_str())
                            .unwrap_or("?");
                        ui.label(egui::RichText::new(format!("   📦 {}", zip_name)).strong());
                    } else {
                        ui.label(format!("{} passende Map-Mods gefunden:", zip_count));
                        ui.add_space(4.0);
                        for (i, zip_path) in
                            ui_state.post_load_dialog.matching_zips.iter().enumerate()
                        {
                            let zip_name =
                                zip_path.file_name().and_then(|n| n.to_str()).unwrap_or("?");
                            ui.radio_value(
                                &mut ui_state.post_load_dialog.selected_zip_index,
                                i,
                                format!("📦 {}", zip_name),
                            );
                        }
                    }
                    ui.add_space(12.0);
                }

                // Buttons
                ui.horizontal(|ui| {
                    if !ui_state.post_load_dialog.matching_zips.is_empty() {
                        let selected_idx = ui_state.post_load_dialog.selected_zip_index;
                        if let Some(zip_path) =
                            ui_state.post_load_dialog.matching_zips.get(selected_idx)
                        {
                            if let Some(zip_str) = zip_path.to_str() {
                                if ui.button("Übersichtskarte generieren").clicked() {
                                    events.push(AppIntent::PostLoadGenerateOverview {
                                        zip_path: zip_str.to_string(),
                                    });
                                }
                            }
                        }
                    }
                    if ui.button("Schließen").clicked() {
                        events.push(AppIntent::PostLoadDialogDismissed);
                    }
                });
            });
        });

    events
}
