use crate::app::{AppIntent, UiState};

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

    // Übersichtskarten-ZIP-Dialog
    if ui_state.show_overview_dialog {
        ui_state.show_overview_dialog = false;

        if let Some(path) = rfd::FileDialog::new()
            .add_filter("FS25 Map-Mod ZIP", &["zip"])
            .pick_file()
        {
            events.push(AppIntent::GenerateOverviewFromZip {
                path: path_to_ui_string(&path),
            });
        }
    }

    events
}
