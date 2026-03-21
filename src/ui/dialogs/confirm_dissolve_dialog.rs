//! Bestaetigungsdialog fuer das Aufloesen einer Segment-Gruppe.

use crate::app::AppIntent;
use crate::shared::{t, I18nKey, Language};

/// Zeigt den Bestaetigungs-Dialog zum Aufloesen einer Gruppe.
///
/// Wird angezeigt wenn der Benutzer "Gruppe aufloesen" waehlt.
/// Nach Bestaetigung wird `DissolveSegmentConfirmed` emittiert,
/// bei Abbruch wird der Dialog geschlossen ohne Aktion.
pub fn show_confirm_dissolve_dialog(
    ctx: &egui::Context,
    confirm_dissolve_id: &mut Option<u64>,
    language: Language,
) -> Vec<AppIntent> {
    let mut events = Vec::new();

    let segment_id = match *confirm_dissolve_id {
        Some(id) => id,
        None => return events,
    };

    let mut open = true;
    egui::Window::new(t(language, I18nKey::ConfirmDissolveTitle))
        .collapsible(false)
        .resizable(false)
        .anchor(egui::Align2::CENTER_CENTER, [0.0, 0.0])
        .open(&mut open)
        .show(ctx, |ui| {
            ui.label(t(language, I18nKey::ConfirmDissolveMessage));
            ui.add_space(8.0);
            ui.horizontal(|ui| {
                if ui.button(t(language, I18nKey::ConfirmDissolveOk)).clicked() {
                    events.push(AppIntent::DissolveSegmentConfirmed { segment_id });
                    *confirm_dissolve_id = None;
                }
                if ui
                    .button(t(language, I18nKey::ConfirmDissolveCancel))
                    .clicked()
                {
                    *confirm_dissolve_id = None;
                }
            });
        });

    if !open {
        *confirm_dissolve_id = None;
    }

    events
}
