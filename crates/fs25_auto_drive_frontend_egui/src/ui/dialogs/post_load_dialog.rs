//! Wiederverwendbarer Overview-Source-Dialog fuer Post-Load und Menue-Einstieg.

use super::{dialog_button_row_with_spacing, dialog_three_action_row, DialogThreeAction};
use crate::app::{AppIntent, OverviewSourceContext};
use fs25_auto_drive_host_bridge::HostLocalDialogState;

/// Zeigt den wiederverwendbaren Overview-Source-Dialog.
///
/// Im Post-Load-Kontext zeigt der Dialog automatisch erkannte Assets und
/// passende Map-Mod-ZIPs. Im Menue-Kontext dient er als Einstieg fuer die
/// manuelle ZIP-Auswahl vor dem Layer-Options-Dialog.
pub fn show_post_load_dialog(
    ctx: &egui::Context,
    ui_state: &mut HostLocalDialogState,
) -> Vec<AppIntent> {
    let mut events = Vec::new();

    if !ui_state.post_load_dialog.visible {
        return events;
    }

    let context = ui_state.post_load_dialog.context;
    let title = match context {
        OverviewSourceContext::PostLoadDetected => "Nach dem Laden erkannt",
        OverviewSourceContext::ManualMenu => "Uebersichtskarte generieren",
    };

    egui::Window::new(title)
        .collapsible(false)
        .resizable(false)
        .anchor(egui::Align2::CENTER_CENTER, [0.0, 0.0])
        .show(ctx, |ui| {
            ui.set_min_width(400.0);
            ui.vertical_centered(|ui| {
                ui.add_space(8.0);

                if matches!(context, OverviewSourceContext::ManualMenu) {
                    ui.label(
                        "Waehlen Sie eine FS25-Map-Mod-ZIP aus, um die Uebersichtskarte im naechsten Schritt zu konfigurieren.",
                    );
                    ui.add_space(12.0);
                } else {
                    if ui_state.post_load_dialog.heightmap_set {
                        ui.horizontal(|ui| {
                            ui.label(
                                egui::RichText::new("✓ Heightmap automatisch geladen")
                                    .color(egui::Color32::from_rgb(100, 200, 100)),
                            );
                        });
                        if let Some(ref hm_path) = ui_state.post_load_dialog.heightmap_path
                            && let Some(filename) = std::path::Path::new(hm_path).file_name()
                        {
                            ui.label(
                                egui::RichText::new(format!(
                                    "   {}",
                                    filename.to_string_lossy()
                                ))
                                .weak(),
                            );
                        }
                        ui.add_space(8.0);
                    }

                    if ui_state.post_load_dialog.overview_loaded {
                        ui.horizontal(|ui| {
                            ui.label(
                                egui::RichText::new("✓ Hintergrundbild automatisch geladen")
                                    .color(egui::Color32::from_rgb(100, 200, 100)),
                            );
                        });
                        ui.add_space(8.0);
                    }

                    if !ui_state.post_load_dialog.matching_zips.is_empty() {
                        if !ui_state.post_load_dialog.map_name.is_empty() {
                            ui.label(format!(
                                "Karte: \"{}\"",
                                ui_state.post_load_dialog.map_name
                            ));
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
                }

                let selected_zip = ui_state
                    .post_load_dialog
                    .matching_zips
                    .get(ui_state.post_load_dialog.selected_zip_index)
                    .and_then(|path| path.to_str())
                    .map(str::to_owned);
                let can_generate = selected_zip.is_some();

                if !can_generate {
                    ui.label(egui::RichText::new("Waehlen Sie zuerst eine gueltige ZIP-Datei aus.").weak());
                }

                if can_generate {
                    if let Some(action) = dialog_three_action_row(
                        ui,
                        "Uebersichtskarte generieren",
                        "ZIP-Datei auswaehlen",
                        "Schliessen",
                    ) {
                        match action {
                            DialogThreeAction::Primary => {
                                if let Some(path) = selected_zip {
                                    events.push(AppIntent::GenerateOverviewFromZip { path });
                                }
                            }
                            DialogThreeAction::Secondary => {
                                events.push(AppIntent::OverviewZipBrowseRequested);
                            }
                            DialogThreeAction::Tertiary => {
                                events.push(AppIntent::PostLoadDialogDismissed);
                            }
                        }
                    }
                } else {
                    dialog_button_row_with_spacing(ui, |ui| {
                        ui.add_enabled(false, egui::Button::new("Uebersichtskarte generieren"));
                        if ui.button("ZIP-Datei auswaehlen").clicked() {
                            events.push(AppIntent::OverviewZipBrowseRequested);
                        }
                        if ui.button("Schliessen").clicked() {
                            events.push(AppIntent::PostLoadDialogDismissed);
                        }
                    });
                }
            });
        });

    events
}
