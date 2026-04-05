use fs25_auto_drive_host_bridge::{HostDialogRequest, HostDialogRequestKind, HostDialogResult};

fn path_to_ui_string(path: &std::path::Path) -> String {
    path.to_string_lossy().into_owned()
}

fn effective_file_name(
    kind: HostDialogRequestKind,
    suggested_file_name: Option<&str>,
) -> Option<&str> {
    match kind {
        HostDialogRequestKind::SaveFile => suggested_file_name,
        HostDialogRequestKind::CurseplayExport => {
            Some(suggested_file_name.unwrap_or("customField"))
        }
        _ => None,
    }
}

fn result_from_selected_path(
    kind: HostDialogRequestKind,
    selected_path: Option<String>,
) -> HostDialogResult {
    selected_path.map_or(HostDialogResult::Cancelled { kind }, |path| {
        HostDialogResult::PathSelected { kind, path }
    })
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
    let effective_file_name = effective_file_name(kind, suggested_file_name.as_deref());

    let selected_path = match kind {
        HostDialogRequestKind::OpenFile => rfd::FileDialog::new()
            .add_filter("AutoDrive Config", &["xml"])
            .pick_file()
            .map(|path| path_to_ui_string(&path)),
        HostDialogRequestKind::SaveFile => {
            let mut dialog = rfd::FileDialog::new().add_filter("AutoDrive Config", &["xml"]);
            if let Some(file_name) = effective_file_name {
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
            if let Some(file_name) = effective_file_name {
                dialog = dialog.set_file_name(file_name);
            }
            dialog.save_file().map(|path| path_to_ui_string(&path))
        }
    };

    result_from_selected_path(kind, selected_path)
}

#[cfg(test)]
mod tests {
    use fs25_auto_drive_host_bridge::{HostDialogRequestKind, HostDialogResult};

    use super::{effective_file_name, result_from_selected_path};

    #[test]
    fn effective_file_name_uses_request_specific_defaults() {
        assert_eq!(
            effective_file_name(HostDialogRequestKind::SaveFile, Some("savegame.xml")),
            Some("savegame.xml")
        );
        assert_eq!(
            effective_file_name(HostDialogRequestKind::SaveFile, None),
            None
        );
        assert_eq!(
            effective_file_name(HostDialogRequestKind::CurseplayExport, Some("field_7.xml")),
            Some("field_7.xml")
        );
        assert_eq!(
            effective_file_name(HostDialogRequestKind::CurseplayExport, None),
            Some("customField")
        );
    }

    #[test]
    fn result_from_selected_path_preserves_kind_for_selected_and_cancelled_results() {
        assert_eq!(
            result_from_selected_path(
                HostDialogRequestKind::SaveFile,
                Some("/tmp/savegame.xml".to_string())
            ),
            HostDialogResult::PathSelected {
                kind: HostDialogRequestKind::SaveFile,
                path: "/tmp/savegame.xml".to_string(),
            }
        );
        assert_eq!(
            result_from_selected_path(HostDialogRequestKind::CurseplayExport, None),
            HostDialogResult::Cancelled {
                kind: HostDialogRequestKind::CurseplayExport,
            }
        );
    }
}
