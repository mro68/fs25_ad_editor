use fs25_auto_drive_host_bridge::{HostDialogRequest, HostDialogRequestKind, HostDialogResult};

fn path_to_ui_string(path: &std::path::Path) -> String {
    path.to_string_lossy().into_owned()
}

/// Verarbeitet ausstehende Datei-Dialog-Requests und gibt semantische Resultate zurueck.
pub fn handle_file_dialogs(dialog_requests: Vec<HostDialogRequest>) -> Vec<HostDialogResult> {
    dialog_requests
        .into_iter()
        .map(handle_dialog_request)
        .collect()
}

fn handle_dialog_request(request: HostDialogRequest) -> HostDialogResult {
    let kind = request.kind;
    let suggested_file_name = request.suggested_file_name;

    let selected_path = match kind {
        HostDialogRequestKind::OpenFile => rfd::FileDialog::new()
            .add_filter("AutoDrive Config", &["xml"])
            .pick_file()
            .map(|path| path_to_ui_string(&path)),
        HostDialogRequestKind::SaveFile => {
            let mut dialog = rfd::FileDialog::new().add_filter("AutoDrive Config", &["xml"]);
            if let Some(file_name) = suggested_file_name.as_deref() {
                dialog = dialog.set_file_name(file_name);
            }
            dialog.save_file().map(|path| path_to_ui_string(&path))
        }
        HostDialogRequestKind::Heightmap => rfd::FileDialog::new()
            .add_filter("Heightmap Image", &["png", "jpg", "jpeg"])
            .pick_file()
            .map(|path| path_to_ui_string(&path)),
        HostDialogRequestKind::BackgroundMap => rfd::FileDialog::new()
            .add_filter("Map Background", &["png", "jpg", "jpeg", "dds", "zip"])
            .pick_file()
            .map(|path| path_to_ui_string(&path)),
        HostDialogRequestKind::OverviewZip => rfd::FileDialog::new()
            .add_filter("FS25 Map-Mod ZIP", &["zip"])
            .pick_file()
            .map(|path| path_to_ui_string(&path)),
        HostDialogRequestKind::CurseplayImport => rfd::FileDialog::new()
            .pick_file()
            .map(|path| path_to_ui_string(&path)),
        HostDialogRequestKind::CurseplayExport => {
            let mut dialog = rfd::FileDialog::new();
            if let Some(file_name) = suggested_file_name.as_deref() {
                dialog = dialog.set_file_name(file_name);
            } else {
                dialog = dialog.set_file_name("customField");
            }
            dialog.save_file().map(|path| path_to_ui_string(&path))
        }
    };

    selected_path.map_or(HostDialogResult::Cancelled { kind }, |path| {
        HostDialogResult::PathSelected { kind, path }
    })
}
