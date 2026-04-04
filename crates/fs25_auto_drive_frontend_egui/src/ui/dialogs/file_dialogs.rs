use crate::app::ui_contract::{DialogRequest, DialogRequestKind, DialogResult};

fn path_to_ui_string(path: &std::path::Path) -> String {
    path.to_string_lossy().into_owned()
}

/// Verarbeitet ausstehende Datei-Dialog-Requests und gibt semantische Resultate zurueck.
pub fn handle_file_dialogs(dialog_requests: Vec<DialogRequest>) -> Vec<DialogResult> {
    dialog_requests
        .into_iter()
        .map(handle_dialog_request)
        .collect()
}

fn handle_dialog_request(request: DialogRequest) -> DialogResult {
    let kind = request.kind();

    let selected_path = match request {
        DialogRequest::PickPath {
            kind: DialogRequestKind::OpenFile,
            ..
        } => rfd::FileDialog::new()
            .add_filter("AutoDrive Config", &["xml"])
            .pick_file()
            .map(|path| path_to_ui_string(&path)),
        DialogRequest::PickPath {
            kind: DialogRequestKind::SaveFile,
            suggested_file_name,
        } => {
            let mut dialog = rfd::FileDialog::new().add_filter("AutoDrive Config", &["xml"]);
            if let Some(file_name) = suggested_file_name {
                dialog = dialog.set_file_name(&file_name);
            }
            dialog.save_file().map(|path| path_to_ui_string(&path))
        }
        DialogRequest::PickPath {
            kind: DialogRequestKind::Heightmap,
            ..
        } => rfd::FileDialog::new()
            .add_filter("Heightmap Image", &["png", "jpg", "jpeg"])
            .pick_file()
            .map(|path| path_to_ui_string(&path)),
        DialogRequest::PickPath {
            kind: DialogRequestKind::BackgroundMap,
            ..
        } => rfd::FileDialog::new()
            .add_filter("Map Background", &["png", "jpg", "jpeg", "dds", "zip"])
            .pick_file()
            .map(|path| path_to_ui_string(&path)),
        DialogRequest::PickPath {
            kind: DialogRequestKind::OverviewZip,
            ..
        } => rfd::FileDialog::new()
            .add_filter("FS25 Map-Mod ZIP", &["zip"])
            .pick_file()
            .map(|path| path_to_ui_string(&path)),
        DialogRequest::PickPath {
            kind: DialogRequestKind::CurseplayImport,
            ..
        } => rfd::FileDialog::new()
            .pick_file()
            .map(|path| path_to_ui_string(&path)),
        DialogRequest::PickPath {
            kind: DialogRequestKind::CurseplayExport,
            suggested_file_name,
        } => {
            let mut dialog = rfd::FileDialog::new();
            if let Some(file_name) = suggested_file_name {
                dialog = dialog.set_file_name(&file_name);
            } else {
                dialog = dialog.set_file_name("customField");
            }
            dialog.save_file().map(|path| path_to_ui_string(&path))
        }
    };

    selected_path.map_or(DialogResult::Cancelled { kind }, |path| {
        DialogResult::PathSelected { kind, path }
    })
}
