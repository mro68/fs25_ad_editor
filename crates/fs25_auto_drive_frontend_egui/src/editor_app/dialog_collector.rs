//! Event-Sammlung fuer Dialoge (Marker, Speichern, Laden, Optionen, etc.).

use crate::app::ui_contract::{panel_action_to_intent, HostUiSnapshot};
use crate::app::AppIntent;
use crate::ui;
use eframe::egui;
use fs25_auto_drive_host_bridge::{map_host_action_to_intent, HostDialogResult, HostSessionAction};

use super::EditorApp;

fn map_dialog_results_to_intents(dialog_results: Vec<HostDialogResult>) -> Vec<AppIntent> {
    dialog_results
        .into_iter()
        .filter_map(|result| {
            map_host_action_to_intent(HostSessionAction::SubmitDialogResult { result })
        })
        .collect()
}

impl EditorApp {
    /// Sammelt Events aus allen offenen Dialogen.
    pub(super) fn collect_dialog_events(
        &mut self,
        ctx: &egui::Context,
        host_ui_snapshot: &HostUiSnapshot,
    ) -> Vec<AppIntent> {
        let mut events = Vec::new();

        let dialog_results = ui::handle_file_dialogs(self.session.take_dialog_requests());
        events.extend(map_dialog_results_to_intents(dialog_results));
        let dialog_state = self.session.dialog_ui_state_mut();
        events.extend(ui::show_heightmap_warning(
            ctx,
            dialog_state.ui.show_heightmap_warning,
        ));
        events.extend(ui::show_marker_dialog(
            ctx,
            dialog_state.ui,
            dialog_state.road_map,
        ));
        events.extend(ui::show_dedup_dialog(ctx, dialog_state.ui));
        events.extend(ui::show_confirm_dissolve_dialog(
            ctx,
            &mut dialog_state.ui.confirm_dissolve_group_id,
            dialog_state.options.language,
        ));
        events.extend(ui::show_zip_browser(ctx, dialog_state.ui));
        events.extend(ui::show_overview_options_dialog(
            ctx,
            &mut dialog_state.ui.overview_options_dialog,
        ));
        events.extend(ui::show_post_load_dialog(ctx, dialog_state.ui));
        events.extend(ui::show_save_overview_dialog(ctx, dialog_state.ui));
        events.extend(ui::show_trace_all_fields_dialog(ctx, dialog_state.ui));
        events.extend(ui::show_group_settings_popup(
            ctx,
            &mut dialog_state.ui.group_settings_popup,
            dialog_state.options,
        ));
        if let Some(options_panel_state) = host_ui_snapshot.options_panel_state() {
            let panel_actions = ui::show_options_dialog(
                ctx,
                options_panel_state.visible,
                &options_panel_state.options,
            );
            events.extend(panel_actions.into_iter().map(panel_action_to_intent));
        }

        events
    }
}

#[cfg(test)]
mod tests {
    use fs25_auto_drive_host_bridge::{HostDialogRequestKind, HostDialogResult};

    use super::map_dialog_results_to_intents;
    use crate::app::AppIntent;

    #[test]
    fn map_dialog_results_to_intents_routes_save_file_and_curseplay_export_results() {
        let intents = map_dialog_results_to_intents(vec![
            HostDialogResult::PathSelected {
                kind: HostDialogRequestKind::SaveFile,
                path: "/tmp/savegame.xml".to_string(),
            },
            HostDialogResult::PathSelected {
                kind: HostDialogRequestKind::CurseplayExport,
                path: "/tmp/customField.xml".to_string(),
            },
        ]);

        assert_eq!(intents.len(), 2);
        assert!(matches!(
            &intents[0],
            AppIntent::SaveFilePathSelected { path } if path == "/tmp/savegame.xml"
        ));
        assert!(matches!(
            &intents[1],
            AppIntent::CurseplayExportPathSelected { path } if path == "/tmp/customField.xml"
        ));
    }

    #[test]
    fn map_dialog_results_to_intents_routes_background_zip_selection_to_zip_browse() {
        let intents = map_dialog_results_to_intents(vec![HostDialogResult::PathSelected {
            kind: HostDialogRequestKind::BackgroundMap,
            path: "/tmp/background_map.ZIP".to_string(),
        }]);

        assert_eq!(intents.len(), 1);
        assert!(matches!(
            &intents[0],
            AppIntent::ZipBackgroundBrowseRequested { path } if path == "/tmp/background_map.ZIP"
        ));
    }

    #[test]
    fn map_dialog_results_to_intents_drops_cancelled_results() {
        let intents = map_dialog_results_to_intents(vec![HostDialogResult::Cancelled {
            kind: HostDialogRequestKind::SaveFile,
        }]);

        assert!(intents.is_empty());
    }
}
