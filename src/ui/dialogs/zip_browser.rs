use crate::app::{AppIntent, UiState, ZipImageEntry};

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
            let overview_count = browser
                .entries
                .iter()
                .filter(|e| e.name.to_lowercase().contains("overview"))
                .count();
            let old_filter = browser.filter_overview;
            ui.horizontal(|ui| {
                ui.checkbox(&mut browser.filter_overview, "Nur Overview-Dateien");
                if browser.filter_overview {
                    ui.label(egui::RichText::new(format!("({overview_count} Treffer)")).weak());
                }
            });
            if browser.filter_overview != old_filter {
                browser.selected = None;
            }
            ui.add_space(2.0);

            let filtered: Vec<(usize, &ZipImageEntry)> = browser
                .entries
                .iter()
                .enumerate()
                .filter(|(_, e)| {
                    !browser.filter_overview || e.name.to_lowercase().contains("overview")
                })
                .collect();

            ui.label(egui::RichText::new(format!("{} Bilddateien:", filtered.len())).strong());
            ui.add_space(4.0);

            egui::ScrollArea::vertical()
                .max_height(300.0)
                .show(ui, |ui| {
                    for &(i, entry) in &filtered {
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

    if !open {
        events.push(AppIntent::ZipBrowserCancelled);
    }

    events
}
